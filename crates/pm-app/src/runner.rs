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
use chrono::{DateTime, Utc};
use pm_risk::{PortfolioLimits, PortfolioSnapshot, PortfolioState};
use pm_strategy::{Ctx, OrderRequest, Side, Strategy};
use pm_types::{ReplayEvent, SpotHistory, TradeHistory};
use serde::Serialize;
use std::io::Write;
use std::path::PathBuf;

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
}

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub starting_cash_usdc: f64,
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
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            starting_cash_usdc: 100.0,
            market_close_ns: 0,
            resolved_yes: None,
            portfolio_limits: PortfolioLimits::default(),
            equity_curve_jsonl: None,
            snapshot_every_n: 200,
            maker_rebate_bps: 0.0,
            taker_fee_bps: 0.0,
            max_inventory_imbalance_shares: f64::INFINITY,
            taker_slippage_bps: 0.0,
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

    let mut portfolio = PortfolioState::new(cfg.starting_cash_usdc, cfg.portfolio_limits.clone());
    portfolio.mark(cfg.starting_cash_usdc);

    let mut curve_file = match cfg.equity_curve_jsonl.as_deref() {
        Some(p) => Some(std::fs::File::create(p)?),
        None => None,
    };
    let snap_every = cfg.snapshot_every_n.max(1);

    for (idx, event) in events.iter().enumerate() {
        last_mid = event.yes_mid;

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

        let ctx = Ctx {
            events_seen: (idx + 1) as u64,
            yes_shares,
            no_shares,
            cash_usdc: cash,
            market_close_ns: cfg.market_close_ns,
        };
        let output = strategy.on_event(event, &ctx, spot, trades);
        for req in output.orders {
            counters.orders_submitted += 1;
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
    }

    counters.resting_orders_cancelled_eom = resting.len();
    resting.clear();

    let yes_resolved = cfg.resolved_yes.unwrap_or(last_mid >= 0.5);
    let settlement_cash = if yes_resolved { yes_shares } else { no_shares };
    let end_cash = cash + settlement_cash;
    portfolio.mark(end_cash);

    let final_snapshot = portfolio.snapshot(
        events.last().map(|e| e.ts_ns).unwrap_or(0),
        end_cash,
    );
    let peak_equity_usdc = final_snapshot.peak_equity_usdc;
    let max_drawdown_pct = if peak_equity_usdc > 0.0 {
        1.0 - end_cash / peak_equity_usdc
    } else {
        0.0
    }
    .max(final_snapshot.drawdown_pct);

    Ok(BacktestReport {
        events_processed: events.len(),
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
    })
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
            limit_price: None,
            tag: req.tag,
        };
        apply_taker_order(
            event, &synthetic, cash, yes_shares, no_shares, portfolio, counters, fills,
            taker_fee_bps, taker_slippage_bps,
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
) {
    let (raw_fill, top_size) = match req.side {
        Side::BuyYes => (event.yes_ask, event.asks[0].size as f64),
        Side::SellYes => (event.yes_bid, event.bids[0].size as f64),
        Side::BuyNo => ((1.0 - event.yes_bid).max(0.0), event.bids[0].size as f64),
        Side::SellNo => ((1.0 - event.yes_ask).max(0.0), event.asks[0].size as f64),
    };
    // Apply slippage: buyers pay more, sellers receive less. Clamp into (0,1).
    let slip = taker_slippage_bps / 10_000.0;
    let fill_price = match req.side {
        Side::BuyYes | Side::BuyNo => ((raw_fill as f64) * (1.0 + slip)).min(0.999) as f32,
        Side::SellYes | Side::SellNo => ((raw_fill as f64) * (1.0 - slip)).max(0.001) as f32,
    };
    if fill_price <= 0.0 || fill_price >= 1.0 {
        counters.orders_rejected_bad_price += 1;
        return;
    }
    if top_size <= 0.0 {
        counters.orders_rejected_no_liquidity += 1;
        return;
    }
    let fillable_shares = req.shares.min(top_size);
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
    });
}

