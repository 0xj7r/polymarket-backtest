# BTC5m Regime-Conditional Lane Gate (br2)

## Objective

Amputate the two failing directional lanes (`late_favourite` + `late_confirm`)
ONLY when an ex-ante regime detector says we are in the whippy / reversing
regime, while leaving FULL br2 (all lanes at size 1.0) in trending regimes. The
`high_skew` lane is always left at 1.0 (it is the profit carrier; see the
post-fill diagnostics).

The bar:
- retain br2's strong full-history profit (baseline ~+$7.2k @ $1k, +$9.7k @ $2,800,
  Sharpe ~3.3 across the Feb-May markets), AND
- flip the recent (May) drawdown weeks positive / near-flat. The amputation
  proof-of-concept (config H0: both directional lanes hard-disabled) flips the
  4k May slice from -$79.21 to +$5.54.

## Ex-ante signals (lookahead audit)

The hard correctness rule: the regime trigger may use ONLY information available
at the load decision instant.

USED (ex-ante, no lookahead):
- `ctx.prior_market_range_{1,3,7}d` (PRIMARY). This is the trailing mean of the
  per-market YES-mid range (`max(yes_mid) - min(yes_mid)`) over the last N
  already-CLOSED BTC 5m markets, computed in
  `walkforward.rs::prior_market_range_mean(&results, N)` where `results` only
  ever contains markets that have already resolved. `crates/pm-strategy/src/lib.rs`
  documents the field as "live-safe in portfolio replay because they never
  include the current market." A multi-day drawdown is a persistent regime, so a
  trailing range estimate captures it without any within-market lookahead. This
  is the cleanest robustly-ex-ante detector.
- `WhipsawRiskSnapshot.score` (OPTIONAL blend). Computed in `regime.rs`
  `from_history(now_ns, spot)` strictly from the spot price path in the trailing
  180s window UP TO `now_ns` (`spot.range(start_ns, now_ns)` and
  `price_at_or_before(next_ns <= now_ns)`). It is therefore ex-ante. It is
  available as a blend term (`regime_gate_whipsaw_weight`) but defaults to weight
  0 (trailing-range only).

REJECTED (lookahead):
- `post_fill_cross_mid_rate`: a post-fill / per-market OUTCOME aggregate. Not
  available at decision time. It was only ever a diagnostic and is NOT used as a
  live trigger.

## Gate design

`BonereaperV2::regime_gate_lane_mult(ctx, whipsaw_score)` returns a multiplier in
`[lane_floor, 1.0]` that is multiplied into the `late_favourite` and
`late_confirm` lane clips (high_skew untouched):

1. If `regime_gate_enabled == false` => return 1.0 (byte-identical baseline).
2. Read the trailing range for the configured window
   (`regime_gate_window` in {1,3,7} => prior_market_range_{1,3,7}d).
3. Warmup (no prior-market history, trailing == 0) => return 1.0 (full size).
4. Blend with the live whipsaw score:
   `score = (1-w)*trailing + w*(whipsaw_score*threshold)`, `w = regime_gate_whipsaw_weight`.
5. If `score >= threshold` => return `regime_gate_lane_floor` (whippy => amputate).
6. Else if a soft band is configured, ramp linearly from 1.0 at
   `threshold - soft_band` down to the floor at `threshold`.
7. Else => 1.0 (trending => full br2).

Config (all INERT by default; disabled => byte-identical to baseline, proven by
`regime_gate_disabled_is_byte_identical_to_baseline`):

| field | default | meaning |
|---|---|---|
| `regime_gate_enabled` | false | master switch |
| `regime_gate_window` | 3 | trailing window in days (1/3/7) |
| `regime_gate_threshold` | 0.50 | trailing-range trigger |
| `regime_gate_soft_band` | 0.0 | linear ramp width below threshold |
| `regime_gate_lane_floor` | 0.0 | directional lane mult when whippy |
| `regime_gate_whipsaw_weight` | 0.0 | blend weight for live whipsaw score |

