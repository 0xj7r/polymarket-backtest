# BTC5m Post-Fill Checkpoint Readout

Source run:

- Run: `20260529T062901Z-portfolio-grid-5265`
- Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8`
- Source artifact:
  `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
- Checkpoint analyzed: first `1,000 / 23,705` portfolio markets after the
  `4,500`-market training prefix.
- Calendar covered by this checkpoint: `2026-02-27T15:40:00Z` to
  `2026-03-03T02:55:00Z`.

Important limitation: this is not the final late-regime window yet. It is an
early replay checkpoint used to validate the new post-fill labels and analysis
path while the full-history rerun continues.

## Portfolio Snapshot

- PnL: `+$153.55`
- End equity: `$1,153.55`
- Path max drawdown: `16.38%`
- Active markets: `175 / 1,000`
- Fills: `357`
- Fill log loss: `0.5988`
- Range `>=0.50` fill log loss: `0.6961`
- Range `<0.50` fill log loss: `0.5403`

The range split matters: the model is materially less calibrated once the
market has already expanded. That is consistent with the observed manual
failure mode: we are most fragile when a favourite forms after a wider/choppier
path and then mean-reverts.

## Cross-Mid Failure Mode

The new post-fill labels confirm the shape we wanted to isolate:

| Lane | Crossed-Mid Fills | Crossed-Mid PnL | Held-Side Fills | Held-Side PnL | Moderate-Adverse PnL |
|---|---:|---:|---:|---:|---:|
| `br2_late_favourite_load` | `57` | `-$465.44` | `64` | `+$299.62` | `+$143.43` |
| `br2_late_confirm` | `51` | `-$242.81` | `51` | `+$343.04` | `+$17.60` |
| `br2_high_skew_load` | `33` | `-$69.48` | `38` | `+$69.82` | `+$20.45` |

So the problem is not simply "any adverse excursion." Moderate adverse
excursion can still finish profitably. The toxic case is specifically entering
the loading lane and then crossing back through mid after entry.

## Mid-Wide Confirmation

The final-range `0.78..0.93` bucket is again the damaging post-hoc label:

| Lane | Mid-Wide Fills | Mid-Wide PnL | Range `<0.78` PnL |
|---|---:|---:|---:|
| `br2_late_favourite_load` | `51` | `-$247.41` | `+$284.86` |
| `br2_late_confirm` | `31` | `-$62.98` | `+$233.88` |
| `br2_high_skew_load` | `30` | `-$37.84` | `+$67.20` |

That supports the existing late-regime readout: the bad state is not "all
volatility." It is the mid-wide / failed-break state where the market moves far
enough to invite loading but not far enough to become a committed extreme.

## Tail Coverage

- Tail fills: `13`
- Tail premium: `$13.10`
- Tail PnL: `+$37.31`
- Tail premium in late-favourite losing markets: `$2.25`
- Tail premium / late-favourite losing-market cost: `0.40%`

The tails worked when they hit, but coverage was far too sparse to protect the
main loading book in this checkpoint. This is an important distinction: cheap
tail convexity is useful, but current tail allocation is not yet a true reversal
hedge for the favourite lane.

## Classifier Smoke Test

Target: `toxic_reversal_path`, using only replay-safe fill-time features.

- Train fills: `213`
- Test fills: `131`
- Test base rate: `25.19%`
- Test AUC: `0.5365`
- Test log loss: `0.7158`

This is not strong enough for a global production gate. It does, however, show
why the gate should be lane-specific:

- Removing high predicted risk globally would remove some good fills.
- On `br2_late_favourite_load`, the train `0.80` quantile would have removed
  `-$46.62` from the final one-day test slice.
- On `br2_late_confirm`, the same high-risk bucket would have removed
  profitable fills.

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
| `2026-03-03` | `36` | `11` | `+$71.23` | `$1,153.55` | `10` | `16` | `62.50%` |

The day-level view says cross-mid rate is directionally important but not
sufficient by itself. We need lane, size, entry price, and tail-hit context.
That is exactly why the full post-fill artifact should be used to train and
validate a lane-specific regime model rather than a blunt threshold.

## Next Validation

When the rerun finishes or reaches the final 30-day window:

1. Re-run `scripts/reversal_tail_diagnostics.py` over the full artifact and the
   final `8,633` markets.
2. Re-run `scripts/postfill_reversal_model.py` with the default `--min-fills`
   over the full artifact, targeting `toxic_reversal_path` and
   `crossed_mid_after_fill`.
3. Promote a gate only if train-quantile thresholds remove negative PnL OOS
   without damaging the profitable early period or clean-directional lanes.
