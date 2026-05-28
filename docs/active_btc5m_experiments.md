# Active BTC 5m Experiments

Last updated: 2026-05-28 22:45 UTC.

Scope for this lane is BTC 5m only. Multi-market BTC/ETH and 15m/1h expansion is
paused until the BTC 5m engine has a clean full-history profile.

## Current Working Ranking

These are active checkpoints, not final full-history results.

1. `cap1k_btc_5m_tail08_lc_range50_exact_profile`
   - Run: `20260528T211443Z-portfolio-grid-55343`
   - Markets: `12,250`
   - PnL: `+$7,121.84`
   - Return: `+712.18%`
   - Max drawdown: `23.70%`
   - Reason to keep: exact `$1,000` pilot sizing, range-gated, convex tails
     enabled, and strong compounded PnL so far.
2. `cap3k_btc_5m_tail08_lc_range50_exact_profile`
   - Run: `20260528T212132Z-portfolio-grid-68579`
   - Markets: `11,750`
   - PnL: `+$12,806.39`
   - Return: `+426.88%`
   - Max drawdown: `20.07%`
   - Reason to keep: exact profile scaled to the current wallet size.
3. `cap5k_btc_5m_tail08_lc_range50_exact`
   - Run: `20260528T205238Z-portfolio-grid-12673`
   - Markets: `12,250`
   - PnL: `+$16,342.84`
   - Return: `+326.86%`
   - Max drawdown: `17.94%`
   - Reason to keep: larger-capital validation of the same range + 8c-tail
     profile.
4. `cap5k_btc_5m_notail_lc_range50_exact`
   - Run: `20260528T202628Z-portfolio-grid-62169`
   - Markets: `12,000`
   - PnL: `+$15,270.25`
   - Return: `+305.41%`
   - Max drawdown: `17.88%`
   - Reason to keep: no-tail PnL ceiling and regression baseline; not the
     preferred production candidate because it lacks convex tail coverage.

## Next Focused Tail-Coverage Test

Do not launch this until one of the active completion runners frees capacity.
The goal is to answer whether cheap tails can provide broader favourite
insurance without turning into the lossy 8-10c tail bleed seen in earlier runs.

Candidate label:
`cap1k_btc_5m_tail08_cov75_ladder_lc_range50_exact_profile`

Keep the selected 1K exact profile unchanged except:

- `br2_tail_target_favourite_loss_coverage_frac = 0.75`
- `br2_tail_max_clips = 10`
- `br2_tail_min_skew_step = 0.01`
- `br2_tail_extreme_threshold = 0.25`
- `br2_tail_budget_favourite_spend_frac = 0.30`
- `br2_tail_budget_favourite_upside_frac = 0.40`
- keep `br2_tail_max_ask = 0.08`

Reasoning:

- Current tail spend is only about `0.6%` of favourite notional and appears on
  only about `10%` of favourite positions.
- When a losing favourite actually has opposite tail coverage, the hedge is
  meaningful at roughly `65-70%` net cover.
- The current problem is coverage frequency, not per-fire hedge sizing.
- Keep the 8c cap because earlier 10c tails bled; do not broaden into the
  8-10c bucket until there is stronger evidence.

Launch:

