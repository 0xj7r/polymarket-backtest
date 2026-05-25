//! In-process backtest runner with maker + taker matcher.
//!
//! - Orders with `limit_price = None` are TAKER fills against the opposite
//!   top of book (immediate, pay the spread).
//! - Orders with `limit_price = Some(L)` enter the resting book. On each
//!   subsequent event, the runner checks whether the book crossed the limit
//!   (BuyYes fills when `event.yes_ask <= L_yes`; SellYes when
//!   `event.yes_bid >= L_yes`; mirror for NO). Filled makers earn the
//!   configurable maker rebate (`maker_rebate_bps`).
//!
//! Resolution: `cfg.resolved_yes` overrides; otherwise inferred from
//! `last_yes_mid >= 0.5`.

use anyhow::Result;
use arrow::array::{
    ArrayRef, BooleanArray, Float32Array, Float64Array, Int64Array, StringArray, UInt32Array,
    UInt64Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, Utc};
use parquet::arrow::ArrowWriter;
use pm_model::{
    MetaFeatures, MetaTrainingSample, ModelConfig, ModelOutput, ModelState,
    OnlineMetaCalibratorSnapshot, edge_vs_mid,
};
use pm_risk::{PortfolioLimits, PortfolioSnapshot, PortfolioState};
use pm_strategy::regime::WhipsawRiskSnapshot;
use pm_strategy::{Ctx, OrderRequest, Side, Strategy};
use pm_types::{ReplayEvent, SpotHistory, TradeHistory};
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize)]
pub struct Fill {
    pub ts_ns: i64,
    pub side: String,
    pub shares: f64,
    pub price: f32,
    pub notional: f64,
    pub tag: String,
    pub maker: bool,
    pub rebate_usdc: f64,
    pub slippage_bps: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_mid: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_bid: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yes_ask: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side_model_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side_edge_vs_mid: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub side_edge_vs_fill: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub calibrated_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_since_open: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seconds_to_close: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regime_whipsaw_score: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regime_path_efficiency: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regime_reversal_pressure: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regime_sign_flip_rate: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub regime_realized_vol_180s_bps: Option<f32>,
}

#[derive(Debug, Clone, Copy)]
struct FillModelContext {
    yes_mid: f32,
    yes_bid: f32,
    yes_ask: f32,
    side_model_p: f32,
    side_edge_vs_mid: f32,
    direction_score: f32,
    confidence_score: f32,
    calibrated_p: f32,
    risk_score: f32,
    seconds_since_open: f32,
    seconds_to_close: f32,
    regime_whipsaw_score: f32,
    regime_path_efficiency: f32,
    regime_reversal_pressure: f32,
    regime_sign_flip_rate: f32,
    regime_realized_vol_180s_bps: f32,
}

