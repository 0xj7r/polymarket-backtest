# Active BTC 5m Experiments

Last updated: 2026-05-29 07:35 UTC.

Scope for this lane is BTC 5m only. Multi-market BTC/ETH and 15m/1h expansion is
paused until the BTC 5m engine has a clean full-history profile.

## Active Rerun

Current active rerun for path diagnostics:

- Run: `20260529T062901Z-portfolio-grid-5265`
- Label suffix: `cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8`
- Instance: `i-069194bfa6e6aa332`
- Source commit: `40ac170edc295cf05b9311af27823ca03e423ad0`
- Purpose: rerun the selected BTC 5m profile with per-fill post-entry path
  telemetry (`adverse_excursion`, `crossed_mid_after_fill`, final side mid), so
  the reversal failure mode can be measured directly instead of inferred from
  final range buckets.

Latest checkpoint readout:

- S3 checkpoint analyzed: `4,500 / 23,705` markets.
- Checkpoint calendar: `2026-02-27T15:40:00Z` to `2026-03-15T06:35:00Z`.
- Checkpoint PnL: `+$2,600.36`; active markets `838 / 4,500` (`18.62%`).
- Path labels are present on all analyzed tracked lane fills.
- The failure mode is now clearly path-dependent, not just a final-range label:
  `crossed_mid_after_fill` is toxic, while held-side and moderate adverse paths
  are strongly profitable.
- `br2_late_favourite_load`: crossed-mid fills lost `-$2,739.40`; held-side
  fills made `+$2,858.32`; moderate-adverse fills made `+$1,139.26`.
- `br2_late_confirm`: crossed-mid fills lost `-$2,574.76`; held-side fills made
  `+$3,346.72`; moderate-adverse fills made `+$320.94`.
- `br2_high_skew_load`: crossed-mid fills lost `-$637.31`; held-side fills made
  `+$630.84`; moderate-adverse fills made `+$247.96`.
- Final-range `0.78..0.93` remains a useful post-hoc diagnostic bucket:
  late-favourite fills in that bucket lost `-$2,153.53`, while final range
  `<0.78` made `+$3,811.93`.
- Daily toxic-path evolution now confirms the actual negative-day mechanism:
  cross-mid PnL is a persistent drag even on profitable days, but it becomes
  account-level damage when non-crossing late breaks stop offsetting it. In this
  checkpoint, March 12 lost `-$187.65` with `-$958.21` cross-mid PnL and March
  13 lost `-$327.80` with `-$859.31` cross-mid PnL. March 9 had worse cross-mid
  drag (`-$874.67`) but still finished positive because non-cross paths made
  `+$1,055.25`.
- Replay-safe logistic toxic-reversal model on a short final-5-day split remains
  weak: test AUC `0.5804`, log loss `0.6897`. It ranks risk directionally, but
  top-risk test buckets still made money, so it is not yet an actionable gate.
- The 4,500-market walk-forward gate simulation has only one OOS fold. The best
  candidate was small (`br2_late_confirm:q0.95`, `+$35.29` full-removal
  improvement); broad/global throttles removed profitable fills. Treat this as
  a diagnostic signal only.
- Replay-safe hard-regime throttles were also tested in the same OOS fold
  (`expanded_not_decisive`, sign-flip/path-efficiency, observed-range/reversal,
  high-price choppy favourite variants). They all removed positive PnL in this
  checkpoint. That means the broad regime label is not enough; we need a sharper
  classifier for "late break that fails back through mid" rather than a blanket
  choppy/mid-wide throttle.
- See `docs/btc5m_postfill_checkpoint_4500_regime_evolution.md`,
  `docs/btc5m_postfill_checkpoint_4500_reversal_tail.md`,
  `docs/btc5m_postfill_checkpoint_4500_toxic_reversal_path_model.md`, and
  `docs/btc5m_postfill_checkpoint_4500_gate_sim.md` for the current reports.

This checkpoint is not yet the final late-regime window. Do not promote a
post-fill reversal gate until the full-history artifact reaches the final 30d
slice and the OOS diagnostics are rerun there.