CLI: `--br2-regime-gate-enabled`, `--br2-regime-gate-window`,
`--br2-regime-gate-threshold`, `--br2-regime-gate-soft-band`,
`--br2-regime-gate-lane-floor`, `--br2-regime-gate-whipsaw-weight`.

## Calibration (local, May drawdown slice)

`/tmp/recent_slice_4k.jsonl` = May 7-20 (the drawdown), 4,000 markets. On this
slice the trailing-3d mean range sits at ~0.58-0.82 (per-market range median
~0.92): a deeply whippy regime. A threshold of 0.50 therefore fires on
essentially the entire slice.

Chosen calibration: `window=3` (3-day trailing), `threshold=0.50`,
`lane_floor=0.0` (full amputation), `whipsaw_weight=0.0` (trailing-range only),
`soft_band=0.10` for the production run (the local sweep used a hard step;
the soft band is a safety ramp that only matters near the boundary which the May
slice never sits at).

Local run sizing matched the `run_local_E` sandbox (clip-fraction 0.015, gross
250, max-clip 30, kelly 0.5) reusing the 062901 snapshot. Same 4,000 May markets,
baseline (gate disabled) vs regime-gated:

| metric | baseline | regime_gated | delta |
|---|---|---|---|
| total PnL | -$79.21 | +$5.54 | +$84.75 |
| maxDD | 12.00% | 0.95% | -11.05pp |
| worst market | -$33.80 | -$3.97 | +$29.83 |
| Sharpe (per-market x sqrt n) | -0.939 | +0.471 | +1.410 |

Weekly PnL (baseline -> regime_gated):
- 2026-W19: -32.27 -> +4.09
- 2026-W20: -37.29 -> -7.49
- 2026-W21:  -9.65 -> +8.93

The regime-gated arm hits +$5.54, exactly reproducing the config-H0 full-lane
amputation proof-of-concept. The gate fire-rate on the May slice is 100%: the
entire slice is the whippy regime (trailing-3d range 0.58-0.82, well above the
0.50 threshold), so the gate correctly amputates throughout. The
baseline run reproduces the prior config-E baseline of -$79.21 exactly,
confirming reproducibility.

Whether the gate stays INERT in trending regimes (the other half of the bar) is
not testable on the local cache (it only holds May 7-20). That is what the
full-history EC2 run validates.

## Full-history signal analysis (the decisive finding)

The local slice contains ONLY the drawdown regime, so a 100% fire-rate there
looks like a win. On the full Feb-May history (champion run 98353: baseline
+$9,027 @ $1k / +$11,058 @ $2,800, maxDD 23.7%/15.4%, Sharpe ~3.3, 21,750
markets W09-W20), the picture inverts.

### 1. The trailing-range signal cannot separate the regimes

Weekly mean trailing-3d YES-mid range vs realized directional-lane PnL:

| week | trailing-3d range | late_favourite PnL | late_confirm PnL |
|---|---|---|---|
| W10 | 0.681 | +657 | +885 |
| W12 | 0.667 | +771 | +1062 |
| W14 | 0.814 | +823 | +39 |
| W15 | 0.813 | +687 | -309 |
| W16 | 0.809 | +1010 | +490 |
| W17 | 0.803 | -10 | +81 |
| W18 | 0.818 | -141 | -255 |
| W19 | 0.829 | -331 | +292 |
| W20 | 0.812 | +12 | -81 |

The profitable trending weeks W14-W16 (directional lanes +$2.7k) have trailing
range ~0.81 — indistinguishable from the drawdown weeks W18-W20 (~0.82). A
threshold of 0.50 fires on 100% of markets; even 0.80 fires 58% and still catches
W14-W16. **The trailing-range detector cannot discriminate high-range-profitable
from high-range-toxic, so the configured gate (threshold 0.50) amputates the
directional lanes across essentially the whole history**, destroying the ~$7.2k
those lanes earn (late_favourite +$4,544, late_confirm +$2,643 over the full run).

