//! BonereaperDirectional v2.1 — directional pyramid + late directional +
//! high-skew load + convex tail. Taker-only until cancel infra lands.
//!
//! Failure-mode notes shaping this design:
//!
//!   * **Maker stranding** (legacy polymarket-exec bug) — posting maker bids
//!     near mid invites adverse selection. We stay taker-only until the
//!     runner gains cancel-on-adverse-move infrastructure.
//!
//!   * **Whipsaw at high yes_mid** — naively treating yes_mid ≥ 0.80 as
//!     committed gets crushed in high-vol regimes. The high-skew lane has
//!     three guards: spot agrees, skew sustained ≥5s, regime != Whipsaw.
//!
//! Lanes:
//!
//!   1. **Early directional probe** (0–30s) — small clip on composite signal,
//!      establishes a position before the pile-on.
//!   2. **Mid-ladder** (30–240s) — pyramids up as the book moves further
//!      in our direction. Capped at N rungs × min-step price movement.
//!   3. **Late directional** (240–300s) — permissive directional taker.
//!      Workhorse lane — high fill rate, modest edge per fill.
//!   4. **High-skew load** (any phase) — once yes_mid ≥ 0.80 (or ≤ 0.20)
//!      AND whipsaw guards pass, layers multiple clips at high prices.
//!      Matches real Bonereaper's late favourite_load behaviour.
//!   5. **Convex tail** — small one-shot cheap bet on the losing side.

use crate::regime::{BtcRegime, BtcRegimeSnapshot, WhipsawRiskSnapshot};
use crate::signals::{Ring, direction_score};
use crate::spot_momentum::spot_momentum_stack;
use crate::{Ctx, OrderRequest, Side, Strategy, StrategyOutput};
use pm_types::{ReplayEvent, SpotHistory, TradeHistory, compute_trade_flow};
use serde::{Deserialize, Serialize};

const BETTING_WINDOW_SECS: i64 = 300;
const MOM_WINDOW: usize = 32;
const MICRO_DEV_SCALE: f32 = 0.6;
const TRADE_FLOW_LOOKBACK_NS: i64 = 60 * 1_000_000_000;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
#[serde(default)]
pub struct BonereaperV2GateStats {
    pub late_confirm_checks: u64,
    pub late_confirm_price_fail: u64,
    pub late_confirm_book_favourite_fail: u64,
    pub late_confirm_book_skew_fail: u64,
    pub late_confirm_model_missing: u64,
    pub late_confirm_model_side_fail: u64,
    pub late_confirm_model_confidence_fail: u64,
    pub late_confirm_model_risk_fail: u64,
    pub late_confirm_model_side_p_fail: u64,
    pub late_confirm_model_edge_fail: u64,
    pub late_confirm_whipsaw_fail: u64,
    pub late_confirm_shares_fail: u64,
    pub late_confirm_emits: u64,

    pub high_skew_checks: u64,
    pub high_skew_regime_fail: u64,
    pub high_skew_threshold_fail: u64,
    pub high_skew_sustain_fail: u64,
    pub high_skew_spot_alignment_fail: u64,
    pub high_skew_price_fail: u64,
    pub high_skew_model_missing: u64,
    pub high_skew_model_side_fail: u64,
    pub high_skew_model_confidence_fail: u64,
    pub high_skew_model_risk_fail: u64,
    pub high_skew_model_side_p_fail: u64,
    pub high_skew_model_edge_fail: u64,
    pub high_skew_whipsaw_fail: u64,
    pub high_skew_shares_fail: u64,
    pub high_skew_emits: u64,

    pub late_favourite_window_checks: u64,
    pub late_favourite_capacity_fail: u64,
    pub late_favourite_refresh_fail: u64,
    pub late_favourite_checks: u64,
    pub late_favourite_skew_fail: u64,
    pub late_favourite_sustain_fail: u64,
    pub late_favourite_alignment_fail: u64,
    pub late_favourite_price_fail: u64,
    pub late_favourite_model_missing: u64,
    pub late_favourite_model_side_fail: u64,
    pub late_favourite_model_confidence_fail: u64,
    pub late_favourite_model_risk_fail: u64,
    pub late_favourite_model_side_p_fail: u64,
    pub late_favourite_model_edge_fail: u64,
    pub late_favourite_model_direction_fail: u64,
    pub late_favourite_whipsaw_fail: u64,
    pub late_favourite_reversal_pressure_fail: u64,
    pub late_favourite_path_efficiency_fail: u64,
    pub late_favourite_market_range_fail: u64,
    pub late_favourite_adverse_momentum_fail: u64,
    pub late_favourite_entry_pullback_fail: u64,
    pub late_favourite_avg_entry_drawdown_fail: u64,
    pub late_favourite_shares_fail: u64,
    pub late_favourite_emits: u64,
}