impl FillModelContext {
    fn from_event(
        event: &ReplayEvent,
        model_output: &ModelOutput,
        side: Side,
        seconds_since_open: f32,
        market_close_ns: i64,
        whipsaw: WhipsawRiskSnapshot,
    ) -> Self {
        let yes_side = order_adds_yes_exposure(side);
        let side_market_mid = if yes_side {
            event.yes_mid
        } else {
            1.0 - event.yes_mid
        };
        let predicted_yes = model_output.direction_score >= 0.0;
        let side_model_p = if yes_side == predicted_yes {
            model_output.calibrated_p
        } else {
            1.0 - model_output.calibrated_p
        };
        Self {
            yes_mid: event.yes_mid,
            yes_bid: event.yes_bid,
            yes_ask: event.yes_ask,
            side_model_p,
            side_edge_vs_mid: side_model_p - side_market_mid,
            direction_score: model_output.direction_score,
            confidence_score: model_output.confidence_score,
            calibrated_p: model_output.calibrated_p,
            risk_score: model_output.risk_score,
            seconds_since_open,
            seconds_to_close: ((market_close_ns - event.ts_ns).max(0) as f32) / 1e9,
            regime_whipsaw_score: whipsaw.score,
            regime_path_efficiency: whipsaw.path_efficiency,
            regime_reversal_pressure: whipsaw.reversal_pressure,
            regime_sign_flip_rate: whipsaw.sign_flip_rate,
            regime_realized_vol_180s_bps: whipsaw.realized_vol_180s_bps,
        }
    }
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct StrategyCounters {
    pub orders_submitted: usize,
    pub orders_filled_taker: usize,
    pub orders_filled_maker: usize,
    pub orders_rejected_no_cash: usize,
    pub orders_rejected_no_liquidity: usize,
    pub orders_rejected_bad_price: usize,
    pub orders_rejected_no_inventory: usize,
    pub orders_rejected_risk_gate: usize,
    pub orders_rejected_model_gate: usize,
    pub orders_rejected_model_gate_confidence: usize,
    pub orders_rejected_model_gate_risk: usize,
    pub orders_rejected_model_gate_edge: usize,
    pub resting_orders_active: usize,
    pub resting_orders_cancelled_eom: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BacktestReport {
    pub events_processed: usize,
    pub counters: StrategyCounters,
    pub start_equity_usdc: f64,
    pub end_equity_usdc: f64,
    pub pnl_usdc: f64,
    pub maker_rebates_usdc: f64,
    pub peak_equity_usdc: f64,
    pub max_drawdown_pct: f64,
    pub final_yes_shares: f64,
    pub final_no_shares: f64,
    pub final_cash_usdc: f64,
    pub yes_resolved: bool,
    pub last_yes_mid: f32,
    pub fills: Vec<Fill>,
    pub final_portfolio: PortfolioSnapshot,
    pub model_training_samples: Vec<MetaTrainingSample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionLogRow {
    pub market_id: u32,
    pub event_idx: u64,
    pub ts_ns: i64,
    pub market_mid: f32,
    pub yes_mid: f32,
    pub yes_bid: f32,
    pub yes_ask: f32,
    pub cash_usdc_before: f64,
    pub yes_shares_before: f64,
    pub no_shares_before: f64,
    pub direction_score: f32,
    pub confidence_score: f32,
    pub calibrated_p: f32,
    pub risk_score: f32,
    pub edge: f32,
    pub has_model_output: bool,
    pub strategy_emitted_model_output: bool,
    pub has_model_attribution: bool,
    pub side_is_yes: bool,
    pub feature_momentum: f32,
    pub feature_book_imbalance_top3: f32,
    pub feature_microprice_dev: f32,
    pub feature_microprice_spot_alignment: f32,
    pub feature_top3_delta_5s: f32,
    pub feature_top3_delta_15s: f32,
    pub feature_spot_score: f32,
    pub feature_spot_fast_momentum: f32,
    pub feature_spot_broad_momentum: f32,
    pub feature_spot_momentum_600s: f32,
    pub feature_spot_momentum_900s: f32,
    pub feature_spot_momentum_1800s: f32,
    pub feature_spot_momentum_3600s: f32,
    pub feature_spot_fast_long_alignment: f32,
    pub feature_spot_broad_trend_consistency: f32,
    pub feature_spot_broad_acceleration: f32,
    pub feature_direction_raw: f32,
    pub feature_stability: f32,
    pub feature_sign_persistence: f32,
    pub feature_markov_persistence: f32,
    pub feature_early_market_penalty: f32,
    pub feature_time_of_day_edge: f32,
    pub feature_time_of_day_advantage: f32,
    pub feature_whipsaw: f32,
    pub feature_liquidity: f32,
    pub feature_path_risk: f32,
    pub feature_imbalance_turn: f32,
    pub feature_markov_reversal_risk: f32,
    pub feature_skew_penalty: f32,
    pub feature_volatility_penalty: f32,
    pub feature_time_of_day_penalty: f32,
    pub feature_volatility_regime: f32,
    pub feature_dir_flip_rate_8: f32,
    pub feature_dir_std_8: f32,
    pub feature_dir_abs_mean_8: f32,
    pub feature_side_p_pre_meta: f32,
    pub feature_side_p_post_meta: f32,
    pub meta_calibrator_updates: u32,
    pub orders_requested: usize,
    pub requested_shares: f64,
    pub requested_notional_usdc: f64,
    pub order_tags: Vec<String>,
    pub event_fill_notional_usdc: f64,
    pub event_fills: usize,
    pub event_slippage_bps: f32,
    pub event_cash_delta_usdc: f64,
    pub event_mtm_delta_usdc: f64,
}

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub starting_cash_usdc: f64,
    /// Market open timestamp in nanoseconds. When set, replay ignores stale
    /// book snapshots before this point and timing gates are measured from it.
    pub market_open_ns: i64,
    pub market_close_ns: i64,
    pub resolved_yes: Option<bool>,
    pub portfolio_limits: PortfolioLimits,
    pub equity_curve_jsonl: Option<PathBuf>,
    pub snapshot_every_n: usize,
    /// Maker rebate (in basis points of notional). Polymarket has run
    /// programs in the 5–20 bps range — default 0 keeps it neutral.
    pub maker_rebate_bps: f64,
    /// Taker fee (bps). Default 0; configure per market regime.
    pub taker_fee_bps: f64,
    /// If |yes_shares - no_shares| exceeds this AFTER a maker fill, cancel
    /// resting orders on the heavy side. Critical safety for paired-MM
    /// strategies: without this, a one-sided book trend can run inventory
    /// far beyond the strategy's emission caps. `f64::INFINITY` disables.
    pub max_inventory_imbalance_shares: f64,
    /// Slippage on taker fills (basis points). Worsens the fill price (buyer
    /// pays more, seller gets less). Approximates queue / latency cost
    /// between strategy decision and venue execution. Default 0.
    pub taker_slippage_bps: f64,
    /// Optional per-decision log (JSONL). Useful for attribution and
    /// post-hoc analysis of every strategy callback.
    pub decision_log_jsonl: Option<PathBuf>,
    /// Optional per-decision attribution log (Parquet).
    pub decision_log_parquet: Option<PathBuf>,
    /// Optional shared canonical model state for walk-forward or portfolio
    /// calibration continuity across markets.
    pub shared_model_state: Option<Arc<Mutex<ModelState>>>,
    /// Allow `record_market_result` to update shared/local model state after
    /// resolution. Disable for frozen snapshot evaluation.
    pub update_model_state_on_resolution: bool,
    /// Frozen meta-calibrator snapshot loaded into a local canonical model
    /// state. Intended for walk-forward test folds.
    pub meta_calibrator_snapshot: Option<OnlineMetaCalibratorSnapshot>,
    /// Enable the online meta-calibrator adjustment in the canonical model.
    /// Disable this for strategy-only A/B runs against the hand-crafted score.
    pub enable_meta_calibration: bool,
    /// Log every Nth decision event to avoid enormous files.
    pub decision_log_every_n: usize,
    /// Gate per-tick orders by model-derived entry constraints.
    pub enforce_model_gate: bool,
    /// Minimum `ModelOutput::confidence_score` required for an order.
    pub model_gate_min_confidence: f32,
    /// Maximum `ModelOutput::risk_score` allowed for an order.
    pub model_gate_max_risk: f32,
    /// Minimum edge over implied side probability required for an order.
    pub model_gate_min_edge: f32,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            starting_cash_usdc: 100.0,
            market_open_ns: 0,
            market_close_ns: 0,
            resolved_yes: None,
            portfolio_limits: PortfolioLimits::default(),
            equity_curve_jsonl: None,
            snapshot_every_n: 200,
            maker_rebate_bps: 0.0,
            taker_fee_bps: 0.0,
            max_inventory_imbalance_shares: f64::INFINITY,
            taker_slippage_bps: 0.0,
            decision_log_jsonl: None,
            decision_log_parquet: None,
            shared_model_state: None,
            update_model_state_on_resolution: true,
            meta_calibrator_snapshot: None,
            enable_meta_calibration: true,
            decision_log_every_n: 1,
            enforce_model_gate: false,
            model_gate_min_confidence: 0.68,
            model_gate_max_risk: 0.72,
            model_gate_min_edge: 0.05,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RestingOrder {
    side: Side,
    /// Always stored in YES-terms. For NO orders the strategy submits a
    /// NO-side limit; we convert: `limit_yes = 1 - limit_no`.
    limit_yes: f32,
    shares: f64,
    submit_ts_ns: i64,
    tag: &'static str,
    model_context: Option<FillModelContext>,
}

pub fn run_backtest<S: Strategy>(
    events: &[ReplayEvent],
    spot: &SpotHistory,
    trades: &TradeHistory,
    strategy: &mut S,
    cfg: &RunnerConfig,
) -> Result<BacktestReport> {
    let mut cash = cfg.starting_cash_usdc;
    let mut yes_shares = 0.0f64;
    let mut no_shares = 0.0f64;
    let mut counters = StrategyCounters::default();
    let mut fills: Vec<Fill> = Vec::new();
    let mut last_mid = 0.0f32;
    let mut total_rebates = 0.0f64;
    let mut resting: Vec<RestingOrder> = Vec::new();
    let mut trade_cursor = 0usize;
    let mut events_processed = 0usize;
    let mut market_yes_min = f32::INFINITY;
    let mut market_yes_max = f32::NEG_INFINITY;
    let mut model_state = ModelState::new();
    if let Some(snapshot) = cfg.meta_calibrator_snapshot.clone() {
        model_state.load_meta_calibrator_snapshot(snapshot);
    }
    let model_cfg = ModelConfig {
        enable_meta_calibration: cfg.enable_meta_calibration,
        ..ModelConfig::default()
    };
    let market_open_ts_ns = if cfg.market_open_ns > 0 {
        cfg.market_open_ns
    } else if cfg.market_close_ns > 300_000_000_000 {
        cfg.market_close_ns - 300_000_000_000
    } else {
        events.first().map_or(0, |e| e.ts_ns).max(0)
    };

    let mut portfolio = PortfolioState::new(cfg.starting_cash_usdc, cfg.portfolio_limits.clone());
    portfolio.mark(cfg.starting_cash_usdc);

    let mut curve_file = match cfg.equity_curve_jsonl.as_deref() {
        Some(p) => Some(std::fs::File::create(p)?),
        None => None,
    };
    let mut decision_file = match cfg.decision_log_jsonl.as_deref() {
        Some(p) => Some(std::fs::File::create(p)?),
        None => None,
    };
    let mut decision_rows = if cfg.decision_log_parquet.is_some() {
        Some(Vec::new())
    } else {
        None
    };
    let snap_every = cfg.snapshot_every_n.max(1);
    let decision_every = cfg.decision_log_every_n.max(1);

    let mut last_window_idx: isize = -1;
    let mut last_canonical_prediction_is_yes: Option<bool> = None;
    let mut last_canonical_sample_point: Option<(MetaFeatures, f32, bool)> = None;
    let mut last_meta_sample_bucket: Option<i64> = None;
    let mut canonical_meta_sample_points: Vec<(MetaFeatures, f32, bool)> = Vec::new();
    for (idx, event) in events.iter().enumerate() {
        if market_open_ts_ns > 0 && event.ts_ns < market_open_ts_ns {
            continue;
        }
        if cfg.market_close_ns > 0 && event.ts_ns > cfg.market_close_ns {
            break;
        }
        last_mid = event.yes_mid;
        market_yes_min = market_yes_min.min(event.yes_mid);
        market_yes_max = market_yes_max.max(event.yes_mid);
        let market_yes_range_so_far = if market_yes_min.is_finite() && market_yes_max.is_finite() {
            market_yes_max - market_yes_min
        } else {
            0.0
        };
        last_window_idx = idx as isize;
        events_processed += 1;

        check_trade_driven_resting_fills(
            event,
            trades,
            &mut trade_cursor,
            &mut resting,
            &mut cash,
            &mut yes_shares,
            &mut no_shares,
            &mut portfolio,
            &mut counters,
            &mut fills,
            &mut total_rebates,
            cfg.maker_rebate_bps,
        );

        check_resting_fills(
            event,
            &mut resting,
            &mut cash,
            &mut yes_shares,
            &mut no_shares,
            &mut portfolio,
            &mut counters,
            &mut fills,
            &mut total_rebates,
            cfg.maker_rebate_bps,
        );

        // Inventory imbalance circuit-breaker: cancel resting orders on the
        // heavy side once we go too long one outcome.
        let imbalance = yes_shares - no_shares;
        if imbalance.abs() > cfg.max_inventory_imbalance_shares {
            let heavy_long_yes = imbalance > 0.0;
            resting.retain(|r| {
                let adds_to_heavy = match (heavy_long_yes, r.side) {
                    (true, Side::BuyYes) => true,
                    (false, Side::BuyNo) => true,
                    _ => false,
                };
                !adds_to_heavy
            });
        }

        let pre_cash = cash;
        let pre_yes_shares = yes_shares;
        let pre_no_shares = no_shares;
        let pre_fill_count = fills.len();
        let pre_mtm = mark_to_market(cash, yes_shares, no_shares, last_mid);
        let secs_since_open = ((event.ts_ns - market_open_ts_ns).max(0) as f64) / 1e9;
        let canonical_model_eval = if let Some(shared) = &cfg.shared_model_state {
            let mut state = shared.lock().expect("shared model mutex poisoned");
            state.evaluate_detailed(event, spot, secs_since_open as f32, &model_cfg)
        } else {
            model_state.evaluate_detailed(event, spot, secs_since_open as f32, &model_cfg)
        };
        let canonical_prediction_is_yes = canonical_model_eval.output.direction_score >= 0.0;
        last_canonical_prediction_is_yes = Some(canonical_prediction_is_yes);
        let canonical_sample_point = (
            canonical_model_eval.attribution.meta_features,
            canonical_model_eval.attribution.side_probability_pre_meta,
            canonical_prediction_is_yes,
        );
        let meta_sample_bucket = (secs_since_open / 15.0).floor() as i64;
        if last_meta_sample_bucket != Some(meta_sample_bucket) {
            canonical_meta_sample_points.push(canonical_sample_point);
            last_meta_sample_bucket = Some(meta_sample_bucket);
        }
        last_canonical_sample_point = Some(canonical_sample_point);
        let ctx = Ctx {
            events_seen: events_processed as u64,
            yes_shares,
            no_shares,
            cash_usdc: cash,
            market_yes_range_so_far,
            model_output: Some(canonical_model_eval.output),
            market_close_ns: cfg.market_close_ns,
        };
        let (output, strategy_model_output) = strategy.on_event_scored(event, &ctx, spot, trades);
        let strategy_emitted_model_output = strategy_model_output.is_some();
        let model_output = strategy_model_output.unwrap_or(canonical_model_eval.output);
        let model_attribution = canonical_model_eval.attribution;
        let whipsaw_snapshot = if spot.is_empty() {
            WhipsawRiskSnapshot::default()
        } else {
            WhipsawRiskSnapshot::from_history(event.ts_ns, spot)
        };
        let has_model_attribution = true;
        let edge = edge_vs_mid(&model_output, event.yes_mid);
        let direction_score = model_output.direction_score;
        let confidence_score = model_output.confidence_score;
        let calibrated_p = model_output.calibrated_p;
        let risk_score = model_output.risk_score;
        let has_model_output = true;
        let orders_requested = output.orders.len();
        let mut order_tags = Vec::with_capacity(orders_requested);
        for req in &output.orders {
            order_tags.push(req.tag.to_string());
        }
        let mut requested_shares = 0.0;
        let mut requested_notional = 0.0;

        for req in output.orders {
            let yes_side = order_adds_yes_exposure(req.side);
            if cfg.enforce_model_gate {
                let side_edge = pm_model::side_edge_vs_mid(&model_output, event.yes_mid, yes_side)
                    .clamp(0.0, 1.0);
                if model_output.confidence_score < cfg.model_gate_min_confidence {
                    counters.orders_rejected_model_gate += 1;
                    counters.orders_rejected_model_gate_confidence += 1;
                    continue;
                }
                if model_output.risk_score > cfg.model_gate_max_risk {
                    counters.orders_rejected_model_gate += 1;
                    counters.orders_rejected_model_gate_risk += 1;
                    continue;
                }
                if side_edge < cfg.model_gate_min_edge {
                    counters.orders_rejected_model_gate += 1;
                    counters.orders_rejected_model_gate_edge += 1;
                    continue;
                }
            }
            counters.orders_submitted += 1;
            requested_shares += req.shares;
            requested_notional += order_request_notional_usdc(req, event).unwrap_or(0.0);
            let fill_context = Some(FillModelContext::from_event(
                event,
                &model_output,
                req.side,
                secs_since_open as f32,
                cfg.market_close_ns,
                whipsaw_snapshot,
            ));
            match req.limit_price {
                None => {
                    apply_taker_order(
                        event,
                        &req,
                        &mut cash,
                        &mut yes_shares,
                        &mut no_shares,
                        &mut portfolio,
                        &mut counters,
                        &mut fills,
                        cfg.taker_fee_bps,
                        cfg.taker_slippage_bps,
                        fill_context,
                    );
                }
                Some(limit) => {
                    submit_maker_order(
                        event,
                        &req,
                        limit,
                        &mut resting,
                        &mut cash,
                        &mut yes_shares,
                        &mut no_shares,
                        &mut portfolio,
                        &mut counters,
                        &mut fills,
                        &mut total_rebates,
                        cfg.maker_rebate_bps,
                        cfg.taker_fee_bps,
                        cfg.taker_slippage_bps,
                        fill_context,
                    );
                }
            }
        }

        counters.resting_orders_active = resting.len();
        let mtm = mark_to_market(cash, yes_shares, no_shares, last_mid);
        portfolio.mark(mtm);

        if let Some(f) = curve_file.as_mut() {
            if idx % snap_every == 0 {
                let snap = portfolio.snapshot(event.ts_ns, mtm);
                writeln!(f, "{}", serde_json::to_string(&snap)?)?;
            }
        }

        let event_fill_notional = fills
            .iter()
            .skip(pre_fill_count)
            .map(|f| f.notional)
            .sum::<f64>();
        let event_fill_count = fills.len() - pre_fill_count;
        let event_fills_window = &fills[pre_fill_count..];
        let (window_notional, slippage_notional) =
            event_fills_window
                .iter()
                .fold((0.0f64, 0.0f32), |(n, slip), fill| {
                    (
                        n + fill.notional,
                        slip + fill.slippage_bps * fill.notional as f32,
                    )
                });
        let event_slippage_bps = if window_notional > 0.0 {
            slippage_notional / window_notional as f32
        } else {
            0.0
        };
        let event_cash_delta = cash - pre_cash;
        let event_mtm_after = mark_to_market(cash, yes_shares, no_shares, last_mid);
        let event_mtm_delta = event_mtm_after - pre_mtm;
        if let Some(f) = decision_file.as_mut() {
            if idx % decision_every == 0 {
                let row = DecisionLogRow {
                    market_id: event.market_id.0,
                    event_idx: (idx + 1) as u64,
                    ts_ns: event.ts_ns,
                    market_mid: last_mid,
                    yes_mid: event.yes_mid,
                    yes_bid: event.yes_bid,
                    yes_ask: event.yes_ask,
                    cash_usdc_before: pre_cash,
                    yes_shares_before: pre_yes_shares,
                    no_shares_before: pre_no_shares,
                    direction_score,
                    confidence_score,
                    calibrated_p,
                    risk_score,
                    edge,
                    has_model_output,
                    strategy_emitted_model_output,
                    has_model_attribution,
                    side_is_yes: direction_score >= 0.0,
                    feature_momentum: model_attribution.direction.momentum,
                    feature_book_imbalance_top3: model_attribution.book_imbalance_top3,
                    feature_microprice_dev: model_attribution.direction.microprice_dev,
                    feature_microprice_spot_alignment: model_attribution
                        .direction
                        .microprice_spot_alignment,
                    feature_top3_delta_5s: model_attribution.direction.top3_delta_5s,
                    feature_top3_delta_15s: model_attribution.direction.top3_delta_15s,
                    feature_spot_score: model_attribution.spot_score,
                    feature_spot_fast_momentum: model_attribution.direction.spot_fast_momentum,
                    feature_spot_broad_momentum: model_attribution.direction.spot_broad_momentum,
                    feature_spot_momentum_600s: model_attribution.direction.spot_momentum_600s,
                    feature_spot_momentum_900s: model_attribution.direction.spot_momentum_900s,
                    feature_spot_momentum_1800s: model_attribution.direction.spot_momentum_1800s,
                    feature_spot_momentum_3600s: model_attribution.direction.spot_momentum_3600s,
                    feature_spot_fast_long_alignment: model_attribution
                        .direction
                        .spot_fast_long_alignment,
                    feature_spot_broad_trend_consistency: model_attribution
                        .direction
                        .spot_broad_trend_consistency,
                    feature_spot_broad_acceleration: model_attribution
                        .direction
                        .spot_broad_acceleration,
                    feature_direction_raw: model_attribution.direction_raw,
                    feature_stability: model_attribution.confidence.stability,
                    feature_sign_persistence: model_attribution.confidence.sign_persistence,
                    feature_markov_persistence: model_attribution.confidence.markov_persistence,
                    feature_early_market_penalty: model_attribution.confidence.early_market_penalty,
                    feature_time_of_day_edge: model_attribution.time_of_day_edge,
                    feature_time_of_day_advantage: model_attribution
                        .confidence
                        .time_of_day_advantage,
                    feature_whipsaw: model_attribution.risk.whipsaw,
                    feature_liquidity: model_attribution.risk.liquidity,
                    feature_path_risk: model_attribution.risk.path_risk,
                    feature_imbalance_turn: model_attribution.risk.imbalance_turn,
                    feature_markov_reversal_risk: model_attribution.risk.markov_reversal_risk,
                    feature_skew_penalty: model_attribution.risk.skew_penalty,
                    feature_volatility_penalty: model_attribution.risk.volatility_penalty,
                    feature_time_of_day_penalty: model_attribution.risk.time_of_day_penalty,
                    feature_volatility_regime: model_attribution.volatility_regime,
                    feature_dir_flip_rate_8: model_attribution.sequence.dir_flip_rate_8,
                    feature_dir_std_8: model_attribution.sequence.dir_std_8,
                    feature_dir_abs_mean_8: model_attribution.sequence.dir_abs_mean_8,
                    feature_side_p_pre_meta: model_attribution.side_probability_pre_meta,
                    feature_side_p_post_meta: model_attribution.side_probability_post_meta,
                    meta_calibrator_updates: model_attribution.meta_calibrator_updates,
                    orders_requested,
                    requested_shares,
                    requested_notional_usdc: requested_notional,
                    order_tags,
                    event_fill_notional_usdc: event_fill_notional,
                    event_fills: event_fill_count,
                    event_slippage_bps,
                    event_cash_delta_usdc: event_cash_delta,
                    event_mtm_delta_usdc: event_mtm_delta,
                };
                if let Some(rows) = decision_rows.as_mut() {
                    rows.push(row.clone());
                }
                writeln!(f, "{}", serde_json::to_string(&row)?)?;
            }
        } else if let Some(rows) = decision_rows.as_mut() {
            if idx % decision_every == 0 {
                rows.push(DecisionLogRow {
                    market_id: event.market_id.0,
                    event_idx: (idx + 1) as u64,
                    ts_ns: event.ts_ns,
                    market_mid: last_mid,
                    yes_mid: event.yes_mid,
                    yes_bid: event.yes_bid,
                    yes_ask: event.yes_ask,
                    cash_usdc_before: pre_cash,
                    yes_shares_before: pre_yes_shares,
                    no_shares_before: pre_no_shares,
                    direction_score,
                    confidence_score,
                    calibrated_p,
                    risk_score,
                    edge,
                    has_model_output,
                    strategy_emitted_model_output,
                    has_model_attribution,
                    side_is_yes: direction_score >= 0.0,
                    feature_momentum: model_attribution.direction.momentum,
                    feature_book_imbalance_top3: model_attribution.book_imbalance_top3,
                    feature_microprice_dev: model_attribution.direction.microprice_dev,
                    feature_microprice_spot_alignment: model_attribution
                        .direction
                        .microprice_spot_alignment,
                    feature_top3_delta_5s: model_attribution.direction.top3_delta_5s,
                    feature_top3_delta_15s: model_attribution.direction.top3_delta_15s,
                    feature_spot_score: model_attribution.spot_score,
                    feature_spot_fast_momentum: model_attribution.direction.spot_fast_momentum,
                    feature_spot_broad_momentum: model_attribution.direction.spot_broad_momentum,
                    feature_spot_momentum_600s: model_attribution.direction.spot_momentum_600s,
                    feature_spot_momentum_900s: model_attribution.direction.spot_momentum_900s,
                    feature_spot_momentum_1800s: model_attribution.direction.spot_momentum_1800s,
                    feature_spot_momentum_3600s: model_attribution.direction.spot_momentum_3600s,
                    feature_spot_fast_long_alignment: model_attribution
                        .direction
                        .spot_fast_long_alignment,
                    feature_spot_broad_trend_consistency: model_attribution
                        .direction
                        .spot_broad_trend_consistency,
                    feature_spot_broad_acceleration: model_attribution
                        .direction
                        .spot_broad_acceleration,
                    feature_direction_raw: model_attribution.direction_raw,
                    feature_stability: model_attribution.confidence.stability,
                    feature_sign_persistence: model_attribution.confidence.sign_persistence,
                    feature_markov_persistence: model_attribution.confidence.markov_persistence,
                    feature_early_market_penalty: model_attribution.confidence.early_market_penalty,
                    feature_time_of_day_edge: model_attribution.time_of_day_edge,
                    feature_time_of_day_advantage: model_attribution
                        .confidence
                        .time_of_day_advantage,
                    feature_whipsaw: model_attribution.risk.whipsaw,
                    feature_liquidity: model_attribution.risk.liquidity,
                    feature_path_risk: model_attribution.risk.path_risk,
                    feature_imbalance_turn: model_attribution.risk.imbalance_turn,
                    feature_markov_reversal_risk: model_attribution.risk.markov_reversal_risk,
                    feature_skew_penalty: model_attribution.risk.skew_penalty,
                    feature_volatility_penalty: model_attribution.risk.volatility_penalty,
                    feature_time_of_day_penalty: model_attribution.risk.time_of_day_penalty,
                    feature_volatility_regime: model_attribution.volatility_regime,
                    feature_dir_flip_rate_8: model_attribution.sequence.dir_flip_rate_8,
                    feature_dir_std_8: model_attribution.sequence.dir_std_8,
                    feature_dir_abs_mean_8: model_attribution.sequence.dir_abs_mean_8,
                    feature_side_p_pre_meta: model_attribution.side_probability_pre_meta,
                    feature_side_p_post_meta: model_attribution.side_probability_post_meta,
                    meta_calibrator_updates: model_attribution.meta_calibrator_updates,
                    orders_requested,
                    requested_shares,
                    requested_notional_usdc: requested_notional,
                    order_tags,
                    event_fill_notional_usdc: event_fill_notional,
                    event_fills: event_fill_count,
                    event_slippage_bps,
                    event_cash_delta_usdc: event_cash_delta,
                    event_mtm_delta_usdc: event_mtm_delta,
                });
            }
        }
    }

    if last_window_idx < 0 {
        last_mid = events.last().map_or(0.0, |e| e.yes_mid);
    }

    counters.resting_orders_cancelled_eom = resting.len();
    resting.clear();

    let yes_resolved = cfg.resolved_yes.unwrap_or(last_mid >= 0.5);
    let settlement_cash = if yes_resolved { yes_shares } else { no_shares };
    let end_cash = cash + settlement_cash;
    portfolio.mark(end_cash);

    let final_ts_ns = if last_window_idx >= 0 {
        events[last_window_idx as usize].ts_ns
    } else {
        events.last().map(|e| e.ts_ns).unwrap_or(0)
    };
    let final_snapshot = portfolio.snapshot(final_ts_ns, end_cash);
    let peak_equity_usdc = final_snapshot.peak_equity_usdc;
    let max_drawdown_pct = if peak_equity_usdc > 0.0 {
        1.0 - end_cash / peak_equity_usdc
    } else {
        0.0
    }
    .max(final_snapshot.drawdown_pct);

    strategy.on_market_resolved(last_mid, yes_resolved);
    if let Some(last) = last_canonical_sample_point {
        if canonical_meta_sample_points.last().copied() != Some(last) {
            canonical_meta_sample_points.push(last);
        }
    }
    let model_training_samples = canonical_meta_sample_points
        .into_iter()
        .map(
            |(features, base_side_probability, predicted_yes)| MetaTrainingSample {
                features,
                market_idx: 0,
                base_side_probability,
                side_observed: if predicted_yes {
                    yes_resolved
                } else {
                    !yes_resolved
                },
            },
        )
        .collect();
    if cfg.update_model_state_on_resolution {
        if let Some(predicted_yes) = last_canonical_prediction_is_yes {
            if let Some(shared) = &cfg.shared_model_state {
                let mut state = shared.lock().expect("shared model mutex poisoned");
                state.record_market_result(last_mid, predicted_yes, yes_resolved);
            } else {
                model_state.record_market_result(last_mid, predicted_yes, yes_resolved);
            }
        }
    }

    if let (Some(path), Some(rows)) = (cfg.decision_log_parquet.as_deref(), decision_rows.as_ref())
    {
        write_decision_rows_parquet(path, rows)?;
    }

    Ok(BacktestReport {
        events_processed,
        counters,
        start_equity_usdc: cfg.starting_cash_usdc,
        end_equity_usdc: end_cash,
        pnl_usdc: end_cash - cfg.starting_cash_usdc,
        maker_rebates_usdc: total_rebates,
        peak_equity_usdc,
        max_drawdown_pct,
        final_yes_shares: yes_shares,
        final_no_shares: no_shares,
        final_cash_usdc: cash,
        yes_resolved,
        last_yes_mid: last_mid,
        fills,
        final_portfolio: final_snapshot,
        model_training_samples,
    })
}

fn write_decision_rows_parquet(path: &Path, rows: &[DecisionLogRow]) -> Result<()> {
    let schema = Arc::new(Schema::new(vec![
        Field::new("market_id", DataType::UInt32, false),
        Field::new("event_idx", DataType::UInt64, false),
        Field::new("ts_ns", DataType::Int64, false),
        Field::new("market_mid", DataType::Float32, false),
        Field::new("yes_mid", DataType::Float32, false),
        Field::new("yes_bid", DataType::Float32, false),
        Field::new("yes_ask", DataType::Float32, false),
        Field::new("cash_usdc_before", DataType::Float64, false),
        Field::new("yes_shares_before", DataType::Float64, false),
        Field::new("no_shares_before", DataType::Float64, false),
        Field::new("direction_score", DataType::Float32, false),
        Field::new("confidence_score", DataType::Float32, false),
        Field::new("calibrated_p", DataType::Float32, false),
        Field::new("risk_score", DataType::Float32, false),
        Field::new("edge", DataType::Float32, false),
        Field::new("has_model_output", DataType::Boolean, false),
        Field::new("strategy_emitted_model_output", DataType::Boolean, false),
        Field::new("has_model_attribution", DataType::Boolean, false),
        Field::new("side_is_yes", DataType::Boolean, false),
        Field::new("feature_momentum", DataType::Float32, false),
        Field::new("feature_book_imbalance_top3", DataType::Float32, false),
        Field::new("feature_microprice_dev", DataType::Float32, false),
        Field::new(
            "feature_microprice_spot_alignment",
            DataType::Float32,
            false,
        ),
        Field::new("feature_top3_delta_5s", DataType::Float32, false),
        Field::new("feature_top3_delta_15s", DataType::Float32, false),
        Field::new("feature_spot_score", DataType::Float32, false),
        Field::new("feature_spot_fast_momentum", DataType::Float32, false),
        Field::new("feature_spot_broad_momentum", DataType::Float32, false),
        Field::new("feature_spot_momentum_600s", DataType::Float32, false),
        Field::new("feature_spot_momentum_900s", DataType::Float32, false),
        Field::new("feature_spot_momentum_1800s", DataType::Float32, false),
        Field::new("feature_spot_momentum_3600s", DataType::Float32, false),
        Field::new("feature_spot_fast_long_alignment", DataType::Float32, false),
        Field::new(
            "feature_spot_broad_trend_consistency",
            DataType::Float32,
            false,
        ),
        Field::new("feature_spot_broad_acceleration", DataType::Float32, false),
        Field::new("feature_direction_raw", DataType::Float32, false),
        Field::new("feature_stability", DataType::Float32, false),
        Field::new("feature_sign_persistence", DataType::Float32, false),
        Field::new("feature_markov_persistence", DataType::Float32, false),
        Field::new("feature_early_market_penalty", DataType::Float32, false),
        Field::new("feature_time_of_day_edge", DataType::Float32, false),
        Field::new("feature_time_of_day_advantage", DataType::Float32, false),
        Field::new("feature_whipsaw", DataType::Float32, false),
        Field::new("feature_liquidity", DataType::Float32, false),
        Field::new("feature_path_risk", DataType::Float32, false),
        Field::new("feature_imbalance_turn", DataType::Float32, false),
        Field::new("feature_markov_reversal_risk", DataType::Float32, false),
        Field::new("feature_skew_penalty", DataType::Float32, false),
        Field::new("feature_volatility_penalty", DataType::Float32, false),
        Field::new("feature_time_of_day_penalty", DataType::Float32, false),
        Field::new("feature_volatility_regime", DataType::Float32, false),
        Field::new("feature_dir_flip_rate_8", DataType::Float32, false),
        Field::new("feature_dir_std_8", DataType::Float32, false),
        Field::new("feature_dir_abs_mean_8", DataType::Float32, false),
        Field::new("feature_side_p_pre_meta", DataType::Float32, false),
        Field::new("feature_side_p_post_meta", DataType::Float32, false),
        Field::new("meta_calibrator_updates", DataType::UInt32, false),
        Field::new("orders_requested", DataType::UInt64, false),
        Field::new("requested_shares", DataType::Float64, false),
        Field::new("requested_notional_usdc", DataType::Float64, false),
        Field::new("order_tags", DataType::Utf8, false),
        Field::new("event_fill_notional_usdc", DataType::Float64, false),
        Field::new("event_fills", DataType::UInt64, false),
        Field::new("event_slippage_bps", DataType::Float32, false),
        Field::new("event_cash_delta_usdc", DataType::Float64, false),
        Field::new("event_mtm_delta_usdc", DataType::Float64, false),
    ]));

    let cols: Vec<ArrayRef> = vec![
        Arc::new(UInt32Array::from_iter_values(
            rows.iter().map(|r| r.market_id),
        )),
        Arc::new(UInt64Array::from_iter_values(
            rows.iter().map(|r| r.event_idx),
        )),
        Arc::new(Int64Array::from_iter_values(rows.iter().map(|r| r.ts_ns))),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.market_mid),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.yes_mid),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.yes_bid),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.yes_ask),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.cash_usdc_before),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.yes_shares_before),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.no_shares_before),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.direction_score),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.confidence_score),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.calibrated_p),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.risk_score),
        )),
        Arc::new(Float32Array::from_iter_values(rows.iter().map(|r| r.edge))),
        Arc::new(BooleanArray::from_iter(
            rows.iter().map(|r| Some(r.has_model_output)),
        )),
        Arc::new(BooleanArray::from_iter(
            rows.iter().map(|r| Some(r.strategy_emitted_model_output)),
        )),
        Arc::new(BooleanArray::from_iter(
            rows.iter().map(|r| Some(r.has_model_attribution)),
        )),
        Arc::new(BooleanArray::from_iter(
            rows.iter().map(|r| Some(r.side_is_yes)),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_momentum),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_book_imbalance_top3),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_microprice_dev),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_microprice_spot_alignment),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_top3_delta_5s),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_top3_delta_15s),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_score),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_fast_momentum),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_broad_momentum),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_momentum_600s),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_momentum_900s),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_momentum_1800s),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_momentum_3600s),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_fast_long_alignment),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_broad_trend_consistency),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_spot_broad_acceleration),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_direction_raw),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_stability),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_sign_persistence),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_markov_persistence),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_early_market_penalty),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_time_of_day_edge),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_time_of_day_advantage),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_whipsaw),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_liquidity),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_path_risk),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_imbalance_turn),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_markov_reversal_risk),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_skew_penalty),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_volatility_penalty),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_time_of_day_penalty),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_volatility_regime),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_dir_flip_rate_8),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_dir_std_8),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_dir_abs_mean_8),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_side_p_pre_meta),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.feature_side_p_post_meta),
        )),
        Arc::new(UInt32Array::from_iter_values(
            rows.iter().map(|r| r.meta_calibrator_updates),
        )),
        Arc::new(UInt64Array::from_iter_values(
            rows.iter().map(|r| r.orders_requested as u64),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.requested_shares),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.requested_notional_usdc),
        )),
        Arc::new(StringArray::from_iter_values(
            rows.iter().map(|r| r.order_tags.join(",")),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.event_fill_notional_usdc),
        )),
        Arc::new(UInt64Array::from_iter_values(
            rows.iter().map(|r| r.event_fills as u64),
        )),
        Arc::new(Float32Array::from_iter_values(
            rows.iter().map(|r| r.event_slippage_bps),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.event_cash_delta_usdc),
        )),
        Arc::new(Float64Array::from_iter_values(
            rows.iter().map(|r| r.event_mtm_delta_usdc),
        )),
    ];

    let batch = RecordBatch::try_new(schema.clone(), cols)?;
    let file = std::fs::File::create(path)?;
    let mut writer = ArrowWriter::try_new(file, schema, None)?;
    writer.write(&batch)?;
    writer.close()?;
    Ok(())
}

