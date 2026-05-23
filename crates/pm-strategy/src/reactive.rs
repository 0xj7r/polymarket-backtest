//! ReactiveDirectional — the spec's primary directional strategy.
//!
//! Phases per market:
//!   1. **Pre-window** (>5min from resolution): hold.
//!   2. **Early window** (0–60s into the 5-min betting window): paired YES/NO
//!      probes around the mid.
//!   3. **Mid/late window** (60s+): when `calibrated_p` clearly favours a side
//!      AND confidence ≥ threshold AND risk ≤ threshold, load the dominant side.
//!
//! Signal stack: book-internal direction_score is *augmented* with multi-TF
//! spot momentum and regime gating when spot data is available. Strategy
//! degrades gracefully (book-only) when `spot` is empty.

use crate::regime::{BtcRegime, BtcRegimeSnapshot};
use crate::signals::{
    ConfidenceScore, DirectionScore, Ring, RiskScore, calibrated_p, confidence_score,
    direction_score, risk_score,
};
use crate::spot_momentum::weighted_multi_tf_return;
use crate::{Ctx, OrderRequest, Side, Strategy, StrategyOutput};
use pm_types::{ReplayEvent, SpotHistory, TradeHistory, compute_trade_flow};

const MOMENTUM_WINDOW: usize = 32;
const DIR_HISTORY_WINDOW: usize = 16;
const MICRO_DEV_SCALE: f32 = 0.6;
const BETTING_WINDOW_SECS: i64 = 300;
const EARLY_PHASE_SECS: f32 = 60.0;
const DEPTH_FULL_AT_SHARES: f32 = 1500.0;
/// Multi-TF spot return is in raw fractional return space (e.g. 0.001 = 10 bps).
/// Scale it to roughly `[-1, 1]` over a ±50bps move.
const SPOT_MOMENTUM_SCALE: f32 = 200.0;
const TRADE_FLOW_LOOKBACK_NS: i64 = 60 * 1_000_000_000;

#[derive(Debug, Clone)]
pub struct ReactiveDirectionalConfig {
    pub bankroll_usdc: f64,
    pub kelly_fraction: f64,
    pub max_clip_usdc: f64,
    pub early_pair_clip_usdc: f64,
    /// Conviction floor for BuyYes (Up) entries.
    pub conviction_threshold_yes: f32,
    /// Conviction floor for BuyNo (Down) entries. Set this LOWER than the
    /// YES threshold to counter the empirical long-YES structural bias —
    /// the strategy historically takes too few BuyNo positions even when
    /// the bearish signal is present.
    pub conviction_threshold_no: f32,
    /// Weight of book-internal direction in the composite (0..1).
    pub book_weight: f32,
    /// Weight of spot momentum in the composite (0..1). Auto-zeroed when no spot.
    pub spot_weight: f32,
}

impl Default for ReactiveDirectionalConfig {
    fn default() -> Self {
        Self {
            bankroll_usdc: 100.0,
            kelly_fraction: 0.25,
            max_clip_usdc: 5.0,
            early_pair_clip_usdc: 0.5,
            conviction_threshold_yes: 0.45,
            conviction_threshold_no: 0.25,
            book_weight: 0.4,
            spot_weight: 0.6,
        }
    }
}

pub struct ReactiveDirectional {
    cfg: ReactiveDirectionalConfig,
    recent_mids: Ring,
    recent_dirs: Ring,
    early_paired_emitted: bool,
    last_load_ts_ns: i64,
}

impl ReactiveDirectional {
    pub fn new(cfg: ReactiveDirectionalConfig) -> Self {
        Self {
            cfg,
            recent_mids: Ring::new(MOMENTUM_WINDOW),
            recent_dirs: Ring::new(DIR_HISTORY_WINDOW),
            early_paired_emitted: false,
            last_load_ts_ns: 0,
        }
    }

    fn window_open_ns(&self, ctx: &Ctx) -> i64 {
        ctx.market_close_ns - BETTING_WINDOW_SECS * 1_000_000_000
    }

    fn seconds_since_window_open(&self, event: &ReplayEvent, ctx: &Ctx) -> f32 {
        ((event.ts_ns - self.window_open_ns(ctx)) as f64 / 1e9) as f32
    }

