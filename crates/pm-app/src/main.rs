use anyhow::{Context, Result, anyhow};
use arrow::array::{Array, Int64Array, StringArray};
use arrow::record_batch::RecordBatch;
use chrono::{DateTime, NaiveDate, Utc};
use clap::{Parser, Subcommand};
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
use pm_model::MetaTrainingConfig;
use pm_risk::PortfolioLimits;
use pm_strategy::{
    BonereaperLite, BonereaperV2, BuyYesAtOpen, DeltaNeutralMm, LateBigBet, LateConfirmation,
    LateConvexTail, PairedMmDense, ReactiveDirectional, SpotMomentumFollower, Strategy,
    bonereaper::BonereaperLiteConfig, bonereaper_v2::BonereaperV2Config,
    delta_neutral_mm::DeltaNeutralMmConfig, late_big_bet::LateBigBetConfig,
    late_confirmation::LateConfirmationConfig, late_convex_tail::LateConvexTailConfig,
    paired_mm::PairedMmDenseConfig, reactive::ReactiveDirectionalConfig,
    spot_follower::SpotMomentumFollowerConfig,
};
use pm_telonex_loader::{
    Channel, TelonexStore, TelonexStoreConfig, load_binance_agg_trades_async,
    load_book_snapshot_async, load_pm_trades_async, polymarket_instrument_id, resolve_binance_day,
    resolve_pm_trades_day, to_quote_tick,
};
use pm_types::{MarketId, SpotHistory, TradeHistory};
use std::fs::File;
use std::path::PathBuf;
use std::time::Instant;

mod discovery;
mod prep_cache;
mod runner;
mod walkforward;

use runner::{RunnerConfig, pretty_print, run_backtest};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use walkforward::{
    StratId, WalkForwardConfig, print_summary, run_walkforward, write_market_results_jsonl_atomic,
    write_summary_json_atomic,
};

#[derive(Parser, Debug)]
#[command(
    name = "pm-app",
    version,
    about = "Polymarket backtest engine (Nautilus pure Rust)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(clap::ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum StrategyKind {
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

#[derive(Debug, Clone, Copy)]
enum MarketRunMode {
    Backtest,
    Paper,
    Live,
}

