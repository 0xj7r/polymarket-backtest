use anyhow::{Context, Result, anyhow};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use pm_strategy::{
    BuyYesAtOpen, PairedMmDense, ReactiveDirectional, Strategy,
    paired_mm::PairedMmDenseConfig,
    reactive::ReactiveDirectionalConfig,
};
use pm_risk::PortfolioLimits;
use pm_telonex_loader::{
    Channel, TelonexStore, TelonexStoreConfig, load_binance_agg_trades_async,
    load_book_snapshot_async, polymarket_instrument_id, resolve_binance_day, to_quote_tick,
};
use pm_types::{MarketId, SpotHistory};
use std::path::PathBuf;
use std::time::Instant;

mod discovery;
mod prep_cache;
mod runner;
mod walkforward;

use runner::{RunnerConfig, pretty_print, run_backtest};
use std::io::{BufRead, BufReader, Write};
use walkforward::{StratId, WalkForwardConfig, print_summary, run_walkforward};

#[derive(Parser, Debug)]
#[command(name = "pm-app", version, about = "Polymarket backtest engine (Nautilus pure Rust)")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum StrategyKind {
    BuyYesAtOpen,
    ReactiveDirectional,
    PairedMm,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Stream a Telonex book_snapshot parquet from S3 and print sanity stats.
    InspectS3 {
        #[arg(long, default_value = "polymarket")]
        exchange: String,
        #[arg(long, default_value = "book_snapshot_25")]
        channel: String,
        #[arg(long)]
        date: String,
        #[arg(long)]
        asset_id: String,
        #[arg(long, default_value = "1")]
        market_id: u32,
        #[arg(long, default_value = "5")]
        head: usize,
    },
    /// Run a backtest on one Polymarket asset using the selected strategy.
    BacktestS3 {
        #[arg(long, default_value = "polymarket")]
        exchange: String,
        #[arg(long, default_value = "book_snapshot_25")]
        channel: String,
        #[arg(long)]
        date: String,
        #[arg(long)]
        asset_id: String,
        /// Slug of the market (e.g. btc-updown-5m-1778587500). Used to parse
        /// the resolution timestamp when --close-ts is omitted.
        #[arg(long)]
        slug: Option<String>,
        /// Resolution Unix epoch seconds. Overrides --slug parsing.
        #[arg(long)]
        close_ts: Option<i64>,
        /// Force the resolution outcome (true = YES won). When omitted, infer
        /// from final yes_mid >= 0.5.
        #[arg(long)]
        resolved_yes: Option<bool>,
        #[arg(long, default_value = "1")]
        market_id: u32,
        #[arg(long, value_enum, default_value = "reactive-directional")]
        strategy: StrategyKind,
        #[arg(long, default_value = "100.0")]
        starting_cash: f64,
        /// For BuyYesAtOpen only: number of YES shares to buy. For
        /// ReactiveDirectional this is ignored.
        #[arg(long, default_value = "10.0")]
        clip_shares: f64,
        #[arg(long, default_value = "0.25")]
        kelly_fraction: f64,
        #[arg(long, default_value = "5.0")]
        max_clip_usdc: f64,
        #[arg(long, default_value = "0.30")]
        max_drawdown_pct: f64,
        #[arg(long, default_value = "250.0")]
        max_daily_exposure_usdc: f64,
        /// Binance spot symbol to load for momentum + regime signals (e.g.
        /// BTCUSDT). Set to empty string to disable spot.
        #[arg(long, default_value = "BTCUSDT")]
        spot_symbol: String,
        /// If set, write the JSON report to this path.
        #[arg(long)]
        out: Option<PathBuf>,
        /// If set, write per-snapshot portfolio rows (JSONL) to this path.
        #[arg(long)]
        equity_curve: Option<PathBuf>,
    },
    /// Stream the tape from S3 and emit Nautilus QuoteTicks (validates that
    /// nautilus-model types are usable downstream).
    QuotesS3 {
        #[arg(long, default_value = "polymarket")]
        exchange: String,
        #[arg(long, default_value = "book_snapshot_25")]
        channel: String,
        #[arg(long)]
        date: String,
        #[arg(long)]
        asset_id: String,
        #[arg(long)]
        slug: String,
        #[arg(long, default_value = "1")]
        market_id: u32,
        #[arg(long, default_value = "5")]
        head: usize,
    },
    /// Discover Polymarket BTC-updown-5m markets for a given date and write
    /// the (asset_id, slug, close_ts, outcome) list to JSONL.
    DiscoverDay {
        #[arg(long)]
        date: String,
        #[arg(long, default_value = "btc-updown-5m-")]
        slug_prefix: String,
        #[arg(long, default_value = "32")]
        max_concurrent: usize,
        #[arg(long)]
        out: PathBuf,
    },
    /// Pre-download all parquets for a market list to a local cache directory.
    /// Once cached, walk-forward with `--local-cache-dir <dir>` is mmap-fast.
    PrepCache {
        #[arg(long)]
        markets: PathBuf,
        #[arg(long)]
        cache_dir: PathBuf,
        #[arg(long, default_value = "BTCUSDT")]
        spot_symbol: String,
        #[arg(long, default_value = "32")]
        max_concurrent: usize,
        #[arg(long, default_value_t = true)]
        skip_existing: bool,
    },
    /// Run a walk-forward backtest over many markets.
    WalkForward {
        /// JSONL of `MarketHandle` rows from `discover-day`.
        #[arg(long)]
        markets: PathBuf,
        #[arg(long, default_value = "100.0")]
        starting_cash: f64,
        #[arg(long, default_value = "0.25")]
        kelly_fraction: f64,
        #[arg(long, default_value = "5.0")]
        max_clip_usdc: f64,
        #[arg(long, default_value = "BTCUSDT")]
        spot_symbol: String,
        /// Comma-separated strategy IDs: buy_yes_at_open, reactive_directional, paired_mm
        #[arg(long, default_value = "reactive_directional,paired_mm")]
        strategies: String,
        #[arg(long, default_value = "16")]
        max_concurrent_fetches: usize,
        /// If set, use the outcome label from discovery instead of inferring
        /// from yes_mid.
        #[arg(long, default_value_t = false)]
        use_outcome_label: bool,
        /// Portfolio mode: process markets in chronological order, compound
        /// equity across markets. Disables parallelism.
        #[arg(long, default_value_t = false)]
        portfolio_mode: bool,
        /// In portfolio mode, override max_clip_usdc per market to be this
        /// fraction of current equity (e.g., 0.005 = 0.5% per bet).
        /// Omitted = use static max_clip_usdc.
        #[arg(long)]
        clip_fraction_of_equity: Option<f64>,
        /// Local directory mirroring the S3 prefix structure. When set, the
        /// loader reads parquets from local disk instead of S3 (use after
        /// `pm-app prep-cache`).
        #[arg(long)]
        local_cache_dir: Option<PathBuf>,
        /// Per-market JSONL output.
        #[arg(long)]
        out_markets: Option<PathBuf>,
        /// Summary JSON output.
        #[arg(long)]
        out_summary: Option<PathBuf>,
    },
    Paper,
    Live,
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::InspectS3 {
            exchange,
            channel,
            date,
            asset_id,
            market_id,
            head,
        } => {
            let channel: Channel = channel
                .parse()
                .map_err(|e: String| anyhow!("bad --channel: {e}"))?;
            inspect_s3(exchange, channel, date, asset_id, MarketId(market_id), head).await
        }
        Cmd::BacktestS3 {
            exchange,
            channel,
            date,
            asset_id,
            slug,
            close_ts,
            resolved_yes,
            market_id,
            strategy,
            starting_cash,
            clip_shares,
            kelly_fraction,
            max_clip_usdc,
            max_drawdown_pct,
            max_daily_exposure_usdc,
            spot_symbol,
            out,
            equity_curve,
        } => {
            let channel: Channel = channel
                .parse()
                .map_err(|e: String| anyhow!("bad --channel: {e}"))?;
            let close_ts_s = match (close_ts, slug.as_deref()) {
                (Some(ts), _) => ts,
                (None, Some(s)) => parse_close_ts_from_slug(s)?,
                (None, None) => {
                    return Err(anyhow!(
                        "need either --close-ts or --slug to determine market resolution time"
                    ));
                }
            };
            let limits = PortfolioLimits {
                max_drawdown_pct,
                max_daily_exposure_usdc,
                max_clip_usdc,
                max_per_market_exposure_usdc: 15.0,
            };
            backtest_s3(
                exchange,
                channel,
                date,
                asset_id,
                MarketId(market_id),
                strategy,
                starting_cash,
                clip_shares,
                kelly_fraction,
                limits,
                close_ts_s,
                resolved_yes,
                spot_symbol,
                out,
                equity_curve,
            )
            .await
        }
        Cmd::QuotesS3 {
            exchange,
            channel,
            date,
            asset_id,
            slug,
            market_id,
            head,
        } => {
            let channel: Channel = channel
                .parse()
                .map_err(|e: String| anyhow!("bad --channel: {e}"))?;
            quotes_s3(exchange, channel, date, asset_id, slug, MarketId(market_id), head).await
        }
        Cmd::DiscoverDay {
            date,
            slug_prefix,
            max_concurrent,
            out,
        } => discover_day(date, slug_prefix, max_concurrent, out).await,
        Cmd::PrepCache {
            markets,
            cache_dir,
            spot_symbol,
            max_concurrent,
            skip_existing,
        } => prep_cache_cmd(markets, cache_dir, spot_symbol, max_concurrent, skip_existing).await,
        Cmd::WalkForward {
            markets,
            starting_cash,
            kelly_fraction,
            max_clip_usdc,
            spot_symbol,
            strategies,
            max_concurrent_fetches,
            use_outcome_label,
            portfolio_mode,
            clip_fraction_of_equity,
            local_cache_dir,
            out_markets,
            out_summary,
        } => walk_forward(
            markets,
            starting_cash,
            kelly_fraction,
            max_clip_usdc,
            spot_symbol,
            strategies,
            max_concurrent_fetches,
            use_outcome_label,
            portfolio_mode,
            clip_fraction_of_equity,
            local_cache_dir,
            out_markets,
            out_summary,
        )
        .await,
        Cmd::Paper => {
            tracing::info!("paper stub — Phase 6");
            Ok(())
        }
        Cmd::Live => {
            tracing::info!("live stub — Phase 6");
            Ok(())
        }
    }
}