- Run: `20260528T222500Z-portfolio-grid-89380`
- Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_cov75_ladder_lc_range50_exact_profile`
- Starting capital: `$1,000`
- Instance: `i-04582d750613706e2`
- Instance type: `c7i.2xlarge`
- Status: active

Capacity note: the no-tail 5K baseline runner was stopped at its latest S3
checkpoint because it was no longer the preferred production candidate and had
not advanced its uploaded checkpoint. Latest preserved no-tail checkpoint:
`12,000` markets, `+$15,270.25`, max drawdown `17.88%`.

First same-prefix checkpoint at `750` markets:

- Baseline 1K tail08: `+$191.31`, max drawdown `11.11%`,
  `br2_convex_tail = +$41.69`
- Coverage variant: `+$212.33`, max drawdown `10.83%`,
  `br2_convex_tail = +$62.15`
- Tail spend:
  - Baseline: `$8.72`, `145.5` tail shares
  - Coverage variant: `$13.60`, `221.6` tail shares
- Favourite-tail frequency:
  - Baseline: `5 / 63` favourite positions, `7.9%`
  - Coverage variant: `6 / 63` favourite positions, `9.5%`
- Net favourite coverage across all favourite positions:
  - Baseline: `7.9%`
  - Coverage variant: `11.8%`

Interpretation: too early to promote, but this is a clean positive first
checkpoint. The broader-tail variant improved PnL, drawdown, tail PnL, and
coverage frequency at the same prefix.

Second same-prefix checkpoint at `2,500` markets:

- Baseline 1K tail08: `+$1,436.68`, max drawdown `18.23%`,
  `br2_convex_tail = +$16.77`
- Coverage variant: `+$1,459.91`, max drawdown `18.28%`,
  `br2_convex_tail = +$26.67`
- Tail spend:
  - Baseline: `$65.65`, `1,033.7` tail shares
  - Coverage variant: `$98.64`, `1,533.9` tail shares
- Favourite-tail frequency:
  - Baseline: `28 / 250` favourite positions, `11.2%`
  - Coverage variant: `29 / 250` favourite positions, `11.6%`
- Net favourite coverage across all favourite positions:
  - Baseline: `8.2%`
  - Coverage variant: `11.9%`

Interpretation: still a small positive comparison. The coverage variant is not
decisively better yet, but it increases hedge coverage without hurting PnL or
drawdown through this prefix.

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

Second exact-profile checkpoint:

- Markets: `1,250`
- PnL: `+$1,412.30`
- Return: `+28.25%`
- Max drawdown: `14.84%`
- Fills: `530`
- Active markets: `19.0%`
- Attribution:
  - `br2_late_confirm`: `+$579.80`
  - `br2_late_favourite_load`: `+$462.53`
  - `br2_high_skew_load`: `+$359.43`
  - `br2_convex_tail`: `+$10.55`

Same-prefix comparison at `1,250` markets:

- No-tail leader: `+$274.69`, max drawdown `27.57%`
- Fixed-tail leader: `+$208.55`, max drawdown `26.73%`
- Exact range + tail: `+$1,412.30`, max drawdown `14.84%`

Interpretation: this is the cleanest early evidence so far that the
`late_confirm` observed-range cap addresses the early reversion damage without
removing the favourite/high-skew workhorses or convex tail upside.

Third exact-profile checkpoint:

- Markets: `2,250`
- PnL: `+$4,150.50`
- Return: `+83.01%`
- Max drawdown: `18.49%`
- Fills: `1,058`
- Active markets: `21.0%`
- Attribution:
  - `br2_late_confirm`: `+$2,623.97`
  - `br2_late_favourite_load`: `+$1,410.50`
  - `br2_high_skew_load`: `+$322.67`
  - `br2_convex_tail`: `-$206.63`

Interpretation: the range gate remains strongly positive, but the current
10c-tail settings are again a drag by this prefix. Tail is still strategically
valid as convex insurance, but this result supports testing a cheaper tail cap
such as `tail_max_ask = 0.08` or reducing only the 8-10c rung.

Fourth exact-profile checkpoint:

- Markets: `4,000`
- PnL: `+$7,587.21`
- Return: `+151.74%`
- Max drawdown: `18.49%`
- Fills: `1,764`
- Active markets: `19.9%`
- Attribution:
  - `br2_late_confirm`: `+$3,698.24`
  - `br2_late_favourite_load`: `+$3,413.24`
  - `br2_high_skew_load`: `+$787.44`
  - `br2_convex_tail`: `-$311.71`

Interpretation: 10c-tail remains behind no-tail range-gated. The range gate is
working; the broad 8-10c tail spend is the main open issue.

Cheaper-tail exact relaunches:

- `8c` cap:
  - Run: `20260528T205238Z-portfolio-grid-12673`
  - Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail08_lc_range50_exact`
  - Instance: `i-0261b49082277a123`
