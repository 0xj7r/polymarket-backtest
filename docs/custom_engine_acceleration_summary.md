# Custom Backtesting Engine Acceleration Summary

Date: 2026-05-27

## Goal

Make the high-fidelity custom backtesting path (`runner.rs` + `walkforward.rs`) fast enough for large historical grids while preserving research fidelity: the same strategy logic, model gates, fill accounting, attribution, and portfolio-mode compounding.

The current direction is to keep the custom engine as the authoritative research path. Nautilus prototypes may remain useful for future scale experiments, but the custom path is the one used for current strategy iteration and PnL validation.

## Delivered

### Parallel loading and independent-market execution

- Raised default `--max-concurrent-fetches` from 16 to 64.
- Refactored independent walk-forward runs into two phases:
  - bounded async loading of book events, spot, and PM trades
  - Rayon parallel execution across loaded markets when `!portfolio_mode`
- Preserved serial `portfolio_mode`, because compounding requires chronological equity continuity.

### ReplayEvent disk cache

- Added `--replay-event-cache-dir <path>`.
- Cache format is JSONL, keyed by date and asset id.
- Added `Serialize`, `Deserialize`, and `PartialEq` support for replay tape types.
- Cache writes are explicit opt-in to avoid duplicating large local caches on machines with limited disk.
- Use this on AWS or large disks for repeated sweeps over the same market list.
- The cache is used by independent runs, portfolio-mode runs, and meta-calibration sample collection. Cached events rebind `market_id` to the current run so attribution remains deterministic when a cache is reused with different market slices.

### Runner hot-path cleanup

- Split resting orders into direction-grouped vectors:
  - `BuyYes`/`SellNo`
  - `SellYes`/`BuyNo`
- Skips resting-order scans when the relevant side is empty.
- Added capacity reserves for resting order and fill vectors.
- Removed debug-format allocation in maker fill side labeling.

### Strategy/profile workflow

- Added `--profile <path>` for walk-forward runs.
- Added example profiles:
  - `configs/bonereaper_v2_leader.toml`
  - `configs/bonereaper_v2_smoke.toml`
- Profile files use a `[bonereaper_v2]` section and optional fields.
- Missing fields do not zero out defaults.
- Profile values override CLI/default values for fields present in the profile.
- A `run_manifest.json` is written next to `summary.json` when a profile is used.

### BonereaperV2 sizing

- Added continuous risk-based sizing to high-skew and late-favourite lanes:
  - higher model `risk_score` reduces emitted size
  - this turns BTC regime risk into proportional sizing, not just hard gating

## Verification

Validated after cleanup:

```bash
cargo fmt --all --check
cargo check -q --all-targets
cargo test -q
```

Profile smoke run:

```bash
cargo run -q -p pm-app -- walk-forward \
  --markets /tmp/pm-local-smoke/markets-2026-05-05-head20.jsonl \
  --local-cache-dir data/cache \
  --profile configs/bonereaper_v2_smoke.toml \
  --strategies bonereaper_v2 \
  --portfolio-mode \
  --use-outcome-label \
  --disable-pm-trades \
  --max-markets 3 \
  --starting-cash 1000 \
  --clip-fraction-of-equity 0.03 \
  --max-clip-usdc 30 \
  --max-order-clip-multiplier 10 \
  --max-per-market-exposure-frac 0.25 \
  --disable-model-gate \
  --disable-meta-calibration \
  --out-markets /tmp/pm-refactor-smoke/markets.jsonl \
  --out-summary /tmp/pm-refactor-smoke/summary.json
```

Observed:

- 3 markets attempted, 3 succeeded
- profile values appeared in `summary.run_config`
- `run_manifest.json` was written
- no replay-event cache was auto-created

## Usage

Recommended AWS repeated-sweep shape:

```bash
cargo run --release -p pm-app -- walk-forward \
  --markets /opt/pm/markets.jsonl \
  --local-cache-dir /data/cache \
  --replay-event-cache-dir /data/replay-event-cache \
  --profile configs/bonereaper_v2_leader.toml \
  --strategies bonereaper_v2 \
  --portfolio-mode \
  --use-outcome-label \
  --starting-cash 1000 \
  --clip-fraction-of-equity 0.03 \
  --max-clip-usdc 30 \
  --max-order-clip-multiplier 10 \
  --max-per-market-exposure-frac 0.25 \
  --out-markets /opt/pm/results/run/markets.jsonl \
  --out-summary /opt/pm/results/run/summary.json
```

For local smoke/debug runs on this Mac, avoid `--replay-event-cache-dir` unless there is enough free disk.

## Current caveats

- `--profile` currently applies profile values last. Do not combine a profile with CLI overrides for the same fields unless the profile should win.
- Portfolio mode remains serial by design.
- Spot loading still dominates tiny local smoke runs.
- The latest acceleration and profile changes need a larger historical rerun before we can rank the new profile against the previous leader.

## Next

1. Run larger local smoke if disk allows.
2. Deploy latest code to AWS once SSH is reachable.
3. Rerun the prior leader profile over the larger historical set.
4. Sweep BTC regime risk weights and risk-size floors against the previous leader.