async fn discover_day(
    date: String,
    slug_prefix: String,
    max_concurrent: usize,
    out: PathBuf,
) -> Result<()> {
    let cfg = TelonexStoreConfig::from_env()?;
    let store = TelonexStore::try_new(&cfg)?;
    let markets =
        discovery::discover_markets(&store, &date, &slug_prefix, max_concurrent).await?;
    let mut f = std::fs::File::create(&out)?;
    for m in &markets {
        writeln!(f, "{}", serde_json::to_string(m)?)?;
    }
    tracing::info!(
        date = %date,
        markets = markets.len(),
        out = ?out,
        "discovery complete"
    );
    println!("discovered {} markets for {} -> {}", markets.len(), date, out.display());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
async fn walk_forward(
    markets_path: PathBuf,
    starting_cash: f64,
    kelly_fraction: f64,
    max_clip_usdc: f64,
    spot_symbol: String,
    strategies_csv: String,
    max_concurrent_fetches: usize,
    use_outcome_label: bool,
    portfolio_mode: bool,
    clip_fraction_of_equity: Option<f64>,
    local_cache_dir: Option<PathBuf>,
    out_markets: Option<PathBuf>,
    out_summary: Option<PathBuf>,
) -> Result<()> {
    let file = std::fs::File::open(&markets_path)
        .with_context(|| format!("open markets file {}", markets_path.display()))?;
    let mut markets: Vec<discovery::MarketHandle> = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        markets.push(serde_json::from_str(&line)?);
    }
    if markets.is_empty() {
        return Err(anyhow!("no markets in {}", markets_path.display()));
    }

    let strategies = parse_strategies(&strategies_csv)?;

    let store = if let Some(dir) = local_cache_dir {
        tracing::info!(?dir, "using local cache");
        TelonexStore::try_new_local(dir)?
    } else {
        let cfg = TelonexStoreConfig::from_env()?;
        TelonexStore::try_new(&cfg)?
    };

    let wf_cfg = WalkForwardConfig {
        starting_cash_usdc: starting_cash,
        kelly_fraction,
        max_clip_usdc,
        spot_symbol,
        strategies,
        max_concurrent_fetches,
        use_outcome_label,
        maker_rebate_bps: 10.0,
        taker_fee_bps: 0.0,
        portfolio_mode,
        clip_fraction_of_equity,
    };

    tracing::info!(markets = markets.len(), "starting walk-forward");
    let started = Instant::now();
    let (results, summary) = run_walkforward(&store, &markets, &wf_cfg).await?;
    let elapsed = started.elapsed().as_secs_f64();
    tracing::info!(elapsed_s = elapsed, "walk-forward complete");

    print_summary(&summary);

    if let Some(p) = out_markets {
        let mut f = std::fs::File::create(&p)?;
        for r in &results {
            writeln!(f, "{}", serde_json::to_string(r)?)?;
        }
        tracing::info!(?p, "wrote per-market results");
    }
    if let Some(p) = out_summary {
        std::fs::write(&p, serde_json::to_string_pretty(&summary)?)?;
        tracing::info!(?p, "wrote summary");
    }
    Ok(())
}