fn order_request_notional_usdc(req: OrderRequest, event: &ReplayEvent) -> Option<f64> {
    match req.side {
        Side::BuyYes => {
            if event.yes_ask <= 0.0 {
                return None;
            }
            let px = if let Some(limit) = req.limit_price {
                if limit > 0.0 && limit < 1.0 {
                    limit
                } else {
                    event.yes_ask
                }
            } else {
                event.yes_ask
            };
            Some(px as f64 * req.shares)
        }
        Side::BuyNo => {
            let implied = (1.0 - event.yes_bid).max(0.0);
            if implied <= 0.0 {
                return None;
            }
            let px = if let Some(limit) = req.limit_price {
                let no_px = (1.0 - limit).clamp(0.0, 1.0);
                if no_px > 0.0 && no_px < 1.0 {
                    no_px
                } else {
                    implied
                }
            } else {
                implied
            };
            Some(px as f64 * req.shares)
        }
        Side::SellYes => {
            if event.yes_bid <= 0.0 {
                return None;
            }
            let px = if let Some(limit) = req.limit_price {
                if limit > 0.0 && limit < 1.0 {
                    limit
                } else {
                    event.yes_bid
                }
            } else {
                event.yes_bid
            };
            Some(px as f64 * req.shares)
        }
        Side::SellNo => {
            let implied = (1.0 - event.yes_ask).max(0.0);
            if implied <= 0.0 {
                return None;
            }
            let px = if let Some(limit) = req.limit_price {
                let no_px = (1.0 - limit).clamp(0.0, 1.0);
                if no_px > 0.0 && no_px < 1.0 {
                    no_px
                } else {
                    implied
                }
            } else {
                implied
            };
            Some(px as f64 * req.shares)
        }
    }
}

