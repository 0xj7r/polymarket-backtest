# BTC5m Late-Regime Readout

Source report: `docs/btc5m_regime_evolution_diagnostics.md`

## What Changed

The late history did not simply become "more mid-wide." The final-range
`0.78..0.93` bucket was already bad early and its market frequency actually fell
from `17.50%` in the first 30d to `12.04%` in the last 30d. The bigger change
was that participation collapsed:

- First 30d: `8,640` markets, `1,214` active, `2,413` fills, `+$5,394`.
- Last 30d: `8,633` markets, `335` active, `549` fills, `+$20`.
- Last third: `7,902` markets, `256` active, `406` fills, `-$227`.

So the later strategy is not getting enough profitable clean-directional volume
to offset the still-toxic mid-wide/reversion bucket.

## The Toxic Bucket

The post-hoc mid-wide range remains the clearest damage label. In the last 30d:

- All mid-wide active markets: `-$3,006`.
- Non-mid-wide markets therefore carried the portfolio back to roughly flat.
- `br2_late_confirm` in mid-wide: `30` fills, `-$1,301`, `33.3%` win rate.
- `br2_late_favourite_load` in mid-wide: `52` fills, `-$1,067`, `44.2%` win rate.
- `br2_high_skew_load` in mid-wide: `50` fills, `-$616`, `60.0%` win rate.

More extreme final ranges were not the problem. In the last 30d, `range_ge_097`
was positive for favourite, confirm, and high-skew lanes. This argues against a
blunt volatility/range throttle. The bad pattern is specifically "large enough
move to make us load, but not enough commitment to finish as an obvious extreme."

## Live-Safe Clues

The currently logged live features do not fully explain the bucket by themselves.
Some simple filters would have helped recently but would have removed profitable
fills earlier:

- Late confirm `sign_flip_rate 0.40..0.457`: removes `-$1,092` in last 30d, but
  would remove `+$190` in first 30d.
- Late confirm `reversal_pressure 0.24..0.34`: removes `-$1,064` in last 30d,
  but would remove `+$828` in first 30d.
- Late confirm `confidence < 0.81`: removes `-$605` in last 30d, but would
  remove `+$304` in first 30d.
- Late favourite observed range `0.40..0.50`: removes `-$660` in last 30d, but
  would remove `+$833` in first 30d.

That means a static global threshold is likely the wrong shape. We need a
regime-conditioned gate or sizing curve that changes behavior when the broader
market tape is in the later low-participation / mid-wide-toxic state.

## Mid-Wide Model Diagnostics

Source reports:

- `docs/btc5m_midwide_membership_model.md`
- `docs/btc5m_midwide_regime_model.md`

Two replay-safe logistic classifiers were checked:

1. `midwide`: whether the fill belongs to a market whose final range was
   `0.78..0.93`.
2. `toxic_midwide`: whether the fill belongs to that final range and the fill
   lost money.

The model can partly detect mid-wide membership out of sample:

- `midwide` OOS AUC: `0.6587`.
- Test base rate: `25.72%`.
- Strongest drivers: observed range, range x sign-flip, prior range features.

But that is not enough for a trading gate. The highest predicted mid-wide bucket
was profitable in the final 30d (`+$443`), while the worst bucket was the middle
risk bucket (`-$1,275`). Removing high predicted mid-wide probability would have
removed profitable late-window fills.

The more directly useful `toxic_midwide` target failed OOS:

- `toxic_midwide` OOS AUC: `0.4412`.
- Test base rate: `13.63%`.
- Highest predicted risk bucket was also profitable (`+$655`).

Conclusion: current fill-time summary features can identify some mid-wide
structure, but they do not robustly identify which mid-wide entries are toxic.
The next model label should be post-entry path failure, not final range alone.

## Interpretation

The model still sees good edge in individual late-favourite and late-confirm
fills, but in later history that edge is less reliable when the market is in the
"late-forming favourite then reverts/soft-finishes" pattern. Feature medians did
not shift dramatically; realized vol and whipsaw measures were often lower in
the last 30d, which is exactly why simple whipsaw gating can miss this. The
failure mode is not always visibly chaotic before entry.

The active post-fill rerun will add direct labels for:

- `crossed_mid_after_fill`
- post-entry adverse excursion
- final side mid after entry

Those fields should become the primary labels for training the next regime
classifier, because they match the real trading failure: favourite loading after
a late break, followed by a cross back through mid or weak finish.

## Strategy Implications

Near-term candidates to test:

1. Add a late-confirm/favourite "mid-wide risk" model that uses live features
   plus prior-day/3-day/7-day range stats and only adjusts these lanes.
2. Avoid hard disabling favourite loading globally. Prefer size throttles:
   reduce size in suspected mid-wide risk, require stronger edge, or shift some
   spend into cheap tail coverage.
3. Treat extreme committed ranges separately from mid-wide. The `>=0.97` final
   range bucket was profitable, so extreme favourite loading should not be
   punished just because range is high.
4. Use post-fill labels from the rerun to train a "crossed-mid-after-entry"
   classifier. That label is more actionable than final range alone.
5. Keep tail spend cheap but revisit coverage only where the classifier says
   late-reversion risk is high. The current tail lane is too sparse to hedge the
   worst late-favourite losses.
