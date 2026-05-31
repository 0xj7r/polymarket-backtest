# BTC-5m Strategy Session Handoff (2026-05-30)

Self-contained brief for a fresh agent. The deep detail lives in the memory files
(`~/.claude/projects/-Users-jackreid-go-polymarket-backtest/memory/`:
`br2-loss-limiting-hedge-first.md`, `br2-future-maker-direction.md`, `signal-build-plan.md`)
and the `docs/` files referenced below.

## Objective
Maximize a PERSISTENT (out-of-sample-robust) risk-adjusted return for the br2 BTC-5m
directional-taker strategy. DRAWDOWN-TOLERANT: the user accepts some lower/drawdown weeks;
do NOT sacrifice overall return to flatten the drawdown. HARD DISCIPLINE (learned the hard way):
validate that an edge survives REALISTIC EXECUTION before building it.

## The base strategy (br2) — strong, already clears the bar
Authoritative full-history run = `98353` (S3 `pm-research-backtest-prod/results/20260529T205000Z-gapfill-baseline-98353`),
champion sizing, 23,705 eval markets Feb12-May20 INCLUDING the drawdown:
- **+$10,466 @ $2,800 (maxDD 15.4%) / +$8,990 @ $1k, daily Sharpe ~3.3.**
- Lane Sharpes: late_favourite 6.58 (durable core), high_skew 4.27, late_confirm 2.42 (volatile), convex_tail **-1.42 (a NET COST)**.
- Net is concentrated: top-10 days = 69% of net.

## The May drawdown — diagnosed (run 98353 + docs/btc5m_may_tranche_diagnosis.md)
-$596 over May7-20 against +$11k entering May; gradual, no cliff. NO data problem (coverage dense every day).
Mechanism: May is CALMER (spot vol 2.06->1.72), the low-vol regime disengages the vol/whipsaw gates so
participation COLLAPSES 12%->6%->2%, and the thin residual that still fires is toxic (crosses mid 54-56% vs 35-44%).
It is the structural crossed-mid tail UNMASKED by participation collapse in a calm/pinned regime. A THIN tail (~139 fills), not a broad regime.