When the active artifact reaches the final 30d slice or completes, regenerate
the diagnostic pack with:

```bash
AWS_PROFILE=visumlabs python3 scripts/run_postfill_diagnostics.py \
  s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl \
  --aws-profile visumlabs \
  --recent-days 30 \
  --last-markets 8633 \
  --test-days 30 \
  --out-prefix docs/btc5m_postfill_full
```

This emits:

- `docs/btc5m_postfill_full_regime_evolution.md`
- `docs/btc5m_postfill_full_reversal_tail.md`
- `docs/btc5m_postfill_full_toxic_reversal_path_model.md`
- `docs/btc5m_postfill_full_crossed_mid_after_fill_model.md`
- `docs/btc5m_postfill_full_gate_sim.md`

To poll until the full artifact is ready and then run the same pack:

```bash
AWS_PROFILE=visumlabs python3 scripts/watch_postfill_diagnostics.py \
  s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl \
  --aws-profile visumlabs \
  --ready-markets 23705 \
  --poll-seconds 300 \
  --out-prefix docs/btc5m_postfill_full
```

## Final Selection

Selected 1K BTC 5m path:
`cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8`

- Run: `20260528T225810Z-portfolio-grid-52322`
- Label:
  `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8`
- Markets: `23,705`
- Equivalent 5m-market days: `82.31`
- Starting capital: `$1,000`
- Final PnL: `+$8,990.21`
- Return: `+899.02%`
- Implied compounded daily return: `2.836%`
- Max drawdown: `23.70%`
- Fills: `3,723`
- Active market rate: `8.50%`
- Attribution:
  - `br2_late_favourite_load`: `+$4,726.87`
  - `br2_late_confirm`: `+$2,379.56`
  - `br2_high_skew_load`: `+$1,977.61`
  - `br2_convex_tail`: `-$93.83`

Reason for selection: the base profile beat the broader-tail cov75 variant on
full history, with slightly lower drawdown and materially less tail bleed. It
keeps cheap 8c convex tails enabled without paying for the heavier cov75
coverage budget that did not improve drawdown or final PnL on this history.

Rejected comparator:
`cap1k_btc_5m_tail08_cov75_ladder_lc_range50_exact_profile_mem128_cf8`

- Run: `20260528T225904Z-portfolio-grid-53933`
- Markets: `23,705`
- Final PnL: `+$8,969.77`
- Return: `+896.98%`
- Implied compounded daily return: `2.833%`
- Max drawdown: `23.72%`
- Fills: `3,731`
- Attribution:
  - `br2_late_favourite_load`: `+$4,737.56`
  - `br2_late_confirm`: `+$2,382.87`
  - `br2_high_skew_load`: `+$1,979.40`
  - `br2_convex_tail`: `-$130.06`

Selection status: chosen. Do not launch additional broad grids for this lane
unless new data, live/paper fill evidence, or a specific failure mode justifies
another narrow test.

## Previous Leading Checkpoints

These were strong but incomplete checkpoints before the memory-safe full-history
relaunches:

1. `cap1k_btc_5m_tail08_lc_range50_exact_profile`
   - Run: `20260528T211443Z-portfolio-grid-55343`
   - Markets: `12,250`
   - PnL: `+$7,121.84`
   - Return: `+712.18%`
   - Max drawdown: `23.70%`
2. `cap3k_btc_5m_tail08_lc_range50_exact_profile`
   - Run: `20260528T212132Z-portfolio-grid-68579`
   - Markets: `12,250`
   - PnL: `+$12,814.25`
   - Return: `+427.14%`
   - Max drawdown: `20.07%`
3. `cap5k_btc_5m_tail08_lc_range50_exact`
   - Run: `20260528T205238Z-portfolio-grid-12673`
   - Markets: `12,250`
   - PnL: `+$16,342.84`
   - Return: `+326.86%`
   - Max drawdown: `17.94%`

## Completed Full-History Relaunches