impl BonereaperV2GateStats {
    pub fn add_assign(&mut self, other: Self) {
        self.late_confirm_checks += other.late_confirm_checks;
        self.late_confirm_price_fail += other.late_confirm_price_fail;
        self.late_confirm_book_favourite_fail += other.late_confirm_book_favourite_fail;
        self.late_confirm_book_skew_fail += other.late_confirm_book_skew_fail;
        self.late_confirm_model_missing += other.late_confirm_model_missing;
        self.late_confirm_model_side_fail += other.late_confirm_model_side_fail;
        self.late_confirm_model_confidence_fail += other.late_confirm_model_confidence_fail;
        self.late_confirm_model_risk_fail += other.late_confirm_model_risk_fail;
        self.late_confirm_model_side_p_fail += other.late_confirm_model_side_p_fail;
        self.late_confirm_model_edge_fail += other.late_confirm_model_edge_fail;
        self.late_confirm_whipsaw_fail += other.late_confirm_whipsaw_fail;
        self.late_confirm_shares_fail += other.late_confirm_shares_fail;
        self.late_confirm_emits += other.late_confirm_emits;

        self.high_skew_checks += other.high_skew_checks;
        self.high_skew_regime_fail += other.high_skew_regime_fail;
        self.high_skew_threshold_fail += other.high_skew_threshold_fail;
        self.high_skew_sustain_fail += other.high_skew_sustain_fail;
        self.high_skew_spot_alignment_fail += other.high_skew_spot_alignment_fail;
        self.high_skew_price_fail += other.high_skew_price_fail;
        self.high_skew_model_missing += other.high_skew_model_missing;
        self.high_skew_model_side_fail += other.high_skew_model_side_fail;
        self.high_skew_model_confidence_fail += other.high_skew_model_confidence_fail;
        self.high_skew_model_risk_fail += other.high_skew_model_risk_fail;
        self.high_skew_model_side_p_fail += other.high_skew_model_side_p_fail;
        self.high_skew_model_edge_fail += other.high_skew_model_edge_fail;
        self.high_skew_whipsaw_fail += other.high_skew_whipsaw_fail;
        self.high_skew_shares_fail += other.high_skew_shares_fail;
        self.high_skew_emits += other.high_skew_emits;

        self.late_favourite_window_checks += other.late_favourite_window_checks;
        self.late_favourite_capacity_fail += other.late_favourite_capacity_fail;
        self.late_favourite_refresh_fail += other.late_favourite_refresh_fail;
        self.late_favourite_checks += other.late_favourite_checks;
        self.late_favourite_skew_fail += other.late_favourite_skew_fail;
        self.late_favourite_sustain_fail += other.late_favourite_sustain_fail;
        self.late_favourite_alignment_fail += other.late_favourite_alignment_fail;
        self.late_favourite_price_fail += other.late_favourite_price_fail;
        self.late_favourite_model_missing += other.late_favourite_model_missing;
        self.late_favourite_model_side_fail += other.late_favourite_model_side_fail;
        self.late_favourite_model_confidence_fail += other.late_favourite_model_confidence_fail;
        self.late_favourite_model_risk_fail += other.late_favourite_model_risk_fail;
        self.late_favourite_model_side_p_fail += other.late_favourite_model_side_p_fail;
        self.late_favourite_model_edge_fail += other.late_favourite_model_edge_fail;
        self.late_favourite_model_direction_fail += other.late_favourite_model_direction_fail;
        self.late_favourite_whipsaw_fail += other.late_favourite_whipsaw_fail;
        self.late_favourite_reversal_pressure_fail += other.late_favourite_reversal_pressure_fail;
        self.late_favourite_path_efficiency_fail += other.late_favourite_path_efficiency_fail;
        self.late_favourite_market_range_fail += other.late_favourite_market_range_fail;
        self.late_favourite_adverse_momentum_fail += other.late_favourite_adverse_momentum_fail;
        self.late_favourite_entry_pullback_fail += other.late_favourite_entry_pullback_fail;
        self.late_favourite_avg_entry_drawdown_fail += other.late_favourite_avg_entry_drawdown_fail;
        self.late_favourite_shares_fail += other.late_favourite_shares_fail;
        self.late_favourite_emits += other.late_favourite_emits;
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BonereaperV2Config {
    pub bankroll_usdc: f64,
    pub max_clip_usdc: f64,
    pub tick: f64,
    pub disable_internal_model_gates: bool,

    pub early_phase_end_secs: f32,
    pub mid_phase_end_secs: f32,

    pub min_composite_direction: f32,
    pub mid_ladder_min_step: f32,
    pub mid_ladder_max_rungs: usize,
    pub early_clip_frac: f32,
    pub mid_clip_frac: f32,
    pub late_clip_frac: f32,
    pub late_max_fires: usize,
    pub late_refresh_secs: f32,
    pub late_sweep_depth: usize,
    pub late_confirm_min_model_confidence: f32,
    pub late_confirm_max_model_risk: f32,
    pub late_confirm_min_model_side_p: f32,
    pub late_confirm_min_model_edge: f32,
    pub late_confirm_min_book_skew: f32,
    pub late_confirm_max_whipsaw_score: f32,

    // High-skew load lane with whipsaw guards
    pub high_skew_threshold: f32,
    pub high_skew_max_ask: f32,
    pub high_skew_clip_frac: f32,
    pub high_skew_max_clips: usize,
    pub high_skew_refresh_secs: f32,
    pub high_skew_sweep_depth: usize,
    pub high_skew_min_sustain_secs: f32,
    pub high_skew_min_spot_alignment: f32,
    pub high_skew_skip_whipsaw: bool,
    pub high_skew_max_whipsaw_score: f32,

    // Late favourite loading: heavier than generic high-skew, but only after
    // the market has a clear favourite and book/spot direction agree.
    pub late_favourite_start_secs: f32,
    pub late_favourite_threshold: f32,
    pub late_favourite_min_ask: f32,
    pub late_favourite_high_cert_ask: f32,
    pub late_favourite_max_ask: f32,
    pub late_favourite_clip_frac: f32,
    pub late_favourite_high_cert_clip_frac: f32,
    pub late_favourite_high_cert_full_clip_edge: f32,
    pub late_favourite_max_clips: usize,
    pub late_favourite_refresh_secs: f32,
    pub late_favourite_min_sustain_secs: f32,
    pub late_favourite_sweep_depth: usize,
    pub late_favourite_min_composite_alignment: f32,
    pub late_favourite_min_model_confidence: f32,
    pub late_favourite_min_model_direction_abs: f32,
    pub late_favourite_max_model_risk: f32,
    pub late_favourite_min_model_side_p: f32,
    pub late_favourite_min_model_edge: f32,
    pub late_favourite_high_cert_min_model_edge: f32,
    /// High-cert favourites are a par-discount trade, not a raw calibrated-p
    /// edge trade. When enabled, still require model side/confidence/risk, but
    /// do not require calibrated_p >= entry_px for asks above the high-cert
    /// threshold.
    pub late_favourite_high_cert_bypass_model_edge: bool,
    pub late_favourite_max_whipsaw_score: f32,
    pub late_favourite_max_reversal_pressure: f32,
    pub late_favourite_min_path_efficiency: f32,
    pub late_favourite_max_observed_range: f32,
    /// Start scaling down late-favourite size once the live market has already
    /// traversed this much YES-mid range. Disabled when >= hard range.
    pub late_favourite_range_soft_throttle: f32,
    /// Scale late-favourite size to zero at this live-observed YES-mid range.
    pub late_favourite_range_hard_throttle: f32,
    /// Extra edge required at/above hard range; interpolated from zero at the
    /// soft range. This is independent from the hard max-observed-range gate.
    pub late_favourite_range_extra_edge: f32,
    /// Extra confidence required at/above hard range; interpolated from zero at
    /// the soft range.
    pub late_favourite_range_extra_confidence: f32,
    pub late_favourite_max_adverse_fast_momentum: f32,
    pub late_favourite_max_entry_pullback: f32,
    pub late_favourite_max_avg_entry_drawdown: f32,

    // Convex tail ladder. Real Bonereaper buys the losing side in multiple
    // rungs as the book moves further away from the tail side; each rung is
    // roughly USD-constant so cheaper rungs get more shares. NOTE: our
    // matcher assumes 100% taker fill at yes_ask, which is optimistic in the
    // 0.02–0.05 zone where venue depth is thin. `tail_min_ask` enforces a
    // floor below which we don't pretend we can fill.
    pub paired_tail_clip_frac: f32,
    pub tail_clip_frac: f32,
    pub tail_extreme_threshold: f32,
    pub tail_min_skew_step: f32,
    pub tail_max_clips: usize,
    pub tail_refresh_secs: f32,
    pub tail_min_ask: f32,
    pub tail_max_ask: f32,
    /// Minimum live-observed YES-mid range required before buying convex tail.
    /// Useful for avoiding steady favourite markets where tail bleed is most
    /// likely and reserving spend for reversal-prone expanded-range regimes.
    pub tail_min_observed_range: f32,
    pub tail_target_favourite_loss_coverage_frac: f32,
    pub tail_budget_favourite_spend_frac: f32,
    pub tail_budget_favourite_upside_frac: f32,
}

impl Default for BonereaperV2Config {
    fn default() -> Self {
        Self {
            bankroll_usdc: 1000.0,
            max_clip_usdc: 5.0,
            tick: 0.01,
            disable_internal_model_gates: false,
            early_phase_end_secs: 30.0,
            mid_phase_end_secs: 240.0,
            min_composite_direction: 0.10,
            mid_ladder_min_step: 0.02,
            mid_ladder_max_rungs: 4,
            // early_dir and mid_ladder are both net-negative on PM 5m (pay
            // spread + slippage at coin-flip prices). Disabled via zero clip;
            // re-enable if a future signal stack adds enough edge to overcome
            // the spread.
            early_clip_frac: 0.00,
            mid_clip_frac: 0.00,
            late_clip_frac: 1.00,
            late_max_fires: 3,
            late_refresh_secs: 10.0,
            late_sweep_depth: 3,
            late_confirm_min_model_confidence: 0.58,
            late_confirm_max_model_risk: 0.80,
            late_confirm_min_model_side_p: 0.58,
            late_confirm_min_model_edge: 0.00,
            late_confirm_min_book_skew: 0.06,
            late_confirm_max_whipsaw_score: 0.85,
            // Favourite-loading lane. Keep this stricter than the old probe
            // defaults: the looser settings overtraded early skew and paid
            // taker spread before the market had a durable favourite.
            high_skew_threshold: 0.16, // yes_mid >= 0.66 or <= 0.34
            high_skew_max_ask: 0.95,
            high_skew_clip_frac: 0.60,
            high_skew_max_clips: 5,
            high_skew_refresh_secs: 4.0,
            high_skew_sweep_depth: 5,
            high_skew_min_sustain_secs: 12.0,
            high_skew_min_spot_alignment: 0.02,
            high_skew_skip_whipsaw: true,
            high_skew_max_whipsaw_score: 0.75,
            late_favourite_start_secs: 180.0,
            late_favourite_threshold: 0.22, // yes_mid >= 0.72 or <= 0.28
            late_favourite_min_ask: 0.70,
            late_favourite_high_cert_ask: 0.90,
            late_favourite_max_ask: 0.97,
            late_favourite_clip_frac: 1.00,
            late_favourite_high_cert_clip_frac: 1.00,
            late_favourite_high_cert_full_clip_edge: 0.00,
            late_favourite_max_clips: 12,
            late_favourite_refresh_secs: 4.0,
            late_favourite_min_sustain_secs: 0.0,
            late_favourite_sweep_depth: 7,
            late_favourite_min_composite_alignment: 0.05,
            late_favourite_min_model_confidence: 0.68,
            late_favourite_min_model_direction_abs: 0.0,
            late_favourite_max_model_risk: 0.72,
            late_favourite_min_model_side_p: 0.62,
            // Strict 5c edge makes 90c+ favourite loading impossible while
            // the model probability is capped at 94c. Keep the lane gated by
            // side probability/confidence/risk, and require a smaller positive
            // edge for the high-cert ladder.
            late_favourite_min_model_edge: 0.00,
            late_favourite_high_cert_min_model_edge: 0.00,
            late_favourite_high_cert_bypass_model_edge: false,
            late_favourite_max_whipsaw_score: 0.75,
            late_favourite_max_reversal_pressure: 1.0,
            late_favourite_min_path_efficiency: 0.0,
            // Disabled by default. Set below 1.0 to reject late favourite
            // loads in markets that already traversed too much YES-mid range.
            late_favourite_max_observed_range: 1.0,
            late_favourite_range_soft_throttle: 1.0,
            late_favourite_range_hard_throttle: 1.0,
            late_favourite_range_extra_edge: 0.0,
            late_favourite_range_extra_confidence: 0.0,
            // Disabled by default for backward-compatible sweeps. Set to a
            // small positive value (e.g. 0.04) to reject favourite loads when
            // the fast BTC impulse is actively moving against the favourite.
            late_favourite_max_adverse_fast_momentum: 1.0,
            // Disabled by default. Set to a small price distance (e.g. 0.015)
            // to stop adding to the same favourite after it rolls over from
            // the best prior entry price.
            late_favourite_max_entry_pullback: 1.0,
            // Disabled by default. Set to a small price distance (e.g. 0.01)
            // to stop adding while the current entry is below the same-side
            // average emitted late-favourite entry.
            late_favourite_max_avg_entry_drawdown: 1.0,
            // Tail ladder: cheap convex bets. Threshold raised to match the
            // skew level where a "cheap" side actually exists.
            // Paired late tails were negative in walk-forward attribution:
            // keep the standalone convex-tail ladder, disable the automatic
            // hedge emitted alongside every late confirmation.
            paired_tail_clip_frac: 0.00,
            tail_clip_frac: 0.10,
            tail_extreme_threshold: 0.30,
            tail_min_skew_step: 0.02,
            tail_max_clips: 3,
            tail_refresh_secs: 5.0,
            tail_min_ask: 0.01,
            tail_max_ask: 0.10,
            tail_min_observed_range: 0.0,
            tail_target_favourite_loss_coverage_frac: 0.0,
            tail_budget_favourite_spend_frac: 0.05,
            tail_budget_favourite_upside_frac: 0.25,
        }
    }
}

pub struct BonereaperV2 {
    cfg: BonereaperV2Config,
    recent_mids: Ring,
    early_emitted: bool,
    ladder_side: Option<Side>,
    last_ladder_mid: f32,
    mid_rungs: usize,
    late_fires: usize,
    last_late_ns: i64,
    high_skew_clips: usize,
    last_high_skew_ns: i64,
    late_favourite_clips: usize,
    last_late_favourite_ns: i64,
    late_favourite_side: Option<Side>,
    late_favourite_shares_emitted: f64,
    late_favourite_notional_emitted: f64,
    late_favourite_peak_entry_px: f64,
    late_favourite_side_shares_emitted: f64,
    late_favourite_side_notional_emitted: f64,
    skew_high_first_ns: Option<i64>,
    skew_low_first_ns: Option<i64>,
    tail_clips: usize,
    last_tail_skew_mag: f32,
    last_tail_ns: i64,
    tail_notional_emitted: f64,
    gate_stats: BonereaperV2GateStats,
}

impl BonereaperV2 {
    pub fn new(cfg: BonereaperV2Config) -> Self {
        Self {
            cfg,
            recent_mids: Ring::new(MOM_WINDOW),
            early_emitted: false,
            ladder_side: None,
            last_ladder_mid: 0.0,
            mid_rungs: 0,
            late_fires: 0,
            last_late_ns: i64::MIN / 2,
            high_skew_clips: 0,
            last_high_skew_ns: i64::MIN / 2,
            late_favourite_clips: 0,
            last_late_favourite_ns: i64::MIN / 2,
            late_favourite_side: None,
            late_favourite_shares_emitted: 0.0,
            late_favourite_notional_emitted: 0.0,
            late_favourite_peak_entry_px: 0.0,
            late_favourite_side_shares_emitted: 0.0,
            late_favourite_side_notional_emitted: 0.0,
            skew_high_first_ns: None,
            skew_low_first_ns: None,
            tail_clips: 0,
            last_tail_skew_mag: 0.0,
            last_tail_ns: i64::MIN / 2,
            tail_notional_emitted: 0.0,
            gate_stats: BonereaperV2GateStats::default(),
        }
    }

    pub fn gate_stats(&self) -> BonereaperV2GateStats {
        self.gate_stats
    }

    fn model_support_for_side(
        &self,
        ctx: &Ctx,
        side: Side,
        min_confidence: f32,
        max_risk: f32,
        min_side_p: f32,
        entry_px: f32,
        min_edge: f32,
    ) -> ModelSupport {
        if self.cfg.disable_internal_model_gates {
            ModelSupport::Supported
        } else {
            model_support_for_side(
                ctx,
                side,
                min_confidence,
                max_risk,
                min_side_p,
                entry_px,
                min_edge,
            )
        }
    }
}

fn shares_capped(usdc: f64, fill_px: f64) -> f64 {
    let raw = (usdc * 0.98) / fill_px;
    ((raw * 1000.0).floor() / 1000.0).max(0.0)
}

fn buy_px(event: &ReplayEvent, side: Side) -> f64 {
    match side {
        Side::BuyYes => event.yes_ask as f64,
        Side::BuyNo => (1.0 - event.yes_bid as f64).max(0.01),
        _ => 0.0,
    }
}

fn side_shares(ctx: &Ctx, side: Side) -> f64 {
    match side {
        Side::BuyYes => ctx.yes_shares,
        Side::BuyNo => ctx.no_shares,
        _ => 0.0,
    }
}

fn opposite_buy_side(side: Side) -> Option<Side> {
    match side {
        Side::BuyYes => Some(Side::BuyNo),
        Side::BuyNo => Some(Side::BuyYes),
        _ => None,
    }
}

fn late_favourite_ladder_levels(favourite_ask: f64, secs_in: f32) -> usize {
    let base = if favourite_ask < 0.75 {
        1
    } else if favourite_ask < 0.80 {
        2
    } else if favourite_ask < 0.90 {
        3
    } else {
        4
    };
    if favourite_ask >= 0.90 && secs_in >= BETTING_WINDOW_SECS as f32 - 120.0 {
        5
    } else {
        base
    }
}

fn late_favourite_high_cert_price_taper(favourite_ask: f64) -> f64 {
    if favourite_ask < 0.95 {
        return 1.0;
    }
    if favourite_ask >= 0.99 {
        return 0.18;
    }
    let progress = ((favourite_ask - 0.95) / 0.04).clamp(0.0, 1.0);
    1.0 - progress * 0.82
}

fn late_favourite_high_cert_edge_taper(edge: f32, min_edge: f32, full_clip_edge: f32) -> f64 {
    if full_clip_edge <= min_edge {
        return 1.0;
    }
    ((edge - min_edge) / (full_clip_edge - min_edge))
        .clamp(0.0, 1.0)
        .into()
}

fn late_favourite_high_cert_max_levels(favourite_ask: f64, base_levels: usize) -> usize {
    if favourite_ask >= 0.99 {
        base_levels.min(1)
    } else if favourite_ask >= 0.97 {
        base_levels.min(2)
    } else if favourite_ask >= 0.95 {
        base_levels.min(3)
    } else {
        base_levels
    }
}

fn tail_observed_range_allowed(range: f32, min_range: f32) -> bool {
    range >= min_range.clamp(0.0, 1.0)
}

fn range_throttle(range: f32, soft: f32, hard: f32) -> f32 {
    if hard <= soft || soft >= 1.0 {
        return 0.0;
    }
    ((range - soft) / (hard - soft)).clamp(0.0, 1.0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModelSupport {
    Supported,
    Missing,
    SideMismatch,
    LowConfidence,
    HighRisk,
    LowSideProbability,
    LowEdge,
}

impl ModelSupport {
    fn is_supported(self) -> bool {
        matches!(self, Self::Supported)
    }
}

fn model_support_for_side(
    ctx: &Ctx,
    side: Side,
    min_confidence: f32,
    max_risk: f32,
    min_side_p: f32,
    entry_px: f32,
    min_edge: f32,
) -> ModelSupport {
    let Some(model) = ctx.model_output else {
        return ModelSupport::Missing;
    };
    let model_side_is_yes = model.direction_score >= 0.0;
    let target_side_is_yes = matches!(side, Side::BuyYes);
    let side_p = if target_side_is_yes == model_side_is_yes {
        model.calibrated_p
    } else {
        1.0 - model.calibrated_p
    };
    if model_side_is_yes != target_side_is_yes {
        ModelSupport::SideMismatch
    } else if model.confidence_score < min_confidence {
        ModelSupport::LowConfidence
    } else if model.risk_score > max_risk {
        ModelSupport::HighRisk
    } else if side_p < min_side_p {
        ModelSupport::LowSideProbability
    } else if side_p - entry_px < min_edge {
        ModelSupport::LowEdge
    } else {
        ModelSupport::Supported
    }
}

fn model_side_probability(ctx: &Ctx, side: Side) -> Option<f32> {
    let model = ctx.model_output?;
    let model_side_is_yes = model.direction_score >= 0.0;
    let target_side_is_yes = matches!(side, Side::BuyYes);
    Some(if target_side_is_yes == model_side_is_yes {
        model.calibrated_p
    } else {
        1.0 - model.calibrated_p
    })
}

fn model_limited_buy_price(ctx: &Ctx, side: Side, max_ask: f32, min_edge: f32) -> f32 {
    model_side_probability(ctx, side)
        .map(|side_p| (side_p - min_edge).min(max_ask).clamp(0.0, 1.0))
        .unwrap_or(max_ask)
}

fn bump_model_fail(stats: &mut BonereaperV2GateStats, prefix: GatePrefix, support: ModelSupport) {
    match (prefix, support) {
        (_, ModelSupport::Supported) => {}
        (GatePrefix::LateConfirm, ModelSupport::Missing) => stats.late_confirm_model_missing += 1,
        (GatePrefix::LateConfirm, ModelSupport::SideMismatch) => {
            stats.late_confirm_model_side_fail += 1;
        }
        (GatePrefix::LateConfirm, ModelSupport::LowConfidence) => {
            stats.late_confirm_model_confidence_fail += 1;
        }
        (GatePrefix::LateConfirm, ModelSupport::HighRisk) => {
            stats.late_confirm_model_risk_fail += 1;
        }
        (GatePrefix::LateConfirm, ModelSupport::LowSideProbability) => {
            stats.late_confirm_model_side_p_fail += 1;
        }
        (GatePrefix::LateConfirm, ModelSupport::LowEdge) => {
            stats.late_confirm_model_edge_fail += 1;
        }
        (GatePrefix::HighSkew, ModelSupport::Missing) => stats.high_skew_model_missing += 1,
        (GatePrefix::HighSkew, ModelSupport::SideMismatch) => {
            stats.high_skew_model_side_fail += 1;
        }
        (GatePrefix::HighSkew, ModelSupport::LowConfidence) => {
            stats.high_skew_model_confidence_fail += 1;
        }
        (GatePrefix::HighSkew, ModelSupport::HighRisk) => stats.high_skew_model_risk_fail += 1,
        (GatePrefix::HighSkew, ModelSupport::LowSideProbability) => {
            stats.high_skew_model_side_p_fail += 1;
        }
        (GatePrefix::HighSkew, ModelSupport::LowEdge) => {
            stats.high_skew_model_edge_fail += 1;
        }
        (GatePrefix::LateFavourite, ModelSupport::Missing) => {
            stats.late_favourite_model_missing += 1;
        }
        (GatePrefix::LateFavourite, ModelSupport::SideMismatch) => {
            stats.late_favourite_model_side_fail += 1;
        }
        (GatePrefix::LateFavourite, ModelSupport::LowConfidence) => {
            stats.late_favourite_model_confidence_fail += 1;
        }
        (GatePrefix::LateFavourite, ModelSupport::HighRisk) => {
            stats.late_favourite_model_risk_fail += 1;
        }
        (GatePrefix::LateFavourite, ModelSupport::LowSideProbability) => {
            stats.late_favourite_model_side_p_fail += 1;
        }
        (GatePrefix::LateFavourite, ModelSupport::LowEdge) => {
            stats.late_favourite_model_edge_fail += 1;
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum GatePrefix {
    LateConfirm,
    HighSkew,
    LateFavourite,
}

fn side_is_book_favourite(event: &ReplayEvent, side: Side) -> bool {
    match side {
        Side::BuyYes => event.yes_mid >= 0.5,
        Side::BuyNo => event.yes_mid <= 0.5,
        _ => false,
    }
}

fn side_book_skew(event: &ReplayEvent, side: Side) -> f32 {
    match side {
        Side::BuyYes => event.yes_mid - 0.5,
        Side::BuyNo => 0.5 - event.yes_mid,
        _ => f32::NEG_INFINITY,
    }
}

fn buy_side_sign(side: Side) -> f32 {
    match side {
        Side::BuyYes => 1.0,
        Side::BuyNo => -1.0,
        _ => 0.0,
    }
}

fn tail_clip_notional(
    base_clip: f64,
    tail_clip_frac: f32,
    favourite_notional: f64,
    tail_notional: f64,
    tail_px: f64,
    coverage_frac: f32,
    remaining_tail_budget: f64,
) -> f64 {
    let remaining_tail_budget = remaining_tail_budget.max(0.0);
    if remaining_tail_budget <= 0.0 {
        return 0.0;
    }
    if coverage_frac > 0.0 {
        let target_tail_notional =
            favourite_notional * coverage_frac.clamp(0.0, 1.0) as f64 * tail_px.clamp(0.0, 1.0);
        return (target_tail_notional - tail_notional)
            .max(0.0)
            .min(remaining_tail_budget);
    }

    (base_clip * tail_clip_frac as f64).min(remaining_tail_budget)
}

impl Strategy for BonereaperV2 {
    fn on_event(
        &mut self,
        event: &ReplayEvent,
        ctx: &Ctx,
        spot: &SpotHistory,
        trades: &TradeHistory,
    ) -> StrategyOutput {
        self.recent_mids.push(event.yes_mid);
        let book_dir = direction_score(event, &self.recent_mids, MICRO_DEV_SCALE);
        let (spot_mom, spot_fast_mom) = if !spot.is_empty() {
            let stack = spot_momentum_stack(event.ts_ns, spot);
            (stack.blended_score, stack.fast_score)
        } else {
            (0.0, 0.0)
        };
        let whipsaw = if !spot.is_empty() {
            WhipsawRiskSnapshot::from_history(event.ts_ns, spot)
        } else {
            WhipsawRiskSnapshot::default()
        };
        let trade_flow = if !trades.is_empty() {
            compute_trade_flow(event.ts_ns, TRADE_FLOW_LOOKBACK_NS, trades).flow_imbalance
        } else {
            0.0
        };
        let composite_dir =
            (0.30 * book_dir.composite + 0.55 * spot_mom + 0.15 * (-trade_flow)).clamp(-1.0, 1.0);

        if event.yes_bid <= 0.0 || event.yes_ask <= 0.0 {
            return StrategyOutput::hold();
        }
        let window_open_ns = ctx.market_close_ns - BETTING_WINDOW_SECS * 1_000_000_000;
        let secs_in = ((event.ts_ns - window_open_ns) as f64 / 1e9) as f32;
        if !(0.0..=BETTING_WINDOW_SECS as f32).contains(&secs_in) {
            return StrategyOutput::hold();
        }

        // Update skew sustain trackers (used by high-skew anti-spike guard).
        let skew_hi = 0.5 + self.cfg.high_skew_threshold;
        let skew_lo = 0.5 - self.cfg.high_skew_threshold;
        if event.yes_mid >= skew_hi {
            self.skew_high_first_ns.get_or_insert(event.ts_ns);
        } else {
            self.skew_high_first_ns = None;
        }
        if event.yes_mid <= skew_lo {
            self.skew_low_first_ns.get_or_insert(event.ts_ns);
        } else {
            self.skew_low_first_ns = None;
        }

        let mut orders: Vec<OrderRequest> = Vec::new();

        // ---- LANE 1: Early directional probe ----
        if !self.early_emitted
            && secs_in <= self.cfg.early_phase_end_secs
            && composite_dir.abs() >= self.cfg.min_composite_direction
        {
            let side = if composite_dir > 0.0 {
                Side::BuyYes
            } else {
                Side::BuyNo
            };
            let px = buy_px(event, side);
            if px > 0.0 {
                let clip = self.cfg.max_clip_usdc * self.cfg.early_clip_frac as f64;
                let shares = shares_capped(clip, px);
                if shares > 0.0 {
                    self.early_emitted = true;
                    orders.push(OrderRequest {
                        side,
                        shares,
                        max_depth: 1,
                        limit_price: None,
                        tag: "br2_early_dir",
                    });
                    self.ladder_side = Some(side);
                    self.last_ladder_mid = event.yes_mid;
                }
            }
        }

        // ---- LANE 2: Mid-ladder ----
        if secs_in > self.cfg.early_phase_end_secs
            && secs_in <= self.cfg.mid_phase_end_secs
            && self.mid_rungs < self.cfg.mid_ladder_max_rungs
            && composite_dir.abs() >= self.cfg.min_composite_direction
        {
            let target = if composite_dir > 0.0 {
                Side::BuyYes
            } else {
                Side::BuyNo
            };
            let same_side = self.ladder_side == Some(target);
            let book_moved = match target {
                Side::BuyYes => {
                    event.yes_mid - self.last_ladder_mid >= self.cfg.mid_ladder_min_step
                }
                Side::BuyNo => self.last_ladder_mid - event.yes_mid >= self.cfg.mid_ladder_min_step,
                _ => false,
            };
            if self.ladder_side.is_none() || (same_side && book_moved) {
                let px = buy_px(event, target);
                if px > 0.0 {
                    let clip = self.cfg.max_clip_usdc * self.cfg.mid_clip_frac as f64;
                    let shares = shares_capped(clip, px);
                    if shares > 0.0 {
                        orders.push(OrderRequest {
                            side: target,
                            shares,
                            max_depth: 1,
                            limit_price: None,
                            tag: "br2_mid_ladder",
                        });
                        self.ladder_side = Some(target);
                        self.last_ladder_mid = event.yes_mid;
                        self.mid_rungs += 1;
                    }
                }
            }
        }

        // ---- LANE 3: Late directional (workhorse) + paired tail hedge ----
        // Each late_confirm fire automatically emits a paired cheap-side tail
        // bet — convex hedge against the directional bet getting whipsawed.
        if secs_in > self.cfg.mid_phase_end_secs
            && self.late_fires < self.cfg.late_max_fires
            && composite_dir.abs() >= self.cfg.min_composite_direction
            && (event.ts_ns - self.last_late_ns) as f64 / 1e9 > self.cfg.late_refresh_secs as f64
        {
            self.gate_stats.late_confirm_checks += 1;
            let target = if composite_dir > 0.0 {
                Side::BuyYes
            } else {
                Side::BuyNo
            };
            let px = buy_px(event, target);
            if px <= 0.0 {
                self.gate_stats.late_confirm_price_fail += 1;
            } else if !side_is_book_favourite(event, target) {
                self.gate_stats.late_confirm_book_favourite_fail += 1;
            } else if side_book_skew(event, target) < self.cfg.late_confirm_min_book_skew {
                self.gate_stats.late_confirm_book_skew_fail += 1;
            } else if whipsaw.score > self.cfg.late_confirm_max_whipsaw_score {
                self.gate_stats.late_confirm_whipsaw_fail += 1;
            } else {
                let model_support = self.model_support_for_side(
                    ctx,
                    target,
                    self.cfg.late_confirm_min_model_confidence,
                    self.cfg.late_confirm_max_model_risk,
                    self.cfg.late_confirm_min_model_side_p,
                    px as f32,
                    self.cfg.late_confirm_min_model_edge,
                );
                if !model_support.is_supported() {
                    bump_model_fail(&mut self.gate_stats, GatePrefix::LateConfirm, model_support);
                } else {
                    let clip = self.cfg.max_clip_usdc * self.cfg.late_clip_frac as f64;
                    let shares = shares_capped(clip, px);
                    if shares > 0.0 {
                        orders.push(OrderRequest {
                            side: target,
                            shares,
                            max_depth: self.cfg.late_sweep_depth,
                            limit_price: None,
                            tag: "br2_late_confirm",
                        });
                        self.late_fires += 1;
                        self.last_late_ns = event.ts_ns;
                        self.gate_stats.late_confirm_emits += 1;

                        // Paired tail hedge: cheap-side bet, sized as a fraction of
                        // the directional clip. Skips its own ladder gates because
                        // it's an explicit hedge, but respects min/max ask floor.
                        if self.cfg.paired_tail_clip_frac > 0.0 {
                            let tail_side = match target {
                                Side::BuyYes => Side::BuyNo,
                                Side::BuyNo => Side::BuyYes,
                                _ => unreachable!(),
                            };
                            let tail_px = buy_px(event, tail_side);
                            let tail_px32 = tail_px as f32;
                            if tail_px32 >= self.cfg.tail_min_ask
                                && tail_px32 <= self.cfg.tail_max_ask
                            {
                                let tail_clip =
                                    self.cfg.max_clip_usdc * self.cfg.paired_tail_clip_frac as f64;
                                let tail_shares = shares_capped(tail_clip, tail_px);
                                if tail_shares > 0.0 {
                                    orders.push(OrderRequest {
                                        side: tail_side,
                                        shares: tail_shares,
                                        max_depth: 1,
                                        limit_price: Some(self.cfg.tail_max_ask),
                                        tag: "br2_paired_tail",
                                    });
                                }
                            }
                        }
                    } else {
                        self.gate_stats.late_confirm_shares_fail += 1;
                    }
                }
            }
        }

        // ---- LANE 4: High-skew load with whipsaw guards ----
        if secs_in >= self.cfg.late_favourite_start_secs
            && self.high_skew_clips < self.cfg.high_skew_max_clips
            && (event.ts_ns - self.last_high_skew_ns) as f64 / 1e9
                > self.cfg.high_skew_refresh_secs as f64
        {
            self.gate_stats.high_skew_checks += 1;
            let regime_ok = if self.cfg.high_skew_skip_whipsaw {
                let snap = BtcRegimeSnapshot::from_history(event.ts_ns, spot);
                !matches!(snap.regime(), Some(BtcRegime::Whipsaw))
            } else {
                true
            };
            let skew_signed = event.yes_mid - 0.5;
            let skew_mag = skew_signed.abs();
            if !regime_ok || whipsaw.score > self.cfg.high_skew_max_whipsaw_score {
                self.gate_stats.high_skew_regime_fail += 1;
                if whipsaw.score > self.cfg.high_skew_max_whipsaw_score {
                    self.gate_stats.high_skew_whipsaw_fail += 1;
                }
            } else if skew_mag < self.cfg.high_skew_threshold {
                self.gate_stats.high_skew_threshold_fail += 1;
            } else {
                let first_ns = if skew_signed > 0.0 {
                    self.skew_high_first_ns
                } else {
                    self.skew_low_first_ns
                };
                let sustained = first_ns
                    .map(|t0| {
                        (event.ts_ns - t0) as f64 / 1e9
                            >= self.cfg.high_skew_min_sustain_secs as f64
                    })
                    .unwrap_or(false);
                let spot_aligned = spot_fast_mom.signum() == skew_signed.signum()
                    && spot_fast_mom.abs() >= self.cfg.high_skew_min_spot_alignment;
                if !sustained {
                    self.gate_stats.high_skew_sustain_fail += 1;
                } else if !spot_aligned {
                    self.gate_stats.high_skew_spot_alignment_fail += 1;
                } else {
                    let side = if skew_signed > 0.0 {
                        Side::BuyYes
                    } else {
                        Side::BuyNo
                    };
                    let px = buy_px(event, side);
                    if px <= 0.0 || px as f32 > self.cfg.high_skew_max_ask {
                        self.gate_stats.high_skew_price_fail += 1;
                    } else {
                        let model_support = self.model_support_for_side(
                            ctx,
                            side,
                            self.cfg.late_favourite_min_model_confidence,
                            self.cfg.late_favourite_max_model_risk,
                            self.cfg.late_favourite_min_model_side_p,
                            px as f32,
                            self.cfg.late_favourite_min_model_edge,
                        );
                        if !model_support.is_supported() {
                            bump_model_fail(
                                &mut self.gate_stats,
                                GatePrefix::HighSkew,
                                model_support,
                            );
                        } else {
                            let clip = self.cfg.max_clip_usdc * self.cfg.high_skew_clip_frac as f64;
                            let shares = shares_capped(clip, px);
                            if shares > 0.0 {
                                orders.push(OrderRequest {
                                    side,
                                    shares,
                                    max_depth: self.cfg.high_skew_sweep_depth,
                                    limit_price: None,
                                    tag: "br2_high_skew_load",
                                });
                                self.high_skew_clips += 1;
                                self.last_high_skew_ns = event.ts_ns;
                                self.gate_stats.high_skew_emits += 1;
                            } else {
                                self.gate_stats.high_skew_shares_fail += 1;
                            }
                        }
                    }
                }
            }
        }

        // ---- LANE 5: Late favourite load ----
        if secs_in >= self.cfg.late_favourite_start_secs {
            self.gate_stats.late_favourite_window_checks += 1;
            let refreshed = (event.ts_ns - self.last_late_favourite_ns) as f64 / 1e9
                > self.cfg.late_favourite_refresh_secs as f64;

            if self.late_favourite_clips >= self.cfg.late_favourite_max_clips {
                self.gate_stats.late_favourite_capacity_fail += 1;
            } else if !refreshed {
                self.gate_stats.late_favourite_refresh_fail += 1;
            } else {
                self.gate_stats.late_favourite_checks += 1;
                let skew_signed = event.yes_mid - 0.5;
                let skew_mag = skew_signed.abs();
                let composite_aligned = composite_dir.signum() == skew_signed.signum()
                    && composite_dir.abs() >= self.cfg.late_favourite_min_composite_alignment;
                let spot_aligned = spot_fast_mom.signum() == skew_signed.signum()
                    && spot_fast_mom.abs() >= self.cfg.high_skew_min_spot_alignment;
                let favourite_sustained = if self.cfg.late_favourite_min_sustain_secs > 0.0 {
                    let first_ns = if skew_signed > 0.0 {
                        self.skew_high_first_ns
                    } else {
                        self.skew_low_first_ns
                    };
                    first_ns
                        .map(|t0| {
                            (event.ts_ns - t0) as f64 / 1e9
                                >= self.cfg.late_favourite_min_sustain_secs as f64
                        })
                        .unwrap_or(false)
                } else {
                    true
                };

                if skew_mag < self.cfg.late_favourite_threshold {
                    self.gate_stats.late_favourite_skew_fail += 1;
                } else if !favourite_sustained {
                    self.gate_stats.late_favourite_sustain_fail += 1;
                } else if whipsaw.score > self.cfg.late_favourite_max_whipsaw_score {
                    self.gate_stats.late_favourite_whipsaw_fail += 1;
                } else if whipsaw.reversal_pressure > self.cfg.late_favourite_max_reversal_pressure
                {
                    self.gate_stats.late_favourite_reversal_pressure_fail += 1;
                } else if whipsaw.path_efficiency < self.cfg.late_favourite_min_path_efficiency {
                    self.gate_stats.late_favourite_path_efficiency_fail += 1;
                } else if ctx.market_yes_range_so_far > self.cfg.late_favourite_max_observed_range {
                    self.gate_stats.late_favourite_market_range_fail += 1;
                } else {
                    let side = if skew_signed > 0.0 {
                        Side::BuyYes
                    } else {
                        Side::BuyNo
                    };
                    let adverse_fast_momentum = buy_side_sign(side) * spot_fast_mom
                        < -self.cfg.late_favourite_max_adverse_fast_momentum;
                    let px = buy_px(event, side);
                    let entry_pullback = if self.cfg.late_favourite_max_entry_pullback < 1.0
                        && self.late_favourite_side == Some(side)
                        && self.late_favourite_peak_entry_px > 0.0
                    {
                        self.late_favourite_peak_entry_px - px
                    } else {
                        0.0
                    };
                    let avg_entry_drawdown = if self.cfg.late_favourite_max_avg_entry_drawdown < 1.0
                        && self.late_favourite_side == Some(side)
                        && self.late_favourite_side_shares_emitted > 0.0
                    {
                        (self.late_favourite_side_notional_emitted
                            / self.late_favourite_side_shares_emitted)
                            - px
                    } else {
                        0.0
                    };
                    let high_cert_favourite = px as f32 >= self.cfg.late_favourite_high_cert_ask;
                    if px <= 0.0
                        || px as f32 > self.cfg.late_favourite_max_ask
                        || (px as f32) < self.cfg.late_favourite_min_ask
                    {
                        self.gate_stats.late_favourite_price_fail += 1;
                    } else if adverse_fast_momentum {
                        self.gate_stats.late_favourite_adverse_momentum_fail += 1;
                    } else if entry_pullback > self.cfg.late_favourite_max_entry_pullback as f64 {
                        self.gate_stats.late_favourite_entry_pullback_fail += 1;
                    } else if avg_entry_drawdown
                        > self.cfg.late_favourite_max_avg_entry_drawdown as f64
                    {
                        self.gate_stats.late_favourite_avg_entry_drawdown_fail += 1;
                    } else if !high_cert_favourite && !(composite_aligned || spot_aligned) {
                        self.gate_stats.late_favourite_alignment_fail += 1;
                    } else if self.cfg.late_favourite_min_model_direction_abs > 0.0
                        && ctx.model_output.is_some_and(|model| {
                            model.direction_score.abs()
                                < self.cfg.late_favourite_min_model_direction_abs
                        })
                    {
                        self.gate_stats.late_favourite_model_direction_fail += 1;
                    } else {
                        let min_model_edge = if high_cert_favourite {
                            self.cfg.late_favourite_high_cert_min_model_edge
                        } else {
                            self.cfg.late_favourite_min_model_edge
                        };
                        let range_throttle = range_throttle(
                            ctx.market_yes_range_so_far,
                            self.cfg.late_favourite_range_soft_throttle,
                            self.cfg.late_favourite_range_hard_throttle,
                        );
                        let throttled_min_confidence = self.cfg.late_favourite_min_model_confidence
                            + self.cfg.late_favourite_range_extra_confidence * range_throttle;
                        let throttled_min_edge = min_model_edge
                            + self.cfg.late_favourite_range_extra_edge * range_throttle;
                        let bypass_high_cert_edge = high_cert_favourite
                            && self.cfg.late_favourite_high_cert_bypass_model_edge;
                        let model_support = self.model_support_for_side(
                            ctx,
                            side,
                            throttled_min_confidence,
                            self.cfg.late_favourite_max_model_risk,
                            self.cfg.late_favourite_min_model_side_p,
                            px as f32,
                            if bypass_high_cert_edge {
                                -1.0
                            } else {
                                throttled_min_edge
                            },
                        );
                        if !model_support.is_supported() {
                            bump_model_fail(
                                &mut self.gate_stats,
                                GatePrefix::LateFavourite,
                                model_support,
                            );
                        } else {
                            let remaining_clips = self
                                .cfg
                                .late_favourite_max_clips
                                .saturating_sub(self.late_favourite_clips);
                            let base_levels = late_favourite_ladder_levels(px, secs_in);
                            let levels = late_favourite_high_cert_max_levels(px, base_levels)
                                .min(self.cfg.late_favourite_sweep_depth.max(1))
                                .min(remaining_clips.max(1));
                            let price_taper = late_favourite_high_cert_price_taper(px);
                            let clip_frac = if high_cert_favourite {
                                self.cfg.late_favourite_high_cert_clip_frac
                            } else {
                                self.cfg.late_favourite_clip_frac
                            };
                            let edge_taper = if high_cert_favourite {
                                if bypass_high_cert_edge {
                                    1.0
                                } else {
                                    model_side_probability(ctx, side)
                                        .map(|side_p| {
                                            late_favourite_high_cert_edge_taper(
                                                side_p - px as f32,
                                                throttled_min_edge,
                                                self.cfg.late_favourite_high_cert_full_clip_edge,
                                            )
                                        })
                                        .unwrap_or(0.0)
                                }
                            } else {
                                1.0
                            };
                            let clip = self.cfg.max_clip_usdc * clip_frac as f64;
                            let range_size_taper = (1.0 - range_throttle as f64).clamp(0.0, 1.0);
                            let desired_notional =
                                clip * levels as f64 * price_taper * edge_taper * range_size_taper;
                            let shares = shares_capped(desired_notional, px);
                            if shares > 0.0 {
                                orders.push(OrderRequest {
                                    side,
                                    shares,
                                    max_depth: levels,
                                    limit_price: Some(if bypass_high_cert_edge {
                                        self.cfg.late_favourite_max_ask
                                    } else {
                                        model_limited_buy_price(
                                            ctx,
                                            side,
                                            self.cfg.late_favourite_max_ask,
                                            throttled_min_edge,
                                        )
                                    }),
                                    tag: "br2_late_favourite_load",
                                });
                                self.late_favourite_clips += levels;
                                self.last_late_favourite_ns = event.ts_ns;
                                if self.late_favourite_side != Some(side) {
                                    self.late_favourite_peak_entry_px = px;
                                    self.late_favourite_side_shares_emitted = shares;
                                    self.late_favourite_side_notional_emitted = shares * px;
                                } else {
                                    self.late_favourite_peak_entry_px =
                                        self.late_favourite_peak_entry_px.max(px);
                                    self.late_favourite_side_shares_emitted += shares;
                                    self.late_favourite_side_notional_emitted += shares * px;
                                }
                                self.late_favourite_side = Some(side);
                                self.late_favourite_shares_emitted += shares;
                                self.late_favourite_notional_emitted += shares * px;
                                self.gate_stats.late_favourite_emits += 1;
                            } else {
                                self.gate_stats.late_favourite_shares_fail += 1;
                            }
                        }
                    }
                }
            }
        }

        // ---- LANE 6: Cheap-tail ladder anchored to late favourite exposure ----
        // This is not a standalone long-shot strategy. It only spends a small
        // fraction of an existing favourite sleeve's spend/upside.
        if self.tail_clips < self.cfg.tail_max_clips
            && (event.ts_ns - self.last_tail_ns) as f64 / 1e9 > self.cfg.tail_refresh_secs as f64
        {
            let skew_mag = (event.yes_mid - 0.5).abs();
            let starting_fresh = self.tail_clips == 0;
            let advanced = skew_mag - self.last_tail_skew_mag >= self.cfg.tail_min_skew_step;
            if skew_mag >= self.cfg.tail_extreme_threshold
                && (starting_fresh || advanced)
                && self.late_favourite_notional_emitted > 0.0
                && tail_observed_range_allowed(
                    ctx.market_yes_range_so_far,
                    self.cfg.tail_min_observed_range,
                )
            {
                let Some(favourite_side) = self.late_favourite_side else {
                    return StrategyOutput { orders };
                };
                let Some(tail_side) = opposite_buy_side(favourite_side) else {
                    return StrategyOutput { orders };
                };
                let favourite_shares = side_shares(ctx, favourite_side).max(
                    self.late_favourite_shares_emitted
                        .min(side_shares(ctx, favourite_side)),
                );
                if favourite_shares <= 0.0 {
                    return StrategyOutput { orders };
                }
                let tail_px = buy_px(event, tail_side);
                let px32 = tail_px as f32;
                if px32 >= self.cfg.tail_min_ask && px32 <= self.cfg.tail_max_ask {
                    let avg_favourite_px =
                        self.late_favourite_notional_emitted / self.late_favourite_shares_emitted;
                    let favourite_win_upside =
                        self.late_favourite_shares_emitted * (1.0 - avg_favourite_px);
                    let cap_by_fav_spend = self.late_favourite_notional_emitted
                        * self.cfg.tail_budget_favourite_spend_frac as f64;
                    let cap_by_win_upside =
                        favourite_win_upside * self.cfg.tail_budget_favourite_upside_frac as f64;
                    let remaining_tail_budget =
                        cap_by_fav_spend.min(cap_by_win_upside) - self.tail_notional_emitted;
                    let clip = tail_clip_notional(
                        self.cfg.max_clip_usdc,
                        self.cfg.tail_clip_frac,
                        self.late_favourite_notional_emitted,
                        self.tail_notional_emitted,
                        tail_px,
                        self.cfg.tail_target_favourite_loss_coverage_frac,
                        remaining_tail_budget,
                    );
                    let shares = if clip > 0.0 {
                        shares_capped(clip, tail_px)
                    } else {
                        0.0
                    };
                    if shares > 0.0 {
                        orders.push(OrderRequest {
                            side: tail_side,
                            shares,
                            max_depth: 2,
                            limit_price: Some(self.cfg.tail_max_ask),
                            tag: "br2_convex_tail",
                        });
                        self.tail_clips += 1;
                        self.tail_notional_emitted += shares * tail_px;
                        self.last_tail_skew_mag = skew_mag;
                        self.last_tail_ns = event.ts_ns;
                    }
                }
            }
        }

        StrategyOutput { orders }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pm_model::ModelOutput;

    #[test]
    fn late_favourite_ladder_scales_by_price_and_final_window() {
        assert_eq!(late_favourite_ladder_levels(0.72, 200.0), 1);
        assert_eq!(late_favourite_ladder_levels(0.77, 200.0), 2);
        assert_eq!(late_favourite_ladder_levels(0.84, 200.0), 3);
        assert_eq!(late_favourite_ladder_levels(0.91, 120.0), 4);
        assert_eq!(late_favourite_ladder_levels(0.91, 190.0), 5);
    }

    #[test]
    fn high_cert_late_favourite_tapers_size_as_upside_collapses() {
        assert_eq!(late_favourite_high_cert_price_taper(0.94), 1.0);
        assert_eq!(late_favourite_high_cert_price_taper(0.95), 1.0);
        assert!((late_favourite_high_cert_price_taper(0.97) - 0.59).abs() < 1e-9);
        assert_eq!(late_favourite_high_cert_price_taper(0.99), 0.18);

        assert_eq!(late_favourite_high_cert_max_levels(0.94, 5), 5);
        assert_eq!(late_favourite_high_cert_max_levels(0.95, 5), 3);
        assert_eq!(late_favourite_high_cert_max_levels(0.97, 5), 2);
        assert_eq!(late_favourite_high_cert_max_levels(0.99, 5), 1);
    }

    #[test]
    fn high_cert_late_favourite_tapers_size_by_model_edge() {
        assert_eq!(
            late_favourite_high_cert_edge_taper(0.010, 0.010, 0.030),
            0.0
        );
        assert!((late_favourite_high_cert_edge_taper(0.020, 0.010, 0.030) - 0.5).abs() < 1e-6);
        assert_eq!(
            late_favourite_high_cert_edge_taper(0.030, 0.010, 0.030),
            1.0
        );
        assert_eq!(
            late_favourite_high_cert_edge_taper(0.040, 0.010, 0.030),
            1.0
        );
        assert_eq!(
            late_favourite_high_cert_edge_taper(0.004, 0.005, 0.030),
            0.0
        );
        assert_eq!(late_favourite_high_cert_edge_taper(0.010, 0.010, 0.0), 1.0);
    }

    #[test]
    fn range_throttle_interpolates_between_soft_and_hard() {
        assert_eq!(range_throttle(0.80, 0.90, 0.98), 0.0);
        assert!((range_throttle(0.94, 0.90, 0.98) - 0.5).abs() < 1e-6);
        assert_eq!(range_throttle(0.99, 0.90, 0.98), 1.0);
        assert_eq!(range_throttle(0.99, 1.0, 1.0), 0.0);
    }

    #[test]
    fn late_favourite_whipsaw_component_stats_accumulate() {
        let mut a = BonereaperV2GateStats {
            late_favourite_sustain_fail: 11,
            late_favourite_reversal_pressure_fail: 2,
            late_favourite_path_efficiency_fail: 3,
            late_favourite_model_direction_fail: 5,
            ..BonereaperV2GateStats::default()
        };
        a.add_assign(BonereaperV2GateStats {
            late_favourite_sustain_fail: 13,
            late_favourite_reversal_pressure_fail: 5,
            late_favourite_path_efficiency_fail: 7,
            late_favourite_model_direction_fail: 8,
            ..BonereaperV2GateStats::default()
        });
        assert_eq!(a.late_favourite_sustain_fail, 24);
        assert_eq!(a.late_favourite_reversal_pressure_fail, 7);
        assert_eq!(a.late_favourite_path_efficiency_fail, 10);
        assert_eq!(a.late_favourite_model_direction_fail, 13);
    }

    #[test]
    fn buy_side_sign_matches_resolution_direction() {
        assert_eq!(buy_side_sign(Side::BuyYes), 1.0);
        assert_eq!(buy_side_sign(Side::BuyNo), -1.0);
    }

    #[test]
    fn model_limited_buy_price_caps_sweeps_to_model_edge() {
        let ctx = Ctx {
            events_seen: 1,
            yes_shares: 0.0,
            no_shares: 0.0,
            cash_usdc: 100.0,
            market_yes_range_so_far: 0.0,
            model_output: Some(ModelOutput {
                direction_score: 0.8,
                confidence_score: 0.9,
                calibrated_p: 0.88,
                risk_score: 0.1,
            }),
            market_close_ns: 0,
        };

        assert!((model_limited_buy_price(&ctx, Side::BuyYes, 0.93, 0.02) - 0.86).abs() < 1e-6);
        assert!((model_limited_buy_price(&ctx, Side::BuyNo, 0.93, 0.02) - 0.10).abs() < 1e-6);
    }

    #[test]
    fn gate_stats_accumulate_late_favourite_pullback_failures() {
        let mut a = BonereaperV2GateStats {
            late_favourite_entry_pullback_fail: 3,
            late_favourite_avg_entry_drawdown_fail: 2,
            ..BonereaperV2GateStats::default()
        };
        a.add_assign(BonereaperV2GateStats {
            late_favourite_entry_pullback_fail: 5,
            late_favourite_avg_entry_drawdown_fail: 7,
            ..BonereaperV2GateStats::default()
        });
        assert_eq!(a.late_favourite_entry_pullback_fail, 8);
        assert_eq!(a.late_favourite_avg_entry_drawdown_fail, 9);
    }

    #[test]
    fn tail_clip_can_target_favourite_loss_coverage() {
        let clip = tail_clip_notional(40.0, 0.15, 240.0, 8.0, 0.08, 0.80, 60.0);
        assert!((clip - 7.36).abs() < 1e-6);
    }

    #[test]
    fn tail_clip_uses_legacy_clip_when_coverage_disabled() {
        let clip = tail_clip_notional(40.0, 0.15, 240.0, 8.0, 0.08, 0.0, 60.0);
        assert!((clip - 6.0).abs() < 1e-6);
    }

    #[test]
    fn tail_clip_respects_budget_under_coverage_target() {
        let clip = tail_clip_notional(40.0, 0.15, 240.0, 0.0, 0.08, 0.80, 5.0);
        assert!((clip - 5.0).abs() < 1e-9);
    }

    #[test]
    fn tail_observed_range_gate_requires_configured_expansion() {
        assert!(tail_observed_range_allowed(0.74, 0.0));
        assert!(!tail_observed_range_allowed(0.74, 0.75));
        assert!(tail_observed_range_allowed(0.75, 0.75));
        assert!(!tail_observed_range_allowed(0.95, 2.0));
    }
}