    fn in_betting_window(&self, event: &ReplayEvent, ctx: &Ctx) -> bool {
        let secs = self.seconds_since_window_open(event, ctx);
        (0.0..=BETTING_WINDOW_SECS as f32).contains(&secs)
    }

    fn composite_direction(
        &self,
        event: &ReplayEvent,
        book_dir: DirectionScore,
        spot: &SpotHistory,
        trades: &TradeHistory,
    ) -> f32 {
        let spot_mom = if !spot.is_empty() {
            weighted_multi_tf_return(event.ts_ns, spot)
                .map(|r| (r as f32 * SPOT_MOMENTUM_SCALE).clamp(-1.0, 1.0))
                .unwrap_or(0.0)
        } else {
            0.0
        };
        let trade_flow = if !trades.is_empty() {
            compute_trade_flow(event.ts_ns, TRADE_FLOW_LOOKBACK_NS, trades).flow_imbalance
        } else {
            0.0
        };
        // EMPIRICAL (2026-05-23, 1-day walk-forward, 288 markets):
        //   no trade flow:                 RD +$106/day  ← best
        //   +30% trade flow (continuation): RD -$331/day  ← worst
        //   -10% trade flow (contrarian):   RD   -$2/day  ← break-even
        //
        // 60s aggressor-flow at either sign adds no edge; the signal is
        // dominated by spread cost from extra fills it triggers. Suppressed
        // for now; revisit with a different lookback or feature transform.
        let _ = trade_flow;
        (self.cfg.book_weight * book_dir.composite + self.cfg.spot_weight * spot_mom)
            .clamp(-1.0, 1.0)
    }
}