impl MarketRunMode {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Backtest => "backtest",
            Self::Paper => "paper",
            Self::Live => "live",
        }
    }
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
        /// Read from a local cache mirror instead of S3.
        #[arg(long)]
        local_cache_dir: Option<PathBuf>,
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
        /// Write per-decision attribution rows (JSONL) to this path.
        #[arg(long)]
        decision_log: Option<PathBuf>,
        /// Only log every Nth decision event when writing `--decision-log`.
        #[arg(long, default_value = "1")]
        decision_log_every_n: usize,
        /// Read data from a local cache mirror instead of S3.
        #[arg(long)]
        local_cache_dir: Option<PathBuf>,
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
        /// Read from a local cache mirror instead of S3.
        #[arg(long)]
        local_cache_dir: Option<PathBuf>,
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
    /// Discover Polymarket BTC-updown-5m markets for a date range.
    DiscoverRange {
        #[arg(long)]
        start_date: String,
        #[arg(long)]
        end_date: String,
        #[arg(long, default_value = "btc-updown-5m-")]
        slug_prefix: String,
        #[arg(long, default_value = "32")]
        max_concurrent: usize,
        #[arg(long)]
        out: PathBuf,
    },
    /// Generate BTC-updown-5m MarketHandle JSONL from the master Polymarket
    /// markets parquet.
    DiscoverMarketsParquet {
        #[arg(long)]
        markets_parquet: PathBuf,
        #[arg(long)]
        start_date: String,
        #[arg(long)]
        end_date: String,
        #[arg(long, default_value = "btc-updown-5m-")]
        slug_prefix: String,
        #[arg(long, default_value_t = false)]
        require_book_s3: bool,
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
        /// Per-order risk cap as a multiple of the base strategy clip. Allows
        /// heavy lanes while `max_clip_usdc` still defines normal entry size.
        #[arg(long, default_value = "2.0")]
        max_order_clip_multiplier: f64,
        /// Maximum cumulative gross buy outlay allowed per market.
        #[arg(long, default_value = "50.0")]
        max_per_market_exposure_usdc: f64,
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
        /// Per-market volatility split for summary stats: `high` if
        /// max(yes_mid) - min(yes_mid) > threshold.
        #[arg(long, default_value = "0.08")]
        volatility_regime_threshold: f64,
        /// In portfolio mode, override max_clip_usdc per market to be this
        /// fraction of current equity (e.g., 0.005 = 0.5% per bet).
        /// Omitted = use static max_clip_usdc.
        #[arg(long)]
        clip_fraction_of_equity: Option<f64>,
        /// Portfolio drawdown fraction where clip sizing starts scaling down.
        /// Example: 0.12 starts de-risking at 12% below peak. Disabled unless
        /// below --clip-drawdown-hard-pct.
        #[arg(long, default_value = "1.0")]
        clip_drawdown_soft_pct: f64,
        /// Portfolio drawdown fraction where clip sizing reaches zero.
        /// Example: 0.25 stops new sizing at 25% below peak.
        #[arg(long, default_value = "1.0")]
        clip_drawdown_hard_pct: f64,
        /// Bonereaper v2 late confirmation clip multiplier.
        #[arg(long, default_value = "1.0")]
        br2_late_clip_frac: f32,
        /// Bonereaper v2 maximum late confirmation fires.
        #[arg(long, default_value = "3")]
        br2_late_max_fires: usize,
        /// Bonereaper v2 minimum ML confidence for late confirmation entries.
        #[arg(long, default_value = "0.58")]
        br2_late_confirm_min_model_confidence: f32,
        /// Bonereaper v2 maximum ML risk for late confirmation entries.
        #[arg(long, default_value = "0.80")]
        br2_late_confirm_max_model_risk: f32,
        /// Bonereaper v2 minimum ML predicted-side probability for late confirmation entries.
        #[arg(long, default_value = "0.58")]
        br2_late_confirm_min_model_side_p: f32,
        /// Bonereaper v2 high-skew clip multiplier.
        #[arg(long, default_value = "0.60")]
        br2_high_skew_clip_frac: f32,
        /// Bonereaper v2 maximum high-skew load clips.
        #[arg(long, default_value = "5")]
        br2_high_skew_max_clips: usize,
        /// Bonereaper v2 late-favourite absolute skew threshold from 0.5.
        #[arg(long, default_value = "0.22")]
        br2_late_favourite_threshold: f32,
        /// Bonereaper v2 late-favourite clip multiplier.
        #[arg(long, default_value = "1.00")]
        br2_late_favourite_clip_frac: f32,
        /// Bonereaper v2 maximum late-favourite load clips.
        #[arg(long, default_value = "12")]
        br2_late_favourite_max_clips: usize,
        /// Bonereaper v2 book depth to sweep for late-favourite loads.
        #[arg(long, default_value = "7")]
        br2_late_favourite_sweep_depth: usize,
        /// Bonereaper v2 minimum ML confidence for late-favourite loads.
        #[arg(long, default_value = "0.68")]
        br2_late_favourite_min_model_confidence: f32,
        /// Bonereaper v2 maximum ML risk for late-favourite loads.
        #[arg(long, default_value = "0.72")]
        br2_late_favourite_max_model_risk: f32,
        /// Bonereaper v2 minimum ML predicted-side probability for late-favourite loads.
        #[arg(long, default_value = "0.62")]
        br2_late_favourite_min_model_side_p: f32,
        /// Bonereaper v2 minimum ML probability edge over entry price for late-favourite loads.
        #[arg(long, default_value = "0.00")]
        br2_late_favourite_min_model_edge: f32,
        /// Enable the runner-level model gate after strategy emission.
        #[arg(long, default_value_t = true)]
        enforce_model_gate: bool,
        /// Runner-level model gate minimum confidence.
        #[arg(long, default_value = "0.68")]
        model_gate_min_confidence: f32,
        /// Runner-level model gate maximum risk.
        #[arg(long, default_value = "0.72")]
        model_gate_max_risk: f32,
        /// Runner-level model gate minimum explicit side edge.
        #[arg(long, default_value = "0.00")]
        model_gate_min_edge: f32,
        /// Local directory mirroring the S3 prefix structure. When set, the
        /// loader reads parquets from local disk instead of S3 (use after
        /// `pm-app prep-cache`).
        #[arg(long)]
        local_cache_dir: Option<PathBuf>,
        /// Split markets into this many chronological forward-chaining folds.
        /// Mutually exclusive with `--fold-size`.
        #[arg(long)]
        walk_forward_folds: Option<usize>,
        /// Split markets into explicit fold windows of this size.
        /// Mutually exclusive with `--walk-forward-folds`.
        #[arg(long)]
        fold_size: Option<usize>,
        /// Purge this many markets around each train/test boundary.
        /// Placeholder for future walk-forward training leakage control.
        #[arg(long, default_value_t = 0)]
        purge_markets: usize,
        /// Do not evaluate fold windows until at least this many prior markets
        /// are available for meta-calibrator training.
        #[arg(long, default_value_t = 0)]
        min_train_markets: usize,
        /// Meta-calibrator training epochs for walk-forward/portfolio training.
        #[arg(long, default_value_t = 24)]
        meta_epochs: usize,
        /// Meta-calibrator learning rate.
        #[arg(long, default_value = "0.04")]
        meta_learning_rate: f32,
        /// Meta-calibrator L2 decay applied on each update.
        #[arg(long, default_value = "0.001")]
        meta_l2: f32,
        /// Absolute clip for meta-calibrator weights and bias.
        #[arg(long, default_value = "1.50")]
        meta_weight_clip: f32,
        /// JSON cache for extracted meta-calibrator training samples.
        #[arg(long)]
        meta_training_samples_cache: Option<PathBuf>,
        /// Load a frozen meta-calibrator snapshot instead of training one.
        #[arg(long)]
        meta_calibrator_snapshot_in: Option<PathBuf>,
        /// Write the trained meta-calibrator snapshot to this path.
        #[arg(long)]
        meta_calibrator_snapshot_out: Option<PathBuf>,
        /// In portfolio mode, write partial outputs every N evaluated markets.
        /// Set to zero to disable.
        #[arg(long, default_value = "0")]
        portfolio_checkpoint_every_markets: usize,
        /// Per-market JSONL output.
        #[arg(long)]
        out_markets: Option<PathBuf>,
        /// Summary JSON output.
        #[arg(long)]
        out_summary: Option<PathBuf>,
    },
    /// Run a paper-mode replay on one market (historical tape + same execution stack as backtest).
    Paper {
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
        /// Write per-decision attribution rows (JSONL) to this path.
        #[arg(long)]
        decision_log: Option<PathBuf>,
        /// Only log every Nth decision event when writing `--decision-log`.
        #[arg(long, default_value = "1")]
        decision_log_every_n: usize,
        /// Read data from a local cache mirror instead of S3.
        #[arg(long)]
        local_cache_dir: Option<PathBuf>,
    },
    /// Run a live-mode replay on one market (historical tape + same execution stack for parity scaffolding).
    Live {
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
        /// Write per-decision attribution rows (JSONL) to this path.
        #[arg(long)]
        decision_log: Option<PathBuf>,
        /// Only log every Nth decision event when writing `--decision-log`.
        #[arg(long, default_value = "1")]
        decision_log_every_n: usize,
        /// Read data from a local cache mirror instead of S3.
        #[arg(long)]
        local_cache_dir: Option<PathBuf>,
    },
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
            local_cache_dir,
        } => {
            let channel: Channel = channel
                .parse()
                .map_err(|e: String| anyhow!("bad --channel: {e}"))?;
            inspect_s3(
                exchange,
                channel,
                date,
                asset_id,
                MarketId(market_id),
                head,
                local_cache_dir,
            )
            .await
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
            decision_log,
            decision_log_every_n,
            local_cache_dir,
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
                decision_log,
                decision_log_every_n,
                local_cache_dir,
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
            local_cache_dir,
        } => {
            let channel: Channel = channel
                .parse()
                .map_err(|e: String| anyhow!("bad --channel: {e}"))?;
            quotes_s3(
                exchange,
                channel,
                date,
                asset_id,
                slug,
                MarketId(market_id),
                head,
                local_cache_dir,
            )
            .await
        }
        Cmd::DiscoverDay {
            date,
            slug_prefix,
            max_concurrent,
            out,
        } => discover_day(date, slug_prefix, max_concurrent, out).await,
        Cmd::DiscoverRange {
            start_date,
            end_date,
            slug_prefix,
            max_concurrent,
            out,
        } => discover_range(start_date, end_date, slug_prefix, max_concurrent, out).await,
        Cmd::DiscoverMarketsParquet {
            markets_parquet,
            start_date,
            end_date,
            slug_prefix,
            require_book_s3,
            out,
        } => {
            discover_markets_parquet(
                markets_parquet,
                start_date,
                end_date,
                slug_prefix,
                require_book_s3,
                out,
            )
            .await
        }
        Cmd::PrepCache {
            markets,
            cache_dir,
            spot_symbol,
            max_concurrent,
            skip_existing,
        } => {
            prep_cache_cmd(
                markets,
                cache_dir,
                spot_symbol,
                max_concurrent,
                skip_existing,
            )
            .await
        }
        Cmd::WalkForward {
            markets,
            starting_cash,
            kelly_fraction,
            max_clip_usdc,
            max_order_clip_multiplier,
            max_per_market_exposure_usdc,
            spot_symbol,
            strategies,
            max_concurrent_fetches,
            use_outcome_label,
            portfolio_mode,
            volatility_regime_threshold,
            clip_fraction_of_equity,
            clip_drawdown_soft_pct,
            clip_drawdown_hard_pct,
            br2_late_clip_frac,
            br2_late_max_fires,
            br2_late_confirm_min_model_confidence,
            br2_late_confirm_max_model_risk,
            br2_late_confirm_min_model_side_p,
            br2_high_skew_clip_frac,
            br2_high_skew_max_clips,
            br2_late_favourite_threshold,
            br2_late_favourite_clip_frac,
            br2_late_favourite_max_clips,
            br2_late_favourite_sweep_depth,
            br2_late_favourite_min_model_confidence,
            br2_late_favourite_max_model_risk,
            br2_late_favourite_min_model_side_p,
            br2_late_favourite_min_model_edge,
            enforce_model_gate,
            model_gate_min_confidence,
            model_gate_max_risk,
            model_gate_min_edge,
            walk_forward_folds,
            fold_size,
            purge_markets,
            min_train_markets,
            meta_epochs,
            meta_learning_rate,
            meta_l2,
            meta_weight_clip,
            meta_training_samples_cache,
            meta_calibrator_snapshot_in,
            meta_calibrator_snapshot_out,
            portfolio_checkpoint_every_markets,
            local_cache_dir,
            out_markets,
            out_summary,
        } => {
            walk_forward(
                markets,
                starting_cash,
                kelly_fraction,
                max_clip_usdc,
                max_order_clip_multiplier,
                max_per_market_exposure_usdc,
                spot_symbol,
                strategies,
                max_concurrent_fetches,
                use_outcome_label,
                portfolio_mode,
                volatility_regime_threshold,
                clip_fraction_of_equity,
                clip_drawdown_soft_pct,
                clip_drawdown_hard_pct,
                br2_late_clip_frac,
                br2_late_max_fires,
                br2_late_confirm_min_model_confidence,
                br2_late_confirm_max_model_risk,
                br2_late_confirm_min_model_side_p,
                br2_high_skew_clip_frac,
                br2_high_skew_max_clips,
                br2_late_favourite_threshold,
                br2_late_favourite_clip_frac,
                br2_late_favourite_max_clips,
                br2_late_favourite_sweep_depth,
                br2_late_favourite_min_model_confidence,
                br2_late_favourite_max_model_risk,
                br2_late_favourite_min_model_side_p,
                br2_late_favourite_min_model_edge,
                enforce_model_gate,
                model_gate_min_confidence,
                model_gate_max_risk,
                model_gate_min_edge,
                walk_forward_folds,
                fold_size,
                purge_markets,
                min_train_markets,
                meta_epochs,
                meta_learning_rate,
                meta_l2,
                meta_weight_clip,
                meta_training_samples_cache,
                meta_calibrator_snapshot_in,
                meta_calibrator_snapshot_out,
                portfolio_checkpoint_every_markets,
                local_cache_dir,
                out_markets,
                out_summary,
            )
            .await
        }
        Cmd::Paper {
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
            decision_log,
            decision_log_every_n,
            local_cache_dir,
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
            run_market_backtest(
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
                decision_log,
                decision_log_every_n,
                local_cache_dir,
                MarketRunMode::Paper,
            )
            .await
        }
        Cmd::Live {
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
            decision_log,
            decision_log_every_n,
            local_cache_dir,
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
            run_market_backtest(
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
                decision_log,
                decision_log_every_n,
                local_cache_dir,
                MarketRunMode::Live,
            )
            .await
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
    let markets = discovery::discover_markets(&store, &date, &slug_prefix, max_concurrent).await?;
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
    println!(
        "discovered {} markets for {} -> {}",
        markets.len(),
        date,
        out.display()
    );
    Ok(())
}