The earlier exact-profile runners did not finish the `23,705`-market history.
SSM/systemd showed `cloud-final.service` failed with `Result: oom-kill`, and
the `pm-app` process was killed after the local run logs reached about
`12,500 / 23,705` markets. The durable S3 summaries are still valid up to their
last uploaded checkpoints, but those runs should not be treated as complete.

Idle OOM-killed instances terminated to control cost:

- `i-026438638670e3522` (`1K` exact profile)
- `i-0b97808c7b58a5dda` (`3K` exact profile)
- `i-0261b49082277a123` (`5K` exact profile)

The smaller coverage runner also became stale with pending SSM commands and no
new S3 checkpoint beyond `3,500` markets, so it was terminated:

- `i-04582d750613706e2`

Completed memory-safe full-history candidates:

1. Base 1K exact tail08 profile
   - Run: `20260528T225810Z-portfolio-grid-52322`
   - Label:
     `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8`
   - Instance: `i-03916a47348b6d96b`
   - Instance type: `r7i.4xlarge`
   - Memory-safety change: `max_concurrent_fetches = 8`, larger memory host.
   - Strategy logic: same 1K exact profile and frozen meta-calibrator.
   - Final status: completed all `23,705` markets and self-terminated.
   - Final result: `+$8,990.21`, max drawdown `23.70%`, `3,723` fills.
2. Broader tail-coverage variant
   - Run: `20260528T225904Z-portfolio-grid-53933`
   - Label:
     `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_cov75_ladder_lc_range50_exact_profile_mem128_cf8`
   - Instance: `i-0e76ad30421811c0d`
   - Instance type: `r7i.4xlarge`
   - Memory-safety change: `max_concurrent_fetches = 8`, larger memory host.
   - Strategy logic: same exact profile except broader tail coverage:
     `target_favourite_loss_coverage_frac = 0.75`, `tail_max_clips = 10`,
     `tail_min_skew_step = 0.01`, `tail_extreme_threshold = 0.25`,
     `budget_favourite_spend_frac = 0.30`, and
     `budget_favourite_upside_frac = 0.40`.
   - Final status: completed all `23,705` markets and self-terminated.
   - Final result: `+$8,969.77`, max drawdown `23.72%`, `3,731` fills.

Common-prefix read: cov75 had a small early lead, but the base profile overtook
it after the larger drawdown/recovery section. On the full history, base finished
`$20.44` ahead with slightly lower drawdown and `$36.23` less tail bleed. The
extra cov75 insurance improved tail payoff in the few tail-hit markets, but did
not improve portfolio-level drawdown or final PnL.

## Tail Hedge Readout

Final full-history base tail08 readout:

- Tail fills: `174`
- Tail markets: `173`
- Gross tail premium spent: `$382.66`
- Tail payout: `$288.83`
- Net tail PnL: `-$93.83`
- Tail shares: `6,131.84`
- Average tail fill price: `6.2c`
- Tail wins: `11 / 174`, `6.32%`
- Directional non-tail notional: about `$173.6k`
- Tail premium as share of directional non-tail notional: about `0.22%`
- Main loss in markets where a tail was present and hit: `$589.20`
- Tail profit in those same markets: `$270.80`
- Tail coverage in those hit markets: `45.96%` net, `49.02%` payout.

Final full-history cov75 readout:

- Tail fills: `183`
- Tail markets: `176`
- Gross tail premium spent: `$565.51`
- Tail payout: `$435.45`
- Net tail PnL: `-$130.06`
- Tail shares: `9,124.35`
- Average tail fill price: `6.2c`
- Tail wins: `12 / 183`, `6.56%`
- Main loss in markets where a tail was present and hit: `$641.62`
- Tail profit in those same markets: `$408.07`
- Tail coverage in those hit markets: `63.60%` net, `67.87%` payout.

Interpretation: tails are working as cheap convexity when they hit, but the
current tail budget is too small and sparse to be a full portfolio hedge. The
selected base profile keeps the cheaper convexity lane because it loses less
premium while preserving most of the full-run PnL. A future live/paper variant
can revisit broader tail participation, but cov75 did not earn promotion on this
historical pass.