fn mark_to_market(cash: f64, yes_shares: f64, no_shares: f64, yes_mid: f32) -> f64 {
    let p = yes_mid.clamp(0.0, 1.0) as f64;
    cash + yes_shares * p + no_shares * (1.0 - p)
}

/// Convert a strategy-side limit price (in YES- or NO-native terms) into a
/// canonical YES-side limit price used by the resting book. For NO orders, the
/// strategy's "limit_price" is in NO-terms; we flip via `1 - L_no`.
fn limit_to_yes_terms(side: Side, limit_price: f32) -> f32 {
    match side {
        Side::BuyYes | Side::SellYes => limit_price,
        Side::BuyNo | Side::SellNo => (1.0 - limit_price).clamp(0.0, 1.0),
    }
}

fn order_adds_yes_exposure(side: Side) -> bool {
    matches!(side, Side::BuyYes | Side::SellNo)
}

#[allow(clippy::too_many_arguments)]
fn apply_maker_fill(
    market_id: u32,
    ts_ns: i64,
    side: Side,
    fill_price_native: f32,
    shares: f64,
    tag: &'static str,
    model_context: Option<FillModelContext>,
    cash: &mut f64,
    yes_shares: &mut f64,
    no_shares: &mut f64,
    portfolio: &mut PortfolioState,
    counters: &mut StrategyCounters,
    fills: &mut Vec<Fill>,
    total_rebates: &mut f64,
    maker_rebate_bps: f64,
) -> bool {
    let notional = shares * fill_price_native as f64;
    match side {
        Side::BuyYes | Side::BuyNo => {
            if !portfolio.can_open_position(market_id, notional) {
                counters.orders_rejected_risk_gate += 1;
                return false;
            }
            if notional > *cash {
                counters.orders_rejected_no_cash += 1;
                return false;
            }
        }
        Side::SellYes => {
            if shares > *yes_shares {
                counters.orders_rejected_no_inventory += 1;
                return false;
            }
        }
        Side::SellNo => {
            if shares > *no_shares {
                counters.orders_rejected_no_inventory += 1;
                return false;
            }
        }
    }

    match side {
        Side::BuyYes => {
            *cash -= notional;
            *yes_shares += shares;
            portfolio.record_outlay(market_id, ts_ns, notional);
        }
        Side::SellYes => {
            *cash += notional;
            *yes_shares -= shares;
        }
        Side::BuyNo => {
            *cash -= notional;
            *no_shares += shares;
            portfolio.record_outlay(market_id, ts_ns, notional);
        }
        Side::SellNo => {
            *cash += notional;
            *no_shares -= shares;
        }
    }

    let rebate = notional * maker_rebate_bps / 10_000.0;
    *cash += rebate;
    *total_rebates += rebate;

    counters.orders_filled_maker += 1;
    fills.push(Fill {
        ts_ns,
        side: format!("{:?}", side),
        shares,
        price: fill_price_native,
        notional,
        tag: tag.to_string(),
        maker: true,
        rebate_usdc: rebate,
        slippage_bps: 0.0,
        yes_mid: model_context.map(|m| m.yes_mid),
        yes_bid: model_context.map(|m| m.yes_bid),
        yes_ask: model_context.map(|m| m.yes_ask),
        side_model_p: model_context.map(|m| m.side_model_p),
        side_edge_vs_mid: model_context.map(|m| m.side_edge_vs_mid),
        side_edge_vs_fill: model_context.map(|m| m.side_model_p - fill_price_native),
        direction_score: model_context.map(|m| m.direction_score),
        confidence_score: model_context.map(|m| m.confidence_score),
        calibrated_p: model_context.map(|m| m.calibrated_p),
        risk_score: model_context.map(|m| m.risk_score),
        seconds_since_open: model_context.map(|m| m.seconds_since_open),
        seconds_to_close: model_context.map(|m| m.seconds_to_close),
        regime_whipsaw_score: model_context.map(|m| m.regime_whipsaw_score),
        regime_path_efficiency: model_context.map(|m| m.regime_path_efficiency),
        regime_reversal_pressure: model_context.map(|m| m.regime_reversal_pressure),
        regime_sign_flip_rate: model_context.map(|m| m.regime_sign_flip_rate),
        regime_realized_vol_180s_bps: model_context.map(|m| m.regime_realized_vol_180s_bps),
    });
    true
}

