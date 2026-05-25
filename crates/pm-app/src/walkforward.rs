//! Walk-forward harness — run many markets through one or more strategies and
//! aggregate the per-market results into a summary table.
//!
//! Concurrency:
//!   * tokio for S3 fetches (bounded by `max_concurrent`).
//!   * rayon for in-process matcher (the matcher itself is so fast that this
//!     is largely irrelevant, but we keep the structure for when it matters).

use anyhow::{Context, Result, anyhow};
use chrono::{Duration, NaiveDate};
use futures::StreamExt;
use pm_model::{
    MetaFeatureWeight, MetaTrainingConfig, MetaTrainingSample, MetaTrainingStats, ModelState,
    OnlineMetaCalibrator, OnlineMetaCalibratorSnapshot, SkewWinRateTable,
};
use pm_risk::PortfolioLimits;
use pm_strategy::{
    BonereaperLite, BonereaperV2, BuyYesAtOpen, DeltaNeutralMm, LateBigBet, LateConfirmation,
    LateConvexTail, NoopStrategy, PairedMmDense, ReactiveDirectional, SpotMomentumFollower,
    bonereaper::BonereaperLiteConfig,
    bonereaper_v2::{BonereaperV2Config, BonereaperV2GateStats},
    delta_neutral_mm::DeltaNeutralMmConfig,
    late_big_bet::LateBigBetConfig,
    late_confirmation::LateConfirmationConfig,
    late_convex_tail::LateConvexTailConfig,
    paired_mm::PairedMmDenseConfig,
    reactive::ReactiveDirectionalConfig,
    spot_follower::SpotMomentumFollowerConfig,
};
use pm_telonex_loader::{
    Channel, TelonexStore, load_binance_agg_trades_async, load_book_snapshot_async,
    load_pm_trades_async, resolve_binance_day, resolve_pm_trades_day,
};
use pm_types::{MarketId, SpotHistory, SpotTick, TradeHistory};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::discovery::{MarketHandle, parse_close_ts};
use crate::runner::{RunnerConfig, run_backtest};

const DEFAULT_META_MAX_FIT_SAMPLES: usize = 120_000;
const DEFAULT_META_MAX_VALIDATION_SAMPLES: usize = 60_000;
const DEFAULT_META_MAX_OOS_EVALUATION_SAMPLES: usize = 120_000;
const DEFAULT_META_MAX_SAMPLES_PER_MARKET: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum StratId {
    BuyYesAtOpen,
    ReactiveDirectional,
    PairedMm,
    SpotMomentumFollower,
    LateBigBet,
    BonereaperLite,
    BonereaperV2,
    DeltaNeutralMm,
    LateConfirmation,
    LateConvexTail,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize)]
pub enum VolatilityBand {
    Low,
    High,
}

impl VolatilityBand {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low_vol",
            Self::High => "high_vol",
        }
    }
}

