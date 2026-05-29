# BTC5m Flow-Conditional Convex Tail (reversal-risk score)

Date: 2026-05-29
Status: validated on the drawdown slice (config B); full-history validation pending.

## What this is
A replay-safe, fill-time **reversal-risk score** built from Binance order-flow microstructure (the signals the prior reversal models never used), used to **target the convex_tail coverage and modulate directional sizing** — instead of paying convex premium blindly on every load.

## Why (the chain)
- Blanket convexity LOST on the recent window (premium drag on all loads; even deepened drawdown). It only hedged extreme favourites (>=85c) and missed the moderate 0.70-0.85 favourites that actually reverse.
- Discovery: Binance adverse aggressor volume / flow imbalance in the 30s pre-fill separates toxic crossed-mid loads (binance_adverse_vol_30s single-feature AUC 0.638).
- Phase 2 score: L2 logistic over {binance_adverse_vol_30s/15s, flow_imbal_30s, trade_intensity_15s, spot_accel_15s_vs_30s, spot_ret_30s, side_model_p, risk_score, price, side_edge_vs_fill}, target = crossed_mid_after_fill. Test AUC 0.7132. Top coef: binance_adverse_vol_30s +0.339. Frozen coeffs: `configs/reversal_score_coeffs.json`.

## Engine
- Phase 1: flow features computed at the load decision and logged per-fill (SpotHistory methods in `crates/pm-types/src/spot.rs`; BinanceFlowFeatures in `crates/pm-app/src/runner.rs`).
- Phase 3: `reversal_risk_score()` + modulators in `crates/pm-strategy/src/bonereaper_v2.rs`. score = sigmoid(intercept + sum coef*(x-mean)/std), HIGH = fragile.
  - Convex coverage = lerp(cov_min, cov_max, score) — convex only on fragile loads.
  - Directional size_mult = lerp(size_floor, size_ceiling, 1-score) — size down fragile, up stable.
- OFF by default (`reversal_score_enabled=false`, sentinel knobs) -> baseline byte-identical.

## Winning recipe (config B), local 4k May-drawdown slice
Flags: `--br2-reversal-score-enabled --br2-reversal-score-coeffs configs/reversal_score_coeffs.json --br2-reversal-score-cov-min 0.0 --br2-reversal-score-cov-max 1.0 --br2-reversal-score-size-floor 0.5 --br2-reversal-score-size-ceiling 1.0 --br2-tail-max-ask 0.30`

| cfg | PnL | maxDD | worst | tail fills | tail PnL |
|---|---|---|---|---|---|
| baseline | -102.41 | 17.1% | -49.7 | 13 | -5.2 |
| **B (target+widen+size-down fragile)** | **-89.69** | **14.0%** | **-41.5** | 110 | +0.8 |

Config B improves PnL (+$12.7), drawdown (-3.1pp), and worst-market (+$8) simultaneously, and flips the tail lane +EV. The decisive lever is **sizing down the flagged-fragile loads** (more than the hedge itself); **widening `tail_max_ask` to 0.30** is what lets convex reach the reversing moderate favourites (fills 13 -> 110). Sizing UP stable loads is counterproductive (do not).

## Honest limits
Config B does NOT flip the drawdown slice positive (-$89.69) and Sharpe stays negative — targeted convexity blunts the tail but doesn't overcome the directional losses on this slice.

## Next: full-history validation
Blanket convex lost -$106..-219 over the full window from premium drag. Targeted config B should AVOID that (pays only on fragile loads) while keeping the drawdown improvement -> the ship/no-ship number. Run config B vs baseline full-history (Feb->May) at $1k and $2,800.
