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

use crate::regime::{BtcRegime, BtcRegimeSnapshot};
use crate::signals::{Ring, direction_score};
use crate::spot_momentum::weighted_multi_tf_return;
use crate::{Ctx, OrderRequest, Side, Strategy, StrategyOutput};
use pm_types::{ReplayEvent, SpotHistory, TradeHistory, compute_trade_flow};

const BETTING_WINDOW_SECS: i64 = 300;
const MOM_WINDOW: usize = 32;
const MICRO_DEV_SCALE: f32 = 0.6;
const SPOT_SCALE: f32 = 300.0;
const TRADE_FLOW_LOOKBACK_NS: i64 = 60 * 1_000_000_000;

#[derive(Debug, Clone, Copy)]
pub struct BonereaperV2Config {
    pub bankroll_usdc: f64,
    pub max_clip_usdc: f64,
    pub tick: f64,

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

    // High-skew load lane with whipsaw guards
    pub high_skew_threshold: f32,
    pub high_skew_max_ask: f32,
    pub high_skew_clip_frac: f32,
    pub high_skew_max_clips: usize,
    pub high_skew_refresh_secs: f32,
    pub high_skew_min_sustain_secs: f32,
    pub high_skew_min_spot_alignment: f32,
    pub high_skew_skip_whipsaw: bool,

    // Convex tail ladder. Real Bonereaper buys the losing side in multiple
    // rungs as the book moves further away from the tail side; each rung is
    // roughly USD-constant so cheaper rungs get more shares. NOTE: our
    // matcher assumes 100% taker fill at yes_ask, which is optimistic in the
    // 0.02–0.05 zone where venue depth is thin. `tail_min_ask` enforces a
    // floor below which we don't pretend we can fill.
    pub tail_clip_frac: f32,
    pub tail_extreme_threshold: f32,
    pub tail_min_skew_step: f32,
    pub tail_max_clips: usize,
    pub tail_refresh_secs: f32,
    pub tail_min_ask: f32,
    pub tail_max_ask: f32,
}