#[allow(clippy::too_many_arguments)]
fn check_trade_driven_resting_fills(
    event: &ReplayEvent,
    trades: &TradeHistory,
    trade_cursor: &mut usize,
    resting: &mut Vec<RestingOrder>,
    cash: &mut f64,
    yes_shares: &mut f64,
    no_shares: &mut f64,
    portfolio: &mut PortfolioState,
    counters: &mut StrategyCounters,
    fills: &mut Vec<Fill>,
    total_rebates: &mut f64,
    maker_rebate_bps: f64,
) {
    let samples = trades.samples();
    while *trade_cursor < samples.len() && samples[*trade_cursor].ts_ns <= event.ts_ns {
        let trade = samples[*trade_cursor];
        *trade_cursor += 1;
        let mut remaining = trade.size as f64;
        let mut i = 0;
        while remaining > 0.0 && i < resting.len() {
            let r = resting[i];
            if trade.ts_ns <= r.submit_ts_ns {
                i += 1;
                continue;
            }
            let fills_order = match r.side {
                Side::BuyYes | Side::SellNo => {
                    !trade.aggressor_buy && trade.price <= r.limit_yes
                }
                Side::SellYes | Side::BuyNo => trade.aggressor_buy && trade.price >= r.limit_yes,
            };
            if !fills_order {
                i += 1;
                continue;
            }

            let fill_shares = remaining.min(r.shares);
            let fill_price_native = match r.side {
                Side::BuyYes | Side::SellYes => r.limit_yes,
                Side::BuyNo | Side::SellNo => 1.0 - r.limit_yes,
            };
            if !apply_maker_fill(
                event.market_id.0,
                trade.ts_ns,
                r.side,
                fill_price_native,
                fill_shares,
                r.tag,
                r.model_context,
                cash,
                yes_shares,
                no_shares,
                portfolio,
                counters,
                fills,
                total_rebates,
                maker_rebate_bps,
            ) {
                resting.swap_remove(i);
                continue;
            }
            remaining -= fill_shares;
            if fill_shares >= resting[i].shares {
                resting.swap_remove(i);
            } else {
                resting[i].shares -= fill_shares;
                i += 1;
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn check_resting_fills(
    event: &ReplayEvent,
    resting: &mut Vec<RestingOrder>,
    cash: &mut f64,
    yes_shares: &mut f64,
    no_shares: &mut f64,
    portfolio: &mut PortfolioState,
    counters: &mut StrategyCounters,
    fills: &mut Vec<Fill>,
    total_rebates: &mut f64,
    maker_rebate_bps: f64,
) {
    // Walk resting orders; collect ones that filled this tick. The book must
    // STRICTLY CROSS past the limit (not just touch), modeling queue priority —
    // touching the limit means we're at the front but other resting orders
    // are also there; if the book actually moves past, those queue holders
    // (and us) get filled.
    let mut i = 0;
    while i < resting.len() {
        let r = resting[i];
        let crossed = match r.side {
            Side::BuyYes | Side::SellNo => {
                // Bidding YES at limit_yes; fill when ask strictly drops below.
                event.yes_ask > 0.0 && event.yes_ask < r.limit_yes
            }
            Side::SellYes | Side::BuyNo => {
                // Asking YES at limit_yes; fill when bid strictly rises above.
                event.yes_bid > 0.0 && event.yes_bid > r.limit_yes
            }
        };
        if !crossed {
            i += 1;
            continue;
        }
        // Translate fill price back to native side for notional accounting.
        let fill_price_native = match r.side {
            Side::BuyYes | Side::SellYes => r.limit_yes,
            Side::BuyNo | Side::SellNo => 1.0 - r.limit_yes,
        };
        let notional = (r.shares as f64) * fill_price_native as f64;

        // Sanity gates (inventory, cash, risk).
        let mut rejected = false;
        match r.side {
            Side::BuyYes | Side::BuyNo => {
                if !portfolio.can_open_position(event.market_id.0, notional) {
                    counters.orders_rejected_risk_gate += 1;
                    rejected = true;
                } else if notional > *cash {
                    counters.orders_rejected_no_cash += 1;
                    rejected = true;
                }
            }
            Side::SellYes => {
                if r.shares > *yes_shares {
                    counters.orders_rejected_no_inventory += 1;
                    rejected = true;
                }
            }
            Side::SellNo => {
                if r.shares > *no_shares {
                    counters.orders_rejected_no_inventory += 1;
                    rejected = true;
                }
            }
        }
        if rejected {
            resting.swap_remove(i);
            continue;
        }

        // Apply.
        match r.side {
            Side::BuyYes => {
                *cash -= notional;
                *yes_shares += r.shares;
                portfolio.record_outlay(event.market_id.0, event.ts_ns, notional);
            }
            Side::SellYes => {
                *cash += notional;
                *yes_shares -= r.shares;
            }
            Side::BuyNo => {
                *cash -= notional;
                *no_shares += r.shares;
                portfolio.record_outlay(event.market_id.0, event.ts_ns, notional);
            }
            Side::SellNo => {
                *cash += notional;
                *no_shares -= r.shares;
            }
        }
        let rebate = notional * maker_rebate_bps / 10_000.0;
        *cash += rebate;
        *total_rebates += rebate;

        counters.orders_filled_maker += 1;
        fills.push(Fill {
            ts_ns: event.ts_ns,
            side: format!("{:?}", r.side),
            shares: r.shares,
            price: fill_price_native,
            notional,
            tag: r.tag.to_string(),
            maker: true,
            rebate_usdc: rebate,
            slippage_bps: 0.0,
            yes_mid: r.model_context.map(|m| m.yes_mid),
            yes_bid: r.model_context.map(|m| m.yes_bid),
            yes_ask: r.model_context.map(|m| m.yes_ask),
            side_model_p: r.model_context.map(|m| m.side_model_p),
            side_edge_vs_mid: r.model_context.map(|m| m.side_edge_vs_mid),
            side_edge_vs_fill: r.model_context.map(|m| m.side_model_p - fill_price_native),
            direction_score: r.model_context.map(|m| m.direction_score),
            confidence_score: r.model_context.map(|m| m.confidence_score),
            calibrated_p: r.model_context.map(|m| m.calibrated_p),
            risk_score: r.model_context.map(|m| m.risk_score),
            seconds_since_open: r.model_context.map(|m| m.seconds_since_open),
            seconds_to_close: r.model_context.map(|m| m.seconds_to_close),
            regime_whipsaw_score: r.model_context.map(|m| m.regime_whipsaw_score),
            regime_path_efficiency: r.model_context.map(|m| m.regime_path_efficiency),
            regime_reversal_pressure: r.model_context.map(|m| m.regime_reversal_pressure),
            regime_sign_flip_rate: r.model_context.map(|m| m.regime_sign_flip_rate),
            regime_realized_vol_180s_bps: r.model_context.map(|m| m.regime_realized_vol_180s_bps),
        });
        let _ = r.submit_ts_ns;
        resting.swap_remove(i);
    }
}

#[allow(clippy::too_many_arguments)]
fn submit_maker_order(
    event: &ReplayEvent,
    req: &OrderRequest,
    limit: f32,
    resting: &mut Vec<RestingOrder>,
    cash: &mut f64,
    yes_shares: &mut f64,
    no_shares: &mut f64,
    portfolio: &mut PortfolioState,
    counters: &mut StrategyCounters,
    fills: &mut Vec<Fill>,
    total_rebates: &mut f64,
    maker_rebate_bps: f64,
    taker_fee_bps: f64,
    taker_slippage_bps: f64,
    model_context: Option<FillModelContext>,
) {
    let limit_yes = limit_to_yes_terms(req.side, limit);
    // Crosses immediately = strategy was actually a taker. Apply taker fill at
    // the limit (not better than the opposite top of book) for realism.
    let immediate = match req.side {
        Side::BuyYes | Side::SellNo => event.yes_ask > 0.0 && limit_yes >= event.yes_ask,
        Side::SellYes | Side::BuyNo => event.yes_bid > 0.0 && limit_yes <= event.yes_bid,
    };
    if immediate {
        // Treat as a taker fill at the opposite top of book (better for buyer
        // than the limit, conservative for seller).
        let synthetic = OrderRequest {
            side: req.side,
            shares: req.shares,
            max_depth: req.max_depth,
            limit_price: Some(limit),
            tag: req.tag,
        };
        apply_taker_order(
            event,
            &synthetic,
            cash,
            yes_shares,
            no_shares,
            portfolio,
            counters,
            fills,
            taker_fee_bps,
            taker_slippage_bps,
            model_context,
        );
        return;
    }

    // Risk-gate quote-side check on the prospective notional (use limit price).
    let prospective_notional = match req.side {
        Side::BuyYes | Side::BuyNo => {
            let px = match req.side {
                Side::BuyYes => limit_yes,
                Side::BuyNo => 1.0 - limit_yes,
                _ => unreachable!(),
            };
            (req.shares as f64) * px as f64
        }
        _ => 0.0,
    };
    if matches!(req.side, Side::BuyYes | Side::BuyNo)
        && !portfolio.can_open_position(event.market_id.0, prospective_notional)
    {
        counters.orders_rejected_risk_gate += 1;
        return;
    }
    let _ = (total_rebates, maker_rebate_bps); // not credited until fill

    resting.push(RestingOrder {
        side: req.side,
        limit_yes,
        shares: req.shares,
        submit_ts_ns: event.ts_ns,
        tag: req.tag,
        model_context,
    });
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
fn apply_taker_order(
    event: &ReplayEvent,
    req: &OrderRequest,
    cash: &mut f64,
    yes_shares: &mut f64,
    no_shares: &mut f64,
    portfolio: &mut PortfolioState,
    counters: &mut StrategyCounters,
    fills: &mut Vec<Fill>,
    taker_fee_bps: f64,
    taker_slippage_bps: f64,
    model_context: Option<FillModelContext>,
) {
    let Some((raw_fill, fillable_shares)) = depth_weighted_fill(event, req) else {
        counters.orders_rejected_no_liquidity += 1;
        return;
    };
    // Apply slippage: buyers pay more, sellers receive less. Clamp into (0,1).
    // At extreme prices (thin books near 0 or 1), bp-slippage is too small —
    // add absolute tick slippage to model the realistic walk-the-book cost
    // of executing in shallow liquidity zones.
    let slip = taker_slippage_bps / 10_000.0;
    let raw_f64 = raw_fill as f64;
    let extreme_ticks = if raw_f64 <= 0.08 || raw_f64 >= 0.92 {
        2.0_f64
    } else if raw_f64 <= 0.15 || raw_f64 >= 0.85 {
        1.0_f64
    } else {
        0.0_f64
    };
    let tick = 0.01_f64;
    let fill_price = match req.side {
        Side::BuyYes | Side::BuyNo => {
            (raw_f64 * (1.0 + slip) + extreme_ticks * tick).min(0.999) as f32
        }
        Side::SellYes | Side::SellNo => {
            (raw_f64 * (1.0 - slip) - extreme_ticks * tick).max(0.001) as f32
        }
    };
    if fill_price <= 0.0 || fill_price >= 1.0 {
        counters.orders_rejected_bad_price += 1;
        return;
    }
    if !fill_respects_limit(req.side, fill_price, req.limit_price) {
        counters.orders_rejected_no_liquidity += 1;
        return;
    }
    if fillable_shares <= 0.0 {
        counters.orders_rejected_no_liquidity += 1;
        return;
    }
    let notional = fillable_shares * fill_price as f64;
    let fee = notional * taker_fee_bps / 10_000.0;

    match req.side {
        Side::BuyYes | Side::BuyNo => {
            if !portfolio.can_open_position(event.market_id.0, notional + fee) {
                counters.orders_rejected_risk_gate += 1;
                return;
            }
            if notional + fee > *cash {
                counters.orders_rejected_no_cash += 1;
                return;
            }
        }
        Side::SellYes => {
            if fillable_shares > *yes_shares {
                counters.orders_rejected_no_inventory += 1;
                return;
            }
        }
        Side::SellNo => {
            if fillable_shares > *no_shares {
                counters.orders_rejected_no_inventory += 1;
                return;
            }
        }
    }
    match req.side {
        Side::BuyYes => {
            *cash -= notional + fee;
            *yes_shares += fillable_shares;
            portfolio.record_outlay(event.market_id.0, event.ts_ns, notional);
        }
        Side::SellYes => {
            *cash += notional - fee;
            *yes_shares -= fillable_shares;
        }
        Side::BuyNo => {
            *cash -= notional + fee;
            *no_shares += fillable_shares;
            portfolio.record_outlay(event.market_id.0, event.ts_ns, notional);
        }
        Side::SellNo => {
            *cash += notional - fee;
            *no_shares -= fillable_shares;
        }
    }
    counters.orders_filled_taker += 1;
    fills.push(Fill {
        ts_ns: event.ts_ns,
        side: format!("{:?}", req.side),
        shares: fillable_shares,
        price: fill_price,
        notional,
        tag: req.tag.to_string(),
        maker: false,
        rebate_usdc: -fee,
        slippage_bps: (((fill_price as f64 - raw_fill as f64).abs() / (raw_fill as f64).max(1e-12))
            * 10_000.0) as f32,
        yes_mid: model_context.map(|m| m.yes_mid),
        yes_bid: model_context.map(|m| m.yes_bid),
        yes_ask: model_context.map(|m| m.yes_ask),
        side_model_p: model_context.map(|m| m.side_model_p),
        side_edge_vs_mid: model_context.map(|m| m.side_edge_vs_mid),
        side_edge_vs_fill: model_context.map(|m| m.side_model_p - fill_price),
        direction_score: model_context.map(|m| m.direction_score),
        confidence_score: model_context.map(|m| m.confidence_score),
        calibrated_p: model_context.map(|m| m.calibrated_p),
        risk_score: model_context.map(|m| m.risk_score),
        seconds_since_open: model_context.map(|m| m.seconds_since_open),
        seconds_to_close: model_context.map(|m| m.seconds_to_close),
        regime_whipsaw_score: model_context.map(|m| m.regime_whipsaw_score),
        regime_path_efficiency: model_context.map(|m| m.regime_path_efficiency),
        regime_reversal_pressure: model_context.map(|m| m.regime_reversal_pressure),
        regime_sign_flip_rate: model_context.map(|m| m.regime_sign_flip_rate),
        regime_realized_vol_180s_bps: model_context.map(|m| m.regime_realized_vol_180s_bps),
    });
}

fn depth_weighted_fill(event: &ReplayEvent, req: &OrderRequest) -> Option<(f32, f64)> {
    let depth = req.max_depth.clamp(1, pm_types::TAPE_DEPTH);
    let mut remaining = req.shares.max(0.0);
    let mut filled = 0.0;
    let mut notional = 0.0;

    for level in 0..depth {
        let (price, size) = match req.side {
            Side::BuyYes => (event.asks[level].price, event.asks[level].size),
            Side::SellYes => (event.bids[level].price, event.bids[level].size),
            Side::BuyNo => (
                (1.0 - event.bids[level].price).max(0.0),
                event.bids[level].size,
            ),
            Side::SellNo => (
                (1.0 - event.asks[level].price).max(0.0),
                event.asks[level].size,
            ),
        };
        if price <= 0.0 || price >= 1.0 || size <= 0.0 {
            continue;
        }
        if !fill_respects_limit(req.side, price, req.limit_price) {
            continue;
        }
        let take = remaining.min(size as f64);
        if take <= 0.0 {
            break;
        }
        filled += take;
        notional += take * price as f64;
        remaining -= take;
        if remaining <= 1e-9 {
            break;
        }
    }

    if filled <= 0.0 {
        return None;
    }
    Some(((notional / filled) as f32, filled))
}

fn fill_respects_limit(side: Side, price: f32, limit_price: Option<f32>) -> bool {
    let Some(limit) = limit_price else {
        return true;
    };
    match side {
        Side::BuyYes | Side::BuyNo => price <= limit,
        Side::SellYes | Side::SellNo => price >= limit,
    }
}

pub fn pretty_print(rep: &BacktestReport) {
    println!("== backtest report ==");
    println!("events_processed  : {}", rep.events_processed);
    println!(
        "orders            : submitted={}  filled[taker={} maker={}]  rejected[cash={} liq={} px={} inv={} risk={} model={} model_reason[conf={} risk={} edge={}]]  resting_active={}  resting_cancelled_eom={}",
        rep.counters.orders_submitted,
        rep.counters.orders_filled_taker,
        rep.counters.orders_filled_maker,
        rep.counters.orders_rejected_no_cash,
        rep.counters.orders_rejected_no_liquidity,
        rep.counters.orders_rejected_bad_price,
        rep.counters.orders_rejected_no_inventory,
        rep.counters.orders_rejected_risk_gate,
        rep.counters.orders_rejected_model_gate,
        rep.counters.orders_rejected_model_gate_confidence,
        rep.counters.orders_rejected_model_gate_risk,
        rep.counters.orders_rejected_model_gate_edge,
        rep.counters.resting_orders_active,
        rep.counters.resting_orders_cancelled_eom,
    );
    println!(
        "equity            : {:>10.4} -> {:>10.4} USDC  (pnl {:>+.4}; rebates {:>+.4})",
        rep.start_equity_usdc, rep.end_equity_usdc, rep.pnl_usdc, rep.maker_rebates_usdc
    );
    println!(
        "peak / max_dd     : {:>10.4} USDC   {:.2}%",
        rep.peak_equity_usdc,
        rep.max_drawdown_pct * 100.0
    );
    println!(
        "final position    : yes={:.4}  no={:.4}  cash={:.4}",
        rep.final_yes_shares, rep.final_no_shares, rep.final_cash_usdc
    );
    println!(
        "resolution        : yes_resolved={}  last_mid={:.4}",
        rep.yes_resolved, rep.last_yes_mid
    );
    if let Some(reason) = rep.final_portfolio.halt_reason.as_deref() {
        println!("HALTED            : {reason}");
    }
    if rep.fills.is_empty() {
        return;
    }
    println!("\nfills (showing first 20):");
    for f in rep.fills.iter().take(20) {
        let dt = DateTime::<Utc>::from_timestamp_nanos(f.ts_ns);
        println!(
            "  {} {:>7} {:>22}  shares={:>8.4} price={:.4} notional={:.4}  {}",
            dt.format("%Y-%m-%dT%H:%M:%S%.3fZ"),
            f.side,
            f.tag,
            f.shares,
            f.price,
            f.notional,
            if f.maker { "MAKER" } else { "TAKER" }
        );
    }
    if rep.fills.len() > 20 {
        println!("  ... and {} more", rep.fills.len() - 20);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pm_model::ModelOutput;
    use pm_strategy::{BuyYesAtOpen, OrderRequest, Side, Strategy, StrategyOutput};
    use pm_types::{BookLevel, MarketId, ReplayFlags, tape::TAPE_DEPTH};

    fn evt(ts_ns: i64, bid: f32, ask: f32, size: f32) -> ReplayEvent {
        let mut bids = [BookLevel::default(); TAPE_DEPTH];
        let mut asks = [BookLevel::default(); TAPE_DEPTH];
        bids[0] = BookLevel { price: bid, size };
        asks[0] = BookLevel { price: ask, size };
        ReplayEvent {
            ts_ns,
            market_id: MarketId(1),
            yes_mid: 0.5 * (bid + ask),
            yes_bid: bid,
            yes_ask: ask,
            volume: 0.0,
            bids,
            asks,
            spot_price: 0.0,
            flags: ReplayFlags::BOOK_UPDATE,
        }
    }

    #[test]
    fn taker_buy_yes_can_sweep_deeper_book_levels() {
        let mut event = evt(0, 0.49, 0.50, 5.0);
        event.asks[1] = BookLevel {
            price: 0.55,
            size: 5.0,
        };
        event.asks[2] = BookLevel {
            price: 0.60,
            size: 10.0,
        };

        let shallow = OrderRequest {
            side: Side::BuyYes,
            shares: 12.0,
            max_depth: 1,
            limit_price: None,
            tag: "shallow",
        };
        let deep = OrderRequest {
            side: Side::BuyYes,
            shares: 12.0,
            max_depth: 3,
            limit_price: None,
            tag: "deep",
        };

        let mut shallow_cash = 100.0;
        let mut shallow_yes = 0.0;
        let mut shallow_no = 0.0;
        let limits = PortfolioLimits {
            max_clip_usdc: 10.0,
            ..PortfolioLimits::default()
        };
        let mut shallow_portfolio = PortfolioState::new(100.0, limits.clone());
        let mut shallow_counters = StrategyCounters::default();
        let mut shallow_fills = Vec::new();
        apply_taker_order(
            &event,
            &shallow,
            &mut shallow_cash,
            &mut shallow_yes,
            &mut shallow_no,
            &mut shallow_portfolio,
            &mut shallow_counters,
            &mut shallow_fills,
            0.0,
            0.0,
            None,
        );

        let mut deep_cash = 100.0;
        let mut deep_yes = 0.0;
        let mut deep_no = 0.0;
        let mut deep_portfolio = PortfolioState::new(100.0, limits);
        let mut deep_counters = StrategyCounters::default();
        let mut deep_fills = Vec::new();
        apply_taker_order(
            &event,
            &deep,
            &mut deep_cash,
            &mut deep_yes,
            &mut deep_no,
            &mut deep_portfolio,
            &mut deep_counters,
            &mut deep_fills,
            0.0,
            0.0,
            None,
        );

        assert_eq!(shallow_fills.len(), 1);
        assert_eq!(deep_fills.len(), 1);
        assert_eq!(shallow_fills[0].shares, 5.0);
        assert_eq!(deep_fills[0].shares, 12.0);
        assert!((shallow_fills[0].price - 0.50).abs() < 1e-6);
        assert!((deep_fills[0].price - 0.5375).abs() < 1e-5);
    }

    #[test]
    fn taker_buy_respects_side_native_limit_price() {
        let mut event = evt(0, 0.49, 0.90, 5.0);
        event.asks[1] = BookLevel {
            price: 0.95,
            size: 10.0,
        };
        let capped = OrderRequest {
            side: Side::BuyYes,
            shares: 12.0,
            max_depth: 2,
            limit_price: Some(0.93),
            tag: "capped",
        };

        let mut cash = 100.0;
        let mut yes = 0.0;
        let mut no = 0.0;
        let mut portfolio = PortfolioState::new(100.0, PortfolioLimits::default());
        let mut counters = StrategyCounters::default();
        let mut fills = Vec::new();
        apply_taker_order(
            &event,
            &capped,
            &mut cash,
            &mut yes,
            &mut no,
            &mut portfolio,
            &mut counters,
            &mut fills,
            0.0,
            0.0,
            None,
        );

        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].shares, 5.0);
        assert!((fills[0].price - 0.91).abs() < 1e-6);
    }

    #[test]
    fn taker_fill_records_model_context_and_side_edge() {
        let event = evt(1_000_000_000, 0.79, 0.80, 10.0);
        let req = OrderRequest {
            side: Side::BuyYes,
            shares: 2.0,
            max_depth: 1,
            limit_price: None,
            tag: "ctx",
        };
        let model = ModelOutput {
            direction_score: 0.5,
            confidence_score: 0.72,
            calibrated_p: 0.88,
            risk_score: 0.20,
        };
        let context = FillModelContext::from_event(
            &event,
            &model,
            req.side,
            1.0,
            301_000_000_000,
            WhipsawRiskSnapshot::default(),
        );

        let mut cash = 100.0;
        let mut yes = 0.0;
        let mut no = 0.0;
        let mut portfolio = PortfolioState::new(100.0, PortfolioLimits::default());
        let mut counters = StrategyCounters::default();
        let mut fills = Vec::new();
        apply_taker_order(
            &event,
            &req,
            &mut cash,
            &mut yes,
            &mut no,
            &mut portfolio,
            &mut counters,
            &mut fills,
            0.0,
            0.0,
            Some(context),
        );

        let fill = fills.first().expect("expected fill");
        assert_eq!(fill.side_model_p, Some(0.88));
        assert_eq!(fill.confidence_score, Some(0.72));
        assert_eq!(fill.risk_score, Some(0.20));
        assert_eq!(fill.seconds_since_open, Some(1.0));
        assert_eq!(fill.seconds_to_close, Some(300.0));
        assert!((fill.side_edge_vs_mid.unwrap() - (0.88 - event.yes_mid)).abs() < 1e-6);
        assert!((fill.side_edge_vs_fill.unwrap() - 0.08).abs() < 1e-6);
    }

    #[test]
    fn taker_fill_records_predicted_no_side_probability() {
        let event = evt(1_000_000_000, 0.19, 0.20, 10.0);
        let req = OrderRequest {
            side: Side::BuyNo,
            shares: 2.0,
            max_depth: 1,
            limit_price: None,
            tag: "ctx_no",
        };
        let model = ModelOutput {
            direction_score: -0.5,
            confidence_score: 0.82,
            calibrated_p: 0.91,
            risk_score: 0.18,
        };
        let context = FillModelContext::from_event(
            &event,
            &model,
            req.side,
            1.0,
            301_000_000_000,
            WhipsawRiskSnapshot::default(),
        );

        let mut cash = 100.0;
        let mut yes = 0.0;
        let mut no = 0.0;
        let mut portfolio = PortfolioState::new(100.0, PortfolioLimits::default());
        let mut counters = StrategyCounters::default();
        let mut fills = Vec::new();
        apply_taker_order(
            &event,
            &req,
            &mut cash,
            &mut yes,
            &mut no,
            &mut portfolio,
            &mut counters,
            &mut fills,
            0.0,
            0.0,
            Some(context),
        );

        let fill = fills.first().expect("expected fill");
        assert_eq!(fill.side_model_p, Some(0.91));
        assert_eq!(fill.calibrated_p, Some(0.91));
        assert!((fill.price - 0.81).abs() < 1e-6);
        assert!((fill.side_edge_vs_mid.unwrap() - (0.91 - (1.0 - event.yes_mid))).abs() < 1e-6);
        assert!((fill.side_edge_vs_fill.unwrap() - 0.10).abs() < 1e-6);
    }

    #[test]
    fn taker_buy_yes_takes_full_loss_when_no_wins() {
        let events = vec![
            evt(0, 0.50, 0.51, 200.0),
            evt(1_000_000_000, 0.30, 0.31, 200.0),
            evt(2_000_000_000, 0.02, 0.03, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            resolved_yes: Some(false),
            portfolio_limits: PortfolioLimits {
                max_clip_usdc: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut strat = BuyYesAtOpen::new(10.0);
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();
        assert_eq!(rep.counters.orders_filled_taker, 1);
        assert_eq!(rep.fills.len(), 1);
        assert!(
            (rep.pnl_usdc - -5.1).abs() < 1e-6,
            "pnl was {}",
            rep.pnl_usdc
        );
    }

    #[test]
    fn runner_skips_stale_pre_open_snapshots() {
        let events = vec![
            evt(0, 0.10, 0.11, 200.0),
            evt(300_000_000_000, 0.50, 0.51, 200.0),
            evt(301_000_000_000, 0.52, 0.53, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_open_ns: 300_000_000_000,
            market_close_ns: 600_000_000_000,
            resolved_yes: Some(true),
            portfolio_limits: PortfolioLimits {
                max_clip_usdc: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut strat = BuyYesAtOpen::new(10.0);
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();

        assert_eq!(rep.counters.orders_filled_taker, 1);
        let fill = rep.fills.first().expect("expected fill");
        assert_eq!(fill.ts_ns, 300_000_000_000);
        assert!((fill.price - 0.51).abs() < 1e-6);
        assert_eq!(fill.seconds_since_open, Some(0.0));
        assert_eq!(fill.seconds_to_close, Some(300.0));
    }

    #[test]
    fn maker_buy_yes_fills_when_book_crosses_down() {
        // Strategy submits a single resting BUY YES at 0.45 at t=0; the ask
        // drops to 0.45 at t=1s; we expect a maker fill at 0.45.
        struct OneShot;
        impl Strategy for OneShot {
            fn on_event(
                &mut self,
                _e: &ReplayEvent,
                ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &TradeHistory,
            ) -> StrategyOutput {
                if ctx.events_seen > 1 {
                    return StrategyOutput::hold();
                }
                StrategyOutput::one(OrderRequest {
                    side: Side::BuyYes,
                    shares: 10.0,
                    max_depth: 1,
                    limit_price: Some(0.45),
                    tag: "test_maker_buy",
                })
            }
        }
        let events = vec![
            evt(0, 0.50, 0.51, 200.0),           // submission tick: ask=0.51, no cross
            evt(500_000_000, 0.46, 0.47, 200.0), // ask=0.47, still no cross
            evt(1_000_000_000, 0.44, 0.45, 200.0), // ask=0.45, cross!
            evt(2_000_000_000, 0.30, 0.31, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            resolved_yes: Some(true),
            portfolio_limits: PortfolioLimits {
                max_clip_usdc: 10.0,
                ..Default::default()
            },
            maker_rebate_bps: 10.0,
            ..Default::default()
        };
        let mut s = OneShot;
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut s,
            &cfg,
        )
        .unwrap();
        assert_eq!(
            rep.counters.orders_filled_maker, 1,
            "expected one maker fill"
        );
        // 10 sh @ 0.45 = 4.50 notional; rebate 10bp = 0.0045; YES wins → +10.
        // Net: -4.50 + 10.00 + 0.0045 = +5.5045
        assert!((rep.pnl_usdc - 5.5045).abs() < 1e-6, "pnl {}", rep.pnl_usdc);
        assert!((rep.maker_rebates_usdc - 0.0045).abs() < 1e-9);
    }

    #[test]
    fn maker_buy_yes_fills_from_trade_print_without_book_cross() {
        struct OneShot;
        impl Strategy for OneShot {
            fn on_event(
                &mut self,
                _e: &ReplayEvent,
                ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &TradeHistory,
            ) -> StrategyOutput {
                if ctx.events_seen > 1 {
                    return StrategyOutput::hold();
                }
                StrategyOutput::one(OrderRequest {
                    side: Side::BuyYes,
                    shares: 5.0,
                    max_depth: 1,
                    limit_price: Some(0.45),
                    tag: "test_trade_maker_buy",
                })
            }
        }
        let events = vec![
            evt(0, 0.44, 0.51, 200.0),
            evt(2_000_000_000, 0.44, 0.51, 200.0),
        ];
        let trades = pm_types::TradeHistory::new(vec![pm_types::TradeTick {
            ts_ns: 1_000_000_000,
            price: 0.45,
            size: 5.0,
            aggressor_buy: false,
        }]);
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            resolved_yes: Some(true),
            maker_rebate_bps: 10.0,
            portfolio_limits: PortfolioLimits {
                max_clip_usdc: 10.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut s = OneShot;
        let rep = run_backtest(&events, &SpotHistory::default(), &trades, &mut s, &cfg).unwrap();
        assert_eq!(rep.counters.orders_filled_maker, 1);
        assert_eq!(rep.fills[0].ts_ns, 1_000_000_000);
        assert_eq!(rep.fills[0].tag, "test_trade_maker_buy");
        assert!((rep.fills[0].price - 0.45).abs() < 1e-6);
    }

    #[test]
    fn model_gate_blocks_low_confidence_orders() {
        struct LowConfidenceShot;
        impl Strategy for LowConfidenceShot {
            fn on_event(
                &mut self,
                _event: &ReplayEvent,
                _ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &TradeHistory,
            ) -> StrategyOutput {
                StrategyOutput::one(OrderRequest {
                    side: Side::BuyYes,
                    shares: 10.0,
                    max_depth: 1,
                    limit_price: None,
                    tag: "blocked_order",
                })
            }

            fn on_event_scored(
                &mut self,
                _event: &ReplayEvent,
                _ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &TradeHistory,
            ) -> (StrategyOutput, Option<ModelOutput>) {
                (
                    StrategyOutput::one(OrderRequest {
                        side: Side::BuyYes,
                        shares: 10.0,
                        max_depth: 1,
                        limit_price: None,
                        tag: "blocked_order",
                    }),
                    Some(ModelOutput {
                        direction_score: 0.20,
                        confidence_score: 0.20,
                        calibrated_p: 0.55,
                        risk_score: 0.95,
                    }),
                )
            }
        }

        let events = vec![evt(0, 0.49, 0.51, 200.0)];
        let cfg = RunnerConfig {
            enforce_model_gate: true,
            ..Default::default()
        };
        let mut strat = LowConfidenceShot;
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();
        assert_eq!(rep.counters.orders_submitted, 0);
        assert_eq!(rep.counters.orders_rejected_model_gate, 1);
        assert_eq!(
            rep.counters.orders_filled_taker + rep.counters.orders_filled_maker,
            0
        );
    }

    #[test]
    fn limit_above_ask_becomes_taker() {
        // Strategy submits a "limit" BUY YES at 0.99 (well above ask=0.51).
        // Should be treated as a taker fill at the actual ask.
        struct OneShot;
        impl Strategy for OneShot {
            fn on_event(
                &mut self,
                _e: &ReplayEvent,
                ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &TradeHistory,
            ) -> StrategyOutput {
                if ctx.events_seen > 1 {
                    return StrategyOutput::hold();
                }
                StrategyOutput::one(OrderRequest {
                    side: Side::BuyYes,
                    shares: 10.0,
                    max_depth: 1,
                    limit_price: Some(0.99),
                    tag: "test_aggressive_limit",
                })
            }
        }
        let events = vec![
            evt(0, 0.50, 0.51, 200.0),
            evt(1_000_000_000, 0.50, 0.51, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            resolved_yes: Some(false),
            portfolio_limits: PortfolioLimits {
                max_clip_usdc: 20.0,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut s = OneShot;
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut s,
            &cfg,
        )
        .unwrap();
        assert_eq!(rep.counters.orders_filled_taker, 1);
        assert_eq!(rep.counters.orders_filled_maker, 0);
    }

    #[derive(Default)]
    struct ResolutionProbe {
        seen: bool,
        last_mid: f32,
        last_result: bool,
    }

    impl Strategy for ResolutionProbe {
        fn on_event(
            &mut self,
            _event: &ReplayEvent,
            _ctx: &Ctx,
            _spot: &SpotHistory,
            _trades: &TradeHistory,
        ) -> StrategyOutput {
            StrategyOutput::hold()
        }

        fn on_market_resolved(&mut self, market_mid: f32, resolved_yes: bool) {
            self.seen = true;
            self.last_mid = market_mid;
            self.last_result = resolved_yes;
        }
    }

    #[test]
    fn run_backtest_calls_market_resolution_hook() {
        let events = vec![
            evt(0, 0.50, 0.51, 200.0),
            evt(1_000_000_000, 0.52, 0.53, 200.0),
            evt(2_000_000_000, 0.48, 0.49, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 3_000_000_000,
            resolved_yes: Some(true),
            portfolio_limits: PortfolioLimits::default(),
            equity_curve_jsonl: None,
            snapshot_every_n: 16,
            ..Default::default()
        };
        let mut probe = ResolutionProbe::default();
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut probe,
            &cfg,
        )
        .unwrap();
        assert!(probe.seen, "expected on_market_resolved callback");
        assert_eq!(probe.last_mid, rep.last_yes_mid);
        assert!((probe.last_mid - 0.485).abs() < 1e-6);
        assert!(probe.last_result);
    }

    #[test]
    fn run_backtest_stops_at_market_close() {
        let events = vec![
            evt(0, 0.20, 0.21, 200.0),
            evt(1_000_000_000, 0.80, 0.82, 200.0), // on-close tick
            evt(2_000_000_000, 0.10, 0.11, 200.0), // after close, must ignore
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 1_000_000_000,
            resolved_yes: None,
            portfolio_limits: PortfolioLimits::default(),
            ..Default::default()
        };
        let mut strat = BuyYesAtOpen::new(10.0);
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();
        assert!((rep.last_yes_mid - 0.81).abs() < 1e-6);
        assert!(rep.yes_resolved);
    }

    #[derive(Default)]
    struct DecisionProbe;

    impl Strategy for DecisionProbe {
        fn on_event(
            &mut self,
            event: &ReplayEvent,
            _ctx: &Ctx,
            _spot: &SpotHistory,
            _trades: &pm_types::TradeHistory,
        ) -> pm_strategy::StrategyOutput {
            let _ = event;
            pm_strategy::StrategyOutput::hold()
        }

        fn on_event_scored(
            &mut self,
            event: &ReplayEvent,
            _ctx: &Ctx,
            _spot: &SpotHistory,
            _trades: &pm_types::TradeHistory,
        ) -> (StrategyOutput, Option<ModelOutput>) {
            let score = ModelOutput {
                direction_score: 0.45,
                confidence_score: 0.80,
                calibrated_p: 0.74,
                risk_score: 0.22,
            };
            let _ = event;
            (StrategyOutput::hold(), Some(score))
        }
    }

    #[test]
    fn decision_log_includes_model_scores_and_edge() {
        let events = vec![evt(0, 0.50, 0.52, 200.0)];
        let log_path = std::env::temp_dir().join("pm_app_decision_log_row_test.jsonl");
        let _ = std::fs::remove_file(&log_path);

        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 5_000_000_000,
            portfolio_limits: PortfolioLimits::default(),
            decision_log_jsonl: Some(log_path.clone()),
            decision_log_every_n: 1,
            ..Default::default()
        };

        let mut strat = DecisionProbe;
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();
        assert_eq!(rep.events_processed, 1);

        let log_txt = std::fs::read_to_string(&log_path).expect("decision log should exist");
        let row: DecisionLogRow =
            serde_json::from_str(log_txt.lines().next().unwrap()).expect("row must be valid JSON");
        assert!(row.has_model_output);
        assert!(row.strategy_emitted_model_output);
        assert!(row.has_model_attribution);
        assert!((row.direction_score - 0.45).abs() < 1e-6);
        assert!((row.confidence_score - 0.80).abs() < 1e-6);
        assert!((row.calibrated_p - 0.74).abs() < 1e-6);
        assert!((row.risk_score - 0.22).abs() < 1e-6);
        assert!((row.edge - (0.74 - 0.51)).abs() < 1e-6);
        assert!(row.side_is_yes);
        assert_eq!(row.orders_requested, 0);
        assert_eq!(row.requested_shares, 0.0);
        assert_eq!(row.event_fills, 0);
        assert_eq!(row.event_fill_notional_usdc, 0.0);
        assert_eq!(row.event_slippage_bps, 0.0);
        assert_eq!(row.event_cash_delta_usdc, 0.0);
        assert_eq!(row.event_mtm_delta_usdc, 0.0);
        assert!((-1.0..=1.0).contains(&row.feature_book_imbalance_top3));
        assert!((0.0..=1.0).contains(&row.feature_stability));
        assert!((0.0..=1.0).contains(&row.feature_side_p_pre_meta));
        assert!((0.0..=1.0).contains(&row.feature_side_p_post_meta));
    }

    #[test]
    fn decision_log_uses_side_oriented_edge_for_no_side() {
        let events = vec![evt(0, 0.58, 0.60, 200.0)];
        let log_path = std::env::temp_dir().join("pm_app_decision_log_no_side_test.jsonl");
        let _ = std::fs::remove_file(&log_path);

        struct DecisionProbeNo;
        impl Strategy for DecisionProbeNo {
            fn on_event(
                &mut self,
                event: &ReplayEvent,
                _ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &pm_types::TradeHistory,
            ) -> pm_strategy::StrategyOutput {
                let _ = event;
                pm_strategy::StrategyOutput::hold()
            }
            fn on_event_scored(
                &mut self,
                event: &ReplayEvent,
                _ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &pm_types::TradeHistory,
            ) -> (StrategyOutput, Option<ModelOutput>) {
                let score = ModelOutput {
                    direction_score: -0.90,
                    confidence_score: 0.85,
                    calibrated_p: 0.69,
                    risk_score: 0.12,
                };
                let _ = event;
                (StrategyOutput::hold(), Some(score))
            }
        }

        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 5_000_000_000,
            portfolio_limits: PortfolioLimits::default(),
            decision_log_jsonl: Some(log_path.clone()),
            decision_log_every_n: 1,
            ..Default::default()
        };

        let mut strat = DecisionProbeNo;
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();
        assert_eq!(rep.events_processed, 1);

        let log_txt = std::fs::read_to_string(&log_path).expect("decision log should exist");
        let row: DecisionLogRow =
            serde_json::from_str(log_txt.lines().next().unwrap()).expect("row must be valid JSON");
        assert!(row.has_model_output);
        assert!(row.strategy_emitted_model_output);
        assert!(!row.side_is_yes);
        let expected_side_edge = 0.69 - (1.0 - 0.59);
        assert!((row.edge - expected_side_edge).abs() < 1e-6);
    }

    #[test]
    fn decision_log_tracks_fill_slippage_and_event_pnl() {
        struct FillShot;

        impl Strategy for FillShot {
            fn on_event(
                &mut self,
                _event: &ReplayEvent,
                _ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &pm_types::TradeHistory,
            ) -> StrategyOutput {
                StrategyOutput::one(OrderRequest {
                    side: Side::BuyYes,
                    shares: 10.0,
                    max_depth: 1,
                    limit_price: None,
                    tag: "fill-shot",
                })
            }

            fn on_event_scored(
                &mut self,
                _event: &ReplayEvent,
                _ctx: &Ctx,
                _spot: &SpotHistory,
                _trades: &pm_types::TradeHistory,
            ) -> (StrategyOutput, Option<ModelOutput>) {
                (
                    StrategyOutput::one(OrderRequest {
                        side: Side::BuyYes,
                        shares: 10.0,
                        max_depth: 1,
                        limit_price: None,
                        tag: "fill-shot",
                    }),
                    Some(ModelOutput {
                        direction_score: 1.0,
                        confidence_score: 1.0,
                        calibrated_p: 0.94,
                        risk_score: 0.0,
                    }),
                )
            }
        }

        let events = vec![evt(0, 0.49, 0.51, 200.0)];
        let log_path = std::env::temp_dir().join("pm_app_decision_log_fill_attrib_test.jsonl");
        let _ = std::fs::remove_file(&log_path);

        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 5_000_000_000,
            portfolio_limits: PortfolioLimits {
                max_clip_usdc: 20.0,
                ..PortfolioLimits::default()
            },
            decision_log_jsonl: Some(log_path.clone()),
            decision_log_every_n: 1,
            taker_slippage_bps: 20.0,
            ..Default::default()
        };

        let mut strat = FillShot;
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();
        assert_eq!(rep.counters.orders_filled_taker, 1);

        let log_txt = std::fs::read_to_string(&log_path).expect("decision log should exist");
        let row: DecisionLogRow =
            serde_json::from_str(log_txt.lines().next().unwrap()).expect("row must be valid JSON");

        assert_eq!(row.event_fills, 1);
        assert!(row.event_fill_notional_usdc > 5.0);
        assert!(row.event_slippage_bps > 0.0);
        assert!(row.event_cash_delta_usdc < 0.0);
        assert!(row.event_cash_delta_usdc > -100.0);
        assert!((row.event_cash_delta_usdc + row.event_fill_notional_usdc).abs() < 1e-6);
        assert!(row.event_mtm_delta_usdc != 0.0);
    }

    #[test]
    fn default_strategy_scorer_emits_model_fields() {
        let events = vec![evt(0, 0.50, 0.52, 200.0)];
        let log_path = std::env::temp_dir().join("pm_app_default_model_fields_test.jsonl");
        let _ = std::fs::remove_file(&log_path);

        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 5_000_000_000,
            portfolio_limits: PortfolioLimits::default(),
            decision_log_jsonl: Some(log_path.clone()),
            decision_log_every_n: 1,
            ..Default::default()
        };

        let mut strat = BuyYesAtOpen::new(10.0);
        let spot = SpotHistory::default();
        let rep = run_backtest(
            &events,
            &spot,
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();
        assert_eq!(rep.events_processed, 1);

        let log_txt = std::fs::read_to_string(&log_path).expect("decision log should exist");
        let row: DecisionLogRow =
            serde_json::from_str(log_txt.lines().next().unwrap()).expect("row must be valid JSON");
        assert!(row.has_model_output);
        assert!(!row.strategy_emitted_model_output);
        assert!(row.has_model_attribution);
        assert!(row.direction_score >= -1.0 && row.direction_score <= 1.0);
        assert!((0.0..=1.0).contains(&row.confidence_score));
        assert!((0.55..=0.94).contains(&row.calibrated_p));
        assert!((0.0..=1.0).contains(&row.risk_score));
        assert!((-1.0..=1.0).contains(&row.edge));
        assert_eq!(row.side_is_yes, row.direction_score >= 0.0);
        assert!((-1.0..=1.0).contains(&row.feature_momentum));
        assert!((-1.0..=1.0).contains(&row.feature_microprice_dev));
        assert!((-1.0..=1.0).contains(&row.feature_spot_score));
        assert!((-1.0..=1.0).contains(&row.feature_direction_raw));
        assert!((0.0..=1.0).contains(&row.feature_markov_persistence));
        assert!((0.0..=1.0).contains(&row.feature_liquidity));
        assert!((0.0..=1.0).contains(&row.feature_path_risk));
        assert!((0.0..=1.0).contains(&row.feature_volatility_regime));
        assert_eq!(row.meta_calibrator_updates, 0);
    }

    #[test]
    fn run_backtest_emits_labeled_model_training_sample() {
        let events = vec![
            evt(0, 0.50, 0.52, 200.0),
            evt(1_000_000_000, 0.54, 0.56, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 5_000_000_000,
            resolved_yes: Some(true),
            portfolio_limits: PortfolioLimits::default(),
            ..Default::default()
        };

        let mut strat = BuyYesAtOpen::new(0.0);
        let rep = run_backtest(
            &events,
            &SpotHistory::default(),
            &pm_types::TradeHistory::default(),
            &mut strat,
            &cfg,
        )
        .unwrap();

        assert!(!rep.model_training_samples.is_empty());
        let sample = rep.model_training_samples[0];
        assert!((0.0..=1.0).contains(&sample.base_side_probability));
        assert_eq!(sample.side_observed, rep.yes_resolved);
    }

    #[test]
    fn run_backtest_passes_trade_history_to_strategy() {
        struct TradeAwareStrategy {
            seen_trade_rows: usize,
        }

        impl Strategy for TradeAwareStrategy {
            fn on_event(
                &mut self,
                _event: &ReplayEvent,
                _ctx: &Ctx,
                _spot: &SpotHistory,
                trades: &pm_types::TradeHistory,
            ) -> StrategyOutput {
                self.seen_trade_rows = trades.len();
                StrategyOutput::hold()
            }
        }

        let trades = pm_types::TradeHistory::new(vec![pm_types::TradeTick {
            ts_ns: 0,
            price: 0.52,
            size: 12.0,
            aggressor_buy: true,
        }]);

        let events = vec![evt(0, 0.50, 0.52, 200.0)];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            market_close_ns: 5_000_000_000,
            portfolio_limits: PortfolioLimits::default(),
            ..Default::default()
        };

        let mut strat = TradeAwareStrategy { seen_trade_rows: 0 };
        let rep =
            run_backtest(&events, &SpotHistory::default(), &trades, &mut strat, &cfg).unwrap();
        assert_eq!(rep.events_processed, 1);
        assert_eq!(strat.seen_trade_rows, trades.len());
        assert_eq!(strat.seen_trade_rows, 1);
    }
}
