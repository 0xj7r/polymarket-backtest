# Active BTC 5m Experiments

Last updated: 2026-05-28 20:07 UTC.

Scope for this lane is BTC 5m only. Multi-market BTC/ETH and 15m/1h expansion is
paused until the BTC 5m engine has a clean full-history profile.

## Current Verified Leader

Run: `20260528T145725Z-portfolio-grid-24140`

Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_vol1p25`

Configuration:

- Starting capital: `$1,000`
- Market family: BTC 5m only
- Markets: `23,705`
- Tails: disabled

Result:

- PnL: `+$8,823.43`
- Return: `+882.34%`
- Max drawdown: `31.14%`
- Fills: `6,010`
- Fill log loss: `0.5458`

Attribution:

- `br2_late_favourite_load`: `+$4,054.44`
- `br2_late_confirm`: `+$3,092.26`
- `br2_high_skew_load`: `+$1,676.73`

This is the benchmark to reproduce at `$5,000` starting capital before live
paper deployment. It does not include convex tail coverage.

## Active 5K Runs

All runs below use:

- Starting capital: `$5,000`
- Market family: BTC 5m only
- Date range: `2026-02-12` to `2026-05-27`
- Training window: first `4,500` markets
- Gross cap: `$1,250`
- Exposure fraction cap: `0.12`
- Clip fraction: `0.015`
- Kelly fraction: `0.5`
- Taker latency: `500ms`

### Scaled No-Tail Reproduction

Run: `20260528T185235Z-portfolio-grid-79610`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_exact_leader_scaled_notails`

Purpose: reproduce the old BTC5m no-tail leader at `$5,000` starting capital.

Latest checkpoint observed:

- Markets: `8,250`
- PnL: `+$13,954.45`
- Return: `+279.09%`
- Max drawdown: `27.57%`
- Fills: `3,695`
- Markets with orders: `1,821`
- Fill log loss: `0.5354`

Latest attribution:

- `br2_late_favourite_load`: `+$5,188.66`
- `br2_high_skew_load`: `+$1,621.74`
- `br2_late_confirm`: `+$7,144.05`

Interpretation: the scaled no-tail reproduction has recovered strongly and is
currently the clean active benchmark. It still has no convex tail insurance.

### Pre-Fix Tail Variant

Run: `20260528T185359Z-portfolio-grid-82390`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_exact_leader_scaled_tail`

Purpose: tail-enabled comparison launched before the directional-tail anchoring
fix.

Status: stopped after preserving artifacts to S3. This run is not promotable
because it predates the tail anchoring fix, but its partial checkpoint remains
useful for rough shape comparison.

Final preserved checkpoint before stop:

- Markets: `2,500`
- PnL: `+$1,968.64`
- Return: `+39.37%`
- Max drawdown: `27.33%`

Earlier attribution at `500` markets:

- `br2_late_favourite_load`: `+$475.32`
- `br2_high_skew_load`: `+$24.14`
- `br2_late_confirm`: `-$739.99`
- `br2_convex_tail`: `-$50.53`

Interpretation: early drag is still mostly `late_confirm`, not convex tails.

### Fixed Directional-Tail Variant

Run: `20260528T190005Z-portfolio-grid-94612`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_directional_tail_fix`

Purpose: same tail variant after commit `df968bcf`, where convex tails are
anchored to all directional exposure rather than only `late_favourite_load`.

Latest checkpoint observed:

- Markets: `7,500`
- PnL: `+$11,881.35`
- Return: `+237.63%`
- Max drawdown: `26.73%`
- Fills: `3,969`
- Markets with orders: `1,715`
- Fill log loss: `0.5026`

Attribution:

- `br2_late_confirm`: `+$7,229.11`
- `br2_late_favourite_load`: `+$4,536.35`
- `br2_high_skew_load`: `+$1,106.83`
- `br2_convex_tail`: `-$990.93`

Comparable checkpoint versus no-tail:

- Through the first `7,250` markets, fixed-tail was `-$1,127.07` behind no-tail:
  `+$10,934.62` versus `+$12,061.70`.
- Directional lanes were roughly comparable; the gap was mostly convex-tail
  insurance spend (`br2_convex_tail = -$793.26` at the `7,250` checkpoint).
- Max drawdown was slightly lower with tails (`26.73%` versus `27.57%`).

Tail price buckets through the latest downloaded fixed-tail market file:

- `4-6c`: `25` fills, `8.0%` hit rate, `+$101.62` raw binary settlement PnL
- `6-8c`: `83` fills, `4.8%` hit rate, `-$74.66` raw binary settlement PnL
- `8-10c`: `334` fills, `5.7%` hit rate, `-$1,107.79` raw binary settlement PnL

Interpretation: cheap tails should be judged as portfolio insurance, not as a
standalone alpha lane. The current evidence says the insurance cost is
acceptable only if it reduces path damage in later reversal regimes. So far it
slightly reduces max drawdown, but `8-10c` dominates the bleed. A future
candidate should keep convex coverage but test `tail_max_ask = 0.08` or a much
smaller 8-10c size, rather than disabling tails outright.

### Late-Confirm Range-Gated Variant

Run: `20260528T191002Z-portfolio-grid-14707`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail_fix_lc_range50`

Purpose: test the post-hoc failure isolation from the 750-market no-tail
checkpoint using a clean forward replay.

Status: stopped after preserving artifacts because it was launched with
`replay_sample_ms = 0`, while the leader comparisons use `replay_sample_ms =
1000`. The early checkpoint is useful only as a non-comparable smoke result.

Only intentional strategy change versus the fixed-tail run:

```text
--br2-late-confirm-max-observed-range 0.50
```

Hypothesis:

- `late_confirm` is profitable only before the market has already traversed a
  large observed YES range.
- Above `0.50` observed range, the lane is often chasing reversal-prone
  dislocations where the model reports high edge but realized payoff is poor.
- The gate should reduce reversion losses without touching profitable
  `late_favourite_load` or `high_skew_load` exposure.

Evidence from the 750-market no-tail checkpoint:

- Baseline checkpoint PnL: `-$256.10`
- Non-`late_confirm` PnL: `+$452.12`
- `late_confirm` PnL: `-$708.22`
- Post-hoc replay of existing fills with `late_confirm` range capped at `0.50`
  estimated total PnL around `+$1,034` on that same checkpoint.

This post-hoc result is a hypothesis generator only. Promotion requires a clean
full-history replay with the gate applied during order generation.

Latest checkpoint observed:

- Markets: `250`
- PnL: `-$162.17`
- Return: `-3.24%`
- Max drawdown: `4.57%`
- Fills: `11`
- Markets with orders: `8`

Interpretation: this blunt `late_confirm` range gate materially reduced early
loss versus the no-tail/tail baselines over the first `250` markets, but it also
starved participation. Because the replay cadence differed from the leaders,
promotion requires the corrected `sample1000` relaunch below.

Corrected relaunch attempt:

- Run: `20260528T200616Z-portfolio-grid-23412`
- Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail_fix_lc_range50_sample1000`
- Replay sample: `1000ms`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Meta retraining: forbidden
- Status: stopped. The leader artifacts had not been uploaded yet, so the
  instance failed on a missing `meta-calibrator-snapshot.json`.

Corrected relaunch retry:

- Run: `20260528T201231Z-portfolio-grid-34818`
- Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail_fix_lc_range50_sample1000_retry`
- Replay sample: `1000ms`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610` after
  manually uploading active leader artifacts
- Meta retraining: forbidden
- Status: stopped. This run was still not apples-to-apples: it differed from
  the leader on late-favourite edge/range throttles and realized-vol gates,
  which inflated participation to `53.5%` by the `750`-market checkpoint.

First corrected checkpoint:

- Markets: `250`
- PnL: `+$1,292.74`
- Return: `+25.85%`
- Max drawdown: `17.12%`
- Fills: `430`
- Active markets: `48.0%`
- Attribution:
  - `br2_late_favourite_load`: `+$940.62`
  - `br2_high_skew_load`: `+$328.91`
  - `br2_late_confirm`: `+$86.45`
  - `br2_convex_tail`: `-$63.24`

Interpretation: discard as config-contaminated. The first `250` markets looked
strong, but the `750`-market checkpoint fell to `-$1,020.65` with `44.26%` max
drawdown because unrelated late-favourite gates were looser than the leader.

Exact-profile relaunch:

- Run: `20260528T202609Z-portfolio-grid-60941`
- Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail_fix_lc_range50_exact`
- Replay sample: `1000ms`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Meta retraining: forbidden
- Only intentional strategy change versus fixed-tail leader:
  `br2_late_confirm_max_observed_range = 0.50`
- Status: active on instance `i-0a3e70b4634752994`

First exact-profile checkpoint:

- Markets: `250`
- PnL: `+$218.45`
- Return: `+4.37%`
- Max drawdown: `3.05%`
- Fills: `35`
- Active markets: `8.0%`
- Attribution:
  - `br2_late_favourite_load`: `+$137.42`
  - `br2_late_confirm`: `+$47.03`
  - `br2_high_skew_load`: `+$38.25`
  - `br2_convex_tail`: `-$4.24`

Config verification:

```bash
python3 scripts/compare_run_configs.py \
  s3://pm-research-backtest-prod/results/20260528T190005Z-portfolio-grid-94612/clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_directional_tail_fix/summary.json \
  s3://pm-research-backtest-prod/results/20260528T202609Z-portfolio-grid-60941/clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail_fix_lc_range50_exact/summary.json \
  --aws-profile visumlabs \
  --allow br2_late_confirm_max_observed_range
```

Result: configs match outside the allowed range-gate key.

### No-Tail Late-Confirm Range-Gated Isolation

Run: `20260528T192139Z-portfolio-grid-37354`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_notail_lc_range50`

Purpose: isolate the `late_confirm_max_observed_range = 0.50` change without
convex tails. This should be compared directly against the scaled no-tail
reproduction run.

Status: stopped after preserving artifacts because it shared the same
`replay_sample_ms = 0` comparability issue as the tail range-gated variant.

Only intentional strategy changes versus the scaled no-tail reproduction:

```text
--br2-late-confirm-max-observed-range 0.50
--br2-tail-clip-frac 0.0
--br2-tail-max-clips 0
```

Corrected relaunch attempt:

- Run: `20260528T200646Z-portfolio-grid-24376`
- Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_notail_lc_range50_sample1000`
- Replay sample: `1000ms`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Meta retraining: forbidden
- Status: stopped for the same missing-snapshot issue as the tail corrected
  relaunch attempt.

Corrected relaunch retry:

- Run: `20260528T201244Z-portfolio-grid-35890`
- Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_notail_lc_range50_sample1000_retry`
- Replay sample: `1000ms`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610` after
  manually uploading active leader artifacts
- Meta retraining: forbidden
- Status: stopped for the same config-contamination issue as the tail retry.

First corrected checkpoint:

- Markets: `250`
- PnL: `+$1,347.13`
- Return: `+26.94%`
- Max drawdown: `18.46%`
- Fills: `393`
- Active markets: `48.0%`
- Attribution:
  - `br2_late_favourite_load`: `+$933.02`
  - `br2_high_skew_load`: `+$326.45`
  - `br2_late_confirm`: `+$87.66`

Interpretation: discard as config-contaminated. At `750` markets it was
`-$1,229.53` with `44.55%` max drawdown and `53.5%` active markets, which is
not comparable to the leader profile.

Exact-profile relaunch:

- Run: `20260528T202628Z-portfolio-grid-62169`
- Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_notail_lc_range50_exact`
- Replay sample: `1000ms`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Meta retraining: forbidden
- Only intentional strategy change versus no-tail leader:
  `br2_late_confirm_max_observed_range = 0.50`
- Status: active on instance `i-097d8531ba2e65a6c`

First exact-profile checkpoint:

- Markets: `250`
- PnL: `+$222.86`
- Return: `+4.46%`
- Max drawdown: `3.05%`
- Fills: `33`
- Active markets: `8.0%`
- Attribution:
  - `br2_late_favourite_load`: `+$137.49`
  - `br2_high_skew_load`: `+$38.28`
  - `br2_late_confirm`: `+$47.09`

Config verification:

```bash
python3 scripts/compare_run_configs.py \
  s3://pm-research-backtest-prod/results/20260528T185235Z-portfolio-grid-79610/clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_exact_leader_scaled_notails/summary.json \
  s3://pm-research-backtest-prod/results/20260528T202628Z-portfolio-grid-62169/clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_notail_lc_range50_exact/summary.json \
  --aws-profile visumlabs \
  --allow br2_late_confirm_max_observed_range
```

Result: configs match outside the allowed range-gate key.

## Decision Rules

Promote a BTC5m candidate only if it beats the verified no-tail leader on a
clean full-history replay after matching market universe and training window.

For each candidate, compare:

- Full-history PnL and compounded return
- Max drawdown and worst local drawdown window
- Active market percentage
- Per-tag PnL for `late_confirm`, `late_favourite_load`, `high_skew_load`, and
  `br2_convex_tail`
- Fill log loss
- Tail spend, payoff, hit rate, and price buckets

Do not promote from a partial checkpoint alone.

Before comparing any strategy candidate, run `scripts/compare_run_configs.py`
against the intended baseline and allow only the intentional experimental keys.