## DEAD ENDS — proven, do NOT re-try (and WHY)
- **Week-level regime gate** (docs/btc5m_regime_conditional_gate.md): FAILS. The toxic drawdown weeks and the profitable trending weeks are ex-ante INDISTINGUISHABLE (same trailing YES-mid range ~0.81). Any gate fires ~100% and erases the +$7.2k directional profit to save a ~$547 drawdown. The local "+$5.54" was a May-only-slice artifact.
- **Fill-level flagging (spot+PM / Phase-1 signals)**: no separation. Phase-1 lead_* features (moneyness, whipsaw count, vol-accel, PM toxicity) are noise on the toxic target (AUC ~0.5), zero marginal lift over the existing reversal score (AUC 0.71); toxic loads interleaved with winners even within drawdown weeks.
- **Pinned-break taker lane**: no edge. Binance flow+spot has ~zero skill on the coinflip break (AUC 0.49-0.56, at/below the book's own implied prob); the book already prices it.
- **15m markets**: no trend edge. 15m bar autocorr NEGATIVE (-0.04), same-sign <0.50, mean-reverts MORE than 5m. Markets exist + engine handles them, but no momentum.
- **Momentum (any 5m)**: dead. 5m bar-to-bar is a coinflip (autocorr ~0).
- **PERP-CONFIRMATION GATE** (the big one): looked like THE win — gating late_favourite/late_confirm on side-signed perp momentum validated at full scale (OOS combined AUC 0.712 vs 0.630 spot+PM, +$8k OOS, fixes May, improves every month, 5:1 avoid-toxic:cut-winner). But the LATENCY-ROBUSTNESS pass KILLED it: the edge is a latency artifact of the clean 1m-kline CLOSE — it collapses by ~20-30s lag (+$10.8k at L=0 -> +$188 at 30s -> NEGATIVE at 60s). A live kline can't be fresher than ~30-60s (bar must close + ingest), past the break point; the fresh sub-minute aggTrade tape carries NO edge (-$49 at L=0). UNTRADEABLE LIVE. (Build plan docs/btc5m_perp_gate_build_plan.md exists but is SHELVED; reusable plumbing pattern only.)
- **Other signals (slow perp funding/OI; cross-asset spot)**: funding/OI latency-robust but NOT predictive (AUC ~0.52); cross-asset spot confirmation dies on latency like the perp.
- **Maker overlay** (calm-regime two-sided quoting): FAILS on current evidence. Adverse selection -7.7c/mkt eats +2.4c capture -> net -5.3c/mkt; the 10bps rebate adds only +0.16c. CALM is NOT lower-adverse-selection (toxic 52% calm vs 48% otherwise) — corrects the regime-complementary premise. ONLY matched-fill (zero net inventory) is positive (+2.0c/mkt); the entire loss is the STRANDED residual. So viable ONLY IF the whale's repair/anti-stranding logic keeps the residual tiny — the unproven IP that killed the user's own live bots; can't be validated offline. "build-to-validate", parked, NOT greenlit. Build plan: docs/btc5m_maker_overlay_build_plan.md.

## CONTESTED / IN-FLIGHT (re-checks of things the agents may have gotten wrong)
- **Real NO/DOWN order book — RESOLVED (it is a true exact mirror, NOT an artifact).** Re-verified rigorously from S3 (ad5b99): pulled the genuine DOWN-token `book_snapshot_25` directly (provenance proven byte-distinct from UP) across 40 markets / 39.4M ladder cells -> `down_bid_k == 1-up_ask_k` + matching sizes for 100.0000% of cells (max err ~1e-16), identical timestamps. Mechanical: Polymarket's CLOB shares one book across complementary tokens. So the real NO book genuinely = 1-yes, synthesized NO is exact, and there is ZERO independent two-sided signal. Consequence: no two-sided signal to engineer; the maker overlay simply uses synthesized NO (Gate 1 unnecessary, simplifies the build). (Caveat: conclusive for arb-pinned btc-updown-5m; less-liquid markets would need a separate check.)
- **Low-vol participation strategy** (running): the user's point — there must be a better way to participate in the low-vol regime where directional sits idle. Testing MEAN-REVERSION / FADE (the untested opposite of the dead pinned-break; motivated by the 56% intra-bar fade) + fair-value-deviation + sell-convexity, as plain takers (no maker/latency trap).

## WHAT SURVIVES (the honest answer so far)
1. **Run the base as-is** — already a strong persistent return (+$10.5k, Sharpe 3.3).
2. **Cross-market diversification** — the ONE growth lever with no execution-realism kill (the SAME proven strategy on uncorrelated assets). ETH/SOL/XRP/BNB/DOGE/HYPE each have ~26k 5m up/down markets in-window with full book coverage. Expected: portfolio Sharpe ~3.3 -> ~4.0 (2 assets) / ~4.8 (4) at rho~0.3 IF the edge replicates and correlations are low — BOTH MUST BE MEASURED. **STILL UNTESTED**: the ETH run failed on setup (see below). This is the #1 next action.

## THE #1 OPEN TASK: cross-market diversification is UNTESTED (ETH run FAILED on setup)
ETH run `56874` (`20260529T235243Z-fullhist-configB-56874`) FAILED, all setup, not strategy:
- train_only: `WALKFORWARD_EXIT=1`, "open markets file /opt/pm/markets-train.jsonl: No such file" — the ETH training markets file was never assembled/placed.
- baseline_eth5m: `EXIT=1`, "open meta-calibrator snapshot .../meta-calibrator-snapshot.json: No such file" — downstream of the failed train (no snapshot).
- configB_eth5m: `EXIT=2`, CLI usage error (passed a stale/wrong --br2 flag; the binary suggests `--br2-recent-regime-gate-enabled`).
FIX + RE-RUN: assemble the ETH-5m markets list correctly (eth-updown-5m slugs, spot ETHUSDT — the engine handles it via `discovery.rs::infer_spot_symbol_from_slug` + `--slug-prefix`; data confirmed: ETHUSDT spot + 26,254 eth-5m markets Feb18-May20 with full book coverage), train a FRESH ETH meta-snapshot, and use a flag set that matches the binary. Then measure: ETH standalone PnL/Sharpe/maxDD, the BTC-ETH weekly PnL CORRELATION, and the combined-portfolio Sharpe/maxDD. Then extend to SOL/XRP.

## DATA + INFRA
- Bucket `pm-research-data-prod` (us-east-1). HAVE: Binance BTC+ETH SPOT agg_trades; Polymarket `book_snapshot_25` AND `book_snapshot_full`, `quotes`, `trades`, `onchain_fills`, `dataset=markets`; all Feb12-May26. NOT in bucket (free on data.binance.vision): Binance perp/futures klines/aggTrades/OI/funding. NO Deribit / cross-exchange.
- `book_snapshot_25`/`_full` schema (flattened, numbered levels): `timestamp_us, local_timestamp_us, exchange, market_id, slug, asset_id, outcome, bid_price_0..N, bid_size_0..N, ask_price_0..N, ask_size_0..N` (prices/sizes are decimal STRINGS). CRUCIAL: each row carries BOTH `asset_id` AND `outcome`, so for one market (slug) the UP-token book and the DOWN-token book are SEPARATE rows/partitions distinguished by asset_id/outcome — the REAL NO book = the rows with the DOWN asset_id (independent, in S3, NOT synthesized 1-yes). `book_snapshot_full` gives deeper levels if 25 is insufficient. Use S3 directly (local cache `data/cache` is UP-only).
- Engine: pure RUST (pm-app) + Python for analysis. NEVER Go. The user's live maker engine `~/go/polymarket-agent` is also Rust.
- Committed (branch add-favourite-config): the flow-conditional reversal-score engine + the per-lane `lane_size_*` multipliers (`--br2-lane-size-late-favourite/_late-confirm/_high-skew`). Phase-1 lead_* signal features are in a worktree (`.claude/worktrees/agent-ac383082a054d3cea`), uncommitted.
- EC2 launcher `scripts/launch_ec2_fullhist_configB.sh`: defaults FIXED to champion sizing + cache + 1s replay (clip-fraction-of-equity 0.015, max-per-market-exposure-usdc 250, max-clip-usdc 30, kelly 0.5, exposure-frac 0.12, taker-latency-ms 500, replay-sample-ms 1000, use-local-cache 1). These were drifted footguns; a contaminated run (oversized) is the failure signature (baseline maxDD 47-76% instead of ~15-24%).

## GOTCHAS / DISCIPLINE
- AWS: `eval "$(aws configure export-credentials --profile visumlabs --format env)"`, `AWS_REGION=us-east-1` (the bucket is us-east-1; the profile default region differs).
- USE S3 DIRECTLY for any book analysis; the LOCAL cache `data/cache` is UP-token-ONLY (the source of the false NO-mirror artifact).
- VALIDATE EXECUTION REALISM BEFORE BUILDING: model latency (for fast signals) and adverse selection (for maker) up front — the perp gate and maker overlay both looked great in backtest and died on execution. A backtest using a clean-but-stale feature, or instantaneous maker fills, is a mirage.
- Baseline must reproduce ~+$9k/+$11k (run 98353) before trusting any comparison; if a baseline blows out (maxDD >40%), the sizing is wrong (drifted launcher defaults).
- Harness task-completion notifications fire SPURIOUSLY mid-run; verify true completion via the `STATUS`/`WALKFORWARD_EXIT=0` marker + a market-count guard, not the notification.
- zsh chokes on heredocs/for-loops — use python scripts. Local python 3.14 has a broken expat that crashes boto3 — use a 3.13 venv (e.g. /tmp/probe_venv).
- Resolution: the markets jsonl carries the true `outcome` field; the backtest resolves against it (`use_outcome_label`). The price-to-beat is the OPEN price (these are "close vs open" markets); the engine proxies the strike with Binance spot-at-open (fine; exact OpenPrice is only in Polymarket Gamma, not ingested).

## RECOMMENDED NEXT ACTIONS (priority order)
1. **Re-run cross-market diversification (ETH) properly** — the #1 surviving lever, currently untested due to the setup bug above. Then SOL/XRP. Measure standalone Sharpe + cross-asset correlation + combined portfolio Sharpe/maxDD.
2. Finish the **real-NO-book independence** re-check (S3 direct) — may reopen a two-sided signal + the maker premise.
3. Finish the **low-vol participation (mean-reversion)** screen.
4. If diversification validates, design the multi-asset PORTFOLIO mode + sizing at the user's ~$2,800 bankroll.