### 2. A trailing realized-directional-PnL signal discriminates the weeks but
still fails the bar

A genuinely different ex-ante signal — the trailing mean realized PnL of the
directional lanes over the last N closed markets — does separate the regimes by
sign (positive +0.1..+0.86 in W09-W16, negative/flat -0.09..+0.00 in W17-W20).
But an offline counterfactual (amputate directional lanes when this trailing
signal < threshold) does NOT clear the bar either:

| N (markets) | thr | gated total | fire-rate | W18-20 PnL (base -547) |
|---|---|---|---|---|
| 288 | 0.0 | +$5,583 | 34% | -$464 |
| 288 | 0.10 | +$4,869 | 45% | -$739 |
| 864 | 0.0 | +$6,648 | 30% | -$1,118 |

It costs $2.4k-$4.2k of total profit and does NOT make the drawdown weeks
positive (it makes them flat-to-worse), because within W18-W20 the directional
lanes have interleaved winners and losers that look identical ex-ante; a trailing
signal removes winners too and lags the regime turn.

### 3. Why amputation is structurally the wrong tool here

At champion sizing the W18-W20 drawdown is only ~-$547 of directional-lane loss
set against +$7.2k of directional profit elsewhere. No ex-ante regime label
isolates that -$547 cleanly, so any lane-level amputation gives up multiples more
trending profit than it saves.

## EC2 validation

Clean re-run launched as `20260529T231822Z-fullhist-configB-85204` (r7i.4xlarge,
local cache), arms = baseline (PURE 062901 champion manifest) vs regime_gated
(= baseline + gate only), champion sizing passed explicitly (max-clip 30, mult
10, kelly 0.5, exposure-frac 0.12, clip-frac 0.015, gross 250, drawdown
0.2/0.4/0.1, replay-sample-ms 1000, latency 500, fetches 8), markets-key
favourite-062901, snapshot 062901 reuse. The baseline arm is the mandatory
reproduction check against 98353 (~+$9k/+$11k). The regime_gated arm is expected
to confirm the offline finding (range gate fires ~100% and guts the trending
profit).

## Verdict

The production plumbing, the ex-ante gate mechanism, and the inert-by-default
safety are all in place and tested (byte-identical when disabled). But the
calibrated detector does NOT meet the bar on the corrected full history:

- The trailing YES-mid-range signal (`prior_market_range_*`) cannot separate the
  profitable high-range trending weeks from the losing high-range whippy weeks;
  at threshold 0.50 it fires on ~100% of markets and would erase the ~$7.2k of
  directional-lane profit to avoid a ~$0.5k drawdown.
- The local-slice "success" (+$5.54, reproducing config H0) was an artifact:
  the May-only slice is entirely the whippy regime, so a 100% fire-rate there has
  no trending profit to lose. It is not representative of the full history.
- A better ex-ante signal (trailing realized directional-lane PnL) discriminates
  the regimes by sign but still costs $2.4k-$4.2k and does not flip the drawdown
  weeks positive, because the toxic markets are interleaved with winners that are
  ex-ante indistinguishable.

Honest conclusion: lane-level amputation, gated by any trailing regime estimate I
could construct ex-ante, does not retain ~baseline trending profit while flipping
the drawdown weeks positive. The drawdown is too small and too interleaved with
profit for a blunt lane on/off gate. The convex-tail hedge path (see
br2-loss-limiting memory) remains the more promising direction for the recent
drawdown: it pays only on flagged-fragile loads rather than removing profitable
exposure wholesale.

The gate ships INERT by default (byte-identical baseline), so it is safe to keep
in-tree as scaffolding, but it should NOT be enabled at threshold 0.50 (or any
range threshold) in production on this strategy.