async fn discover_range(
    start_date: String,
    end_date: String,
    slug_prefix: String,
    max_concurrent: usize,
    out: PathBuf,
) -> Result<()> {
    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .with_context(|| format!("parse --start-date {start_date}"))?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .with_context(|| format!("parse --end-date {end_date}"))?;
    if end < start {
        return Err(anyhow!("--end-date must be >= --start-date"));
    }

    let cfg = TelonexStoreConfig::from_env()?;
    let store = TelonexStore::try_new(&cfg)?;
    let mut all = Vec::new();
    let mut day = start;
    while day <= end {
        let date = day.format("%Y-%m-%d").to_string();
        let markets = discovery::discover_markets(&store, &date, &slug_prefix, max_concurrent)
            .await
            .with_context(|| format!("discover markets for {date}"))?;
        tracing::info!(
            date,
            markets = markets.len(),
            "range discovery day complete"
        );
        all.extend(markets);
        day = day
            .succ_opt()
            .ok_or_else(|| anyhow!("date overflow after {date}"))?;
    }
    all.sort_by_key(|m| m.close_ts);
    all.dedup_by(|a, b| a.asset_id == b.asset_id);

    let mut f = std::fs::File::create(&out).with_context(|| format!("create {}", out.display()))?;
    for m in &all {
        writeln!(f, "{}", serde_json::to_string(m)?)?;
    }
    tracing::info!(
        start = %start_date,
        end = %end_date,
        markets = all.len(),
        out = %out.display(),
        "range discovery complete"
    );
    println!(
        "discovered {} markets for {}..{} -> {}",
        all.len(),
        start_date,
        end_date,
        out.display()
    );
    Ok(())
}