pub fn pretty_print(rep: &BacktestReport) {
    println!("== backtest report ==");
    println!("events_processed  : {}", rep.events_processed);
    println!(
        "orders            : submitted={}  filled[taker={} maker={}]  rejected[cash={} liq={} px={} inv={} risk={}]  resting_active={}  resting_cancelled_eom={}",
        rep.counters.orders_submitted,
        rep.counters.orders_filled_taker,
        rep.counters.orders_filled_maker,
        rep.counters.orders_rejected_no_cash,
        rep.counters.orders_rejected_no_liquidity,
        rep.counters.orders_rejected_bad_price,
        rep.counters.orders_rejected_no_inventory,
        rep.counters.orders_rejected_risk_gate,
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
    fn taker_buy_yes_takes_full_loss_when_no_wins() {
        let events = vec![
            evt(0, 0.50, 0.51, 200.0),
            evt(1_000_000_000, 0.30, 0.31, 200.0),
            evt(2_000_000_000, 0.02, 0.03, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            resolved_yes: Some(false),
            portfolio_limits: PortfolioLimits { max_clip_usdc: 20.0, ..Default::default() },
            ..Default::default()
        };
        let mut strat = BuyYesAtOpen::new(10.0);
        let spot = SpotHistory::default();
        let rep = run_backtest(&events, &spot, &pm_types::TradeHistory::default(), &mut strat, &cfg).unwrap();
        assert_eq!(rep.counters.orders_filled_taker, 1);
        assert!((rep.pnl_usdc - -5.1).abs() < 1e-6, "pnl was {}", rep.pnl_usdc);
    }

    #[test]
    fn maker_buy_yes_fills_when_book_crosses_down() {
        // Strategy submits a single resting BUY YES at 0.45 at t=0; the ask
        // drops to 0.45 at t=1s; we expect a maker fill at 0.45.
        struct OneShot;
        impl Strategy for OneShot {
            fn on_event(&mut self, _e: &ReplayEvent, ctx: &Ctx, _spot: &SpotHistory, _trades: &TradeHistory) -> StrategyOutput {
                if ctx.events_seen > 1 {
                    return StrategyOutput::hold();
                }
                StrategyOutput::one(OrderRequest {
                    side: Side::BuyYes,
                    shares: 10.0,
                    limit_price: Some(0.45),
                    tag: "test_maker_buy",
                })
            }
        }
        let events = vec![
            evt(0, 0.50, 0.51, 200.0),       // submission tick: ask=0.51, no cross
            evt(500_000_000, 0.46, 0.47, 200.0), // ask=0.47, still no cross
            evt(1_000_000_000, 0.44, 0.45, 200.0), // ask=0.45, cross!
            evt(2_000_000_000, 0.30, 0.31, 200.0),
        ];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            resolved_yes: Some(true),
            portfolio_limits: PortfolioLimits { max_clip_usdc: 10.0, ..Default::default() },
            maker_rebate_bps: 10.0,
            ..Default::default()
        };
        let mut s = OneShot;
        let spot = SpotHistory::default();
        let rep = run_backtest(&events, &spot, &pm_types::TradeHistory::default(), &mut s, &cfg).unwrap();
        assert_eq!(rep.counters.orders_filled_maker, 1, "expected one maker fill");
        // 10 sh @ 0.45 = 4.50 notional; rebate 10bp = 0.0045; YES wins → +10.
        // Net: -4.50 + 10.00 + 0.0045 = +5.5045
        assert!((rep.pnl_usdc - 5.5045).abs() < 1e-6, "pnl {}", rep.pnl_usdc);
        assert!((rep.maker_rebates_usdc - 0.0045).abs() < 1e-9);
    }

    #[test]
    fn limit_above_ask_becomes_taker() {
        // Strategy submits a "limit" BUY YES at 0.99 (well above ask=0.51).
        // Should be treated as a taker fill at the actual ask.
        struct OneShot;
        impl Strategy for OneShot {
            fn on_event(&mut self, _e: &ReplayEvent, ctx: &Ctx, _spot: &SpotHistory, _trades: &TradeHistory) -> StrategyOutput {
                if ctx.events_seen > 1 {
                    return StrategyOutput::hold();
                }
                StrategyOutput::one(OrderRequest {
                    side: Side::BuyYes,
                    shares: 10.0,
                    limit_price: Some(0.99),
                    tag: "test_aggressive_limit",
                })
            }
        }
        let events = vec![evt(0, 0.50, 0.51, 200.0), evt(1_000_000_000, 0.50, 0.51, 200.0)];
        let cfg = RunnerConfig {
            starting_cash_usdc: 100.0,
            resolved_yes: Some(false),
            portfolio_limits: PortfolioLimits { max_clip_usdc: 20.0, ..Default::default() },
            ..Default::default()
        };
        let mut s = OneShot;
        let spot = SpotHistory::default();
        let rep = run_backtest(&events, &spot, &pm_types::TradeHistory::default(), &mut s, &cfg).unwrap();
        assert_eq!(rep.counters.orders_filled_taker, 1);
        assert_eq!(rep.counters.orders_filled_maker, 0);
    }
}
