# Active BTC 5m Experiments

Last updated: 2026-05-29 06:55 UTC.

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

- S3 checkpoint analyzed: `2,000 / 23,705` markets.
- Checkpoint calendar: `2026-02-27T15:40:00Z` to `2026-03-06T14:15:00Z`.
- Checkpoint PnL: `+$1,147.81`, max drawdown `18.23%`, `957` fills.
- Path labels are present on all analyzed fills.
- `crossed_mid_after_fill` is the directly confirmed loss shape:
  `br2_late_favourite_load` crossed-mid fills lost `-$1,216.94`, while
  held-side favourite fills made `+$1,194.07`.
- The 2-day OOS toxic-reversal smoke model is promising but not yet decisive:
  test AUC `0.6191`; the highest-risk bucket had `68.97%` cross-mid rate and
  `-$238.27` PnL.
- A stricter 3,250-market walk-forward gate simulation has two OOS folds. It
  suggests the first actionable shape is lane-specific, not a global throttle:
  `br2_late_confirm:q0.95` removed `-$114.68` across `10` high-risk fills,
  while broad late-favourite throttles removed positive PnL.
- See `docs/btc5m_postfill_checkpoint_readout.md` for the checkpoint details.
- See `docs/btc5m_postfill_checkpoint_2000_regime_evolution.md` for the
  current post-fill evolution report.

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
