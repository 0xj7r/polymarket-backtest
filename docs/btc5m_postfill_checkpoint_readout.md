# BTC5m Post-Fill Checkpoint Readout

Source run:

- Run: `20260529T062901Z-portfolio-grid-5265`
- Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8`
- Source artifact:
  `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
- Checkpoint analyzed: first `2,000 / 23,705` portfolio markets after the
  `4,500`-market training prefix.
- Calendar covered by this checkpoint: `2026-02-27T15:40:00Z` to
  `2026-03-06T14:15:00Z`.

Important limitation: this is not the final late-regime window yet. It is an
early replay checkpoint used to validate the new post-fill labels and analysis
path while the full-history rerun continues.

## Portfolio Snapshot

- PnL: `+$1,147.81`
- End equity: `$2,147.81`
- Path max drawdown: `18.23%`
- Active markets: `453 / 2,000`
- Fills: `957`
- Fill log loss: `0.5201` across all fills in the summary.

The path split matters more than a blunt observed-range split in this checkpoint.
The summary range `>=0.50` fill log loss is not worse than range `<0.50`, but
the final mid-wide and crossed-mid post-fill buckets are sharply negative. That
matches the observed manual failure mode: we are fragile when a favourite forms
and then mean-reverts through mid, not merely whenever range is high.

## Cross-Mid Failure Mode

The new post-fill labels confirm the shape we wanted to isolate:

| Lane | Crossed-Mid Fills | Crossed-Mid PnL | Held-Side Fills | Held-Side PnL | Moderate-Adverse PnL |
|---|---:|---:|---:|---:|---:|
| `br2_late_favourite_load` | `120` | `-$1,216.94` | `183` | `+$1,194.07` | `+$439.66` |
| `br2_late_confirm` | `142` | `-$790.85` | `158` | `+$1,344.27` | `+$103.57` |
| `br2_high_skew_load` | `66` | `-$265.80` | `101` | `+$232.97` | `+$96.92` |

So the problem is not simply "any adverse excursion." Moderate adverse
excursion can still finish profitably. The toxic case is specifically entering
the loading lane and then crossing back through mid after entry.

## Mid-Wide Confirmation

The final-range `0.78..0.93` bucket is again the damaging post-hoc label:

| Lane | Mid-Wide Fills | Mid-Wide PnL | Range `<0.78` PnL |
|---|---:|---:|---:|
| `br2_late_favourite_load` | `103` | `-$838.30` | `+$1,358.09` |
| `br2_late_confirm` | `78` | `-$82.00` | `+$761.84` |
| `br2_high_skew_load` | `64` | `-$132.81` | `+$212.42` |

That supports the existing late-regime readout: the bad state is not "all
volatility." It is the mid-wide / failed-break state where the market moves far
enough to invite loading but not far enough to become a committed extreme.

## Tail Coverage

- Tail fills: `39`
- Tail premium: `$56.23`
- Tail PnL: `+$9.93`

The tails worked when they hit, but coverage was far too sparse to protect the
main loading book in this checkpoint. This is an important distinction: cheap
tail convexity is useful, but current tail allocation is not yet a true reversal
hedge for the favourite lane.

## Classifier Smoke Test

Target: `toxic_reversal_path`, using only replay-safe fill-time features.

- Train fills: `624`
- Test fills: `294`
- Test base rate: `23.81%`
- Test AUC: `0.6191`
- Test log loss: `0.5770`

This is not strong enough for a global production gate. It does, however, show
why the gate should be lane-specific:

- The highest predicted risk bucket is the only negative risk bucket in the
  2-day OOS slice: `58` fills, `-$238.27`, `68.97%` cross-mid rate.
- Candidate high-risk removal thresholds remove negative OOS PnL globally, but
  the lane split is still mixed and needs full-history confirmation.

Conclusion: if the full run confirms this, the right implementation is not a
single "mid-wide risk off" switch. It is a lane-specific size curve:

- Favourite loading: throttle or require extra edge when predicted post-fill
  reversal risk is high.
- Late confirm: treat separately; it may need different features or a narrower
  entry rule.
- High skew: avoid broad throttling unless the full-run labels show persistent
  negative OOS removal.
- Tails: keep cheap convexity, but consider increasing tail coverage only in
  predicted reversal-risk states after the full run confirms it.

## Daily Checkpoint View

| Date | Markets | Active | PnL | End Equity | Cross-Mid Fills | Fills | Cross Rate |
|---|---:|---:|---:|---:|---:|---:|---:|
| `2026-02-27` | `100` | `15` | `+$31.19` | `$1,031.19` | `10` | `25` | `40.00%` |
| `2026-02-28` | `288` | `27` | `+$114.17` | `$1,145.36` | `9` | `62` | `14.52%` |
| `2026-03-01` | `288` | `53` | `+$12.91` | `$1,158.27` | `44` | `104` | `42.31%` |
| `2026-03-02` | `288` | `69` | `-$75.95` | `$1,082.31` | `68` | `150` | `45.33%` |
| `2026-03-03` | `288` | `73` | `+$307.92` | `$1,390.23` | `40` | `154` | `25.97%` |
| `2026-03-04` | `288` | `99` | `+$437.38` | `$1,827.61` | `71` | `239` | `29.71%` |
| `2026-03-05` | `288` | `73` | `+$523.26` | `$2,350.87` | `45` | `133` | `33.83%` |
| `2026-03-06` | `172` | `44` | `-$203.06` | `$2,147.81` | `41` | `90` | `45.56%` |

The day-level view says cross-mid rate is directionally important but not
sufficient by itself. We need lane, size, entry price, and tail-hit context.
That is exactly why the full post-fill artifact should be used to train and
validate a lane-specific regime model rather than a blunt threshold.

Detailed current report: `docs/btc5m_postfill_checkpoint_2000_regime_evolution.md`.

## Next Validation

When the rerun finishes or reaches the final 30-day window:

1. Re-run `scripts/reversal_tail_diagnostics.py` over the full artifact and the
   final `8,633` markets.
2. Re-run `scripts/postfill_reversal_model.py` with the default `--min-fills`
   over the full artifact, targeting `toxic_reversal_path` and
   `crossed_mid_after_fill`.
3. Promote a gate only if train-quantile thresholds remove negative PnL OOS
   without damaging the profitable early period or clean-directional lanes.
