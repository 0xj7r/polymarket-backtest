# polymarket-backtest

Pure-Rust backtesting + paper + live framework for Polymarket BTC-5m markets, built on the NautilusTrader Rust crates.

## Workspace

```
crates/
├── pm-types/              # ReplayEvent, SpotTick/SpotHistory, TradeTick/TradeHistory, MarketId, BookLevel, PortfolioLimits
├── pm-telonex-loader/     # S3 streaming loaders + Nautilus QuoteTick conversion
│   ├── book_snapshot       (Polymarket book_snapshot_25)
│   ├── polymarket_trades   (Polymarket trades channel — aggressor flow)
│   ├── polymarket_onchain  (Polymarket onchain_fills — whale_net_flow)
│   ├── binance_trades      (Binance agg_trades — BTC spot)
│   ├── nautilus_conv       (ReplayEvent → nautilus_model::QuoteTick)
│   └── s3                  (object_store-based S3 client)
├── pm-model/              # canonical 4-score model + online meta-calibrator
├── pm-strategy/           # strategy candidates + execution lanes
│   ├── signals             (direction/confidence/calibrated_p/risk scores)
│   ├── regime              (BtcRegime: Flat/Whipsaw/DirectionalSmooth/TrendingVolatile)
│   ├── spot_momentum       (multi-TF weighted spot returns)
│   ├── reactive            (ReactiveDirectional — paired probes + dominant load)
│   ├── paired_mm           (PairedMmDense — unlawful-shear ladder)
│   ├── delta_neutral_mm    (DeltaNeutralMm — touch-tick spread capture)
│   ├── late_big_bet        (LateBigBet — single high-conviction last-60s bet)
│   ├── spot_follower       (SpotMomentumFollower — pure spot trend follower)
│   ├── bonereaper          (BonereaperLite — 4-lane composite)
│   └── trivial             (BuyYesAtOpen baseline)
├── pm-risk/               # Kelly sizing + PortfolioState (drawdown, daily/per-market caps)
└── pm-app/                # CLI: discover-day | inspect-s3 | backtest-s3 | quotes-s3 | walk-forward
```

## Quickstart

```bash
# Build
cargo build --release -p pm-app

# Authenticate to S3 (us-east-1, bucket pm-research-data-prod)
eval "$(AWS_PROFILE=visumlabs aws configure export-credentials --format env)"
export PM_TELONEX_REGION=us-east-1

# Discover markets for a day (one-off — the markets.parquet predicate-filter
# helper in scripts/ is faster; this hits the Telonex API).
./target/release/pm-app discover-day --date 2026-05-12 --out /tmp/markets.jsonl

# OR generate the markets list from the master parquet (recommended):
python3 -c "
import pyarrow.parquet as pq, json, datetime as dt
t = pq.ParquetFile('/tmp/markets-full.parquet').read(columns=['slug','outcome_0','outcome_1','asset_id_0','asset_id_1','status','result_id'])
df = t.to_pandas()
btc5 = df[df['slug'].str.startswith('btc-updown-5m-', na=False) & (df['status'] == 'resolved')].copy()
# ... see scripts/generate_markets_jsonl.py
"

# Run walk-forward across all 7 strategies
./target/release/pm-app walk-forward \\
    --markets /tmp/markets.jsonl \\
    --strategies "reactive_directional,paired_mm,delta_neutral_mm,late_big_bet,bonereaper_lite,spot_momentum_follower,buy_yes_at_open" \\
    --starting-cash 100 --max-clip-usdc 5 --spot-symbol BTCUSDT \\
    --use-outcome-label \\
    --out-markets /tmp/wf.jsonl --out-summary /tmp/wf-summary.json
```

For full AWS portfolio runs, sizing grids, and checkpointed monitoring, see
[docs/aws-backtest-runbook.md](docs/aws-backtest-runbook.md).

## Empirical findings (1-day, 288 BTC-5m markets, 2026-05-12)

| Strategy | Total P&L | Hit | Fills | Worst | Notes |
|---|---|---|---|---|---|
| **LateBigBet** | **+$141.50** | 35.8% | 181 | -$4.90 | Single high-conviction bet in last 60s; negated 15% trade-flow weight |
| ReactiveDirectional | +$106 | 29.1% | 786 | -$16.29 | Long-YES bias; edge tail-concentrated |
| BonereaperLite | -$2.29 | 55.2% | 572 | -$0.09 | 4-lane composite; nearly break-even |
| PairedMm | -$7.14 | 92.4% | 1,662 | -$0.45 | High hit rate, tail-eats-gains |
| DeltaNeutralMm | -$26.22 | 55.6% | 1,614 | -$4.74 | Inventory leaks; needs tighter cancel |
| SpotMomentumFollower | $0 | — | 0 | — | Threshold too tight |

**Key empirical truth**: aggressor flow (`trades.side`) on PM BTC-5m is **contra-indicating**.
- Positive 30% weight to trade flow: RD goes from +$106 → -$331
- Negated 15% weight: brings it within break-even
- Best LBB config uses negated 15% weight

## Architecture notes

- **All data lives in S3** (`s3://pm-research-data-prod`, us-east-1). The binary never writes a local cache — deploys cleanly to AWS without filesystem mounting.
- **Maker + taker matcher** with configurable rebate (bps), fee (bps), and resting-order book that fills when the market crosses the limit.
- **Per-market exposure cap** prevents a single bad market from blowing up daily P&L. Critical: too tight a cap kills the edge ($15 → -$19; $50 → +$106).
- **Inventory-imbalance cancellation** in the runner cancels resting orders on the heavy side once `|yes_shares - no_shares|` exceeds threshold.
- **Markets dataset discovery** filters 1.4M-row Polymarket markets parquet to BTC-5m subset with `result_id` for resolved outcomes — 39,649 markets across 138 days available.

## Tests

```bash
cargo test --workspace
```

41 tests across the workspace, covering: signal math, regime classification, calibrated probability bounds, runner matcher (taker + maker), per-market exposure cap, whale-flow accounting, ring buffer stats.

## Status

The framework is a working research/backtest engine, but not production-live
complete yet:
- ✅ S3 streaming for book / trades / spot / onchain
- ✅ Maker matcher with rebates + per-market cap + inventory cancel
- ✅ Walk-forward harness (parallel pipelining)
- ✅ Markets-dataset discovery (138 days, 39k+ resolved markets)
- ✅ 4-score model + walk-forward online meta-calibrator
- ✅ Strategy candidates, including BonereaperV2 late/favourite lanes
- ✅ Nautilus QuoteTick conversion (linked, validated)
- ✅ AWS portfolio-grid launcher with checkpointed result uploads

Remaining for true production:
- Larger historical walk-forward validation across completed Telonex backfills
- Paper/live execution adapter with the same strategy/model path
- Robust nightly retraining and promotion gates
- Whale-flow features only available on dates ≤ 2026-04-28 (archive boundary)
