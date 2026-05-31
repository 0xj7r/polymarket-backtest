# BTC-5m br2 Live Port Plan

Date: 2026-05-30. Scope: carry forward ONLY the BTC-5m base (the 062901 champion config + its meta-snapshot). ETH/15m/vol-floor are future per-asset strategies, not part of this port.

## The key de-risk: CRATE-REUSE IS FEASIBLE (verified)
`pm-types`, `pm-model`, `pm-risk`, `pm-strategy` (the br2 decision logic, bonereaper_v2.rs) have **ZERO nautilus dependencies**. So the live agent can consume them as **path/git deps** and run the EXACT SAME `BonereaperV2::on_event` logic live and in backtest. **Live == backtest is guaranteed at the code level** - the strategy is shared, only the live-feed ADAPTERS + execution wiring are new. Do NOT copy-paste the strategy code (it would diverge). Example Cargo change: polymarket-agent's workspace deps point pm-strategy/pm-types/pm-model/pm-risk at `../polymarket-backtest/crates/*`.

## Two host options (the open decision)
- **Option A - port into the EXISTING `~/go/polymarket-agent`.** Path-dep the br2 crates; reuse the agent's battle-tested production hardening (kill-switch, reconcile, capital guard, auto-redeem, singleton lock, paper-fill calibration) and its Binance-spot + Polymarket feeds + the TAKER order path. LOWER RISK, faster - the hard live-ops problems are already solved. New code = a feature-adapter + the snapshot/param loader + taker-order wiring.
- **Option B - a NEW nautilus-based agent** (e.g. `crates/pm-live` in the backtest repo) using `nautilus-live` + `nautilus-polymarket` (v0.57, already workspace deps). CLEANER (single repo, no cross-repo deps; Nautilus provides execution/risk/portfolio). REQUIRES a validated `nautilus-polymarket` LIVE IOC execution path against the real CLOB.
- **Decision rule:** if the user's nautilus testing already validated live Polymarket order submit/fill/cancel -> Option B (cleaner). If nautilus was data/backtest only -> Option A (faster, existing hardening). The adapter architecture is identical for both; only the execution runtime differs.

## What br2 needs from the live host (the interface)
At each decision instant, br2 consumes: a trailing `SpotHistory` (Binance BTC spot), the `BinanceFlowFeatures` (signed flow / adverse vol / momentum from the spot tape), the Polymarket YES-book top-of-book (NO = 1 - yes, confirmed exact), the meta side-model + reversal-risk score (from the frozen 062901 snapshot + coeffs), and per-market context (seconds-to-close, the open-price strike proxy = spot-at-open). It outputs TAKER orders (BuyYes/BuyNo with clip sizing). The backtest's replay + fill-model are NOT ported.

## Live-vs-backtest gaps to handle (either option)
- Feature windows are 5m-tuned + hardcoded (180s vol, etc.) - for BTC-5m these are CORRECT, carry as-is.
- The 500ms taker-latency assumption vs real measured latency - measure + match.
- Per-market lifecycle: 5-min markets open/close continuously - discovery + state per active market.
- Meta-model inference live (load the snapshot once; infer per decision).
- Position/inventory state + risk limits + a hard kill-switch.

## Phased sequence (each independently testable)
1. Crate-reuse wiring (path deps) + a live FEATURE-ADAPTER that feeds br2 from the live Binance-spot + PM-book feeds.
2. SHADOW / dry-run: br2 decides and LOGS only, no orders.
3. **EQUIVALENCE TEST (the safety gate):** replay the same telonex data through the live code path and DIFF br2's decisions vs the backtest. Prove live == backtest BEFORE any real order.
4. Execution wiring (taker order submit/fill/cancel) + risk limits + kill-switch.
5. Paper / small-size live; calibrate fill realism.
6. Scale to the ~$2,800 bankroll.

## Open question for the user
Which host (A vs B)? It hinges on what the existing "nautilus testing" actually covered - validated LIVE Polymarket execution, or data/backtest only?