impl StratId {
    pub fn name(self) -> &'static str {
        match self {
            StratId::BuyYesAtOpen => "buy_yes_at_open",
            StratId::ReactiveDirectional => "reactive_directional",
            StratId::PairedMm => "paired_mm",
            StratId::SpotMomentumFollower => "spot_momentum_follower",
            StratId::LateBigBet => "late_big_bet",
            StratId::BonereaperLite => "bonereaper_lite",
            StratId::BonereaperV2 => "bonereaper_v2",
            StratId::DeltaNeutralMm => "delta_neutral_mm",
            StratId::LateConfirmation => "late_confirmation",
            StratId::LateConvexTail => "late_convex_tail",
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalkForwardConfig {
    pub starting_cash_usdc: f64,
    pub kelly_fraction: f64,
    pub max_clip_usdc: f64,
    pub max_order_clip_multiplier: f64,
    pub max_per_market_exposure_usdc: f64,
    pub spot_symbol: String,
    pub strategies: Vec<StratId>,
    pub max_concurrent_fetches: usize,
    /// Optional research-speed replay thinning. `0` keeps every raw event.
    /// Non-zero keeps first/last plus at most one event per interval.
    pub replay_sample_ms: u64,
    pub use_outcome_label: bool,
    pub maker_rebate_bps: f64,
    pub taker_fee_bps: f64,
    /// **Portfolio mode**: process markets in chronological order, compound
    /// equity from one market into the next. Disables parallelism (each
    /// market's starting cash depends on the previous market's end cash).
    /// When `false`, each market is independent and starts from
    /// `starting_cash_usdc`.
    pub portfolio_mode: bool,
    /// In portfolio mode, override `max_clip_usdc` per market to be
    /// `clip_fraction_of_equity × current_equity`. Set to `None` to use
    /// the static `max_clip_usdc` regardless of bankroll. Typical: 0.005
    /// (0.5% of equity per bet).
    pub clip_fraction_of_equity: Option<f64>,
    /// Portfolio-level drawdown where clips begin scaling down. Expressed as
    /// a fraction below peak equity, e.g. `0.12` for 12%. Disabled when this
    /// is greater than or equal to `clip_drawdown_hard_pct`.
    pub clip_drawdown_soft_pct: f64,
    /// Portfolio-level drawdown where clips scale to zero. Expressed as a
    /// fraction below peak equity, e.g. `0.25` for 25%.
    pub clip_drawdown_hard_pct: f64,
    pub br2_disable_internal_model_gates: bool,
    pub br2_min_composite_direction: f32,
    pub br2_early_clip_frac: f32,
    pub br2_mid_clip_frac: f32,
    pub br2_late_clip_frac: f32,
    pub br2_late_max_fires: usize,
    pub br2_late_confirm_min_model_confidence: f32,
    pub br2_late_confirm_max_model_risk: f32,
    pub br2_late_confirm_min_model_side_p: f32,
    pub br2_late_confirm_min_model_edge: f32,
    pub br2_late_confirm_min_book_skew: f32,
    pub br2_late_confirm_max_whipsaw_score: f32,
    pub br2_high_skew_clip_frac: f32,
    pub br2_high_skew_max_clips: usize,
    pub br2_high_skew_max_whipsaw_score: f32,
    pub br2_late_favourite_start_secs: f32,
    pub br2_late_favourite_threshold: f32,
    pub br2_late_favourite_min_ask: f32,
    pub br2_late_favourite_max_ask: f32,
    pub br2_late_favourite_clip_frac: f32,
    pub br2_late_favourite_high_cert_clip_frac: f32,
    pub br2_late_favourite_max_clips: usize,
    pub br2_late_favourite_min_sustain_secs: f32,
    pub br2_late_favourite_sweep_depth: usize,
    pub br2_late_favourite_min_model_confidence: f32,
    pub br2_late_favourite_min_model_direction_abs: f32,
    pub br2_late_favourite_max_model_risk: f32,
    pub br2_late_favourite_min_model_side_p: f32,
    pub br2_late_favourite_min_model_edge: f32,
    pub br2_late_favourite_high_cert_min_model_edge: f32,
    pub br2_late_favourite_max_whipsaw_score: f32,
    pub br2_late_favourite_max_reversal_pressure: f32,
    pub br2_late_favourite_min_path_efficiency: f32,
    pub br2_late_favourite_max_observed_range: f32,
    pub br2_late_favourite_max_adverse_fast_momentum: f32,
    pub br2_late_favourite_max_entry_pullback: f32,
    pub br2_late_favourite_max_avg_entry_drawdown: f32,
    pub br2_tail_clip_frac: f32,
    pub br2_tail_max_clips: usize,
    pub br2_tail_min_ask: f32,
    pub br2_tail_max_ask: f32,
    pub br2_tail_extreme_threshold: f32,
    pub br2_tail_min_skew_step: f32,
    pub br2_tail_budget_favourite_spend_frac: f32,
    pub br2_tail_budget_favourite_upside_frac: f32,
    pub enforce_model_gate: bool,
    pub model_gate_min_confidence: f32,
    pub model_gate_max_risk: f32,
    pub model_gate_min_edge: f32,
    /// Split aggregate reporting by per-market price range in YES mid: high
    /// volatility if `range > threshold`.
    pub volatility_regime_threshold: f64,
    /// Enable walk-forward folds. Mutually exclusive with `fold_size`.
    /// If set, markets are split into this many chronological folds.
    pub walk_forward_folds: Option<usize>,
    /// Enable walk-forward folds with explicit fold-size (in markets).
    /// Mutually exclusive with `walk_forward_folds`.
    pub fold_size: Option<usize>,
    /// Purge this many markets around each train/test boundary.
    /// With forward-purged CV this excludes the immediately adjacent markets
    /// from training to reduce label leakage.
    pub purge_markets: usize,
    /// Do not evaluate a test fold until at least this many prior markets are
    /// available for walk-forward meta-calibrator training.
    pub min_train_markets: usize,
    /// Online meta-calibrator fit hyperparameters. These are intentionally
    /// runtime-tunable because validation often rejects overfit settings.
    pub meta_training_config: MetaTrainingConfig,
    /// Maximum market-balanced samples used for fitting the meta-calibrator.
    /// The raw extracted cache is still retained; this only bounds fit cost
    /// and prevents dense tick markets from dominating the objective.
    pub meta_max_fit_samples: usize,
    /// Maximum market-balanced samples used for validation selection.
    pub meta_max_validation_samples: usize,
    /// Maximum samples retained from a single market for fit/validation/OOS
    /// meta-calibrator diagnostics.
    pub meta_max_samples_per_market: usize,
    /// Maximum market-balanced OOS samples used in summary diagnostics.
    pub meta_max_oos_evaluation_samples: usize,
    /// Optional training/evaluation filter: keep only meta samples with base
    /// predicted-side probability at least this high.
    pub meta_train_min_base_p: f32,
    /// Optional training/evaluation filter: keep only samples past the early
    /// market penalty, e.g. `0.05` for late/candidate-regime calibration.
    pub meta_train_max_early_penalty: f32,
    /// Optional training/evaluation filter on `2 * abs(mid - 0.5)`.
    pub meta_train_min_mid_distance: f32,
    /// Optional JSON cache for extracted meta-calibrator training samples.
    /// Intended for AWS Batch/local sweeps where the train window is fixed.
    pub meta_training_samples_cache: Option<PathBuf>,
    /// Optional frozen meta-calibrator snapshot to load instead of training.
    pub meta_calibrator_snapshot_in: Option<PathBuf>,
    /// Optional path to write the trained meta-calibrator snapshot.
    pub meta_calibrator_snapshot_out: Option<PathBuf>,
    /// Disable to run strategy logic against the hand-crafted model only.
    pub enable_meta_calibration: bool,
    /// In portfolio mode, write partial outputs every N evaluated markets.
    /// Set to zero to disable checkpointing.
    pub portfolio_checkpoint_every_markets: usize,
    /// Optional per-market JSONL path used for portfolio checkpoints.
    pub checkpoint_markets_out: Option<PathBuf>,
    /// Optional summary JSON path used for portfolio checkpoints.
    pub checkpoint_summary_out: Option<PathBuf>,
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            starting_cash_usdc: 100.0,
            kelly_fraction: 0.25,
            max_clip_usdc: 20.0,
            max_order_clip_multiplier: 2.0,
            max_per_market_exposure_usdc: 50.0,
            spot_symbol: "BTCUSDT".to_string(),
            strategies: vec![StratId::ReactiveDirectional, StratId::PairedMm],
            max_concurrent_fetches: 16,
            replay_sample_ms: 0,
            use_outcome_label: false,
            maker_rebate_bps: 0.0,
            taker_fee_bps: 0.0,
            portfolio_mode: false,
            clip_fraction_of_equity: None,
            clip_drawdown_soft_pct: 1.0,
            clip_drawdown_hard_pct: 1.0,
            br2_disable_internal_model_gates: false,
            br2_min_composite_direction: 0.10,
            br2_early_clip_frac: 0.00,
            br2_mid_clip_frac: 0.00,
            br2_late_clip_frac: 1.0,
            br2_late_max_fires: 3,
            br2_late_confirm_min_model_confidence: 0.58,
            br2_late_confirm_max_model_risk: 0.80,
            br2_late_confirm_min_model_side_p: 0.58,
            br2_late_confirm_min_model_edge: 0.00,
            br2_late_confirm_min_book_skew: 0.06,
            br2_late_confirm_max_whipsaw_score: 0.85,
            br2_high_skew_clip_frac: 0.60,
            br2_high_skew_max_clips: 5,
            br2_high_skew_max_whipsaw_score: 0.75,
            br2_late_favourite_start_secs: 180.0,
            br2_late_favourite_threshold: 0.22,
            br2_late_favourite_min_ask: 0.70,
            br2_late_favourite_max_ask: 0.97,
            br2_late_favourite_clip_frac: 1.00,
            br2_late_favourite_high_cert_clip_frac: 1.00,
            br2_late_favourite_max_clips: 12,
            br2_late_favourite_min_sustain_secs: 0.0,
            br2_late_favourite_sweep_depth: 7,
            br2_late_favourite_min_model_confidence: 0.68,
            br2_late_favourite_min_model_direction_abs: 0.0,
            br2_late_favourite_max_model_risk: 0.72,
            br2_late_favourite_min_model_side_p: 0.62,
            br2_late_favourite_min_model_edge: 0.00,
            br2_late_favourite_high_cert_min_model_edge: 0.00,
            br2_late_favourite_max_whipsaw_score: 0.75,
            br2_late_favourite_max_reversal_pressure: 1.0,
            br2_late_favourite_min_path_efficiency: 0.0,
            br2_late_favourite_max_observed_range: 1.0,
            br2_late_favourite_max_adverse_fast_momentum: 1.0,
            br2_late_favourite_max_entry_pullback: 1.0,
            br2_late_favourite_max_avg_entry_drawdown: 1.0,
            br2_tail_clip_frac: 0.10,
            br2_tail_max_clips: 3,
            br2_tail_min_ask: 0.01,
            br2_tail_max_ask: 0.10,
            br2_tail_extreme_threshold: 0.30,
            br2_tail_min_skew_step: 0.02,
            br2_tail_budget_favourite_spend_frac: 0.05,
            br2_tail_budget_favourite_upside_frac: 0.25,
            enforce_model_gate: true,
            model_gate_min_confidence: 0.68,
            model_gate_max_risk: 0.72,
            model_gate_min_edge: 0.00,
            volatility_regime_threshold: 0.08,
            walk_forward_folds: None,
            fold_size: None,
            purge_markets: 0,
            min_train_markets: 0,
            meta_training_config: MetaTrainingConfig::default(),
            meta_max_fit_samples: DEFAULT_META_MAX_FIT_SAMPLES,
            meta_max_validation_samples: DEFAULT_META_MAX_VALIDATION_SAMPLES,
            meta_max_samples_per_market: DEFAULT_META_MAX_SAMPLES_PER_MARKET,
            meta_max_oos_evaluation_samples: DEFAULT_META_MAX_OOS_EVALUATION_SAMPLES,
            meta_train_min_base_p: 0.0,
            meta_train_max_early_penalty: 1.0,
            meta_train_min_mid_distance: 0.0,
            meta_training_samples_cache: None,
            meta_calibrator_snapshot_in: None,
            meta_calibrator_snapshot_out: None,
            enable_meta_calibration: true,
            portfolio_checkpoint_every_markets: 0,
            checkpoint_markets_out: None,
            checkpoint_summary_out: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketResult {
    pub asset_id: String,
    pub slug: String,
    pub close_ts: i64,
    pub outcome_label: String,
    pub volatility_range: f64,
    pub volatility_band: VolatilityBand,
    pub per_strategy: HashMap<&'static str, StrategyMarketResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyMarketResult {
    pub orders_submitted: usize,
    pub orders_filled: usize,
    pub orders_rejected_model_gate: usize,
    pub orders_rejected_model_gate_confidence: usize,
    pub orders_rejected_model_gate_risk: usize,
    pub orders_rejected_model_gate_edge: usize,
    pub pnl_usdc: f64,
    pub start_equity_usdc: f64,
    pub end_equity_usdc: f64,
    pub max_drawdown_pct: f64,
    pub fills: usize,
    pub maker_rebates_usdc: f64,
    pub clip_used_usdc: f64,
    pub yes_resolved: bool,
    /// Per-fill detail: ts, side, shares, price, notional, tag, maker, rebate.
    /// Empty if `fills_count == 0`. Use sparingly for large runs (per-market
    /// rows can grow large).
    pub fills_detail: Vec<crate::runner::Fill>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonereaper_v2_gate_stats: Option<BonereaperV2GateStats>,
    #[serde(skip_serializing)]
    pub model_training_samples: Vec<MetaTrainingSample>,
}

fn market_open_ns(m: &MarketHandle) -> i64 {
    parse_close_ts(&m.slug)
        .unwrap_or_else(|| m.close_ts.saturating_sub(300))
        .saturating_mul(1_000_000_000)
}

#[derive(Debug, Clone, Serialize)]
pub struct WalkForwardSummary {
    pub markets_attempted: usize,
    pub markets_succeeded: usize,
    /// Key runtime controls used to produce this summary.
    pub run_config: Option<SummaryRunConfig>,
    /// Overall aggregate for all markets.
    pub per_strategy: HashMap<&'static str, StrategyAggregate>,
    /// Aggregate split by volatility regime (Low/High).
    pub by_volatility_band: HashMap<VolatilityBand, HashMap<&'static str, StrategyAggregate>>,
    /// Per-fold summaries when walk-forward mode is enabled.
    pub fold_summaries: Vec<WalkForwardFoldSummary>,
    /// Meta-calibrator training/evaluation evidence for train-once portfolio
    /// runs. Empty for legacy independent-market runs without ML training.
    pub meta_calibration: Option<MetaCalibrationReport>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SummaryRunConfig {
    pub starting_cash_usdc: f64,
    pub kelly_fraction: f64,
    pub max_clip_usdc: f64,
    pub max_order_clip_multiplier: f64,
    pub max_per_market_exposure_usdc: f64,
    pub replay_sample_ms: u64,
    pub clip_fraction_of_equity: Option<f64>,
    pub clip_drawdown_soft_pct: f64,
    pub clip_drawdown_hard_pct: f64,
    pub br2_disable_internal_model_gates: bool,
    pub br2_min_composite_direction: f32,
    pub br2_early_clip_frac: f32,
    pub br2_mid_clip_frac: f32,
    pub br2_late_clip_frac: f32,
    pub br2_late_max_fires: usize,
    pub br2_late_confirm_min_model_confidence: f32,
    pub br2_late_confirm_max_model_risk: f32,
    pub br2_late_confirm_min_model_side_p: f32,
    pub br2_late_confirm_min_model_edge: f32,
    pub br2_late_confirm_min_book_skew: f32,
    pub br2_late_confirm_max_whipsaw_score: f32,
    pub br2_high_skew_clip_frac: f32,
    pub br2_high_skew_max_clips: usize,
    pub br2_high_skew_max_whipsaw_score: f32,
    pub br2_late_favourite_start_secs: f32,
    pub br2_late_favourite_threshold: f32,
    pub br2_late_favourite_min_ask: f32,
    pub br2_late_favourite_max_ask: f32,
    pub br2_late_favourite_clip_frac: f32,
    pub br2_late_favourite_high_cert_clip_frac: f32,
    pub br2_late_favourite_max_clips: usize,
    pub br2_late_favourite_min_sustain_secs: f32,
    pub br2_late_favourite_sweep_depth: usize,
    pub br2_late_favourite_min_model_confidence: f32,
    pub br2_late_favourite_min_model_direction_abs: f32,
    pub br2_late_favourite_max_model_risk: f32,
    pub br2_late_favourite_min_model_side_p: f32,
    pub br2_late_favourite_min_model_edge: f32,
    pub br2_late_favourite_high_cert_min_model_edge: f32,
    pub br2_late_favourite_max_whipsaw_score: f32,
    pub br2_late_favourite_max_reversal_pressure: f32,
    pub br2_late_favourite_min_path_efficiency: f32,
    pub br2_late_favourite_max_observed_range: f32,
    pub br2_late_favourite_max_adverse_fast_momentum: f32,
    pub br2_late_favourite_max_entry_pullback: f32,
    pub br2_late_favourite_max_avg_entry_drawdown: f32,
    pub br2_tail_clip_frac: f32,
    pub br2_tail_max_clips: usize,
    pub br2_tail_min_ask: f32,
    pub br2_tail_max_ask: f32,
    pub br2_tail_extreme_threshold: f32,
    pub br2_tail_min_skew_step: f32,
    pub br2_tail_budget_favourite_spend_frac: f32,
    pub br2_tail_budget_favourite_upside_frac: f32,
    pub enforce_model_gate: bool,
    pub model_gate_min_confidence: f32,
    pub model_gate_max_risk: f32,
    pub model_gate_min_edge: f32,
    pub min_train_markets: usize,
    pub meta_epochs: usize,
    pub meta_learning_rate: f32,
    pub meta_l2: f32,
    pub meta_weight_clip: f32,
    pub meta_max_fit_samples: usize,
    pub meta_max_validation_samples: usize,
    pub meta_max_samples_per_market: usize,
    pub meta_max_oos_evaluation_samples: usize,
    pub meta_train_min_base_p: f32,
    pub meta_train_max_early_penalty: f32,
    pub meta_train_min_mid_distance: f32,
    pub enable_meta_calibration: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetaCalibrationReport {
    pub train_markets: usize,
    pub raw_train_samples: usize,
    pub train_samples: usize,
    pub train_updates: u32,
    pub train_log_loss: Option<f32>,
    pub selected_training_config: Option<MetaTrainingConfig>,
    pub candidate_evaluations: Vec<MetaCandidateEvaluation>,
    pub raw_validation_samples: usize,
    pub validation_samples: usize,
    pub validation: Option<MetaEvaluationSummary>,
    pub selected: bool,
    pub rejected_reason: Option<String>,
    pub oos_samples: usize,
    pub oos_evaluation_samples: usize,
    pub oos: Option<MetaEvaluationSummary>,
    pub beta_enabled: bool,
    pub beta_coefficients: (f32, f32, f32),
    pub top_feature_weights: Vec<MetaFeatureWeight>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetaCandidateEvaluation {
    pub training_config: MetaTrainingConfig,
    pub train_log_loss: f32,
    pub updates: u32,
    pub beta_enabled: bool,
    pub beta_coefficients: (f32, f32, f32),
    pub isotonic_bins: usize,
    pub tree_count: usize,
    pub tree_split_count: usize,
    pub top_feature_weights: Vec<MetaFeatureWeight>,
    pub validation: MetaEvaluationSummary,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalkForwardFoldSummary {
    pub fold_idx: usize,
    pub train_end_exclusive: usize,
    pub purge_markets: usize,
    pub test_start: usize,
    pub test_end: usize,
    pub meta_train_samples: usize,
    pub meta_train_log_loss: Option<f32>,
    pub meta_oos: Option<MetaEvaluationSummary>,
    pub fold_results: WalkForwardSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct MetaEvaluationSummary {
    pub samples: usize,
    pub market_count: usize,
    pub positive_rate: f32,
    pub market_equal_weighted_positive_rate: f32,
    pub base_distribution: PredictionDistribution,
    pub calibrated_distribution: PredictionDistribution,
    pub prior_log_loss: f32,
    pub base_log_loss: f32,
    pub calibrated_log_loss: f32,
    pub log_loss_delta: f32,
    pub prior_log_loss_delta: f32,
    pub prior_brier: f32,
    pub base_brier: f32,
    pub calibrated_brier: f32,
    pub brier_delta: f32,
    pub prior_brier_delta: f32,
    pub market_equal_weighted_prior_log_loss: f32,
    pub market_equal_weighted_base_log_loss: f32,
    pub market_equal_weighted_calibrated_log_loss: f32,
    pub market_equal_weighted_prior_brier: f32,
    pub market_equal_weighted_base_brier: f32,
    pub market_equal_weighted_calibrated_brier: f32,
    pub base_accuracy: f32,
    pub calibrated_accuracy: f32,
    pub calibrated_ece: f32,
    pub calibration_bins: Vec<CalibrationBin>,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct PredictionDistribution {
    pub mean: f32,
    pub p10: f32,
    pub p50: f32,
    pub p90: f32,
    pub share_ge_55: f32,
    pub share_ge_60: f32,
    pub share_ge_65: f32,
    pub share_ge_70: f32,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct CalibrationBin {
    pub lower: f32,
    pub upper: f32,
    pub samples: usize,
    pub avg_predicted: f32,
    pub observed_rate: f32,
}

fn market_volatility_range(events: &[pm_types::ReplayEvent]) -> f64 {
    if events.is_empty() {
        return 0.0;
    }
    let mut low = f64::INFINITY;
    let mut high = f64::NEG_INFINITY;
    for e in events {
        if !e.yes_mid.is_finite() {
            continue;
        }
        let v = e.yes_mid as f64;
        if v < low {
            low = v;
        }
        if v > high {
            high = v;
        }
    }
    if !low.is_finite() {
        return 0.0;
    }
    high - low
}

fn volatility_band(range: f64, threshold: f64) -> VolatilityBand {
    if range.is_nan() {
        return VolatilityBand::Low;
    }
    if range > threshold {
        VolatilityBand::High
    } else {
        VolatilityBand::Low
    }
}

fn sample_replay_events(
    events: &[pm_types::ReplayEvent],
    sample_ms: u64,
) -> Vec<pm_types::ReplayEvent> {
    if sample_ms == 0 || events.len() <= 2 {
        return events.to_vec();
    }
    let sample_ns = (sample_ms as i64).saturating_mul(1_000_000).max(1);
    let mut sampled = Vec::with_capacity(events.len().min(320));
    sampled.push(events[0]);

    let first_ns = events[0].ts_ns;
    let mut current_bucket = None::<i64>;
    let mut pending = None::<pm_types::ReplayEvent>;
    for event in events.iter().skip(1).take(events.len().saturating_sub(2)) {
        let bucket = event.ts_ns.saturating_sub(first_ns) / sample_ns;
        if current_bucket != Some(bucket) {
            if let Some(previous) = pending.take() {
                sampled.push(previous);
            }
            current_bucket = Some(bucket);
        }
        pending = Some(*event);
    }
    if let Some(previous) = pending {
        if sampled
            .last()
            .is_none_or(|last| last.ts_ns != previous.ts_ns)
        {
            sampled.push(previous);
        }
    }

    let last = *events.last().expect("events length checked");
    if sampled.last().is_none_or(|event| event.ts_ns != last.ts_ns) {
        sampled.push(last);
    }
    sampled
}

fn compounded_clip(bankroll: f64, frac: f64) -> f64 {
    if !bankroll.is_finite() || !frac.is_finite() || bankroll <= 0.0 || frac <= 0.0 {
        return 0.0;
    }
    let raw = bankroll * frac;
    let cap = (bankroll * 0.10).max(0.0);
    if cap <= 0.50 {
        raw.min(cap).max(0.0)
    } else {
        raw.clamp(0.50, cap)
    }
}

fn drawdown_clip_multiplier(drawdown_pct: f64, soft_pct: f64, hard_pct: f64) -> f64 {
    if !drawdown_pct.is_finite()
        || !soft_pct.is_finite()
        || !hard_pct.is_finite()
        || soft_pct >= hard_pct
    {
        return 1.0;
    }
    if drawdown_pct <= soft_pct {
        1.0
    } else if drawdown_pct >= hard_pct {
        0.0
    } else {
        1.0 - (drawdown_pct - soft_pct) / (hard_pct - soft_pct)
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StrategyAggregate {
    pub total_pnl_usdc: f64,
    pub first_start_equity_usdc: f64,
    pub last_end_equity_usdc: f64,
    pub min_end_equity_usdc: f64,
    pub max_end_equity_usdc: f64,
    pub compounded_return_pct: f64,
    pub path_max_drawdown_pct: f64,
    pub mean_pnl_usdc: f64,
    pub median_pnl_usdc: f64,
    pub stdev_pnl_usdc: f64,
    pub hit_rate: f64,
    pub markets_with_orders: usize,
    pub total_orders_submitted: usize,
    pub total_orders_filled: usize,
    pub total_orders_rejected_model_gate: usize,
    pub total_orders_rejected_model_gate_confidence: usize,
    pub total_orders_rejected_model_gate_risk: usize,
    pub total_orders_rejected_model_gate_edge: usize,
    pub worst_market_pnl: f64,
    pub best_market_pnl: f64,
    pub sharpe_ratio: f64,
    pub by_fill_tag: HashMap<String, FillTagAggregate>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bonereaper_v2_gate_stats: Option<BonereaperV2GateStats>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct FillTagAggregate {
    pub fills: usize,
    pub total_notional_usdc: f64,
    pub total_pnl_usdc: f64,
    pub mean_pnl_usdc: f64,
    pub avg_fill_price: f64,
    pub hit_rate: f64,
    pub avg_side_edge_vs_fill: f64,
    pub avg_regime_whipsaw_score: f64,
    pub avg_regime_path_efficiency: f64,
    pub avg_regime_reversal_pressure: f64,
    pub avg_regime_sign_flip_rate: f64,
    pub avg_regime_realized_vol_180s_bps: f64,
}

#[derive(Debug, Default)]
struct FillTagAccumulator {
    fills: usize,
    wins: usize,
    total_pnl_usdc: f64,
    total_notional_usdc: f64,
    sum_fill_price: f64,
    sum_side_edge_vs_fill: f64,
    side_edge_samples: usize,
    sum_regime_whipsaw_score: f64,
    sum_regime_path_efficiency: f64,
    sum_regime_reversal_pressure: f64,
    sum_regime_sign_flip_rate: f64,
    sum_regime_realized_vol_180s_bps: f64,
    regime_samples: usize,
}

impl FillTagAccumulator {
    fn push(&mut self, fill: &crate::runner::Fill, pnl: f64) {
        self.fills += 1;
        self.total_pnl_usdc += pnl;
        self.total_notional_usdc += fill.notional;
        self.sum_fill_price += fill.price as f64;
        if pnl > 0.0 {
            self.wins += 1;
        }
        if let Some(edge) = fill.side_edge_vs_fill {
            self.sum_side_edge_vs_fill += edge as f64;
            self.side_edge_samples += 1;
        }
        if let (
            Some(whipsaw),
            Some(path_efficiency),
            Some(reversal_pressure),
            Some(sign_flip_rate),
            Some(realized_vol),
        ) = (
            fill.regime_whipsaw_score,
            fill.regime_path_efficiency,
            fill.regime_reversal_pressure,
            fill.regime_sign_flip_rate,
            fill.regime_realized_vol_180s_bps,
        ) {
            self.sum_regime_whipsaw_score += whipsaw as f64;
            self.sum_regime_path_efficiency += path_efficiency as f64;
            self.sum_regime_reversal_pressure += reversal_pressure as f64;
            self.sum_regime_sign_flip_rate += sign_flip_rate as f64;
            self.sum_regime_realized_vol_180s_bps += realized_vol as f64;
            self.regime_samples += 1;
        }
    }

    fn into_aggregate(self) -> FillTagAggregate {
        FillTagAggregate {
            fills: self.fills,
            total_notional_usdc: self.total_notional_usdc,
            total_pnl_usdc: self.total_pnl_usdc,
            mean_pnl_usdc: if self.fills > 0 {
                self.total_pnl_usdc / self.fills as f64
            } else {
                0.0
            },
            avg_fill_price: if self.fills > 0 {
                self.sum_fill_price / self.fills as f64
            } else {
                0.0
            },
            hit_rate: if self.fills > 0 {
                self.wins as f64 / self.fills as f64
            } else {
                0.0
            },
            avg_side_edge_vs_fill: if self.side_edge_samples > 0 {
                self.sum_side_edge_vs_fill / self.side_edge_samples as f64
            } else {
                0.0
            },
            avg_regime_whipsaw_score: if self.regime_samples > 0 {
                self.sum_regime_whipsaw_score / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_path_efficiency: if self.regime_samples > 0 {
                self.sum_regime_path_efficiency / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_reversal_pressure: if self.regime_samples > 0 {
                self.sum_regime_reversal_pressure / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_sign_flip_rate: if self.regime_samples > 0 {
                self.sum_regime_sign_flip_rate / self.regime_samples as f64
            } else {
                0.0
            },
            avg_regime_realized_vol_180s_bps: if self.regime_samples > 0 {
                self.sum_regime_realized_vol_180s_bps / self.regime_samples as f64
            } else {
                0.0
            },
        }
    }
}

/// Per-market spot-history cache so we don't re-download the same Binance day.
#[derive(Default)]
struct SpotCache {
    pub inner: HashMap<String, Arc<SpotHistory>>,
    raw_days: HashMap<String, Arc<Vec<SpotTick>>>,
}

impl SpotCache {
    async fn load_raw_day(
        &mut self,
        store: &TelonexStore,
        symbol: &str,
        date: &str,
        required: bool,
    ) -> Result<Option<Arc<Vec<SpotTick>>>> {
        if let Some(ticks) = self.raw_days.get(date) {
            return Ok(Some(ticks.clone()));
        }

        let path = match resolve_binance_day(store, "agg_trades", symbol, date).await {
            Ok(path) => path,
            Err(err) if required => {
                return Err(err).with_context(|| format!("resolve spot {symbol} {date}"));
            }
            Err(err) => {
                tracing::warn!(
                    symbol,
                    date,
                    error = %err,
                    "optional prior spot day unavailable"
                );
                return Ok(None);
            }
        };
        let (ticks, stats) = load_binance_agg_trades_async(store.store(), path).await?;
        tracing::info!(symbol, date, ticks = stats.rows_emitted, "spot day loaded");
        let ticks = Arc::new(ticks);
        self.raw_days.insert(date.to_string(), ticks.clone());
        Ok(Some(ticks))
    }

    async fn get_or_load(
        &mut self,
        store: &TelonexStore,
        symbol: &str,
        date: &str,
    ) -> Result<Arc<SpotHistory>> {
        if let Some(s) = self.inner.get(date) {
            return Ok(s.clone());
        }
        let current = self
            .load_raw_day(store, symbol, date, true)
            .await?
            .ok_or_else(|| anyhow!("missing required spot day {symbol} {date}"))?;

        let mut ticks = Vec::new();
        if let Some(prev_date) = previous_date(date)? {
            if let Some(prev) = self
                .load_raw_day(store, symbol, &prev_date, false)
                .await
                .with_context(|| format!("load optional prior spot {prev_date}"))?
            {
                ticks.reserve(prev.len() + current.len());
                ticks.extend_from_slice(&prev);
            }
        }
        ticks.extend_from_slice(&current);
        let h = Arc::new(SpotHistory::new(ticks));
        self.inner.insert(date.to_string(), h.clone());
        Ok(h)
    }
}

fn previous_date(date: &str) -> Result<Option<String>> {
    let parsed = NaiveDate::parse_from_str(date, "%Y-%m-%d")
        .with_context(|| format!("parse market date {date}"))?;
    Ok(parsed
        .checked_sub_signed(Duration::days(1))
        .map(|d| d.format("%Y-%m-%d").to_string()))
}

pub async fn run_walkforward(
    store: &TelonexStore,
    markets: &[MarketHandle],
    cfg: &WalkForwardConfig,
) -> Result<(Vec<MarketResult>, WalkForwardSummary)> {
    // Always sort by close_ts so portfolio mode is well-defined and parallel
    // mode logs read sensibly. Single source of truth.
    let mut markets_sorted: Vec<MarketHandle> = markets.to_vec();
    markets_sorted.sort_by_key(|m| m.close_ts);
    let markets = &markets_sorted[..];

    let mut spot_cache = SpotCache::default();
    // Preload all distinct spot days up front to amortize the big download.
    let unique_dates: Vec<String> = markets
        .iter()
        .map(|m| m.date.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    for date in &unique_dates {
        if !cfg.spot_symbol.is_empty() {
            spot_cache
                .get_or_load(store, &cfg.spot_symbol, date)
                .await
                .with_context(|| format!("preload spot {date}"))?;
        }
    }
    let spot_map_top: HashMap<String, Arc<SpotHistory>> = spot_cache.inner.clone();
    if cfg.portfolio_mode && (cfg.walk_forward_folds.is_some() || cfg.fold_size.is_some()) {
        return Err(anyhow!(
            "walk-forward fold configuration is not supported in portfolio mode"
        ));
    }

    if cfg.portfolio_mode {
        let preloaded_snapshot = if cfg.enable_meta_calibration {
            match cfg.meta_calibrator_snapshot_in.as_deref() {
                Some(path) => Some(read_meta_snapshot(path)?),
                None => None,
            }
        } else {
            None
        };
        if cfg.min_train_markets > 0 {
            if cfg.min_train_markets >= markets.len() {
                return Err(anyhow!(
                    "min_train_markets={} leaves no markets for portfolio evaluation",
                    cfg.min_train_markets
                ));
            }
            let mut meta_report = None;
            let meta_snapshot = if !cfg.enable_meta_calibration {
                tracing::info!(
                    min_train_markets = cfg.min_train_markets,
                    "meta-calibration disabled; preserving train/eval split without training snapshot"
                );
                None
            } else if let Some(snapshot) = preloaded_snapshot.clone() {
                meta_report = Some(MetaCalibrationReport {
                    train_markets: 0,
                    raw_train_samples: 0,
                    train_samples: 0,
                    train_updates: snapshot.updates,
                    train_log_loss: None,
                    selected_training_config: None,
                    candidate_evaluations: Vec::new(),
                    raw_validation_samples: 0,
                    validation_samples: 0,
                    validation: None,
                    selected: true,
                    rejected_reason: None,
                    oos_samples: 0,
                    oos_evaluation_samples: 0,
                    oos: None,
                    beta_enabled: snapshot.beta_enabled(),
                    beta_coefficients: snapshot.beta_coefficients(),
                    top_feature_weights: snapshot.top_feature_weights(12),
                });
                Some(snapshot)
            } else {
                let training_samples = load_or_collect_training_samples(
                    store,
                    &markets[..cfg.min_train_markets],
                    cfg,
                    &spot_map_top,
                )
                .await?;
                if training_samples.is_empty() {
                    None
                } else {
                    let selected = train_validated_meta_calibrator(
                        cfg.min_train_markets,
                        &training_samples,
                        cfg.meta_training_config,
                        MetaSampleLimits::from_config(cfg),
                        cfg.meta_calibrator_snapshot_out.as_deref(),
                    )?;
                    meta_report = Some(selected.report);
                    Some(selected.snapshot)
                }
            };
            return run_portfolio(
                store,
                &markets[cfg.min_train_markets..],
                cfg,
                &spot_map_top,
                meta_snapshot,
                meta_report,
            )
            .await;
        }
        let meta_report = preloaded_snapshot
            .as_ref()
            .map(|snapshot| MetaCalibrationReport {
                train_markets: 0,
                raw_train_samples: 0,
                train_samples: 0,
                train_updates: snapshot.updates,
                train_log_loss: None,
                selected_training_config: None,
                candidate_evaluations: Vec::new(),
                raw_validation_samples: 0,
                validation_samples: 0,
                validation: None,
                selected: true,
                rejected_reason: None,
                oos_samples: 0,
                oos_evaluation_samples: 0,
                oos: None,
                beta_enabled: snapshot.beta_enabled(),
                beta_coefficients: snapshot.beta_coefficients(),
                top_feature_weights: snapshot.top_feature_weights(12),
            });
        return run_portfolio(
            store,
            markets,
            cfg,
            &spot_map_top,
            preloaded_snapshot,
            meta_report,
        )
        .await;
    }

    let mut fold_summaries = Vec::new();
    let fold_plan = build_fold_plan(markets.len(), cfg)?;
    let use_folds = cfg.walk_forward_folds.is_some() || cfg.fold_size.is_some();

    let mut results = Vec::new();
    let mut training_samples = Vec::new();
    let mut training_loaded_until = 0usize;
    for (fold_idx, (train_end_exclusive, test_start, test_end)) in fold_plan.iter().enumerate() {
        let mut meta_train_samples = 0usize;
        let mut meta_train_log_loss = None;
        let meta_snapshot = if cfg.enable_meta_calibration && use_folds && *train_end_exclusive > 0
        {
            if *train_end_exclusive > training_loaded_until {
                let new_samples = collect_training_samples(
                    store,
                    &markets[training_loaded_until..*train_end_exclusive],
                    cfg,
                    &spot_map_top,
                )
                .await?;
                training_samples.extend(new_samples);
                training_loaded_until = *train_end_exclusive;
            }
            if training_samples.is_empty() {
                None
            } else {
                let mut state = ModelState::new();
                let stats = state.fit_meta_calibrator(&training_samples, cfg.meta_training_config);
                meta_train_samples = stats.samples;
                meta_train_log_loss = Some(stats.log_loss);
                tracing::info!(
                    fold = fold_idx,
                    samples = stats.samples,
                    updates = stats.updates,
                    train_markets = training_loaded_until,
                    log_loss = stats.log_loss,
                    "trained fold meta-calibrator"
                );
                Some(state.meta_calibrator_snapshot())
            }
        } else {
            None
        };
        let meta_oos = if cfg.enable_meta_calibration && use_folds {
            if let Some(snapshot) = meta_snapshot.as_ref() {
                let test_samples = collect_training_samples(
                    store,
                    &markets[*test_start..*test_end],
                    cfg,
                    &spot_map_top,
                )
                .await?;
                Some(evaluate_meta_calibration(snapshot, &test_samples))
            } else {
                None
            }
        } else {
            None
        };
        let fold_markets = &markets[*test_start..*test_end];
        let fold_markets = run_markets(
            store,
            fold_markets,
            cfg,
            &spot_map_top,
            *test_start,
            cfg.max_concurrent_fetches,
            meta_snapshot,
        )
        .await?;
        if use_folds {
            let mut fold_summary = aggregate(&fold_markets, &cfg.strategies);
            fold_summary.run_config = Some(summary_run_config(cfg));
            fold_summaries.push(WalkForwardFoldSummary {
                fold_idx,
                train_end_exclusive: *train_end_exclusive,
                purge_markets: cfg.purge_markets,
                test_start: *test_start,
                test_end: *test_end,
                meta_train_samples,
                meta_train_log_loss,
                meta_oos,
                fold_results: fold_summary,
            });
        }
        results.extend(fold_markets);
    }

    let mut summary = aggregate(&results, &cfg.strategies);
    summary.run_config = Some(summary_run_config(cfg));
    if use_folds {
        summary.fold_summaries = fold_summaries;
    }
    Ok((results, summary))
}

fn build_fold_plan(total: usize, cfg: &WalkForwardConfig) -> Result<Vec<(usize, usize, usize)>> {
    if cfg.walk_forward_folds.is_some() && cfg.fold_size.is_some() {
        return Err(anyhow!(
            "cannot set both --walk-forward-folds and --fold-size"
        ));
    }
    if total == 0 {
        return Ok(vec![(0, 0, 0)]);
    }
    if let Some(folds) = cfg.walk_forward_folds {
        if folds == 0 {
            return Err(anyhow!("walk-forward-folds must be >= 1"));
        }
        if folds > total {
            return Err(anyhow!(
                "walk-forward-folds ({folds}) cannot exceed number of markets ({total})"
            ));
        }
        let base = total / folds;
        let remainder = total % folds;
        let mut test_start = 0usize;
        let mut out = Vec::with_capacity(folds);
        for fold_idx in 0..folds {
            let size = base + usize::from(fold_idx < remainder);
            let test_end = (test_start + size).min(total);
            if size == 0 || test_start >= total || test_end <= test_start {
                break;
            }
            let train_end_exclusive = test_start.saturating_sub(cfg.purge_markets);
            if train_end_exclusive >= cfg.min_train_markets {
                out.push((train_end_exclusive, test_start, test_end));
            }
            test_start = test_end;
        }
        if out.is_empty() {
            return Err(anyhow!(
                "no walk-forward folds satisfy min_train_markets={} with total markets={total}",
                cfg.min_train_markets
            ));
        }
        Ok(out)
    } else if let Some(fold_size) = cfg.fold_size {
        if fold_size == 0 {
            return Err(anyhow!("fold-size must be >= 1"));
        }
        let mut out = Vec::new();
        let mut test_start = 0usize;
        while test_start < total {
            let test_end = (test_start + fold_size).min(total);
            let train_end_exclusive = test_start.saturating_sub(cfg.purge_markets);
            if train_end_exclusive >= cfg.min_train_markets {
                out.push((train_end_exclusive, test_start, test_end));
            }
            test_start = test_end;
        }
        if out.is_empty() {
            return Err(anyhow!(
                "no walk-forward folds satisfy min_train_markets={} with total markets={total}",
                cfg.min_train_markets
            ));
        }
        Ok(out)
    } else {
        Ok(vec![(0, 0, total)])
    }
}

async fn load_or_collect_training_samples(
    store: &TelonexStore,
    markets: &[MarketHandle],
    cfg: &WalkForwardConfig,
    spot_map: &HashMap<String, Arc<SpotHistory>>,
) -> Result<Vec<MetaTrainingSample>> {
    if let Some(path) = cfg.meta_training_samples_cache.as_deref() {
        if path.exists() {
            let file = File::open(path)
                .with_context(|| format!("open meta training samples cache {}", path.display()))?;
            match serde_json::from_reader::<_, Vec<MetaTrainingSample>>(BufReader::new(file)) {
                Ok(samples) => {
                    tracing::info!(
                        samples = samples.len(),
                        path = %path.display(),
                        "loaded meta training samples cache"
                    );
                    return Ok(samples);
                }
                Err(error) => {
                    tracing::warn!(
                        path = %path.display(),
                        %error,
                        "ignoring incompatible meta training samples cache"
                    );
                }
            }
        }

        let samples = collect_training_samples(store, markets, cfg, spot_map).await?;
        write_meta_training_samples(path, &samples)?;
        return Ok(samples);
    }

    collect_training_samples(store, markets, cfg, spot_map).await
}

fn write_meta_training_samples(
    path: &std::path::Path,
    samples: &[MetaTrainingSample],
) -> Result<()> {
    ensure_parent_dir(path)?;
    let file = File::create(path)
        .with_context(|| format!("create meta training samples cache {}", path.display()))?;
    serde_json::to_writer(BufWriter::new(file), samples)
        .with_context(|| format!("write meta training samples cache {}", path.display()))?;
    tracing::info!(
        samples = samples.len(),
        path = %path.display(),
        "wrote meta training samples cache"
    );
    Ok(())
}

fn read_meta_snapshot(path: &std::path::Path) -> Result<OnlineMetaCalibratorSnapshot> {
    let file = File::open(path)
        .with_context(|| format!("open meta-calibrator snapshot {}", path.display()))?;
    let snapshot = serde_json::from_reader(BufReader::new(file))
        .with_context(|| format!("read meta-calibrator snapshot {}", path.display()))?;
    tracing::info!(
        path = %path.display(),
        "loaded meta-calibrator snapshot"
    );
    Ok(snapshot)
}

fn write_meta_snapshot(
    path: &std::path::Path,
    snapshot: &OnlineMetaCalibratorSnapshot,
) -> Result<()> {
    ensure_parent_dir(path)?;
    let file = File::create(path)
        .with_context(|| format!("create meta-calibrator snapshot {}", path.display()))?;
    serde_json::to_writer_pretty(BufWriter::new(file), snapshot)
        .with_context(|| format!("write meta-calibrator snapshot {}", path.display()))?;
    tracing::info!(
        updates = snapshot.updates,
        path = %path.display(),
        "wrote meta-calibrator snapshot"
    );
    Ok(())
}

fn ensure_parent_dir(path: &std::path::Path) -> Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create parent directory {}", parent.display()))?;
    }
    Ok(())
}

struct SelectedMetaCalibrator {
    snapshot: OnlineMetaCalibratorSnapshot,
    report: MetaCalibrationReport,
}

const META_FEATURE_EARLY_MARKET_PENALTY: usize = 20;
const META_FEATURE_MID_DISTANCE_FROM_HALF: usize = 38;

#[derive(Debug, Clone, Copy)]
struct MetaSampleLimits {
    max_fit_samples: usize,
    max_validation_samples: usize,
    max_samples_per_market: usize,
    min_base_p: f32,
    max_early_penalty: f32,
    min_mid_distance: f32,
}

impl MetaSampleLimits {
    fn from_config(cfg: &WalkForwardConfig) -> Self {
        Self {
            max_fit_samples: cfg.meta_max_fit_samples,
            max_validation_samples: cfg.meta_max_validation_samples,
            max_samples_per_market: cfg.meta_max_samples_per_market,
            min_base_p: cfg.meta_train_min_base_p,
            max_early_penalty: cfg.meta_train_max_early_penalty,
            min_mid_distance: cfg.meta_train_min_mid_distance,
        }
    }
}

fn filter_meta_samples_for_training(
    samples: &[MetaTrainingSample],
    limits: MetaSampleLimits,
) -> Vec<MetaTrainingSample> {
    let min_base_p = limits.min_base_p.clamp(0.0, 1.0);
    let max_early_penalty = limits.max_early_penalty.clamp(0.0, 1.0);
    let min_mid_distance = limits.min_mid_distance.clamp(0.0, 1.0);
    if min_base_p <= 0.0 && max_early_penalty >= 1.0 && min_mid_distance <= 0.0 {
        return samples.to_vec();
    }
    samples
        .iter()
        .copied()
        .filter(|sample| {
            sample.base_side_probability >= min_base_p
                && sample.features.values[META_FEATURE_EARLY_MARKET_PENALTY] <= max_early_penalty
                && sample.features.values[META_FEATURE_MID_DISTANCE_FROM_HALF] >= min_mid_distance
        })
        .collect()
}

fn train_validated_meta_calibrator(
    train_markets: usize,
    training_samples: &[MetaTrainingSample],
    training_config: MetaTrainingConfig,
    limits: MetaSampleLimits,
    snapshot_out: Option<&std::path::Path>,
) -> Result<SelectedMetaCalibrator> {
    let filtered_training_samples = filter_meta_samples_for_training(training_samples, limits);
    if filtered_training_samples.len() != training_samples.len() {
        tracing::info!(
            raw_samples = training_samples.len(),
            filtered_samples = filtered_training_samples.len(),
            min_base_p = limits.min_base_p,
            max_early_penalty = limits.max_early_penalty,
            min_mid_distance = limits.min_mid_distance,
            "filtered meta training samples"
        );
    }
    let training_samples = filtered_training_samples.as_slice();
    if training_samples.len() < 2 {
        let snapshot = OnlineMetaCalibrator::default().snapshot();
        if let Some(path) = snapshot_out {
            write_meta_snapshot(path, &snapshot)?;
        }
        let stats = MetaTrainingStats {
            samples: 0,
            epochs: 0,
            updates: 0,
            log_loss: 0.0,
        };
        return Ok(SelectedMetaCalibrator {
            report: meta_calibration_report(
                train_markets,
                &[],
                &stats,
                &snapshot,
                None,
                Vec::new(),
                0,
                0,
                None,
                0,
                false,
                Some("not enough training samples".to_string()),
            ),
            snapshot,
        });
    }
    let (raw_fit_samples, raw_validation_samples) =
        split_meta_samples_by_market(training_samples, 0.80);
    let fit_samples = market_balanced_meta_samples(
        &raw_fit_samples,
        limits.max_fit_samples,
        limits.max_samples_per_market,
    );
    let validation_samples = market_balanced_meta_samples(
        &raw_validation_samples,
        limits.max_validation_samples,
        limits.max_samples_per_market,
    );
    tracing::info!(
        raw_fit_samples = raw_fit_samples.len(),
        fit_samples = fit_samples.len(),
        raw_validation_samples = raw_validation_samples.len(),
        validation_samples = validation_samples.len(),
        max_samples_per_market = limits.max_samples_per_market,
        "selected market-balanced meta-calibrator samples"
    );

    let candidates = meta_training_candidates(training_config);
    let mut candidate_evaluations = Vec::with_capacity(candidates.len());
    let mut best: Option<(
        MetaTrainingConfig,
        MetaTrainingStats,
        OnlineMetaCalibratorSnapshot,
        MetaEvaluationSummary,
    )> = None;
    for candidate_cfg in candidates {
        let mut state = ModelState::new();
        let stats = state.fit_meta_calibrator(&fit_samples, candidate_cfg);
        let snapshot = state.meta_calibrator_snapshot();
        let validation = evaluate_meta_calibration(&snapshot, &validation_samples);
        let validation_passed = validation.calibrated_log_loss < validation.base_log_loss
            && validation.calibrated_log_loss < validation.prior_log_loss
            && validation.calibrated_brier <= validation.base_brier
            && validation.calibrated_brier <= validation.prior_brier
            && validation.market_equal_weighted_calibrated_log_loss
                < validation.market_equal_weighted_base_log_loss
            && validation.market_equal_weighted_calibrated_log_loss
                < validation.market_equal_weighted_prior_log_loss
            && validation.market_equal_weighted_calibrated_brier
                <= validation.market_equal_weighted_base_brier
            && validation.market_equal_weighted_calibrated_brier
                <= validation.market_equal_weighted_prior_brier;
        if validation_passed {
            let replace = best
                .as_ref()
                .map(|(_, _, _, best_validation)| {
                    validation.calibrated_log_loss < best_validation.calibrated_log_loss
                })
                .unwrap_or(true);
            if replace {
                best = Some((candidate_cfg, stats, snapshot.clone(), validation.clone()));
            }
        }
        candidate_evaluations.push(MetaCandidateEvaluation {
            training_config: candidate_cfg,
            train_log_loss: stats.log_loss,
            updates: stats.updates,
            beta_enabled: snapshot.beta_enabled(),
            beta_coefficients: snapshot.beta_coefficients(),
            isotonic_bins: snapshot.isotonic_bins(),
            tree_count: snapshot.tree_count(),
            tree_split_count: snapshot.tree_split_count(),
            top_feature_weights: snapshot.top_feature_weights(8),
            validation,
            selected: false,
        });
    }
    let (selected_training_config, stats, selected_snapshot, validation, validation_passed) =
        if let Some((selected_cfg, stats, snapshot, validation)) = best {
            if let Some(candidate) = candidate_evaluations
                .iter_mut()
                .find(|candidate| candidate.training_config == selected_cfg)
            {
                candidate.selected = true;
            }
            (Some(selected_cfg), stats, snapshot, validation, true)
        } else {
            let stats = MetaTrainingStats {
                samples: fit_samples.len(),
                epochs: training_config.epochs,
                updates: 0,
                log_loss: 0.0,
            };
            let snapshot = OnlineMetaCalibrator::default().snapshot();
            let validation = evaluate_meta_calibration(&snapshot, &validation_samples);
            (None, stats, snapshot, validation, false)
        };
    if let Some(path) = snapshot_out {
        write_meta_snapshot(path, &selected_snapshot)?;
    }
    tracing::info!(
        fit_samples = stats.samples,
        validation_samples = validation.samples,
        updates = stats.updates,
        train_markets,
        train_log_loss = stats.log_loss,
        validation_market_count = validation.market_count,
        validation_positive_rate = validation.positive_rate,
        validation_market_equal_weighted_positive_rate =
            validation.market_equal_weighted_positive_rate,
        validation_prior_log_loss = validation.prior_log_loss,
        validation_base_log_loss = validation.base_log_loss,
        validation_calibrated_log_loss = validation.calibrated_log_loss,
        validation_market_equal_weighted_prior_log_loss =
            validation.market_equal_weighted_prior_log_loss,
        validation_market_equal_weighted_base_log_loss =
            validation.market_equal_weighted_base_log_loss,
        validation_market_equal_weighted_calibrated_log_loss =
            validation.market_equal_weighted_calibrated_log_loss,
        validation_calibrated_mean = validation.calibrated_distribution.mean,
        validation_calibrated_p50 = validation.calibrated_distribution.p50,
        validation_calibrated_p90 = validation.calibrated_distribution.p90,
        validation_calibrated_share_ge_60 = validation.calibrated_distribution.share_ge_60,
        validation_calibrated_share_ge_65 = validation.calibrated_distribution.share_ge_65,
        validation_prior_brier = validation.prior_brier,
        validation_base_brier = validation.base_brier,
        validation_calibrated_brier = validation.calibrated_brier,
        selected = validation_passed,
        ?selected_training_config,
        "validated portfolio meta-calibrator"
    );
    let rejected_reason = if validation_passed {
        None
    } else {
        Some(
            "validation log loss or brier did not improve over sample and market-equal base/prior baselines"
                .to_string(),
        )
    };
    let report = meta_calibration_report(
        train_markets,
        &fit_samples,
        &stats,
        &selected_snapshot,
        selected_training_config,
        candidate_evaluations,
        raw_fit_samples.len(),
        validation_samples.len(),
        Some(validation),
        raw_validation_samples.len(),
        validation_passed,
        rejected_reason,
    );
    Ok(SelectedMetaCalibrator {
        snapshot: selected_snapshot,
        report,
    })
}

fn split_meta_samples_by_market(
    samples: &[MetaTrainingSample],
    fit_fraction: f64,
) -> (Vec<MetaTrainingSample>, Vec<MetaTrainingSample>) {
    let groups = group_meta_samples_by_market(samples);
    if groups.len() < 2 {
        return (samples.to_vec(), Vec::new());
    }

    let split_at = ((groups.len() as f64) * fit_fraction).round() as usize;
    let split_at = split_at.clamp(1, groups.len().saturating_sub(1));
    let mut fit = Vec::new();
    let mut validation = Vec::new();
    for (idx, (_market_idx, market_samples)) in groups.into_iter().enumerate() {
        if idx < split_at {
            fit.extend(market_samples);
        } else {
            validation.extend(market_samples);
        }
    }
    (fit, validation)
}

fn market_balanced_meta_samples(
    samples: &[MetaTrainingSample],
    max_samples: usize,
    max_samples_per_market: usize,
) -> Vec<MetaTrainingSample> {
    if samples.is_empty() || max_samples == 0 {
        return Vec::new();
    }
    if samples.len() <= max_samples && max_samples_per_market == 0 {
        return samples.to_vec();
    }

    let groups = group_meta_samples_by_market(samples);
    if groups.is_empty() {
        return Vec::new();
    }

    let configured_per_market = if max_samples_per_market == 0 {
        usize::MAX
    } else {
        max_samples_per_market
    };
    let per_market_cap = if max_samples >= groups.len() {
        configured_per_market.min((max_samples / groups.len()).max(1))
    } else {
        1
    };
    let mut selected = Vec::with_capacity(samples.len().min(max_samples));
    for (_market_idx, market_samples) in groups {
        let take = market_samples.len().min(per_market_cap);
        extend_evenly_sampled(&market_samples, take, &mut selected);
    }

    if selected.len() <= max_samples {
        selected
    } else {
        let mut bounded = Vec::with_capacity(max_samples);
        extend_evenly_sampled(&selected, max_samples, &mut bounded);
        bounded
    }
}

fn group_meta_samples_by_market(
    samples: &[MetaTrainingSample],
) -> Vec<(u32, Vec<MetaTrainingSample>)> {
    let mut groups: BTreeMap<u32, Vec<MetaTrainingSample>> = BTreeMap::new();
    for sample in samples {
        groups.entry(sample.market_idx).or_default().push(*sample);
    }
    groups.into_iter().collect()
}

fn extend_evenly_sampled<T: Copy>(items: &[T], take: usize, out: &mut Vec<T>) {
    if take == 0 || items.is_empty() {
        return;
    }
    if take >= items.len() {
        out.extend_from_slice(items);
        return;
    }
    if take == 1 {
        out.push(items[items.len() / 2]);
        return;
    }
    let last = items.len() - 1;
    let denom = take - 1;
    for i in 0..take {
        let idx = (i * last + denom / 2) / denom;
        out.push(items[idx]);
    }
}

fn meta_training_candidates(primary: MetaTrainingConfig) -> Vec<MetaTrainingConfig> {
    let candidates = [
        primary,
        MetaTrainingConfig {
            epochs: 4,
            learning_rate: 0.005,
            l2: 0.01,
            weight_clip: 0.25,
            reset_before_fit: true,
        },
        MetaTrainingConfig {
            epochs: 8,
            learning_rate: 0.01,
            l2: 0.01,
            weight_clip: 0.50,
            reset_before_fit: true,
        },
        MetaTrainingConfig {
            epochs: 12,
            learning_rate: 0.02,
            l2: 0.005,
            weight_clip: 0.75,
            reset_before_fit: true,
        },
        MetaTrainingConfig {
            epochs: 16,
            learning_rate: 0.02,
            l2: 0.01,
            weight_clip: 1.00,
            reset_before_fit: true,
        },
    ];

    let mut deduped = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if !deduped.contains(&candidate) {
            deduped.push(candidate);
        }
    }
    deduped
}

fn meta_calibration_report(
    train_markets: usize,
    training_samples: &[MetaTrainingSample],
    stats: &MetaTrainingStats,
    snapshot: &OnlineMetaCalibratorSnapshot,
    selected_training_config: Option<MetaTrainingConfig>,
    candidate_evaluations: Vec<MetaCandidateEvaluation>,
    raw_train_samples: usize,
    validation_samples: usize,
    validation: Option<MetaEvaluationSummary>,
    raw_validation_samples: usize,
    selected: bool,
    rejected_reason: Option<String>,
) -> MetaCalibrationReport {
    MetaCalibrationReport {
        train_markets,
        raw_train_samples,
        train_samples: training_samples.len(),
        train_updates: stats.updates,
        train_log_loss: Some(stats.log_loss),
        selected_training_config,
        candidate_evaluations,
        raw_validation_samples,
        validation_samples,
        validation,
        selected,
        rejected_reason,
        oos_samples: 0,
        oos_evaluation_samples: 0,
        oos: None,
        beta_enabled: snapshot.beta_enabled(),
        beta_coefficients: snapshot.beta_coefficients(),
        top_feature_weights: snapshot.top_feature_weights(12),
    }
}

fn evaluate_meta_calibration(
    snapshot: &OnlineMetaCalibratorSnapshot,
    samples: &[MetaTrainingSample],
) -> MetaEvaluationSummary {
    let calibrator = OnlineMetaCalibrator::from_snapshot(snapshot.clone());
    if samples.is_empty() {
        return MetaEvaluationSummary {
            samples: 0,
            market_count: 0,
            positive_rate: 0.0,
            market_equal_weighted_positive_rate: 0.0,
            base_distribution: PredictionDistribution::default(),
            calibrated_distribution: PredictionDistribution::default(),
            prior_log_loss: 0.0,
            base_log_loss: 0.0,
            calibrated_log_loss: 0.0,
            log_loss_delta: 0.0,
            prior_log_loss_delta: 0.0,
            prior_brier: 0.0,
            base_brier: 0.0,
            calibrated_brier: 0.0,
            brier_delta: 0.0,
            prior_brier_delta: 0.0,
            market_equal_weighted_prior_log_loss: 0.0,
            market_equal_weighted_base_log_loss: 0.0,
            market_equal_weighted_calibrated_log_loss: 0.0,
            market_equal_weighted_prior_brier: 0.0,
            market_equal_weighted_base_brier: 0.0,
            market_equal_weighted_calibrated_brier: 0.0,
            base_accuracy: 0.0,
            calibrated_accuracy: 0.0,
            calibrated_ece: 0.0,
            calibration_bins: Vec::new(),
        };
    }

    let mut base_log_loss = 0.0f32;
    let mut calibrated_log_loss = 0.0f32;
    let mut base_brier = 0.0f32;
    let mut calibrated_brier = 0.0f32;
    let mut base_correct = 0usize;
    let mut calibrated_correct = 0usize;
    let mut observed_count = 0usize;
    let mut base_predictions = Vec::with_capacity(samples.len());
    let mut calibrated_predictions = Vec::with_capacity(samples.len());
    let mut bins = [CalibrationBinAccumulator::default(); 10];
    let mut market_accumulators: std::collections::BTreeMap<u32, MarketCalibrationAccumulator> =
        std::collections::BTreeMap::new();

    for sample in samples {
        let observed = sample.side_observed;
        let target = if observed { 1.0 } else { 0.0 };
        observed_count += usize::from(observed);
        let base = sample.base_side_probability.clamp(1.0e-6, 1.0 - 1.0e-6);
        let calibrated = calibrator
            .predict_side_win_probability(base, &sample.features)
            .clamp(1.0e-6, 1.0 - 1.0e-6);

        base_log_loss += binary_log_loss(base, observed);
        calibrated_log_loss += binary_log_loss(calibrated, observed);
        base_brier += (base - target) * (base - target);
        calibrated_brier += (calibrated - target) * (calibrated - target);
        let market_acc = market_accumulators.entry(sample.market_idx).or_default();
        market_acc.samples += 1;
        market_acc.observed += usize::from(observed);
        market_acc.base_log_loss += binary_log_loss(base, observed);
        market_acc.calibrated_log_loss += binary_log_loss(calibrated, observed);
        market_acc.base_brier += (base - target) * (base - target);
        market_acc.calibrated_brier += (calibrated - target) * (calibrated - target);
        base_predictions.push(base);
        calibrated_predictions.push(calibrated);
        base_correct += usize::from((base >= 0.5) == observed);
        calibrated_correct += usize::from((calibrated >= 0.5) == observed);
        let bin = ((calibrated * bins.len() as f32).floor() as usize).min(bins.len() - 1);
        bins[bin].samples += 1;
        bins[bin].sum_predicted += calibrated;
        bins[bin].observed += usize::from(observed);
    }

    let n = samples.len() as f32;
    let positive_rate = observed_count as f32 / n;
    let prior = positive_rate.clamp(1.0e-6, 1.0 - 1.0e-6);
    let prior_log_loss = -(positive_rate * prior.ln() + (1.0 - positive_rate) * (1.0 - prior).ln());
    let prior_brier = samples
        .iter()
        .map(|sample| {
            let target = if sample.side_observed { 1.0 } else { 0.0 };
            (prior - target) * (prior - target)
        })
        .sum::<f32>()
        / n;
    let base_log_loss = base_log_loss / n;
    let calibrated_log_loss = calibrated_log_loss / n;
    let base_brier = base_brier / n;
    let calibrated_brier = calibrated_brier / n;
    let market_count = market_accumulators.len();
    let mut market_positive_rate = 0.0f32;
    let mut market_prior_log_loss = 0.0f32;
    let mut market_base_log_loss = 0.0f32;
    let mut market_calibrated_log_loss = 0.0f32;
    let mut market_prior_brier = 0.0f32;
    let mut market_base_brier = 0.0f32;
    let mut market_calibrated_brier = 0.0f32;
    for market in market_accumulators.values() {
        let market_n = market.samples.max(1) as f32;
        let observed_rate = market.observed as f32 / market_n;
        market_positive_rate += observed_rate;
        market_prior_log_loss +=
            -(observed_rate * prior.ln() + (1.0 - observed_rate) * (1.0 - prior).ln());
        market_base_log_loss += market.base_log_loss / market_n;
        market_calibrated_log_loss += market.calibrated_log_loss / market_n;
        market_prior_brier +=
            observed_rate * (1.0 - prior) * (1.0 - prior) + (1.0 - observed_rate) * prior * prior;
        market_base_brier += market.base_brier / market_n;
        market_calibrated_brier += market.calibrated_brier / market_n;
    }
    let market_denom = market_count.max(1) as f32;
    let market_positive_rate = market_positive_rate / market_denom;
    let market_prior_log_loss = market_prior_log_loss / market_denom;
    let market_base_log_loss = market_base_log_loss / market_denom;
    let market_calibrated_log_loss = market_calibrated_log_loss / market_denom;
    let market_prior_brier = market_prior_brier / market_denom;
    let market_base_brier = market_base_brier / market_denom;
    let market_calibrated_brier = market_calibrated_brier / market_denom;
    let mut calibrated_ece = 0.0f32;
    let calibration_bins: Vec<CalibrationBin> = bins
        .iter()
        .enumerate()
        .filter_map(|(idx, bin)| {
            if bin.samples == 0 {
                return None;
            }
            let samples_f = bin.samples as f32;
            let avg_predicted = bin.sum_predicted / samples_f;
            let observed_rate = bin.observed as f32 / samples_f;
            calibrated_ece += (samples_f / n) * (avg_predicted - observed_rate).abs();
            Some(CalibrationBin {
                lower: idx as f32 / 10.0,
                upper: (idx + 1) as f32 / 10.0,
                samples: bin.samples,
                avg_predicted,
                observed_rate,
            })
        })
        .collect();
    MetaEvaluationSummary {
        samples: samples.len(),
        market_count,
        positive_rate,
        market_equal_weighted_positive_rate: market_positive_rate,
        base_distribution: prediction_distribution(&mut base_predictions),
        calibrated_distribution: prediction_distribution(&mut calibrated_predictions),
        prior_log_loss,
        base_log_loss,
        calibrated_log_loss,
        log_loss_delta: calibrated_log_loss - base_log_loss,
        prior_log_loss_delta: calibrated_log_loss - prior_log_loss,
        prior_brier,
        base_brier,
        calibrated_brier,
        brier_delta: calibrated_brier - base_brier,
        prior_brier_delta: calibrated_brier - prior_brier,
        market_equal_weighted_prior_log_loss: market_prior_log_loss,
        market_equal_weighted_base_log_loss: market_base_log_loss,
        market_equal_weighted_calibrated_log_loss: market_calibrated_log_loss,
        market_equal_weighted_prior_brier: market_prior_brier,
        market_equal_weighted_base_brier: market_base_brier,
        market_equal_weighted_calibrated_brier: market_calibrated_brier,
        base_accuracy: base_correct as f32 / n,
        calibrated_accuracy: calibrated_correct as f32 / n,
        calibrated_ece,
        calibration_bins,
    }
}

fn prediction_distribution(predictions: &mut [f32]) -> PredictionDistribution {
    if predictions.is_empty() {
        return PredictionDistribution::default();
    }
    predictions.sort_by(|a, b| a.total_cmp(b));
    let n = predictions.len();
    let mean = predictions.iter().sum::<f32>() / n as f32;
    let percentile = |q: f32| -> f32 {
        let idx = ((n - 1) as f32 * q).round() as usize;
        predictions[idx.min(n - 1)]
    };
    let share_ge = |threshold: f32| -> f32 {
        let count = predictions.iter().filter(|p| **p >= threshold).count();
        count as f32 / n as f32
    };
    PredictionDistribution {
        mean,
        p10: percentile(0.10),
        p50: percentile(0.50),
        p90: percentile(0.90),
        share_ge_55: share_ge(0.55),
        share_ge_60: share_ge(0.60),
        share_ge_65: share_ge(0.65),
        share_ge_70: share_ge(0.70),
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct CalibrationBinAccumulator {
    samples: usize,
    sum_predicted: f32,
    observed: usize,
}

#[derive(Debug, Clone, Copy, Default)]
struct MarketCalibrationAccumulator {
    samples: usize,
    observed: usize,
    base_log_loss: f32,
    calibrated_log_loss: f32,
    base_brier: f32,
    calibrated_brier: f32,
}

fn binary_log_loss(p: f32, observed: bool) -> f32 {
    let p = p.clamp(1.0e-6, 1.0 - 1.0e-6);
    if observed { -p.ln() } else { -(1.0 - p).ln() }
}

async fn collect_training_samples(
    store: &TelonexStore,
    markets: &[MarketHandle],
    cfg: &WalkForwardConfig,
    spot_map: &HashMap<String, Arc<SpotHistory>>,
) -> Result<Vec<MetaTrainingSample>> {
    if markets.is_empty() {
        return Ok(Vec::new());
    }

    let store_inner = store.store();
    let empty_spot = Arc::new(SpotHistory::default());
    let total = markets.len();
    let completed = Arc::new(AtomicUsize::new(0));
    let concurrency = cfg.max_concurrent_fetches.max(1);
    tracing::info!(markets = total, concurrency, "collecting training samples");

    let mut indexed_samples =
        futures::stream::iter(markets.iter().cloned().enumerate().map(|(idx, m)| {
            let store = store.clone();
            let store_inner = store_inner.clone();
            let spot = if cfg.spot_symbol.is_empty() {
                empty_spot.clone()
            } else {
                spot_map
                    .get(&m.date)
                    .cloned()
                    .unwrap_or_else(|| empty_spot.clone())
            };
            let use_outcome_label = cfg.use_outcome_label;
            let starting_cash_usdc = cfg.starting_cash_usdc;
            let completed = completed.clone();
            async move {
                let samples = collect_training_samples_for_market(
                    &store,
                    store_inner,
                    idx,
                    &m,
                    spot,
                    use_outcome_label,
                    starting_cash_usdc,
                )
                .await;
                let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 100 == 0 || done == total {
                    tracing::info!(done, total, "training sample progress");
                }
                (idx, samples)
            }
        }))
        .buffer_unordered(concurrency)
        .collect::<Vec<_>>()
        .await;

    indexed_samples.sort_by_key(|(idx, _)| *idx);
    let mut samples = Vec::with_capacity(markets.len());
    for (_, market_samples) in indexed_samples {
        samples.extend(market_samples);
    }
    Ok(samples)
}

async fn collect_training_samples_for_market(
    store: &TelonexStore,
    store_inner: Arc<dyn object_store::ObjectStore>,
    idx: usize,
    m: &MarketHandle,
    spot: Arc<SpotHistory>,
    use_outcome_label: bool,
    starting_cash_usdc: f64,
) -> Vec<MetaTrainingSample> {
    let path = match store
        .resolve_asset_day("polymarket", Channel::BookSnapshot25, &m.date, &m.asset_id)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!(market = %m.slug, error = %e, "training resolve path failed");
            return Vec::new();
        }
    };
    let (events, _stats) =
        match load_book_snapshot_async(store_inner, path, MarketId(idx as u32 + 1)).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(market = %m.slug, error = %e, "training tape load failed");
                return Vec::new();
            }
        };
    let resolved_yes = if use_outcome_label {
        Some(matches!(m.outcome.as_str(), "Up" | "Yes" | "yes" | "UP"))
    } else {
        None
    };
    let runner_cfg = RunnerConfig {
        starting_cash_usdc,
        market_open_ns: market_open_ns(m),
        market_close_ns: m.close_ts.saturating_mul(1_000_000_000),
        resolved_yes,
        portfolio_limits: PortfolioLimits::default(),
        equity_curve_jsonl: None,
        snapshot_every_n: 1_000_000,
        maker_rebate_bps: 0.0,
        taker_fee_bps: 0.0,
        decision_log_jsonl: None,
        decision_log_parquet: None,
        shared_model_state: None,
        update_model_state_on_resolution: true,
        meta_calibrator_snapshot: None,
        enable_meta_calibration: true,
        decision_log_every_n: 1_000_000,
        max_inventory_imbalance_shares: 1.5,
        taker_slippage_bps: 0.0,
        enforce_model_gate: false,
        model_gate_min_confidence: 0.68,
        model_gate_max_risk: 0.72,
        model_gate_min_edge: 0.05,
    };
    let mut strat = NoopStrategy;
    match run_backtest(
        &events,
        &spot,
        &TradeHistory::default(),
        &mut strat,
        &runner_cfg,
    ) {
        Ok(report) => {
            let mut samples = report.model_training_samples;
            for sample in &mut samples {
                sample.market_idx = idx as u32;
            }
            samples
        }
        Err(e) => {
            tracing::warn!(market = %m.slug, error = %e, "training sample extraction failed");
            Vec::new()
        }
    }
}

async fn run_markets(
    store: &TelonexStore,
    markets: &[MarketHandle],
    cfg: &WalkForwardConfig,
    spot_map: &HashMap<String, Arc<SpotHistory>>,
    market_id_offset: usize,
    max_concurrent_fetches: usize,
    meta_calibrator_snapshot: Option<OnlineMetaCalibratorSnapshot>,
) -> Result<Vec<MarketResult>> {
    if markets.is_empty() {
        return Ok(Vec::new());
    }
    let spot_empty = Arc::new(SpotHistory::default());
    let store_inner = store.store();
    let cfg_arc = Arc::new(cfg.clone());
    let stream = futures::stream::iter(markets.iter().enumerate().map(|(idx, m)| {
        let store_inner = store_inner.clone();
        let spot_map = spot_map.clone();
        let spot_empty = spot_empty.clone();
        let cfg_arc = cfg_arc.clone();
        let meta_calibrator_snapshot = meta_calibrator_snapshot.clone();
        let store_for_resolve = store.clone();
        async move {
            let started = Instant::now();
            let path = match store_for_resolve
                .resolve_asset_day(
                    "polymarket",
                    Channel::BookSnapshot25,
                    &m.date,
                    &m.asset_id,
                )
                .await
            {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(market = %m.slug, error = %e, "resolve path failed");
                    return None;
                }
            };
            let (events, _stats) = match load_book_snapshot_async(
                store_inner.clone(),
                path,
                MarketId((idx + market_id_offset) as u32 + 1),
            )
            .await
            {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(market = %m.slug, error = %e, "tape load failed");
                    return None;
                }
            };
            let sampled_events;
            let events_for_run = if cfg_arc.replay_sample_ms > 0 {
                sampled_events = sample_replay_events(&events, cfg_arc.replay_sample_ms);
                sampled_events.as_slice()
            } else {
                events.as_slice()
            };
            // Per-market trades (best effort: skip if missing/erroring).
            let trades = match resolve_pm_trades_day(&store_for_resolve, &m.date, &m.asset_id).await {
                Ok(tp) => match load_pm_trades_async(store_inner.clone(), tp).await {
                    Ok((ticks, _)) => Arc::new(TradeHistory::new(ticks)),
                    Err(e) => {
                        tracing::debug!(market = %m.slug, error = %e, "trades load failed");
                        Arc::new(TradeHistory::default())
                    }
                },
                Err(_) => Arc::new(TradeHistory::default()),
            };
            let spot = if cfg_arc.spot_symbol.is_empty() {
                spot_empty
            } else {
                spot_map.get(&m.date).cloned().unwrap_or(spot_empty)
            };
            let resolved_yes = if cfg_arc.use_outcome_label {
                Some(matches!(m.outcome.as_str(), "Up" | "Yes" | "yes" | "UP"))
            } else {
                None
            };
            let runner_cfg = RunnerConfig {
                starting_cash_usdc: cfg_arc.starting_cash_usdc,
                market_open_ns: market_open_ns(&m),
                market_close_ns: m.close_ts.saturating_mul(1_000_000_000),
                resolved_yes,
                portfolio_limits: PortfolioLimits {
                    max_clip_usdc: cfg_arc.max_clip_usdc * cfg_arc.max_order_clip_multiplier,
                    max_per_market_exposure_usdc: cfg_arc.max_per_market_exposure_usdc,
                    ..PortfolioLimits::default()
                },
                equity_curve_jsonl: None,
                snapshot_every_n: 1_000_000,
                maker_rebate_bps: cfg_arc.maker_rebate_bps,
                taker_fee_bps: cfg_arc.taker_fee_bps,
                decision_log_jsonl: None,
                decision_log_parquet: None,
                shared_model_state: None,
                update_model_state_on_resolution: meta_calibrator_snapshot.is_none(),
                meta_calibrator_snapshot,
                enable_meta_calibration: cfg_arc.enable_meta_calibration,
                decision_log_every_n: 1_000_000,
                // Hard inventory cap: never let |yes - no| exceed 1.5 shares
                // per market (paired-MM safety net).
                max_inventory_imbalance_shares: 1.5,
                taker_slippage_bps: 15.0,
                enforce_model_gate: cfg_arc.enforce_model_gate,
                model_gate_min_confidence: cfg_arc.model_gate_min_confidence,
                model_gate_max_risk: cfg_arc.model_gate_max_risk,
                model_gate_min_edge: cfg_arc.model_gate_min_edge,
            };

            let mut per_strategy = HashMap::new();
            for &strat in &cfg_arc.strategies {
                match run_one_strategy(
                    strat,
                    &cfg_arc,
                    events_for_run,
                    &spot,
                    &trades,
                    &runner_cfg,
                    cfg_arc.starting_cash_usdc,
                    cfg_arc.max_clip_usdc,
                    None,
                    None,
                ) {
                    Ok(mut r) => {
                        for sample in &mut r.model_training_samples {
                            sample.market_idx = (idx + market_id_offset) as u32;
                        }
                        per_strategy.insert(strat.name(), r);
                    }
                    Err(e) => {
                        tracing::warn!(market = %m.slug, strategy = strat.name(), error = %e, "strategy run failed");
                    }
                }
            }

            let volatility_range = market_volatility_range(events_for_run);
            let volatility_band = volatility_band(volatility_range, cfg_arc.volatility_regime_threshold);

            tracing::debug!(
                market = %m.slug,
                events = events_for_run.len(),
                raw_events = events.len(),
                elapsed_ms = started.elapsed().as_millis() as u64,
                "market done",
            );

            Some(MarketResult {
                asset_id: m.asset_id.clone(),
                slug: m.slug.clone(),
                close_ts: m.close_ts,
                outcome_label: m.outcome.clone(),
                volatility_range,
                volatility_band,
                per_strategy,
            })
        }
    }))
    .buffer_unordered(max_concurrent_fetches);

    let mut results = Vec::with_capacity(markets.len());
    let mut stream = std::pin::pin!(stream);
    let mut completed = 0usize;
    while let Some(maybe_result) = stream.next().await {
        if let Some(r) = maybe_result {
            results.push(r);
        }
        completed += 1;
        if completed % 50 == 0 {
            tracing::info!(done = completed, total = markets.len(), "progress");
        }
    }

    results.sort_by_key(|r| r.close_ts);
    Ok(results)
}

fn run_one_strategy(
    strat: StratId,
    cfg: &WalkForwardConfig,
    events: &[pm_types::ReplayEvent],
    spot: &SpotHistory,
    trades: &TradeHistory,
    runner_cfg: &RunnerConfig,
    bankroll: f64,
    clip: f64,
    shared_skew_table: Option<Arc<Mutex<SkewWinRateTable>>>,
    shared_model_state: Option<Arc<Mutex<ModelState>>>,
) -> Result<StrategyMarketResult> {
    let (report, bonereaper_v2_gate_stats) = match strat {
        StratId::BuyYesAtOpen => {
            let mut s = BuyYesAtOpen::new(10.0);
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::ReactiveDirectional => {
            let mut s = ReactiveDirectional::new(ReactiveDirectionalConfig {
                bankroll_usdc: bankroll,
                kelly_fraction: cfg.kelly_fraction,
                max_clip_usdc: clip,
                early_pair_clip_usdc: 0.5,
                conviction_threshold_yes: 0.68,
                conviction_threshold_no: 0.68,
                book_weight: 0.3,
                spot_weight: 0.7,
                shared_model_state,
                shared_skew_table,
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::PairedMm => {
            let mut s = PairedMmDense::new(PairedMmDenseConfig {
                clip_shares: (clip * 0.3 / 5.0).max(0.05), // scale with clip
                max_rungs_per_leg: 3,
                max_entry_pair_cost: 1.05,
                max_leg_imbalance_shares: 0.6,
                min_refresh_ns: 2_000_000_000,
                ..PairedMmDenseConfig::default()
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::SpotMomentumFollower => {
            let mut s = SpotMomentumFollower::new(SpotMomentumFollowerConfig {
                clip_usdc: clip,
                ..SpotMomentumFollowerConfig::default()
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::LateBigBet => {
            let mut s = LateBigBet::new(LateBigBetConfig {
                bankroll_usdc: bankroll,
                kelly_fraction: 0.5,
                max_clip_usdc: clip,
                late_seconds: 60.0,
                min_conviction: 0.15,
                max_ask_yes: 0.94,
                min_bid_yes: 0.06,
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::BonereaperLite => {
            let mut s = BonereaperLite::new(BonereaperLiteConfig {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip,
                ..BonereaperLiteConfig::default()
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::DeltaNeutralMm => {
            let mut s = DeltaNeutralMm::new(DeltaNeutralMmConfig {
                clip_shares: (clip * 0.3).max(0.1),
                max_pair_cost: 1.02,
                max_inventory_delta_shares: 1.0,
                ..DeltaNeutralMmConfig::default()
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::BonereaperV2 => {
            let mut s = BonereaperV2::new(BonereaperV2Config {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip,
                disable_internal_model_gates: cfg.br2_disable_internal_model_gates,
                min_composite_direction: cfg.br2_min_composite_direction,
                early_clip_frac: cfg.br2_early_clip_frac,
                mid_clip_frac: cfg.br2_mid_clip_frac,
                late_clip_frac: cfg.br2_late_clip_frac,
                late_max_fires: cfg.br2_late_max_fires,
                late_confirm_min_model_confidence: cfg.br2_late_confirm_min_model_confidence,
                late_confirm_max_model_risk: cfg.br2_late_confirm_max_model_risk,
                late_confirm_min_model_side_p: cfg.br2_late_confirm_min_model_side_p,
                late_confirm_min_model_edge: cfg.br2_late_confirm_min_model_edge,
                late_confirm_min_book_skew: cfg.br2_late_confirm_min_book_skew,
                late_confirm_max_whipsaw_score: cfg.br2_late_confirm_max_whipsaw_score,
                high_skew_clip_frac: cfg.br2_high_skew_clip_frac,
                high_skew_max_clips: cfg.br2_high_skew_max_clips,
                high_skew_max_whipsaw_score: cfg.br2_high_skew_max_whipsaw_score,
                late_favourite_start_secs: cfg.br2_late_favourite_start_secs,
                late_favourite_threshold: cfg.br2_late_favourite_threshold,
                late_favourite_min_ask: cfg.br2_late_favourite_min_ask,
                late_favourite_max_ask: cfg.br2_late_favourite_max_ask,
                late_favourite_clip_frac: cfg.br2_late_favourite_clip_frac,
                late_favourite_high_cert_clip_frac: cfg.br2_late_favourite_high_cert_clip_frac,
                late_favourite_max_clips: cfg.br2_late_favourite_max_clips,
                late_favourite_min_sustain_secs: cfg.br2_late_favourite_min_sustain_secs,
                late_favourite_sweep_depth: cfg.br2_late_favourite_sweep_depth,
                late_favourite_min_model_confidence: cfg.br2_late_favourite_min_model_confidence,
                late_favourite_min_model_direction_abs: cfg
                    .br2_late_favourite_min_model_direction_abs,
                late_favourite_max_model_risk: cfg.br2_late_favourite_max_model_risk,
                late_favourite_min_model_side_p: cfg.br2_late_favourite_min_model_side_p,
                late_favourite_min_model_edge: cfg.br2_late_favourite_min_model_edge,
                late_favourite_high_cert_min_model_edge: cfg
                    .br2_late_favourite_high_cert_min_model_edge,
                late_favourite_max_whipsaw_score: cfg.br2_late_favourite_max_whipsaw_score,
                late_favourite_max_reversal_pressure: cfg.br2_late_favourite_max_reversal_pressure,
                late_favourite_min_path_efficiency: cfg.br2_late_favourite_min_path_efficiency,
                late_favourite_max_observed_range: cfg.br2_late_favourite_max_observed_range,
                late_favourite_max_adverse_fast_momentum: cfg
                    .br2_late_favourite_max_adverse_fast_momentum,
                late_favourite_max_entry_pullback: cfg.br2_late_favourite_max_entry_pullback,
                late_favourite_max_avg_entry_drawdown: cfg
                    .br2_late_favourite_max_avg_entry_drawdown,
                tail_clip_frac: cfg.br2_tail_clip_frac,
                tail_max_clips: cfg.br2_tail_max_clips,
                tail_min_ask: cfg.br2_tail_min_ask,
                tail_max_ask: cfg.br2_tail_max_ask,
                tail_extreme_threshold: cfg.br2_tail_extreme_threshold,
                tail_min_skew_step: cfg.br2_tail_min_skew_step,
                tail_budget_favourite_spend_frac: cfg.br2_tail_budget_favourite_spend_frac,
                tail_budget_favourite_upside_frac: cfg.br2_tail_budget_favourite_upside_frac,
                ..BonereaperV2Config::default()
            });
            let report = run_backtest(events, spot, trades, &mut s, runner_cfg)?;
            (report, Some(s.gate_stats()))
        }
        StratId::LateConfirmation => {
            let mut s = LateConfirmation::new(LateConfirmationConfig {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip,
                ..LateConfirmationConfig::default()
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
        StratId::LateConvexTail => {
            let mut s = LateConvexTail::new(LateConvexTailConfig {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip * 0.2,
                ..LateConvexTailConfig::default()
            });
            (
                run_backtest(events, spot, trades, &mut s, runner_cfg)?,
                None,
            )
        }
    };
    Ok(StrategyMarketResult {
        orders_submitted: report.counters.orders_submitted,
        orders_filled: report.counters.orders_filled_taker + report.counters.orders_filled_maker,
        orders_rejected_model_gate: report.counters.orders_rejected_model_gate,
        orders_rejected_model_gate_confidence: report
            .counters
            .orders_rejected_model_gate_confidence,
        orders_rejected_model_gate_risk: report.counters.orders_rejected_model_gate_risk,
        orders_rejected_model_gate_edge: report.counters.orders_rejected_model_gate_edge,
        pnl_usdc: report.pnl_usdc,
        start_equity_usdc: bankroll,
        end_equity_usdc: report.end_equity_usdc,
        max_drawdown_pct: report.max_drawdown_pct,
        fills: report.fills.len(),
        maker_rebates_usdc: report.maker_rebates_usdc,
        clip_used_usdc: clip,
        yes_resolved: report.yes_resolved,
        fills_detail: report.fills,
        bonereaper_v2_gate_stats,
        model_training_samples: report.model_training_samples,
    })
}

/// Portfolio-mode walk-forward: sequential, chronological, compounding equity.
/// Each strategy maintains its own running bankroll; per-market max_clip can
/// scale with equity via `cfg.clip_fraction_of_equity`.
async fn run_portfolio(
    store: &TelonexStore,
    markets: &[MarketHandle],
    cfg: &WalkForwardConfig,
    spot_map: &HashMap<String, Arc<SpotHistory>>,
    meta_calibrator_snapshot: Option<OnlineMetaCalibratorSnapshot>,
    mut meta_report: Option<MetaCalibrationReport>,
) -> Result<(Vec<MarketResult>, WalkForwardSummary)> {
    let store_inner = store.store();
    let empty_spot = Arc::new(SpotHistory::default());
    let mut equity_by_strategy: HashMap<&'static str, f64> = cfg
        .strategies
        .iter()
        .map(|s| (s.name(), cfg.starting_cash_usdc))
        .collect();
    let mut peak_equity_by_strategy = equity_by_strategy.clone();
    let mut shared_skew_tables: HashMap<&'static str, Arc<Mutex<SkewWinRateTable>>> =
        HashMap::new();
    let mut shared_model_states: HashMap<&'static str, Arc<Mutex<ModelState>>> = HashMap::new();
    for strat in &cfg.strategies {
        if *strat == StratId::ReactiveDirectional {
            shared_skew_tables.insert(strat.name(), Arc::new(Mutex::new(SkewWinRateTable::new())));
            shared_model_states.insert(
                strat.name(),
                Arc::new(Mutex::new(model_state_with_snapshot(
                    meta_calibrator_snapshot.as_ref(),
                ))),
            );
        } else {
            shared_model_states.insert(
                strat.name(),
                Arc::new(Mutex::new(model_state_with_snapshot(
                    meta_calibrator_snapshot.as_ref(),
                ))),
            );
        }
    }
    let mut results: Vec<MarketResult> = Vec::with_capacity(markets.len());
    let mut oos_meta_samples: Vec<MetaTrainingSample> = Vec::with_capacity(markets.len());

    for (idx, m) in markets.iter().enumerate() {
        let path = match store
            .resolve_asset_day("polymarket", Channel::BookSnapshot25, &m.date, &m.asset_id)
            .await
        {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!(market = %m.slug, error = %e, "resolve path failed");
                continue;
            }
        };
        let (events, _stats) =
            match load_book_snapshot_async(store_inner.clone(), path, MarketId(idx as u32 + 1))
                .await
            {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(market = %m.slug, error = %e, "tape load failed");
                    continue;
                }
            };
        let sampled_events;
        let events_for_run = if cfg.replay_sample_ms > 0 {
            sampled_events = sample_replay_events(&events, cfg.replay_sample_ms);
            sampled_events.as_slice()
        } else {
            events.as_slice()
        };
        let trades = match resolve_pm_trades_day(store, &m.date, &m.asset_id).await {
            Ok(tp) => match load_pm_trades_async(store_inner.clone(), tp).await {
                Ok((ticks, _)) => Arc::new(TradeHistory::new(ticks)),
                Err(_) => Arc::new(TradeHistory::default()),
            },
            Err(_) => Arc::new(TradeHistory::default()),
        };
        let spot = if cfg.spot_symbol.is_empty() {
            empty_spot.clone()
        } else {
            spot_map
                .get(&m.date)
                .cloned()
                .unwrap_or_else(|| empty_spot.clone())
        };
        let resolved_yes = if cfg.use_outcome_label {
            Some(matches!(m.outcome.as_str(), "Up" | "Yes" | "yes" | "UP"))
        } else {
            None
        };

        let mut per_strategy = HashMap::new();
        let mut captured_meta_sample_for_market = false;
        for &strat in &cfg.strategies {
            let bankroll = *equity_by_strategy
                .get(strat.name())
                .unwrap_or(&cfg.starting_cash_usdc);
            let peak_equity = *peak_equity_by_strategy
                .get(strat.name())
                .unwrap_or(&cfg.starting_cash_usdc);
            let drawdown_pct = if peak_equity > 0.0 {
                1.0 - bankroll / peak_equity
            } else {
                0.0
            };
            let clip_multiplier = drawdown_clip_multiplier(
                drawdown_pct,
                cfg.clip_drawdown_soft_pct,
                cfg.clip_drawdown_hard_pct,
            );
            // Per-market clip: a fraction of current equity (compounds), else
            // static fallback. Hard floor + ceiling for sanity.
            let clip = match cfg.clip_fraction_of_equity {
                Some(frac) => compounded_clip(bankroll, frac),
                None => cfg.max_clip_usdc,
            } * clip_multiplier;
            let runner_cfg = RunnerConfig {
                starting_cash_usdc: bankroll,
                market_open_ns: market_open_ns(&m),
                market_close_ns: m.close_ts.saturating_mul(1_000_000_000),
                resolved_yes,
                portfolio_limits: PortfolioLimits {
                    max_clip_usdc: clip * cfg.max_order_clip_multiplier,
                    max_per_market_exposure_usdc: cfg.max_per_market_exposure_usdc,
                    max_daily_exposure_usdc: bankroll * 5.0,
                    ..PortfolioLimits::default()
                },
                equity_curve_jsonl: None,
                snapshot_every_n: 1_000_000,
                maker_rebate_bps: cfg.maker_rebate_bps,
                taker_fee_bps: cfg.taker_fee_bps,
                decision_log_jsonl: None,
                decision_log_parquet: None,
                shared_model_state: if strat == StratId::ReactiveDirectional {
                    None
                } else {
                    shared_model_states.get(strat.name()).cloned()
                },
                update_model_state_on_resolution: meta_calibrator_snapshot.is_none(),
                meta_calibrator_snapshot: meta_calibrator_snapshot.clone(),
                enable_meta_calibration: cfg.enable_meta_calibration,
                decision_log_every_n: 1_000_000,
                max_inventory_imbalance_shares: 1.5,
                taker_slippage_bps: 15.0,
                enforce_model_gate: cfg.enforce_model_gate,
                model_gate_min_confidence: cfg.model_gate_min_confidence,
                model_gate_max_risk: cfg.model_gate_max_risk,
                model_gate_min_edge: cfg.model_gate_min_edge,
            };
            match run_one_strategy(
                strat,
                cfg,
                events_for_run,
                &spot,
                &trades,
                &runner_cfg,
                bankroll,
                clip,
                shared_skew_tables.get(strat.name()).cloned(),
                shared_model_states.get(strat.name()).cloned(),
            ) {
                Ok(mut r) => {
                    for sample in &mut r.model_training_samples {
                        sample.market_idx = idx as u32;
                    }
                    if !captured_meta_sample_for_market && !r.model_training_samples.is_empty() {
                        oos_meta_samples.extend(r.model_training_samples.iter().copied());
                        captured_meta_sample_for_market = true;
                    }
                    equity_by_strategy.insert(strat.name(), r.end_equity_usdc);
                    peak_equity_by_strategy
                        .entry(strat.name())
                        .and_modify(|peak| *peak = peak.max(r.end_equity_usdc))
                        .or_insert(r.end_equity_usdc);
                    per_strategy.insert(strat.name(), r);
                }
                Err(e) => {
                    tracing::warn!(market = %m.slug, strategy = strat.name(), error = %e, "strategy run failed");
                }
            }
        }

        let volatility_range = market_volatility_range(events_for_run);
        let volatility_band = volatility_band(volatility_range, cfg.volatility_regime_threshold);

        if (idx + 1) % 50 == 0 {
            let equity_strs: Vec<String> = cfg
                .strategies
                .iter()
                .map(|s| {
                    format!(
                        "{}={:.2}",
                        s.name(),
                        equity_by_strategy.get(s.name()).copied().unwrap_or(0.0)
                    )
                })
                .collect();
            tracing::info!(
                done = idx + 1,
                total = markets.len(),
                equity = %equity_strs.join(" "),
                "portfolio progress",
            );
        }

        results.push(MarketResult {
            asset_id: m.asset_id.clone(),
            slug: m.slug.clone(),
            close_ts: m.close_ts,
            outcome_label: m.outcome.clone(),
            volatility_range,
            volatility_band,
            per_strategy,
        });

        if cfg.portfolio_checkpoint_every_markets > 0
            && results.len() % cfg.portfolio_checkpoint_every_markets == 0
        {
            write_portfolio_checkpoint(
                cfg,
                &results,
                meta_report.as_ref(),
                meta_calibrator_snapshot.as_ref(),
                &oos_meta_samples,
            )
            .with_context(|| {
                format!("write portfolio checkpoint after {} markets", results.len())
            })?;
        }
    }

    let mut summary = aggregate(&results, &cfg.strategies);
    summary.run_config = Some(summary_run_config(cfg));
    if let Some(report) = meta_report.as_mut() {
        report.oos_samples = oos_meta_samples.len();
        if let Some(snapshot) = meta_calibrator_snapshot.as_ref() {
            let filtered_oos_samples = filter_meta_samples_for_training(
                &oos_meta_samples,
                MetaSampleLimits::from_config(cfg),
            );
            let evaluation_samples = market_balanced_meta_samples(
                &filtered_oos_samples,
                cfg.meta_max_oos_evaluation_samples,
                cfg.meta_max_samples_per_market,
            );
            report.oos_evaluation_samples = evaluation_samples.len();
            report.oos = Some(evaluate_meta_calibration(snapshot, &evaluation_samples));
        }
        summary.meta_calibration = meta_report;
    }
    Ok((results, summary))
}

fn model_state_with_snapshot(snapshot: Option<&OnlineMetaCalibratorSnapshot>) -> ModelState {
    let mut state = ModelState::new();
    if let Some(snapshot) = snapshot {
        state.load_meta_calibrator_snapshot(snapshot.clone());
    }
    state
}

fn write_portfolio_checkpoint(
    cfg: &WalkForwardConfig,
    results: &[MarketResult],
    meta_report: Option<&MetaCalibrationReport>,
    meta_calibrator_snapshot: Option<&OnlineMetaCalibratorSnapshot>,
    oos_meta_samples: &[MetaTrainingSample],
) -> Result<()> {
    let mut summary = aggregate(results, &cfg.strategies);
    summary.run_config = Some(summary_run_config(cfg));
    if let Some(report) = meta_report {
        let mut report = report.clone();
        report.oos_samples = oos_meta_samples.len();
        if let Some(snapshot) = meta_calibrator_snapshot {
            let filtered_oos_samples = filter_meta_samples_for_training(
                oos_meta_samples,
                MetaSampleLimits::from_config(cfg),
            );
            let evaluation_samples = market_balanced_meta_samples(
                &filtered_oos_samples,
                cfg.meta_max_oos_evaluation_samples,
                cfg.meta_max_samples_per_market,
            );
            report.oos_evaluation_samples = evaluation_samples.len();
            report.oos = Some(evaluate_meta_calibration(snapshot, &evaluation_samples));
        }
        summary.meta_calibration = Some(report);
    }
    if let Some(path) = cfg.checkpoint_markets_out.as_deref() {
        write_market_results_jsonl_atomic(path, results)?;
        tracing::info!(
            ?path,
            markets = results.len(),
            "wrote portfolio markets checkpoint"
        );
    }
    if let Some(path) = cfg.checkpoint_summary_out.as_deref() {
        write_summary_json_atomic(path, &summary)?;
        tracing::info!(
            ?path,
            markets = results.len(),
            "wrote portfolio summary checkpoint"
        );
    }
    Ok(())
}

pub fn write_market_results_jsonl_atomic(
    path: &std::path::Path,
    results: &[MarketResult],
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    let tmp = temp_sibling_path(path);
    {
        let mut file = std::fs::File::create(&tmp)
            .with_context(|| format!("create temp results {}", tmp.display()))?;
        for result in results {
            writeln!(file, "{}", serde_json::to_string(result)?)
                .with_context(|| format!("write temp results {}", tmp.display()))?;
        }
        file.flush()
            .with_context(|| format!("flush temp results {}", tmp.display()))?;
    }
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn write_summary_json_atomic(
    path: &std::path::Path,
    summary: &WalkForwardSummary,
) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create output directory {}", parent.display()))?;
    }
    let tmp = temp_sibling_path(path);
    std::fs::write(&tmp, serde_json::to_string_pretty(summary)?)
        .with_context(|| format!("write temp summary {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} to {}", tmp.display(), path.display()))?;
    Ok(())
}

fn temp_sibling_path(path: &std::path::Path) -> PathBuf {
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy())
        .unwrap_or_else(|| "checkpoint".into());
    path.with_file_name(format!("{file_name}.tmp"))
}

fn aggregate_for_strategy(records: &[&StrategyMarketResult]) -> StrategyAggregate {
    let mut pnls: Vec<f64> = Vec::with_capacity(records.len());
    let mut markets_with_orders = 0usize;
    let mut total_orders_submitted = 0usize;
    let mut total_orders_filled = 0usize;
    let mut total_orders_rejected_model_gate = 0usize;
    let mut total_orders_rejected_model_gate_confidence = 0usize;
    let mut total_orders_rejected_model_gate_risk = 0usize;
    let mut total_orders_rejected_model_gate_edge = 0usize;
    let mut tag_fills: HashMap<String, FillTagAccumulator> = HashMap::new();
    let mut bonereaper_v2_gate_stats: Option<BonereaperV2GateStats> = None;
    let first_start_equity = records
        .first()
        .map(|r| r.start_equity_usdc)
        .unwrap_or_default();
    let mut last_end_equity = records
        .last()
        .map(|r| r.end_equity_usdc)
        .unwrap_or_default();
    let mut min_end_equity = f64::INFINITY;
    let mut max_end_equity = f64::NEG_INFINITY;
    let mut peak_end_equity = first_start_equity.max(0.0);
    let mut path_max_drawdown = 0.0f64;

    for r in records {
        pnls.push(r.pnl_usdc);
        if r.orders_filled > 0 {
            markets_with_orders += 1;
        }
        total_orders_submitted += r.orders_submitted;
        total_orders_filled += r.orders_filled;
        total_orders_rejected_model_gate += r.orders_rejected_model_gate;
        total_orders_rejected_model_gate_confidence += r.orders_rejected_model_gate_confidence;
        total_orders_rejected_model_gate_risk += r.orders_rejected_model_gate_risk;
        total_orders_rejected_model_gate_edge += r.orders_rejected_model_gate_edge;
        for fill in &r.fills_detail {
            let pnl = fill_resolution_pnl(fill, r.yes_resolved);
            tag_fills
                .entry(fill.tag.clone())
                .or_default()
                .push(fill, pnl);
        }
        if let Some(stats) = r.bonereaper_v2_gate_stats {
            bonereaper_v2_gate_stats
                .get_or_insert_with(BonereaperV2GateStats::default)
                .add_assign(stats);
        }
        last_end_equity = r.end_equity_usdc;
        min_end_equity = min_end_equity.min(r.end_equity_usdc);
        max_end_equity = max_end_equity.max(r.end_equity_usdc);
        peak_end_equity = peak_end_equity.max(r.end_equity_usdc);
        if peak_end_equity > 0.0 {
            path_max_drawdown =
                path_max_drawdown.max((peak_end_equity - r.end_equity_usdc) / peak_end_equity);
        }
    }

    let total = pnls.iter().sum::<f64>();
    let n = pnls.len();
    let mean = if n > 0 { total / n as f64 } else { 0.0 };
    let stdev = if n > 1 {
        (pnls.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (n - 1) as f64).sqrt()
    } else {
        0.0
    };
    let mut sorted = pnls.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = if sorted.is_empty() {
        0.0
    } else if sorted.len() % 2 == 1 {
        sorted[sorted.len() / 2]
    } else {
        0.5 * (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2])
    };
    let hit_rate = if pnls.is_empty() {
        0.0
    } else {
        pnls.iter().filter(|p| **p > 0.0).count() as f64 / pnls.len() as f64
    };
    let best = sorted.last().copied().unwrap_or(0.0);
    let worst = sorted.first().copied().unwrap_or(0.0);
    if !min_end_equity.is_finite() {
        min_end_equity = 0.0;
    }
    if !max_end_equity.is_finite() {
        max_end_equity = 0.0;
    }
    let by_fill_tag = tag_fills
        .into_iter()
        .map(|(tag, fills)| (tag, fills.into_aggregate()))
        .collect();
    StrategyAggregate {
        total_pnl_usdc: total,
        first_start_equity_usdc: first_start_equity,
        last_end_equity_usdc: last_end_equity,
        min_end_equity_usdc: min_end_equity,
        max_end_equity_usdc: max_end_equity,
        compounded_return_pct: if first_start_equity > 0.0 {
            (last_end_equity / first_start_equity - 1.0) * 100.0
        } else {
            0.0
        },
        path_max_drawdown_pct: path_max_drawdown * 100.0,
        mean_pnl_usdc: mean,
        median_pnl_usdc: median,
        stdev_pnl_usdc: stdev,
        hit_rate,
        markets_with_orders,
        total_orders_submitted,
        total_orders_filled,
        total_orders_rejected_model_gate,
        total_orders_rejected_model_gate_confidence,
        total_orders_rejected_model_gate_risk,
        total_orders_rejected_model_gate_edge,
        worst_market_pnl: worst,
        best_market_pnl: best,
        sharpe_ratio: if stdev > 0.0 {
            mean / stdev * (records.len() as f64).sqrt()
        } else {
            0.0
        },
        by_fill_tag,
        bonereaper_v2_gate_stats,
    }
}

fn fill_resolution_pnl(fill: &crate::runner::Fill, yes_resolved: bool) -> f64 {
    let payout = match fill.side.as_str() {
        "BuyYes" => {
            if yes_resolved {
                fill.shares
            } else {
                0.0
            }
        }
        "BuyNo" => {
            if yes_resolved {
                0.0
            } else {
                fill.shares
            }
        }
        "SellYes" | "SellNo" => fill.notional,
        _ => 0.0,
    };
    payout - fill.notional + fill.rebate_usdc
}

fn aggregate(results: &[MarketResult], strategies: &[StratId]) -> WalkForwardSummary {
    let mut per_strategy = HashMap::new();
    for &strat in strategies {
        let name = strat.name();
        let records: Vec<_> = results
            .iter()
            .filter_map(|r| r.per_strategy.get(name))
            .collect();
        per_strategy.insert(name, aggregate_for_strategy(&records));
    }

    let mut by_volatility_band: HashMap<VolatilityBand, HashMap<&'static str, StrategyAggregate>> =
        HashMap::new();
    for band in [VolatilityBand::Low, VolatilityBand::High] {
        let mut band_per_strategy = HashMap::new();
        let band_results: Vec<_> = results
            .iter()
            .filter(|r| r.volatility_band == band)
            .collect();
        for &strat in strategies {
            let name = strat.name();
            let records: Vec<_> = band_results
                .iter()
                .filter_map(|r| r.per_strategy.get(name))
                .collect();
            band_per_strategy.insert(name, aggregate_for_strategy(&records));
        }
        by_volatility_band.insert(band, band_per_strategy);
    }

    WalkForwardSummary {
        markets_attempted: results.len(),
        markets_succeeded: results
            .iter()
            .filter(|r| !r.per_strategy.is_empty())
            .count(),
        run_config: None,
        per_strategy,
        by_volatility_band,
        fold_summaries: Vec::new(),
        meta_calibration: None,
    }
}

fn summary_run_config(cfg: &WalkForwardConfig) -> SummaryRunConfig {
    SummaryRunConfig {
        starting_cash_usdc: cfg.starting_cash_usdc,
        kelly_fraction: cfg.kelly_fraction,
        max_clip_usdc: cfg.max_clip_usdc,
        max_order_clip_multiplier: cfg.max_order_clip_multiplier,
        max_per_market_exposure_usdc: cfg.max_per_market_exposure_usdc,
        replay_sample_ms: cfg.replay_sample_ms,
        clip_fraction_of_equity: cfg.clip_fraction_of_equity,
        clip_drawdown_soft_pct: cfg.clip_drawdown_soft_pct,
        clip_drawdown_hard_pct: cfg.clip_drawdown_hard_pct,
        br2_disable_internal_model_gates: cfg.br2_disable_internal_model_gates,
        br2_min_composite_direction: cfg.br2_min_composite_direction,
        br2_early_clip_frac: cfg.br2_early_clip_frac,
        br2_mid_clip_frac: cfg.br2_mid_clip_frac,
        br2_late_clip_frac: cfg.br2_late_clip_frac,
        br2_late_max_fires: cfg.br2_late_max_fires,
        br2_late_confirm_min_model_confidence: cfg.br2_late_confirm_min_model_confidence,
        br2_late_confirm_max_model_risk: cfg.br2_late_confirm_max_model_risk,
        br2_late_confirm_min_model_side_p: cfg.br2_late_confirm_min_model_side_p,
        br2_late_confirm_min_model_edge: cfg.br2_late_confirm_min_model_edge,
        br2_late_confirm_min_book_skew: cfg.br2_late_confirm_min_book_skew,
        br2_late_confirm_max_whipsaw_score: cfg.br2_late_confirm_max_whipsaw_score,
        br2_high_skew_clip_frac: cfg.br2_high_skew_clip_frac,
        br2_high_skew_max_clips: cfg.br2_high_skew_max_clips,
        br2_high_skew_max_whipsaw_score: cfg.br2_high_skew_max_whipsaw_score,
        br2_late_favourite_start_secs: cfg.br2_late_favourite_start_secs,
        br2_late_favourite_threshold: cfg.br2_late_favourite_threshold,
        br2_late_favourite_min_ask: cfg.br2_late_favourite_min_ask,
        br2_late_favourite_max_ask: cfg.br2_late_favourite_max_ask,
        br2_late_favourite_clip_frac: cfg.br2_late_favourite_clip_frac,
        br2_late_favourite_high_cert_clip_frac: cfg.br2_late_favourite_high_cert_clip_frac,
        br2_late_favourite_max_clips: cfg.br2_late_favourite_max_clips,
        br2_late_favourite_min_sustain_secs: cfg.br2_late_favourite_min_sustain_secs,
        br2_late_favourite_sweep_depth: cfg.br2_late_favourite_sweep_depth,
        br2_late_favourite_min_model_confidence: cfg.br2_late_favourite_min_model_confidence,
        br2_late_favourite_min_model_direction_abs: cfg.br2_late_favourite_min_model_direction_abs,
        br2_late_favourite_max_model_risk: cfg.br2_late_favourite_max_model_risk,
        br2_late_favourite_min_model_side_p: cfg.br2_late_favourite_min_model_side_p,
        br2_late_favourite_min_model_edge: cfg.br2_late_favourite_min_model_edge,
        br2_late_favourite_high_cert_min_model_edge: cfg
            .br2_late_favourite_high_cert_min_model_edge,
        br2_late_favourite_max_whipsaw_score: cfg.br2_late_favourite_max_whipsaw_score,
        br2_late_favourite_max_reversal_pressure: cfg.br2_late_favourite_max_reversal_pressure,
        br2_late_favourite_min_path_efficiency: cfg.br2_late_favourite_min_path_efficiency,
        br2_late_favourite_max_observed_range: cfg.br2_late_favourite_max_observed_range,
        br2_late_favourite_max_adverse_fast_momentum: cfg
            .br2_late_favourite_max_adverse_fast_momentum,
        br2_late_favourite_max_entry_pullback: cfg.br2_late_favourite_max_entry_pullback,
        br2_late_favourite_max_avg_entry_drawdown: cfg.br2_late_favourite_max_avg_entry_drawdown,
        br2_tail_clip_frac: cfg.br2_tail_clip_frac,
        br2_tail_max_clips: cfg.br2_tail_max_clips,
        br2_tail_min_ask: cfg.br2_tail_min_ask,
        br2_tail_max_ask: cfg.br2_tail_max_ask,
        br2_tail_extreme_threshold: cfg.br2_tail_extreme_threshold,
        br2_tail_min_skew_step: cfg.br2_tail_min_skew_step,
        br2_tail_budget_favourite_spend_frac: cfg.br2_tail_budget_favourite_spend_frac,
        br2_tail_budget_favourite_upside_frac: cfg.br2_tail_budget_favourite_upside_frac,
        enforce_model_gate: cfg.enforce_model_gate,
        model_gate_min_confidence: cfg.model_gate_min_confidence,
        model_gate_max_risk: cfg.model_gate_max_risk,
        model_gate_min_edge: cfg.model_gate_min_edge,
        min_train_markets: cfg.min_train_markets,
        meta_epochs: cfg.meta_training_config.epochs,
        meta_learning_rate: cfg.meta_training_config.learning_rate,
        meta_l2: cfg.meta_training_config.l2,
        meta_weight_clip: cfg.meta_training_config.weight_clip,
        meta_max_fit_samples: cfg.meta_max_fit_samples,
        meta_max_validation_samples: cfg.meta_max_validation_samples,
        meta_max_samples_per_market: cfg.meta_max_samples_per_market,
        meta_max_oos_evaluation_samples: cfg.meta_max_oos_evaluation_samples,
        meta_train_min_base_p: cfg.meta_train_min_base_p,
        meta_train_max_early_penalty: cfg.meta_train_max_early_penalty,
        meta_train_min_mid_distance: cfg.meta_train_min_mid_distance,
        enable_meta_calibration: cfg.enable_meta_calibration,
    }
}

pub fn print_summary(summary: &WalkForwardSummary) {
    fn print_table(title: &str, per_strategy: &HashMap<&'static str, StrategyAggregate>) {
        println!("{title}");
        println!(
            "{:>22}  {:>10}  {:>10}  {:>9}  {:>8}  {:>10}  {:>10}  {:>10}  {:>8}  {:>9}  {:>14}  {:>10}",
            "strategy",
            "total_pnl",
            "end_eq",
            "return",
            "max_dd",
            "mean_pnl",
            "median",
            "stdev",
            "hit",
            "sharpe",
            "fills",
            "worst",
        );
        let mut rows: Vec<(&&str, &StrategyAggregate)> = per_strategy.iter().collect();
        rows.sort_by(|a, b| {
            b.1.total_pnl_usdc
                .partial_cmp(&a.1.total_pnl_usdc)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (name, agg) in rows {
            println!(
                "{:>22}  {:>+10.4}  {:>10.2}  {:>+8.1}%  {:>7.1}%  {:>+10.4}  {:>+10.4}  {:>10.4}  {:>7.1}%  {:>+10.4}  {:>14}  {:>+10.4}",
                name,
                agg.total_pnl_usdc,
                agg.last_end_equity_usdc,
                agg.compounded_return_pct,
                agg.path_max_drawdown_pct,
                agg.mean_pnl_usdc,
                agg.median_pnl_usdc,
                agg.stdev_pnl_usdc,
                agg.hit_rate * 100.0,
                agg.sharpe_ratio,
                agg.total_orders_filled,
                agg.worst_market_pnl,
            );
        }
        println!();
    }

    println!("== walk-forward summary ==");
    println!(
        "markets: attempted={}  succeeded={}",
        summary.markets_attempted, summary.markets_succeeded
    );
    println!();

    print_table("overall", &summary.per_strategy);

    println!("volatility bands:");
    print_table(
        VolatilityBand::Low.as_str(),
        summary
            .by_volatility_band
            .get(&VolatilityBand::Low)
            .unwrap_or(&HashMap::new()),
    );
    print_table(
        VolatilityBand::High.as_str(),
        summary
            .by_volatility_band
            .get(&VolatilityBand::High)
            .unwrap_or(&HashMap::new()),
    );

    if !summary.fold_summaries.is_empty() {
        println!("folds:");
        for fold in &summary.fold_summaries {
            println!(
                "  [{}] train_end={} purge={} test=[{}, {})",
                fold.fold_idx,
                fold.train_end_exclusive,
                fold.purge_markets,
                fold.test_start,
                fold.test_end
            );
            if fold.test_start >= fold.test_end {
                println!("    (empty)");
                continue;
            }
            print_table(
                &format!(
                    "fold {} metrics ({}..{})",
                    fold.fold_idx, fold.test_start, fold.test_end
                ),
                &fold.fold_results.per_strategy,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn aggregate_handles_per_strategy_metrics() {
        let mut result_reactive = HashMap::new();
        result_reactive.insert(
            StratId::ReactiveDirectional.name(),
            StrategyMarketResult {
                orders_submitted: 10,
                orders_filled: 6,
                orders_rejected_model_gate: 0,
                orders_rejected_model_gate_confidence: 0,
                orders_rejected_model_gate_risk: 0,
                orders_rejected_model_gate_edge: 0,
                pnl_usdc: 4.0,
                start_equity_usdc: 100.0,
                end_equity_usdc: 104.0,
                max_drawdown_pct: 0.12,
                fills: 6,
                maker_rebates_usdc: 0.1,
                clip_used_usdc: 2.0,
                yes_resolved: true,
                fills_detail: vec![],
                bonereaper_v2_gate_stats: None,
                model_training_samples: vec![],
            },
        );

        let mut result_bonereaper = HashMap::new();
        result_bonereaper.insert(
            StratId::BonereaperLite.name(),
            StrategyMarketResult {
                orders_submitted: 3,
                orders_filled: 1,
                orders_rejected_model_gate: 0,
                orders_rejected_model_gate_confidence: 0,
                orders_rejected_model_gate_risk: 0,
                orders_rejected_model_gate_edge: 0,
                pnl_usdc: -2.0,
                start_equity_usdc: 100.0,
                end_equity_usdc: 98.0,
                max_drawdown_pct: 0.05,
                fills: 1,
                maker_rebates_usdc: 0.0,
                clip_used_usdc: 1.5,
                yes_resolved: false,
                fills_detail: vec![],
                bonereaper_v2_gate_stats: None,
                model_training_samples: vec![],
            },
        );

        let mut result_no_order = HashMap::new();
        result_no_order.insert(
            StratId::PairedMm.name(),
            StrategyMarketResult {
                orders_submitted: 0,
                orders_filled: 0,
                orders_rejected_model_gate: 0,
                orders_rejected_model_gate_confidence: 0,
                orders_rejected_model_gate_risk: 0,
                orders_rejected_model_gate_edge: 0,
                pnl_usdc: 0.0,
                start_equity_usdc: 100.0,
                end_equity_usdc: 100.0,
                max_drawdown_pct: 0.0,
                fills: 0,
                maker_rebates_usdc: 0.0,
                clip_used_usdc: 3.0,
                yes_resolved: true,
                fills_detail: vec![],
                bonereaper_v2_gate_stats: None,
                model_training_samples: vec![],
            },
        );

        let results = vec![
            MarketResult {
                asset_id: "1".to_string(),
                slug: "a".to_string(),
                close_ts: 0,
                outcome_label: "Yes".to_string(),
                volatility_range: 0.02,
                volatility_band: VolatilityBand::Low,
                per_strategy: result_reactive,
            },
            MarketResult {
                asset_id: "2".to_string(),
                slug: "b".to_string(),
                close_ts: 0,
                outcome_label: "No".to_string(),
                volatility_range: 0.20,
                volatility_band: VolatilityBand::High,
                per_strategy: result_bonereaper,
            },
            MarketResult {
                asset_id: "3".to_string(),
                slug: "c".to_string(),
                close_ts: 0,
                outcome_label: "Yes".to_string(),
                volatility_range: 0.10,
                volatility_band: VolatilityBand::Low,
                per_strategy: result_no_order,
            },
        ];

        let summary = aggregate(
            &results,
            &[
                StratId::ReactiveDirectional,
                StratId::BonereaperLite,
                StratId::PairedMm,
            ],
        );
        assert_eq!(summary.markets_attempted, 3);
        assert_eq!(summary.markets_succeeded, 3);

        let reactive = summary
            .per_strategy
            .get(StratId::ReactiveDirectional.name())
            .expect("reactive missing");
        assert_eq!(reactive.markets_with_orders, 1);
        assert_eq!(reactive.total_orders_filled, 6);
        assert_eq!(reactive.best_market_pnl, 4.0);
        assert_eq!(reactive.worst_market_pnl, 4.0);
        assert!((reactive.hit_rate - 1.0).abs() < f64::EPSILON);

        let paired = summary
            .per_strategy
            .get(StratId::PairedMm.name())
            .expect("paired missing");
        assert_eq!(paired.markets_with_orders, 0);
        assert_eq!(paired.total_orders_filled, 0);
        assert_eq!(paired.total_pnl_usdc, 0.0);

        let low = summary
            .by_volatility_band
            .get(&VolatilityBand::Low)
            .expect("low band missing");
        let low_reactive = low
            .get(StratId::ReactiveDirectional.name())
            .expect("low reactive missing");
        assert_eq!(low_reactive.total_pnl_usdc, 4.0);

        let high = summary
            .by_volatility_band
            .get(&VolatilityBand::High)
            .expect("high band missing");
        let high_bonereaper = high
            .get(StratId::BonereaperLite.name())
            .expect("high bonereaper missing");
        assert_eq!(high_bonereaper.total_pnl_usdc, -2.0);
        assert_eq!(reactive.sharpe_ratio, 0.0);
    }

    #[test]
    fn fold_plan_with_fold_size() {
        let cfg = WalkForwardConfig {
            fold_size: Some(3),
            ..WalkForwardConfig::default()
        };
        let plan = build_fold_plan(10, &cfg).expect("plan");
        assert_eq!(plan, vec![(0, 0, 3), (3, 3, 6), (6, 6, 9), (9, 9, 10)]);
    }

    #[test]
    fn fold_plan_skips_until_min_train_markets() {
        let cfg = WalkForwardConfig {
            fold_size: Some(3),
            min_train_markets: 6,
            ..WalkForwardConfig::default()
        };
        let plan = build_fold_plan(10, &cfg).expect("plan");
        assert_eq!(plan, vec![(6, 6, 9), (9, 9, 10)]);
    }

    #[test]
    fn fold_plan_errors_when_min_train_markets_impossible() {
        let cfg = WalkForwardConfig {
            fold_size: Some(3),
            min_train_markets: 12,
            ..WalkForwardConfig::default()
        };
        let err = build_fold_plan(10, &cfg).expect_err("should reject impossible min train");
        assert!(
            err.to_string().contains("no walk-forward folds satisfy"),
            "{err}"
        );
    }

    #[test]
    fn fold_plan_with_folds_and_purge() {
        let cfg = WalkForwardConfig {
            walk_forward_folds: Some(2),
            purge_markets: 2,
            ..WalkForwardConfig::default()
        };
        let plan = build_fold_plan(10, &cfg).expect("plan");
        assert_eq!(plan, vec![(0, 0, 5), (3, 5, 10)]);
    }

    #[test]
    fn no_fold_config_uses_single_window() {
        let cfg = WalkForwardConfig::default();
        let plan = build_fold_plan(7, &cfg).expect("plan");
        assert_eq!(plan, vec![(0, 0, 7)]);
    }

    #[test]
    fn compounded_clip_does_not_panic_on_tiny_bankroll() {
        assert_eq!(compounded_clip(0.16, 0.02), 0.0032);
        assert_eq!(compounded_clip(0.0, 0.02), 0.0);
        assert_eq!(compounded_clip(1000.0, 0.02), 20.0);
    }

    #[test]
    fn sample_replay_events_keeps_latest_tick_per_interval() {
        fn event(ts_ms: i64, mid: f32) -> pm_types::ReplayEvent {
            pm_types::ReplayEvent {
                ts_ns: ts_ms * 1_000_000,
                market_id: pm_types::MarketId(1),
                yes_mid: mid,
                yes_bid: mid - 0.01,
                yes_ask: mid + 0.01,
                volume: 0.0,
                bids: [pm_types::BookLevel::default(); pm_types::TAPE_DEPTH],
                asks: [pm_types::BookLevel::default(); pm_types::TAPE_DEPTH],
                spot_price: 100.0,
                flags: pm_types::ReplayFlags::BOOK_UPDATE,
            }
        }

        let events = vec![
            event(0, 0.50),
            event(100, 0.51),
            event(900, 0.52),
            event(1100, 0.53),
            event(1900, 0.54),
            event(2100, 0.55),
            event(3000, 0.56),
        ];

        let sampled = sample_replay_events(&events, 1000);
        let mids: Vec<_> = sampled.iter().map(|event| event.yes_mid).collect();
        assert_eq!(mids, vec![0.50, 0.52, 0.54, 0.55, 0.56]);
    }

    #[test]
    fn market_balanced_meta_samples_caps_each_market_and_total() {
        let mut samples = Vec::new();
        for market_idx in 0..5 {
            for sample_idx in 0..10 {
                let mut features = pm_model::MetaFeatures::default();
                features.values[0] = sample_idx as f32;
                samples.push(MetaTrainingSample {
                    features,
                    market_idx,
                    base_side_probability: 0.5,
                    side_observed: market_idx % 2 == 0,
                });
            }
        }

        let selected = market_balanced_meta_samples(&samples, 12, 4);
        assert_eq!(selected.len(), 10);
        for market_idx in 0..5 {
            let count = selected
                .iter()
                .filter(|sample| sample.market_idx == market_idx)
                .count();
            assert_eq!(count, 2);
        }
    }

    #[test]
    fn split_meta_samples_by_market_uses_chronological_markets() {
        let samples: Vec<MetaTrainingSample> = (0..4)
            .flat_map(|market_idx| {
                (0..3).map(move |_| MetaTrainingSample {
                    features: pm_model::MetaFeatures::default(),
                    market_idx,
                    base_side_probability: 0.5,
                    side_observed: market_idx % 2 == 0,
                })
            })
            .collect();

        let (fit, validation) = split_meta_samples_by_market(&samples, 0.5);
        assert_eq!(fit.len(), 6);
        assert_eq!(validation.len(), 6);
        assert!(fit.iter().all(|sample| sample.market_idx <= 1));
        assert!(validation.iter().all(|sample| sample.market_idx >= 2));
    }

    #[test]
    fn previous_date_handles_month_boundary() {
        assert_eq!(
            previous_date("2026-03-01").unwrap(),
            Some("2026-02-28".to_string())
        );
    }
}