- `6c` cap:
  - Run: `20260528T205254Z-portfolio-grid-13193`
  - Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail06_lc_range50_exact`
  - Instance: `i-0666effd4c4f595da`

Both reuse the frozen meta-calibrator and prebuilt Linux `pm-app` binary. The
only intentional differences versus the 10c exact tail run are
`br2_late_confirm_max_observed_range = 0.50` and `br2_tail_max_ask`.

First cheaper-tail checkpoints:

- `8c` cap: `250` markets, `+$222.86`, max drawdown `3.05%`, no convex-tail
  fills yet.
- `6c` cap: `250` markets, `+$220.99`, max drawdown `3.05%`,
  `br2_convex_tail = -$1.79`.

Config verification: both cheaper-tail runs match the 10c exact-tail run
outside the allowed `br2_tail_max_ask` key.

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

Second exact-profile checkpoint:

- Markets: `1,250`
- PnL: `+$1,402.69`
- Return: `+28.05%`
- Max drawdown: `16.60%`
- Fills: `478`
- Active markets: `19.0%`
- Attribution:
  - `br2_late_confirm`: `+$575.59`
  - `br2_late_favourite_load`: `+$466.68`
  - `br2_high_skew_load`: `+$360.42`

Interpretation: no-tail range-gated performance is very close to the tail
version at this prefix, but drawdown is higher (`16.60%` versus `14.84%`).
Given the strategic preference for convex reversal coverage, the tail version
remains the stronger candidate unless later full-history results show the tail
spend overwhelming path utility.

Third exact-profile checkpoint:

- Markets: `2,250`
- PnL: `+$4,379.35`
- Return: `+87.59%`
- Max drawdown: `17.88%`
- Fills: `952`
- Active markets: `21.0%`
- Attribution:
  - `br2_late_confirm`: `+$2,627.91`
  - `br2_late_favourite_load`: `+$1,429.21`
  - `br2_high_skew_load`: `+$322.24`

Interpretation: at this prefix, exact-profile no-tail range-gated is ahead of
the 10c-tail version by `+$228.85` and has slightly lower drawdown. It is the
cleaner current range-gated benchmark while a cheaper-tail variant is pending.

Fourth exact-profile checkpoint:

- Markets: `4,250`
- PnL: `+$8,494.81`
- Return: `+169.90%`
- Max drawdown: `17.88%`
- Fills: `1,640`
- Active markets: `19.6%`
- Attribution:
  - `br2_late_confirm`: `+$3,829.50`
  - `br2_late_favourite_load`: `+$3,840.94`
  - `br2_high_skew_load`: `+$824.37`

Interpretation: this is the current leading exact-profile candidate. It keeps
the strong range-gated directional edge and avoids the broad-tail bleed. It
must still complete full history before promotion.

Scaled 1K verification run:

- Run: `20260528T210327Z-portfolio-grid-33677`
- Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_notail_lc_range50_exact`
- Starting capital: `$1,000`
- Gross cap: `$250`
- Max clip: `$30`
- Tails: disabled
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Binary: prebuilt `pm-app-al2023-x86_64-607c3156`
- Instance: `i-06b79496ee1e34f04`
- Status: stopped/replaced before first checkpoint. This was a no-tail
  reproduction run; it is not the desired pilot profile because the current
  candidate should keep convex tail coverage enabled.

Scaled 1K range + tail verification run:

- Run: `20260528T210817Z-portfolio-grid-43023`
- Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact`
- Starting capital: `$1,000`
- Gross cap: `$250`
- Max clip: `$30`
- Range gate: `br2_late_confirm_max_observed_range = 0.50`
- Tails: enabled with `br2_tail_max_ask = 0.08`
- Tail budget: `clip_frac = 0.10`, `max_clips = 6`,
  `target_favourite_loss_coverage_frac = 0.50`,
  `budget_favourite_spend_frac = 0.20`,
  `budget_favourite_upside_frac = 0.25`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Binary: prebuilt `pm-app-al2023-x86_64-607c3156`
- Instance: `i-047b07c20f934e3ee`
- Status: stopped/replaced at first checkpoint. Config comparison showed it
  inherited default `kelly_fraction`, drawdown clip throttles, max order
  multiplier, and training metadata, so it was not an exact scaled version of
  the 5K tail08 profile.

Scaled 1K range + tail exact-profile run:

- Run: `20260528T211443Z-portfolio-grid-55343`
- Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile`
- Starting capital: `$1,000`
- Kelly: `0.50`
- Gross cap: `$250`
- Max clip: `$30`
- Max order clip multiplier: `10`
- Drawdown clip throttle: soft `0.2%`, hard `0.4%`, min multiplier `0.1`
- Range gate: `br2_late_confirm_max_observed_range = 0.50`
- Tails: enabled with `br2_tail_max_ask = 0.08`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Binary: prebuilt `pm-app-al2023-x86_64-607c3156`
- Instance: `i-026438638670e3522`
- Status: active

Latest checkpoint:

- Markets: `12,250`
- PnL: `+$7,121.84`
- Return: `+712.18%`
- Max drawdown: `23.70%`
- Fills: `2,866`
- Active markets: `12.1%`
- Attribution:
  - `br2_late_favourite_load`: `+$3,899.21`
  - `br2_late_confirm`: `+$1,930.91`
  - `br2_high_skew_load`: `+$1,363.21`
  - `br2_convex_tail`: `-$71.49`
- Config verification: matches the 5K tail08 exact profile outside scaled
  `starting_cash_usdc`, `max_clip_usdc`, and
  `max_per_market_exposure_usdc`.

Scaled 3K range + tail exact-profile run:

- Run: `20260528T212132Z-portfolio-grid-68579`
- Label: `clip_0p015_gross_750_expfrac_0p12_lat500ms_cap3k_btc_5m_tail08_lc_range50_exact_profile`
- Starting capital: `$3,000`
- Kelly: `0.50`
- Gross cap: `$750`
- Max clip: `$90`
- Max order clip multiplier: `10`
- Drawdown clip throttle: soft `0.2%`, hard `0.4%`, min multiplier `0.1`
- Range gate: `br2_late_confirm_max_observed_range = 0.50`
- Tails: enabled with `br2_tail_max_ask = 0.08`
- Meta calibrator: reused from `20260528T185235Z-portfolio-grid-79610`
- Binary: prebuilt `pm-app-al2023-x86_64-607c3156`
- Instance: `i-0b97808c7b58a5dda`
- Status: active

Latest checkpoint:

- Markets: `11,750`
- PnL: `+$12,806.39`
- Return: `+426.88%`
- Max drawdown: `20.07%`
- Fills: `2,843`
- Active markets: `12.2%`
- Attribution:
  - `br2_late_favourite_load`: `+$5,781.39`
  - `br2_late_confirm`: `+$4,812.43`
  - `br2_high_skew_load`: `+$2,315.12`
  - `br2_convex_tail`: `-$102.55`
- Config verification: matches the 5K tail08 exact profile outside scaled
  `starting_cash_usdc`, `max_clip_usdc`, and
  `max_per_market_exposure_usdc`.

Stopped cheaper-tail branch:

- Run: `20260528T205254Z-portfolio-grid-13193`
- Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail06_lc_range50_exact`
- Final preserved checkpoint before stop: `3,750` markets, `+$8,966.27`,
  max drawdown `17.89%`
- Convex-tail attribution: `-$160.72`
- Reason: underperformed the 8c tail cap on the same family of settings while
  offering less useful convex capture.

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