impl Default for BonereaperV2Config {
    fn default() -> Self {
        Self {
            bankroll_usdc: 1000.0,
            max_clip_usdc: 5.0,
            tick: 0.01,
            early_phase_end_secs: 30.0,
            mid_phase_end_secs: 240.0,
            min_composite_direction: 0.10,
            mid_ladder_min_step: 0.02,
            mid_ladder_max_rungs: 4,
            early_clip_frac: 0.30,
            mid_clip_frac: 0.60,
            late_clip_frac: 1.00,
            late_max_fires: 6,
            late_refresh_secs: 8.0,
            high_skew_threshold: 0.30,
            high_skew_max_ask: 0.95,
            high_skew_clip_frac: 0.80,
            high_skew_max_clips: 4,
            high_skew_refresh_secs: 6.0,
            high_skew_min_sustain_secs: 5.0,
            high_skew_min_spot_alignment: 0.03,
            high_skew_skip_whipsaw: true,
            tail_clip_frac: 0.15,
            tail_extreme_threshold: 0.20,
            tail_min_skew_step: 0.03,
            tail_max_clips: 4,
            tail_refresh_secs: 8.0,
            tail_min_ask: 0.01,
            tail_max_ask: 0.25,
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
    skew_high_first_ns: Option<i64>,
    skew_low_first_ns: Option<i64>,
    tail_clips: usize,
    last_tail_skew_mag: f32,
    last_tail_ns: i64,
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
            skew_high_first_ns: None,
            skew_low_first_ns: None,
            tail_clips: 0,
            last_tail_skew_mag: 0.0,
            last_tail_ns: i64::MIN / 2,
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
        let spot_mom = if !spot.is_empty() {
            weighted_multi_tf_return(event.ts_ns, spot)
                .map(|r| (r * SPOT_SCALE as f64).clamp(-1.0, 1.0) as f32)
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let trade_flow = if !trades.is_empty() {
            compute_trade_flow(event.ts_ns, TRADE_FLOW_LOOKBACK_NS, trades).flow_imbalance
        } else {
            0.0
        };
        let composite_dir = (0.30 * book_dir.composite + 0.55 * spot_mom + 0.15 * (-trade_flow))
            .clamp(-1.0, 1.0);

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
            let side = if composite_dir > 0.0 { Side::BuyYes } else { Side::BuyNo };
            let px = buy_px(event, side);
            if px > 0.0 {
                let clip = self.cfg.max_clip_usdc * self.cfg.early_clip_frac as f64;
                let shares = shares_capped(clip, px);
                if shares > 0.0 {
                    self.early_emitted = true;
                    orders.push(OrderRequest {
                        side,
                        shares,
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
            let target = if composite_dir > 0.0 { Side::BuyYes } else { Side::BuyNo };
            let same_side = self.ladder_side == Some(target);
            let book_moved = match target {
                Side::BuyYes => event.yes_mid - self.last_ladder_mid >= self.cfg.mid_ladder_min_step,
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

        // ---- LANE 3: Late directional (workhorse) ----
        if secs_in > self.cfg.mid_phase_end_secs
            && self.late_fires < self.cfg.late_max_fires
            && composite_dir.abs() >= self.cfg.min_composite_direction
            && (event.ts_ns - self.last_late_ns) as f64 / 1e9 > self.cfg.late_refresh_secs as f64
        {
            let target = if composite_dir > 0.0 { Side::BuyYes } else { Side::BuyNo };
            let px = buy_px(event, target);
            if px > 0.0 {
                let clip = self.cfg.max_clip_usdc * self.cfg.late_clip_frac as f64;
                let shares = shares_capped(clip, px);
                if shares > 0.0 {
                    orders.push(OrderRequest {
                        side: target,
                        shares,
                        limit_price: None,
                        tag: "br2_late_confirm",
                    });
                    self.late_fires += 1;
                    self.last_late_ns = event.ts_ns;
                }
            }
        }

        // ---- LANE 4: High-skew load with whipsaw guards ----
        if self.high_skew_clips < self.cfg.high_skew_max_clips
            && (event.ts_ns - self.last_high_skew_ns) as f64 / 1e9
                > self.cfg.high_skew_refresh_secs as f64
        {
            let regime_ok = if self.cfg.high_skew_skip_whipsaw {
                let snap = BtcRegimeSnapshot::from_history(event.ts_ns, spot);
                !matches!(snap.regime(), Some(BtcRegime::Whipsaw))
            } else {
                true
            };
            let skew_signed = event.yes_mid - 0.5;
            let skew_mag = skew_signed.abs();
            if regime_ok && skew_mag >= self.cfg.high_skew_threshold {
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
                let spot_aligned = spot_mom.signum() == skew_signed.signum()
                    && spot_mom.abs() >= self.cfg.high_skew_min_spot_alignment;
                if sustained && spot_aligned {
                    let side = if skew_signed > 0.0 { Side::BuyYes } else { Side::BuyNo };
                    let px = buy_px(event, side);
                    if px > 0.0 && px as f32 <= self.cfg.high_skew_max_ask {
                        let clip = self.cfg.max_clip_usdc * self.cfg.high_skew_clip_frac as f64;
                        let shares = shares_capped(clip, px);
                        if shares > 0.0 {
                            orders.push(OrderRequest {
                                side,
                                shares,
                                limit_price: None,
                                tag: "br2_high_skew_load",
                            });
                            self.high_skew_clips += 1;
                            self.last_high_skew_ns = event.ts_ns;
                        }
                    }
                }
            }
        }

        // ---- LANE 5: Convex tail ladder ----
        // Buy the losing (cheap) side; each new rung requires both the book to
        // have moved further away (skew advance ≥ tail_min_skew_step) and a
        // minimum refresh time to elapse.
        if self.tail_clips < self.cfg.tail_max_clips
            && (event.ts_ns - self.last_tail_ns) as f64 / 1e9 > self.cfg.tail_refresh_secs as f64
        {
            let skew_mag = (event.yes_mid - 0.5).abs();
            let starting_fresh = self.tail_clips == 0;
            let advanced = skew_mag - self.last_tail_skew_mag >= self.cfg.tail_min_skew_step;
            if skew_mag >= self.cfg.tail_extreme_threshold && (starting_fresh || advanced) {
                let (tail_side, tail_px) = if event.yes_mid > 0.5 {
                    let no_ask = (1.0 - event.yes_bid as f64).max(0.01);
                    (Side::BuyNo, no_ask)
                } else {
                    (Side::BuyYes, event.yes_ask as f64)
                };
                let px32 = tail_px as f32;
                if px32 >= self.cfg.tail_min_ask && px32 <= self.cfg.tail_max_ask {
                    let clip = self.cfg.max_clip_usdc * self.cfg.tail_clip_frac as f64;
                    let shares = shares_capped(clip, tail_px);
                    if shares > 0.0 {
                        orders.push(OrderRequest {
                            side: tail_side,
                            shares,
                            limit_price: None,
                            tag: "br2_convex_tail",
                        });
                        self.tail_clips += 1;
                        self.last_tail_skew_mag = skew_mag;
                        self.last_tail_ns = event.ts_ns;
                    }
                }
            }
        }

        StrategyOutput { orders }
    }
}