async fn discover_markets_parquet(
    markets_parquet: PathBuf,
    start_date: String,
    end_date: String,
    slug_prefix: String,
    require_book_s3: bool,
    out: PathBuf,
) -> Result<()> {
    let start = NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .with_context(|| format!("parse --start-date {start_date}"))?;
    let end = NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .with_context(|| format!("parse --end-date {end_date}"))?;
    if end < start {
        return Err(anyhow!("--end-date must be >= --start-date"));
    }

    let file = File::open(&markets_parquet)
        .with_context(|| format!("open markets parquet {}", markets_parquet.display()))?;
    let builder = ParquetRecordBatchReaderBuilder::try_new(file)
        .with_context(|| format!("read parquet metadata {}", markets_parquet.display()))?;
    let mut reader = builder
        .with_batch_size(8192)
        .build()
        .context("build markets parquet reader")?;

    let mut all = Vec::new();
    for batch in &mut reader {
        let batch = batch.context("read markets parquet batch")?;
        append_market_rows_from_parquet(&batch, &slug_prefix, start, end, &mut all)?;
    }

    all.sort_by_key(|m| (m.close_ts, m.asset_id.clone()));
    all.dedup_by(|a, b| a.slug == b.slug);

    if require_book_s3 {
        let before = all.len();
        let cfg = TelonexStoreConfig::from_env()?;
        let store = TelonexStore::try_new(&cfg)?;
        let available = available_book_assets_by_date(&store, start, end).await?;
        all.retain(|m| {
            available
                .get(&m.date)
                .is_some_and(|assets| assets.contains(&m.asset_id))
        });
        tracing::info!(
            before,
            after = all.len(),
            "filtered parquet markets to cached book_snapshot_25 assets"
        );
    }

    let mut f = std::fs::File::create(&out).with_context(|| format!("create {}", out.display()))?;
    for m in &all {
        writeln!(f, "{}", serde_json::to_string(m)?)?;
    }
    println!(
        "discovered {} markets from {} for {}..{} -> {}",
        all.len(),
        markets_parquet.display(),
        start_date,
        end_date,
        out.display()
    );
    Ok(())
}