fn parse_strategies(csv: &str) -> Result<Vec<StratId>> {
    let mut out = Vec::new();
    for token in csv.split(',').map(str::trim) {
        let id = match token {
            "buy_yes_at_open" => StratId::BuyYesAtOpen,
            "reactive_directional" => StratId::ReactiveDirectional,
            "paired_mm" => StratId::PairedMm,
            "spot_momentum_follower" => StratId::SpotMomentumFollower,
            "late_big_bet" => StratId::LateBigBet,
            "bonereaper_lite" => StratId::BonereaperLite,
            "bonereaper_v2" => StratId::BonereaperV2,
            "delta_neutral_mm" => StratId::DeltaNeutralMm,
            "late_confirmation" => StratId::LateConfirmation,
            "late_convex_tail" => StratId::LateConvexTail,
            other => return Err(anyhow!("unknown strategy: {other}")),
        };
        out.push(id);
    }
    if out.is_empty() {
        return Err(anyhow!("no strategies specified"));
    }
    Ok(out)
}

/// Parses `btc-updown-5m-1778587500` -> 1778587500.
fn parse_close_ts_from_slug(slug: &str) -> Result<i64> {
    slug.rsplit('-')
        .next()
        .and_then(|t| t.parse::<i64>().ok())
        .ok_or_else(|| anyhow!("could not parse resolution timestamp from slug: {slug}"))
}