impl Strategy for ReactiveDirectional {
    fn on_event(
        &mut self,
        event: &ReplayEvent,
        ctx: &Ctx,
        spot: &SpotHistory, _trades: &pm_types::TradeHistory,
    ) -> StrategyOutput {
        self.recent_mids.push(event.yes_mid);
        let book_dir: DirectionScore = direction_score(event, &self.recent_mids, MICRO_DEV_SCALE);
        let composite_dir = self.composite_direction(event, book_dir, spot, _trades);
        self.recent_dirs.push(composite_dir);

        if !self.in_betting_window(event, ctx) {
            return StrategyOutput::hold();
        }
        if event.yes_bid <= 0.0 || event.yes_ask <= 0.0 {
            return StrategyOutput::hold();
        }

        let secs = self.seconds_since_window_open(event, ctx);
        let conf: ConfidenceScore = confidence_score(&self.recent_dirs, secs);
        let risk: RiskScore = risk_score(event, &self.recent_dirs, DEPTH_FULL_AT_SHARES);

        // Regime gating: skip directional loading in Whipsaw; bonus to confidence
        // in DirectionalSmooth.
        let regime = BtcRegimeSnapshot::from_history(event.ts_ns, spot).regime();
        let regime_conf_mul = match regime {
            Some(BtcRegime::DirectionalSmooth) => 1.15,
            Some(BtcRegime::TrendingVolatile) => 0.95,
            Some(BtcRegime::Flat) => 0.85,
            Some(BtcRegime::Whipsaw) => 0.55,
            None => 1.0,
        };
        let conf_composite = (conf.composite * regime_conf_mul).clamp(0.0, 1.0);

        let p = calibrated_p(composite_dir, conf_composite, event.yes_mid, 0.7);

        // Early window: paired probes (once).
        if !self.early_paired_emitted && secs <= EARLY_PHASE_SECS {
            self.early_paired_emitted = true;
            let yes_clip = (self.cfg.early_pair_clip_usdc / event.yes_ask as f64).max(0.0);
            let no_clip =
                (self.cfg.early_pair_clip_usdc / (1.0 - event.yes_bid as f64).max(0.01)).max(0.0);
            return StrategyOutput {
                orders: vec![
                    OrderRequest {
                        side: Side::BuyYes,
                        shares: yes_clip,
                        limit_price: None,
                        tag: "rd_early_pair_yes",
                    },
                    OrderRequest {
                        side: Side::BuyNo,
                        shares: no_clip,
                        limit_price: None,
                        tag: "rd_early_pair_no",
                    },
                ],
            };
        }

        // Mid/late directional load.
        if secs <= EARLY_PHASE_SECS {
            return StrategyOutput::hold();
        }
        if matches!(regime, Some(BtcRegime::Whipsaw)) {
            return StrategyOutput::hold();
        }
        let conviction = (p - 0.5).abs() * 2.0;
        // EMPIRICAL: asymmetric BuyNo threshold (lower than BuyYes) was
        // designed to counter long-YES bias but dropped P&L from +$106 to
        // -$2 in single-day testing. The bias IS the alpha here — being
        // conservative on BuyNo loses the structural edge. Keeping symmetric
        // for now; revisit when we have a real bearish signal.
        let conviction_floor = self
            .cfg
            .conviction_threshold_yes
            .min(self.cfg.conviction_threshold_no);
        if conviction < conviction_floor {
            return StrategyOutput::hold();
        }
        if conf_composite < 0.15 {
            return StrategyOutput::hold();
        }
        if risk.composite > 0.95 {
            return StrategyOutput::hold();
        }
        if event.ts_ns - self.last_load_ts_ns < 5 * 1_000_000_000 {
            return StrategyOutput::hold();
        }
        self.last_load_ts_ns = event.ts_ns;

        let stake = pm_risk::fractional_kelly_stake(
            p as f64,
            event.yes_mid as f64,
            self.cfg.bankroll_usdc,
            self.cfg.kelly_fraction,
            self.cfg.max_clip_usdc,
        );
        if stake <= 0.10 {
            return StrategyOutput::hold();
        }

        let (side, fill_px) = if p >= 0.5 {
            (Side::BuyYes, event.yes_ask)
        } else {
            (Side::BuyNo, (1.0 - event.yes_bid).max(0.01))
        };
        let shares = (stake / fill_px as f64).max(0.0);
        if shares <= 0.0 {
            return StrategyOutput::hold();
        }
        StrategyOutput::one(OrderRequest {
            side,
            shares,
            limit_price: None,
            tag: "rd_dominant_load",
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pm_types::{BookLevel, MarketId, ReplayFlags, tape::TAPE_DEPTH};

    fn evt(ts_ns: i64, bid: f32, ask: f32) -> ReplayEvent {
        let mut bids = [BookLevel::default(); TAPE_DEPTH];
        let mut asks = [BookLevel::default(); TAPE_DEPTH];
        for i in 0..TAPE_DEPTH {
            bids[i] = BookLevel { price: bid - i as f32 * 0.01, size: 200.0 };
            asks[i] = BookLevel { price: ask + i as f32 * 0.01, size: 200.0 };
        }
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
    fn holds_outside_betting_window() {
        let close_ns: i64 = 1_000_000_000_000;
        let ctx = Ctx {
            events_seen: 1,
            yes_shares: 0.0,
            no_shares: 0.0,
            cash_usdc: 100.0,
            market_close_ns: close_ns,
        };
        let mut s = ReactiveDirectional::new(ReactiveDirectionalConfig::default());
        let spot = SpotHistory::default();
        let out = s.on_event(&evt(close_ns - 600 * 1_000_000_000, 0.50, 0.51), &ctx, &spot, &pm_types::TradeHistory::default());
        assert!(out.orders.is_empty(), "should not trade pre-window");
    }

    #[test]
    fn emits_paired_entry_in_early_window_once() {
        let close_ns: i64 = 1_000_000_000_000;
        let ctx = Ctx {
            events_seen: 1,
            yes_shares: 0.0,
            no_shares: 0.0,
            cash_usdc: 100.0,
            market_close_ns: close_ns,
        };
        let mut s = ReactiveDirectional::new(ReactiveDirectionalConfig::default());
        let spot = SpotHistory::default();
        let first = s.on_event(&evt(close_ns - 290 * 1_000_000_000, 0.50, 0.51), &ctx, &spot, &pm_types::TradeHistory::default());
        assert_eq!(first.orders.len(), 2);
        let second = s.on_event(&evt(close_ns - 289 * 1_000_000_000, 0.50, 0.51), &ctx, &spot, &pm_types::TradeHistory::default());
        assert!(
            second.orders.is_empty()
                || second.orders.iter().all(|o| !o.tag.starts_with("rd_early_pair")),
        );
    }
}