async fn available_book_assets_by_date(
    store: &TelonexStore,
    start: NaiveDate,
    end: NaiveDate,
) -> Result<HashMap<String, HashSet<String>>> {
    let mut out = HashMap::new();
    let mut day = start;
    while day <= end {
        let date = day.format("%Y-%m-%d").to_string();
        let assets = discovery::list_asset_ids_for_day(store, &date)
            .await
            .with_context(|| format!("list cached book assets for {date}"))?
            .into_iter()
            .collect::<HashSet<_>>();
        tracing::info!(date, assets = assets.len(), "cached book assets listed");
        out.insert(date.clone(), assets);
        day = day
            .succ_opt()
            .ok_or_else(|| anyhow!("date overflow after {date}"))?;
    }
    Ok(out)
}

fn append_market_rows_from_parquet(
    batch: &RecordBatch,
    slug_prefix: &str,
    start: NaiveDate,
    end: NaiveDate,
    out: &mut Vec<discovery::MarketHandle>,
) -> Result<()> {
    let slug = required_string_col(batch, "slug")?;
    let status = required_string_col(batch, "status")?;
    let result_id = required_string_col(batch, "result_id")?;
    let outcome_0 = required_string_col(batch, "outcome_0")?;
    let outcome_1 = required_string_col(batch, "outcome_1")?;
    let asset_id_0 = required_string_col(batch, "asset_id_0")?;
    let asset_id_1 = required_string_col(batch, "asset_id_1")?;
    let end_date_us = required_i64_col(batch, "end_date_us")?;

    for row in 0..batch.num_rows() {
        let Some(slug_value) = string_value(slug, row) else {
            continue;
        };
        if !slug_value.starts_with(slug_prefix) {
            continue;
        }
        if string_value(status, row) != Some("resolved") {
            continue;
        }
        let Some(start_ts) = discovery::parse_close_ts(slug_value) else {
            continue;
        };
        let Some(start_dt) = DateTime::from_timestamp(start_ts, 0) else {
            continue;
        };
        let date = start_dt.date_naive();
        if date < start || date > end {
            continue;
        }
        let Some((selected_idx, asset_id)) = canonical_up_asset_for_row(
            string_value(outcome_0, row),
            string_value(asset_id_0, row),
            string_value(outcome_1, row),
            string_value(asset_id_1, row),
        ) else {
            continue;
        };
        if asset_id.is_empty() {
            continue;
        }
        let outcome = match string_value(result_id, row) {
            Some("0") if selected_idx == 0 => "Up",
            Some("1") if selected_idx == 1 => "Up",
            Some("0" | "1") => "Down",
            _ => continue,
        };
        let close_ts = if end_date_us.is_valid(row) {
            end_date_us.value(row).div_euclid(1_000_000)
        } else {
            start_ts.saturating_add(300)
        };
        out.push(discovery::MarketHandle {
            asset_id: asset_id.to_string(),
            slug: slug_value.to_string(),
            close_ts,
            outcome: outcome.to_string(),
            date: date.to_string(),
        });
    }
    Ok(())
}