async fn fetch_tape(
    exchange: &str,
    channel: Channel,
    date: &str,
    asset_id: &str,
    market_id: MarketId,
) -> Result<(TelonexStore, Vec<pm_types::ReplayEvent>, pm_telonex_loader::LoadStats)> {
    let cfg = TelonexStoreConfig::from_env()?;
    tracing::info!(bucket = %cfg.bucket, region = %cfg.region, "connecting to S3");
    let store = TelonexStore::try_new(&cfg)?;

    let resolve_started = Instant::now();
    let path = store
        .resolve_asset_day(exchange, channel, date, asset_id)
        .await?;
    tracing::info!(
        ?path,
        took_ms = resolve_started.elapsed().as_millis() as u64,
        "resolved parquet"
    );

    let load_started = Instant::now();
    let (events, stats) = load_book_snapshot_async(store.store(), path.clone(), market_id).await?;
    tracing::info!(
        load_ms = load_started.elapsed().as_millis() as u64,
        rows = stats.rows_emitted,
        "tape loaded"
    );
    Ok((store, events, stats))
}

async fn inspect_s3(
    exchange: String,
    channel: Channel,
    date: String,
    asset_id: String,
    market_id: MarketId,
    head: usize,
) -> Result<()> {
    let (store, events, stats) = fetch_tape(&exchange, channel, &date, &asset_id, market_id).await?;

    println!(
        "== s3://{}/raw/telonex/exchange={}/channel={}/date={}/asset_id={} ==",
        store.bucket, exchange, channel, date, asset_id
    );
    println!("batches         : {}", stats.batches);
    println!("rows_total      : {}", stats.rows_total);
    println!("rows_emitted    : {}", stats.rows_emitted);
    println!("rows_null_top   : {}", stats.rows_null_top);
    if let (Some(f), Some(l)) = (stats.first_ts_ns, stats.last_ts_ns) {
        let fdt = DateTime::<Utc>::from_timestamp_nanos(f);
        let ldt = DateTime::<Utc>::from_timestamp_nanos(l);
        println!("ts_range_utc    : {} -> {}", fdt.to_rfc3339(), ldt.to_rfc3339());
        let dur_s = (l - f) as f64 / 1e9;
        println!("duration_seconds: {:.1}", dur_s);
    }

    if events.is_empty() {
        println!("no events emitted");
        return Ok(());
    }

    let mut min_spread = f32::INFINITY;
    let mut max_spread = f32::NEG_INFINITY;
    let mut sum_spread = 0.0f64;
    let mut crossed = 0usize;
    for e in &events {
        if e.yes_bid > 0.0 && e.yes_ask > 0.0 {
            let sp = e.yes_ask - e.yes_bid;
            if sp < min_spread { min_spread = sp; }
            if sp > max_spread { max_spread = sp; }
            sum_spread += sp as f64;
            if sp < 0.0 { crossed += 1; }
        }
    }
    let avg_spread = sum_spread / events.len() as f64;
    println!(
        "spread (yes)    : min={:.4}  avg={:.4}  max={:.4}  crossed_rows={}",
        min_spread, avg_spread, max_spread, crossed
    );
    println!(
        "yes_mid first/last: {:.4} -> {:.4}",
        events.first().unwrap().yes_mid,
        events.last().unwrap().yes_mid
    );

    println!("\nfirst {head} events:");
    for e in events.iter().take(head) {
        let dt = DateTime::<Utc>::from_timestamp_nanos(e.ts_ns);
        println!(
            "  {} mid={:.4} bid={:.4}x{:>7.1} ask={:.4}x{:>7.1}",
            dt.format("%H:%M:%S%.3f"),
            e.yes_mid,
            e.yes_bid,
            e.bids[0].size,
            e.yes_ask,
            e.asks[0].size
        );
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn backtest_s3(
    exchange: String,
    channel: Channel,
    date: String,
    asset_id: String,
    market_id: MarketId,
    strategy: StrategyKind,
    starting_cash: f64,
    clip_shares: f64,
    kelly_fraction: f64,
    limits: PortfolioLimits,
    close_ts_seconds: i64,
    resolved_yes: Option<bool>,
    spot_symbol: String,
    out: Option<PathBuf>,
    equity_curve: Option<PathBuf>,
) -> Result<()> {
    let (store, events, _stats) = fetch_tape(&exchange, channel, &date, &asset_id, market_id).await?;

    let spot_history = if spot_symbol.is_empty() {
        SpotHistory::default()
    } else {
        load_spot_history(&store, &spot_symbol, &date).await?
    };

    let market_close_ns = close_ts_seconds.saturating_mul(1_000_000_000);
    let max_clip_usdc = limits.max_clip_usdc;
    let cfg = RunnerConfig {
        starting_cash_usdc: starting_cash,
        market_close_ns,
        resolved_yes,
        portfolio_limits: limits,
        equity_curve_jsonl: equity_curve,
        snapshot_every_n: 200,
        maker_rebate_bps: 10.0,
        taker_fee_bps: 0.0,
        max_inventory_imbalance_shares: 1.5,
        taker_slippage_bps: 15.0,
    };
    let started = Instant::now();
    let report = match strategy {
        StrategyKind::BuyYesAtOpen => {
            let mut s = BuyYesAtOpen::new(clip_shares);
            run_backtest(&events, &spot_history, &pm_types::TradeHistory::default(), &mut s, &cfg)?
        }
        StrategyKind::ReactiveDirectional => {
            let mut s = build_reactive(starting_cash, kelly_fraction, max_clip_usdc);
            run_backtest(&events, &spot_history, &pm_types::TradeHistory::default(), &mut s, &cfg)?
        }
        StrategyKind::PairedMm => {
            let mut s = PairedMmDense::new(PairedMmDenseConfig {
                clip_shares: max_clip_usdc / 0.5_f64.max(0.01),
                ..PairedMmDenseConfig::default()
            });
            run_backtest(&events, &spot_history, &pm_types::TradeHistory::default(), &mut s, &cfg)?
        }
    };
    tracing::info!(
        elapsed_ms = started.elapsed().as_millis() as u64,
        events = report.events_processed,
        ?strategy,
        "backtest done"
    );

    pretty_print(&report);

    if let Some(path) = out {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&path, json)?;
        tracing::info!(?path, "wrote report");
    }
    Ok(())
}

fn build_reactive(
    starting_cash: f64,
    kelly_fraction: f64,
    max_clip_usdc: f64,
) -> impl Strategy + use<> {
    let cfg = ReactiveDirectionalConfig {
        bankroll_usdc: starting_cash,
        kelly_fraction,
        max_clip_usdc,
        early_pair_clip_usdc: 0.5,
        conviction_threshold_yes: 0.45,
            conviction_threshold_no: 0.25,
        book_weight: 0.4,
        spot_weight: 0.6,
    };
    ReactiveDirectional::new(cfg)
}

/// Build a Nautilus-conformant symbol from a Polymarket slug. The dotted
/// venue suffix is added inside `polymarket_instrument_id`.
fn slug_to_nautilus_symbol(slug: &str) -> String {
    slug.to_uppercase()
}

async fn load_spot_history(
    store: &TelonexStore,
    symbol: &str,
    date: &str,
) -> Result<SpotHistory> {
    let load_started = Instant::now();
    let path = resolve_binance_day(store, "agg_trades", symbol, date).await?;
    let (ticks, stats) = load_binance_agg_trades_async(store.store(), path).await?;
    tracing::info!(
        symbol = %symbol,
        date = %date,
        ticks = stats.rows_emitted,
        load_ms = load_started.elapsed().as_millis() as u64,
        "spot history loaded"
    );
    Ok(SpotHistory::new(ticks))
}

async fn quotes_s3(
    exchange: String,
    channel: Channel,
    date: String,
    asset_id: String,
    slug: String,
    market_id: MarketId,
    head: usize,
) -> Result<()> {
    let (_store, events, stats) = fetch_tape(&exchange, channel, &date, &asset_id, market_id).await?;
    let symbol = slug_to_nautilus_symbol(&slug);
    let iid = polymarket_instrument_id(&symbol);

    println!("== nautilus QuoteTick conversion ==");
    println!("symbol        : {symbol}");
    println!("instrument_id : {iid}");
    println!("rows_emitted  : {}", stats.rows_emitted);

    let convert_start = Instant::now();
    let mut converted = 0usize;
    println!("\nfirst {head} QuoteTicks:");
    for (idx, e) in events.iter().enumerate() {
        let q = to_quote_tick(e, iid);
        converted += 1;
        if idx < head {
            let dt = DateTime::<Utc>::from_timestamp_nanos(u64::from(q.ts_event) as i64);
            println!(
                "  {} bid={}x{} ask={}x{}",
                dt.format("%H:%M:%S%.3f"),
                q.bid_price,
                q.bid_size,
                q.ask_price,
                q.ask_size
            );
        }
    }
    let convert_ms = convert_start.elapsed().as_millis() as u64;
    println!("converted     : {converted} QuoteTicks in {convert_ms}ms");

    Ok(())
}

async fn prep_cache_cmd(
    markets_path: PathBuf,
    cache_dir: PathBuf,
    spot_symbol: String,
    max_concurrent: usize,
    skip_existing: bool,
) -> Result<()> {
    let file = std::fs::File::open(&markets_path)
        .with_context(|| format!("open markets file {}", markets_path.display()))?;
    let mut markets: Vec<discovery::MarketHandle> = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        markets.push(serde_json::from_str(&line)?);
    }
    if markets.is_empty() {
        return Err(anyhow!("no markets in {}", markets_path.display()));
    }
    let cfg = TelonexStoreConfig::from_env()?;
    let store = TelonexStore::try_new(&cfg)?;
    let prep_cfg = prep_cache::PrepCacheConfig {
        cache_dir,
        spot_symbol,
        max_concurrent,
        skip_existing,
    };
    prep_cache::run_prep_cache(&store, &markets, &prep_cfg).await
}
