//! Walk-forward harness — run many markets through one or more strategies and
//! aggregate the per-market results into a summary table.
//!
//! Concurrency:
//!   * tokio for S3 fetches (bounded by `max_concurrent`).
//!   * rayon for in-process matcher (the matcher itself is so fast that this
//!     is largely irrelevant, but we keep the structure for when it matters).

use anyhow::{Context, Result};
use futures::StreamExt;
use pm_risk::PortfolioLimits;
use pm_strategy::{
    BonereaperLite, BonereaperV2, BuyYesAtOpen, DeltaNeutralMm, LateBigBet, LateConfirmation,
    LateConvexTail, PairedMmDense, ReactiveDirectional, SpotMomentumFollower, Strategy,
    bonereaper::BonereaperLiteConfig,
    bonereaper_v2::BonereaperV2Config,
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
use pm_types::{MarketId, SpotHistory, TradeHistory};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

use crate::discovery::MarketHandle;
use crate::runner::{RunnerConfig, run_backtest};

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
    pub spot_symbol: String,
    pub strategies: Vec<StratId>,
    pub max_concurrent_fetches: usize,
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
}

impl Default for WalkForwardConfig {
    fn default() -> Self {
        Self {
            starting_cash_usdc: 100.0,
            kelly_fraction: 0.25,
            max_clip_usdc: 5.0,
            spot_symbol: "BTCUSDT".to_string(),
            strategies: vec![
                StratId::ReactiveDirectional,
                StratId::PairedMm,
            ],
            max_concurrent_fetches: 16,
            use_outcome_label: false,
            maker_rebate_bps: 0.0,
            taker_fee_bps: 0.0,
            portfolio_mode: false,
            clip_fraction_of_equity: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct MarketResult {
    pub asset_id: String,
    pub slug: String,
    pub close_ts: i64,
    pub outcome_label: String,
    pub per_strategy: HashMap<&'static str, StrategyMarketResult>,
}

#[derive(Debug, Clone, Serialize)]
pub struct StrategyMarketResult {
    pub orders_submitted: usize,
    pub orders_filled: usize,
    pub pnl_usdc: f64,
    pub start_equity_usdc: f64,
    pub end_equity_usdc: f64,
    pub max_drawdown_pct: f64,
    pub fills: usize,
    pub maker_rebates_usdc: f64,
    pub clip_used_usdc: f64,
    /// Per-fill detail: ts, side, shares, price, notional, tag, maker, rebate.
    /// Empty if `fills_count == 0`. Use sparingly for large runs (per-market
    /// rows can grow large).
    pub fills_detail: Vec<crate::runner::Fill>,
}

#[derive(Debug, Clone, Serialize)]
pub struct WalkForwardSummary {
    pub markets_attempted: usize,
    pub markets_succeeded: usize,
    pub per_strategy: HashMap<&'static str, StrategyAggregate>,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct StrategyAggregate {
    pub total_pnl_usdc: f64,
    pub mean_pnl_usdc: f64,
    pub median_pnl_usdc: f64,
    pub stdev_pnl_usdc: f64,
    pub hit_rate: f64,
    pub markets_with_orders: usize,
    pub total_orders_filled: usize,
    pub worst_market_pnl: f64,
    pub best_market_pnl: f64,
}

/// Per-market spot-history cache so we don't re-download the same Binance day.
#[derive(Default)]
struct SpotCache {
    pub inner: HashMap<String, Arc<SpotHistory>>,
}

impl SpotCache {
    async fn get_or_load(
        &mut self,
        store: &TelonexStore,
        symbol: &str,
        date: &str,
    ) -> Result<Arc<SpotHistory>> {
        let key = format!("{symbol}|{date}");
        if let Some(s) = self.inner.get(&key) {
            return Ok(s.clone());
        }
        let path = resolve_binance_day(store, "agg_trades", symbol, date).await?;
        let (ticks, stats) = load_binance_agg_trades_async(store.store(), path).await?;
        tracing::info!(symbol, date, ticks = stats.rows_emitted, "spot day loaded");
        let h = Arc::new(SpotHistory::new(ticks));
        self.inner.insert(key, h.clone());
        Ok(h)
    }
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

    if cfg.portfolio_mode {
        return run_portfolio(store, markets, cfg, &spot_map_top).await;
    }

    // Snapshot the spot cache so worker futures can share Arc<SpotHistory>s
    // without holding a mutable reference to the cache.
    let spot_map: HashMap<String, Arc<SpotHistory>> = spot_cache.inner.clone();
    let empty_spot = Arc::new(SpotHistory::default());

    let store_inner = store.store();
    let cfg_arc = Arc::new(cfg.clone());

    // Per-market futures: each one does fetch + matcher in sequence, but many
    // run concurrently via `buffer_unordered`. While market N's matcher runs,
    // market N+1's tape is being fetched.
    let stream = futures::stream::iter(markets.iter().enumerate().map(|(idx, m)| {
        let store_inner = store_inner.clone();
        let spot_map = spot_map.clone();
        let empty_spot = empty_spot.clone();
        let cfg_arc = cfg_arc.clone();
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
                MarketId(idx as u32 + 1),
            )
            .await
            {
                Ok(t) => t,
                Err(e) => {
                    tracing::warn!(market = %m.slug, error = %e, "tape load failed");
                    return None;
                }
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
                empty_spot
            } else {
                spot_map.get(&m.date).cloned().unwrap_or(empty_spot)
            };
            let resolved_yes = if cfg_arc.use_outcome_label {
                Some(matches!(m.outcome.as_str(), "Up" | "Yes" | "yes" | "UP"))
            } else {
                None
            };
            let runner_cfg = RunnerConfig {
                starting_cash_usdc: cfg_arc.starting_cash_usdc,
                market_close_ns: m.close_ts.saturating_mul(1_000_000_000),
                resolved_yes,
                portfolio_limits: PortfolioLimits {
                    max_clip_usdc: cfg_arc.max_clip_usdc,
                    ..PortfolioLimits::default()
                },
                equity_curve_jsonl: None,
                snapshot_every_n: 1_000_000,
                maker_rebate_bps: cfg_arc.maker_rebate_bps,
                taker_fee_bps: cfg_arc.taker_fee_bps,
                // Hard inventory cap: never let |yes - no| exceed 1.5 shares
                // per market (paired-MM safety net).
                max_inventory_imbalance_shares: 1.5,
                taker_slippage_bps: 15.0,
            };

            let mut per_strategy = HashMap::new();
            for &strat in &cfg_arc.strategies {
                match run_one_strategy(
                    strat,
                    &cfg_arc,
                    &events,
                    &spot,
                    &trades,
                    &runner_cfg,
                    cfg_arc.starting_cash_usdc,
                    cfg_arc.max_clip_usdc,
                ) {
                    Ok(r) => {
                        per_strategy.insert(strat.name(), r);
                    }
                    Err(e) => {
                        tracing::warn!(market = %m.slug, strategy = strat.name(), error = %e, "strategy run failed");
                    }
                }
            }

            tracing::debug!(
                market = %m.slug,
                events = events.len(),
                elapsed_ms = started.elapsed().as_millis() as u64,
                "market done",
            );

            Some(MarketResult {
                asset_id: m.asset_id.clone(),
                slug: m.slug.clone(),
                close_ts: m.close_ts,
                outcome_label: m.outcome.clone(),
                per_strategy,
            })
        }
    }))
    .buffer_unordered(cfg.max_concurrent_fetches);

    use futures::StreamExt as _;
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

    let summary = aggregate(&results, &cfg.strategies);
    Ok((results, summary))
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
) -> Result<StrategyMarketResult> {
    let report = match strat {
        StratId::BuyYesAtOpen => {
            let mut s = BuyYesAtOpen::new(10.0);
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
        StratId::ReactiveDirectional => {
            let mut s = ReactiveDirectional::new(ReactiveDirectionalConfig {
                bankroll_usdc: bankroll,
                kelly_fraction: cfg.kelly_fraction,
                max_clip_usdc: clip,
                early_pair_clip_usdc: 0.5,
                conviction_threshold_yes: 0.20,
                conviction_threshold_no: 0.20,
                book_weight: 0.3,
                spot_weight: 0.7,
            });
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
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
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
        StratId::SpotMomentumFollower => {
            let mut s = SpotMomentumFollower::new(SpotMomentumFollowerConfig {
                clip_usdc: clip,
                ..SpotMomentumFollowerConfig::default()
            });
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
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
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
        StratId::BonereaperLite => {
            let mut s = BonereaperLite::new(BonereaperLiteConfig {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip,
                ..BonereaperLiteConfig::default()
            });
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
        StratId::DeltaNeutralMm => {
            let mut s = DeltaNeutralMm::new(DeltaNeutralMmConfig {
                clip_shares: (clip * 0.3).max(0.1),
                max_pair_cost: 1.02,
                max_inventory_delta_shares: 1.0,
                ..DeltaNeutralMmConfig::default()
            });
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
        StratId::BonereaperV2 => {
            let mut s = BonereaperV2::new(BonereaperV2Config {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip,
                ..BonereaperV2Config::default()
            });
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
        StratId::LateConfirmation => {
            let mut s = LateConfirmation::new(LateConfirmationConfig {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip,
                ..LateConfirmationConfig::default()
            });
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
        StratId::LateConvexTail => {
            let mut s = LateConvexTail::new(LateConvexTailConfig {
                bankroll_usdc: bankroll,
                max_clip_usdc: clip * 0.2,
                ..LateConvexTailConfig::default()
            });
            run_backtest(events, spot, trades, &mut s, runner_cfg)?
        }
    };
    Ok(StrategyMarketResult {
        orders_submitted: report.counters.orders_submitted,
        orders_filled: report.counters.orders_filled_taker + report.counters.orders_filled_maker,
        pnl_usdc: report.pnl_usdc,
        start_equity_usdc: bankroll,
        end_equity_usdc: report.end_equity_usdc,
        max_drawdown_pct: report.max_drawdown_pct,
        fills: report.fills.len(),
        maker_rebates_usdc: report.maker_rebates_usdc,
        clip_used_usdc: clip,
        fills_detail: report.fills,
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
) -> Result<(Vec<MarketResult>, WalkForwardSummary)> {
    let store_inner = store.store();
    let empty_spot = Arc::new(SpotHistory::default());
    let mut equity_by_strategy: HashMap<&'static str, f64> = cfg
        .strategies
        .iter()
        .map(|s| (s.name(), cfg.starting_cash_usdc))
        .collect();
    let mut results: Vec<MarketResult> = Vec::with_capacity(markets.len());

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
        let (events, _stats) = match load_book_snapshot_async(
            store_inner.clone(),
            path,
            MarketId(idx as u32 + 1),
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(market = %m.slug, error = %e, "tape load failed");
                continue;
            }
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
            spot_map.get(&m.date).cloned().unwrap_or_else(|| empty_spot.clone())
        };
        let resolved_yes = if cfg.use_outcome_label {
            Some(matches!(m.outcome.as_str(), "Up" | "Yes" | "yes" | "UP"))
        } else {
            None
        };

        let mut per_strategy = HashMap::new();
        for &strat in &cfg.strategies {
            let bankroll = *equity_by_strategy
                .get(strat.name())
                .unwrap_or(&cfg.starting_cash_usdc);
            // Per-market clip: a fraction of current equity (compounds), else
            // static fallback. Hard floor + ceiling for sanity.
            let clip = match cfg.clip_fraction_of_equity {
                Some(frac) => (bankroll * frac).clamp(0.50, bankroll * 0.10),
                None => cfg.max_clip_usdc,
            };
            let runner_cfg = RunnerConfig {
                starting_cash_usdc: bankroll,
                market_close_ns: m.close_ts.saturating_mul(1_000_000_000),
                resolved_yes,
                portfolio_limits: PortfolioLimits {
                    max_clip_usdc: clip,
                    max_per_market_exposure_usdc: (clip * 10.0).max(50.0),
                    max_daily_exposure_usdc: bankroll * 5.0,
                    ..PortfolioLimits::default()
                },
                equity_curve_jsonl: None,
                snapshot_every_n: 1_000_000,
                maker_rebate_bps: cfg.maker_rebate_bps,
                taker_fee_bps: cfg.taker_fee_bps,
                max_inventory_imbalance_shares: 1.5,
                taker_slippage_bps: 15.0,
            };
            match run_one_strategy(strat, cfg, &events, &spot, &trades, &runner_cfg, bankroll, clip)
            {
                Ok(r) => {
                    equity_by_strategy.insert(strat.name(), r.end_equity_usdc);
                    per_strategy.insert(strat.name(), r);
                }
                Err(e) => {
                    tracing::warn!(market = %m.slug, strategy = strat.name(), error = %e, "strategy run failed");
                }
            }
        }

        if (idx + 1) % 50 == 0 {
            let equity_strs: Vec<String> = cfg
                .strategies
                .iter()
                .map(|s| format!("{}={:.2}", s.name(), equity_by_strategy.get(s.name()).copied().unwrap_or(0.0)))
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
            per_strategy,
        });
    }

    let summary = aggregate(&results, &cfg.strategies);
    Ok((results, summary))
}

fn aggregate(results: &[MarketResult], strategies: &[StratId]) -> WalkForwardSummary {
    let mut per_strategy = HashMap::new();
    for &strat in strategies {
        let name = strat.name();
        let mut pnls: Vec<f64> = Vec::new();
        let mut markets_with_orders = 0usize;
        let mut total_orders_filled = 0usize;
        for r in results {
            if let Some(sr) = r.per_strategy.get(name) {
                pnls.push(sr.pnl_usdc);
                if sr.orders_filled > 0 {
                    markets_with_orders += 1;
                }
                total_orders_filled += sr.orders_filled;
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
        per_strategy.insert(
            name,
            StrategyAggregate {
                total_pnl_usdc: total,
                mean_pnl_usdc: mean,
                median_pnl_usdc: median,
                stdev_pnl_usdc: stdev,
                hit_rate,
                markets_with_orders,
                total_orders_filled,
                worst_market_pnl: worst,
                best_market_pnl: best,
            },
        );
    }
    WalkForwardSummary {
        markets_attempted: results.len(),
        markets_succeeded: results
            .iter()
            .filter(|r| !r.per_strategy.is_empty())
            .count(),
        per_strategy,
    }
}

pub fn print_summary(summary: &WalkForwardSummary) {
    println!("== walk-forward summary ==");
    println!("markets: attempted={}  succeeded={}", summary.markets_attempted, summary.markets_succeeded);
    println!();
    println!(
        "{:>22}  {:>10}  {:>10}  {:>10}  {:>10}  {:>8}  {:>14}  {:>10}",
        "strategy", "total_pnl", "mean_pnl", "median", "stdev", "hit", "fills", "worst"
    );
    let mut rows: Vec<(&&str, &StrategyAggregate)> = summary.per_strategy.iter().collect();
    rows.sort_by(|a, b| b.1.total_pnl_usdc.partial_cmp(&a.1.total_pnl_usdc).unwrap_or(std::cmp::Ordering::Equal));
    for (name, agg) in rows {
        println!(
            "{:>22}  {:>+10.4}  {:>+10.4}  {:>+10.4}  {:>10.4}  {:>7.1}%  {:>14}  {:>+10.4}",
            name,
            agg.total_pnl_usdc,
            agg.mean_pnl_usdc,
            agg.median_pnl_usdc,
            agg.stdev_pnl_usdc,
            agg.hit_rate * 100.0,
            agg.total_orders_filled,
            agg.worst_market_pnl,
        );
    }
}