fn canonical_up_asset_for_row<'a>(
    outcome_0: Option<&str>,
    asset_id_0: Option<&'a str>,
    outcome_1: Option<&str>,
    asset_id_1: Option<&'a str>,
) -> Option<(u8, &'a str)> {
    if outcome_is_up(outcome_0) {
        asset_id_0.map(|asset| (0, asset))
    } else if outcome_is_up(outcome_1) {
        asset_id_1.map(|asset| (1, asset))
    } else {
        None
    }
}

fn outcome_is_up(outcome: Option<&str>) -> bool {
    outcome.is_some_and(|value| {
        value.eq_ignore_ascii_case("up")
            || value.eq_ignore_ascii_case("yes")
            || value.eq_ignore_ascii_case("above")
    })
}

fn required_string_col<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a StringArray> {
    let idx = batch
        .schema()
        .fields()
        .iter()
        .position(|field| field.name() == name)
        .ok_or_else(|| anyhow!("markets parquet missing column {name}"))?;
    batch
        .column(idx)
        .as_any()
        .downcast_ref::<StringArray>()
        .ok_or_else(|| anyhow!("markets parquet column {name} is not string"))
}

fn required_i64_col<'a>(batch: &'a RecordBatch, name: &str) -> Result<&'a Int64Array> {
    let idx = batch
        .schema()
        .fields()
        .iter()
        .position(|field| field.name() == name)
        .ok_or_else(|| anyhow!("markets parquet missing column {name}"))?;
    batch
        .column(idx)
        .as_any()
        .downcast_ref::<Int64Array>()
        .ok_or_else(|| anyhow!("markets parquet column {name} is not int64"))
}

fn string_value(array: &StringArray, row: usize) -> Option<&str> {
    array.is_valid(row).then(|| array.value(row))
}

#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
async fn walk_forward(
    markets_path: PathBuf,
    starting_cash: f64,
    kelly_fraction: f64,
    max_clip_usdc: f64,
    max_order_clip_multiplier: f64,
    max_per_market_exposure_usdc: f64,
    spot_symbol: String,
    strategies_csv: String,
    max_concurrent_fetches: usize,
    use_outcome_label: bool,
    portfolio_mode: bool,
    volatility_regime_threshold: f64,
    clip_fraction_of_equity: Option<f64>,
    clip_drawdown_soft_pct: f64,
    clip_drawdown_hard_pct: f64,
    br2_late_clip_frac: f32,
    br2_late_max_fires: usize,
    br2_late_confirm_min_model_confidence: f32,
    br2_late_confirm_max_model_risk: f32,
    br2_late_confirm_min_model_side_p: f32,
    br2_high_skew_clip_frac: f32,
    br2_high_skew_max_clips: usize,
    br2_late_favourite_threshold: f32,
    br2_late_favourite_clip_frac: f32,
    br2_late_favourite_max_clips: usize,
    br2_late_favourite_sweep_depth: usize,
    br2_late_favourite_min_model_confidence: f32,
    br2_late_favourite_max_model_risk: f32,
    br2_late_favourite_min_model_side_p: f32,
    br2_late_favourite_min_model_edge: f32,
    enforce_model_gate: bool,
    model_gate_min_confidence: f32,
    model_gate_max_risk: f32,
    model_gate_min_edge: f32,
    walk_forward_folds: Option<usize>,
    fold_size: Option<usize>,
    purge_markets: usize,
    min_train_markets: usize,
    meta_epochs: usize,
    meta_learning_rate: f32,
    meta_l2: f32,
    meta_weight_clip: f32,
    meta_training_samples_cache: Option<PathBuf>,
    meta_calibrator_snapshot_in: Option<PathBuf>,
    meta_calibrator_snapshot_out: Option<PathBuf>,
    portfolio_checkpoint_every_markets: usize,
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
    if walk_forward_folds.is_some() && fold_size.is_some() {
        return Err(anyhow!(
            "cannot set both --walk-forward-folds and --fold-size"
        ));
    }
    if walk_forward_folds == Some(0) {
        return Err(anyhow!("--walk-forward-folds must be >= 1"));
    }
    if fold_size == Some(0) {
        return Err(anyhow!("--fold-size must be >= 1"));
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
        max_order_clip_multiplier,
        max_per_market_exposure_usdc,
        spot_symbol,
        strategies,
        max_concurrent_fetches,
        use_outcome_label,
        maker_rebate_bps: 10.0,
        taker_fee_bps: 0.0,
        portfolio_mode,
        clip_fraction_of_equity,
        clip_drawdown_soft_pct,
        clip_drawdown_hard_pct,
        br2_late_clip_frac,
        br2_late_max_fires,
        br2_late_confirm_min_model_confidence,
        br2_late_confirm_max_model_risk,
        br2_late_confirm_min_model_side_p,
        br2_high_skew_clip_frac,
        br2_high_skew_max_clips,
        br2_late_favourite_threshold,
        br2_late_favourite_clip_frac,
        br2_late_favourite_max_clips,
        br2_late_favourite_sweep_depth,
        br2_late_favourite_min_model_confidence,
        br2_late_favourite_max_model_risk,
        br2_late_favourite_min_model_side_p,
        br2_late_favourite_min_model_edge,
        enforce_model_gate,
        model_gate_min_confidence,
        model_gate_max_risk,
        model_gate_min_edge,
        volatility_regime_threshold,
        walk_forward_folds,
        fold_size,
        purge_markets,
        min_train_markets,
        meta_training_config: MetaTrainingConfig {
            epochs: meta_epochs,
            learning_rate: meta_learning_rate,
            l2: meta_l2,
            weight_clip: meta_weight_clip,
            reset_before_fit: true,
        },
        meta_training_samples_cache,
        meta_calibrator_snapshot_in,
        meta_calibrator_snapshot_out,
        portfolio_checkpoint_every_markets,
        checkpoint_markets_out: out_markets.clone(),
        checkpoint_summary_out: out_summary.clone(),
    };

    tracing::info!(markets = markets.len(), "starting walk-forward");
    let started = Instant::now();
    let (results, summary) = run_walkforward(&store, &markets, &wf_cfg).await?;
    let elapsed = started.elapsed().as_secs_f64();
    tracing::info!(elapsed_s = elapsed, "walk-forward complete");

    print_summary(&summary);

    if let Some(p) = out_markets {
        write_market_results_jsonl_atomic(&p, &results)?;
        tracing::info!(?p, "wrote per-market results");
    }
    if let Some(p) = out_summary {
        write_summary_json_atomic(&p, &summary)?;
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
    local_cache_dir: Option<&PathBuf>,
) -> Result<(
    TelonexStore,
    Vec<pm_types::ReplayEvent>,
    pm_telonex_loader::LoadStats,
)> {
    let store = if let Some(cache_dir) = local_cache_dir {
        TelonexStore::try_new_local(cache_dir.clone())?
    } else {
        let cfg = TelonexStoreConfig::from_env()?;
        tracing::info!(bucket = %cfg.bucket, region = %cfg.region, "connecting to S3");
        TelonexStore::try_new(&cfg)?
    };

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
    if stats.out_of_order_rows > 0 {
        tracing::warn!(
            out_of_order = stats.out_of_order_rows,
            "rewound-timestamps in source tape; events re-sorted for determinism"
        );
    }
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
    local_cache_dir: Option<PathBuf>,
) -> Result<()> {
    let (store, events, stats) = fetch_tape(
        &exchange,
        channel,
        &date,
        &asset_id,
        market_id,
        local_cache_dir.as_ref(),
    )
    .await?;

    println!(
        "== s3://{}/raw/telonex/exchange={}/channel={}/date={}/asset_id={} ==",
        store.bucket, exchange, channel, date, asset_id
    );
    println!("batches         : {}", stats.batches);
    println!("rows_total      : {}", stats.rows_total);
    println!("rows_emitted    : {}", stats.rows_emitted);
    println!("rows_null_top   : {}", stats.rows_null_top);
    println!("rows_reordered  : {}", stats.out_of_order_rows);
    if let (Some(f), Some(l)) = (stats.first_ts_ns, stats.last_ts_ns) {
        let fdt = DateTime::<Utc>::from_timestamp_nanos(f);
        let ldt = DateTime::<Utc>::from_timestamp_nanos(l);
        println!(
            "ts_range_utc    : {} -> {}",
            fdt.to_rfc3339(),
            ldt.to_rfc3339()
        );
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
            if sp < min_spread {
                min_spread = sp;
            }
            if sp > max_spread {
                max_spread = sp;
            }
            sum_spread += sp as f64;
            if sp < 0.0 {
                crossed += 1;
            }
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
    decision_log: Option<PathBuf>,
    decision_log_every_n: usize,
    local_cache_dir: Option<PathBuf>,
) -> Result<()> {
    run_market_backtest(
        exchange,
        channel,
        date,
        asset_id,
        market_id,
        strategy,
        starting_cash,
        clip_shares,
        kelly_fraction,
        limits,
        close_ts_seconds,
        resolved_yes,
        spot_symbol,
        out,
        equity_curve,
        decision_log,
        decision_log_every_n,
        local_cache_dir,
        MarketRunMode::Backtest,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
async fn run_market_backtest(
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
    decision_log: Option<PathBuf>,
    decision_log_every_n: usize,
    local_cache_dir: Option<PathBuf>,
    mode: MarketRunMode,
) -> Result<()> {
    let (store, events, _stats) = fetch_tape(
        &exchange,
        channel,
        &date,
        &asset_id,
        market_id,
        local_cache_dir.as_ref(),
    )
    .await?;

    let spot_history = if spot_symbol.is_empty() {
        SpotHistory::default()
    } else {
        load_spot_history(&store, &spot_symbol, &date).await?
    };

    let market_close_ns = close_ts_seconds.saturating_mul(1_000_000_000);
    let market_run_mode = mode.as_str();
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
        decision_log_jsonl: decision_log,
        decision_log_parquet: None,
        shared_model_state: None,
        meta_calibrator_snapshot: None,
        decision_log_every_n,
        enforce_model_gate: true,
        model_gate_min_confidence: 0.68,
        model_gate_max_risk: 0.72,
        model_gate_min_edge: 0.00,
    };
    let trade_history = match resolve_pm_trades_day(&store, &date, &asset_id).await {
        Ok(path) => match load_pm_trades_async(store.store(), path).await {
            Ok((trades, stats)) => {
                tracing::info!(
                    rows = stats.rows_emitted,
                    buys = stats.buy_count,
                    sells = stats.sell_count,
                    "pm trades loaded"
                );
                TradeHistory::new(trades)
            }
            Err(e) => {
                tracing::warn!(error = %e, "pm trades load failed, defaulting to empty trade history");
                TradeHistory::default()
            }
        },
        Err(_) => TradeHistory::default(),
    };

    let started = Instant::now();
    let report = match strategy {
        StrategyKind::BuyYesAtOpen => {
            let mut s = BuyYesAtOpen::new(clip_shares);
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::ReactiveDirectional => {
            let mut s = build_reactive(starting_cash, kelly_fraction, max_clip_usdc);
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::PairedMm => {
            let mut s = PairedMmDense::new(PairedMmDenseConfig {
                clip_shares: max_clip_usdc / 0.5_f64.max(0.01),
                ..PairedMmDenseConfig::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::SpotMomentumFollower => {
            let mut s = SpotMomentumFollower::new(SpotMomentumFollowerConfig {
                clip_usdc: max_clip_usdc,
                ..SpotMomentumFollowerConfig::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::LateBigBet => {
            let mut s = LateBigBet::new(LateBigBetConfig {
                bankroll_usdc: starting_cash,
                max_clip_usdc,
                kelly_fraction,
                ..LateBigBetConfig::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::BonereaperLite => {
            let mut s = BonereaperLite::new(BonereaperLiteConfig {
                bankroll_usdc: starting_cash,
                max_clip_usdc,
                ..BonereaperLiteConfig::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::BonereaperV2 => {
            let mut s = BonereaperV2::new(BonereaperV2Config {
                bankroll_usdc: starting_cash,
                max_clip_usdc,
                ..BonereaperV2Config::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::DeltaNeutralMm => {
            let mut s = DeltaNeutralMm::new(DeltaNeutralMmConfig {
                clip_shares: (max_clip_usdc * 0.3).max(0.1),
                ..DeltaNeutralMmConfig::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::LateConfirmation => {
            let mut s = LateConfirmation::new(LateConfirmationConfig {
                bankroll_usdc: starting_cash,
                max_clip_usdc,
                ..LateConfirmationConfig::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
        StrategyKind::LateConvexTail => {
            let mut s = LateConvexTail::new(LateConvexTailConfig {
                bankroll_usdc: starting_cash,
                max_clip_usdc: max_clip_usdc * 0.2,
                ..LateConvexTailConfig::default()
            });
            run_backtest(&events, &spot_history, &trade_history, &mut s, &cfg)?
        }
    };
    tracing::info!(
        elapsed_ms = started.elapsed().as_millis() as u64,
        events = report.events_processed,
        mode = market_run_mode,
        ?strategy,
        "market run done"
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
        book_weight: 0.4,
        spot_weight: 0.6,
        shared_skew_table: None,
        ..ReactiveDirectionalConfig::default()
    };
    ReactiveDirectional::new(cfg)
}

/// Build a Nautilus-conformant symbol from a Polymarket slug. The dotted
/// venue suffix is added inside `polymarket_instrument_id`.
fn slug_to_nautilus_symbol(slug: &str) -> String {
    slug.to_uppercase()
}

async fn load_spot_history(store: &TelonexStore, symbol: &str, date: &str) -> Result<SpotHistory> {
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
    local_cache_dir: Option<PathBuf>,
) -> Result<()> {
    let (_store, events, stats) = fetch_tape(
        &exchange,
        channel,
        &date,
        &asset_id,
        market_id,
        local_cache_dir.as_ref(),
    )
    .await?;
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
        if line.trim().is_empty() {
            continue;
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parquet_discovery_selects_up_asset_from_either_outcome_slot() {
        assert_eq!(
            canonical_up_asset_for_row(Some("Up"), Some("asset0"), Some("Down"), Some("asset1")),
            Some((0, "asset0"))
        );
        assert_eq!(
            canonical_up_asset_for_row(Some("Down"), Some("asset0"), Some("Up"), Some("asset1")),
            Some((1, "asset1"))
        );
    }

    #[test]
    fn parquet_discovery_rejects_rows_without_canonical_up_side() {
        assert_eq!(
            canonical_up_asset_for_row(Some("No"), Some("asset0"), Some("Down"), Some("asset1")),
            None
        );
    }
}
