# Reversal And Tail Diagnostics

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Window: last `5750` markets.

Definitions:

- `final_range_078_093_mid_wide` is a post-hoc bucket using full resolved-market YES-mid range.
- `expanded_not_decisive` is live-safe: observed range `0.40..0.65` plus sign flips or low path efficiency.

Summary:

- Active markets: `991`
- Total PnL: `$4,141.46`
- Late-favourite losing markets: `103`
- Late-favourite losing-market cost: `$4,555.32`
- Tail premium in late-favourite losing markets: `$5.09`
- Tail premium / late-favourite losing cost: `0.11%`
- Total tail premium: `$158.83`
- Total tail PnL: `$-13.62`

## Final Range Buckets

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | final_range_078_093_mid_wide | 17 | $-30.91 | $30.91 | 0.00% |
| br2_convex_tail | final_range_093_097 | 3 | $22.94 | $1.73 | 100.00% |
| br2_convex_tail | final_range_ge_097 | 6 | $113.65 | $6.88 | 100.00% |
| br2_convex_tail | final_range_lt_078 | 65 | $-119.31 | $119.31 | 0.00% |
| br2_high_skew_load | final_range_078_093_mid_wide | 155 | $-569.46 | $2,233.35 | 60.65% |
| br2_high_skew_load | final_range_093_097 | 9 | $-51.39 | $102.18 | 33.33% |
| br2_high_skew_load | final_range_ge_097 | 4 | $-21.93 | $70.26 | 50.00% |
| br2_high_skew_load | final_range_lt_078 | 282 | $1,216.35 | $3,915.79 | 95.04% |
| br2_late_confirm | final_range_078_093_mid_wide | 170 | $-91.29 | $6,229.20 | 61.18% |
| br2_late_confirm | final_range_093_097 | 14 | $-408.12 | $555.13 | 28.57% |
| br2_late_confirm | final_range_ge_097 | 15 | $-115.28 | $500.30 | 46.67% |
| br2_late_confirm | final_range_lt_078 | 503 | $2,572.58 | $17,723.40 | 82.11% |
| br2_late_favourite_load | final_range_078_093_mid_wide | 198 | $-2,493.90 | $5,903.05 | 52.53% |
| br2_late_favourite_load | final_range_093_097 | 16 | $-316.85 | $544.78 | 31.25% |
| br2_late_favourite_load | final_range_ge_097 | 13 | $-27.13 | $235.24 | 76.92% |
| br2_late_favourite_load | final_range_lt_078 | 534 | $4,461.51 | $18,911.81 | 93.26% |

## Live-Safe Regime Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | expanded_chop | 10 | $9.52 | $11.98 | 20.00% |
| br2_convex_tail | expanded_continuation | 18 | $-4.39 | $26.27 | 11.11% |
| br2_convex_tail | expanded_not_decisive | 44 | $-15.31 | $82.41 | 4.55% |
| br2_convex_tail | neutral | 11 | $21.13 | $13.58 | 27.27% |
| br2_convex_tail | reversal_pressure | 8 | $-24.58 | $24.58 | 0.00% |
| br2_high_skew_load | expanded_chop | 36 | $-121.94 | $458.99 | 58.33% |
| br2_high_skew_load | expanded_continuation | 33 | $-97.10 | $484.24 | 69.70% |
| br2_high_skew_load | expanded_not_decisive | 241 | $239.45 | $3,342.73 | 82.99% |
| br2_high_skew_load | neutral | 94 | $470.63 | $1,396.23 | 88.30% |
| br2_high_skew_load | reversal_pressure | 46 | $82.53 | $639.40 | 86.96% |
| br2_late_confirm | expanded_not_decisive | 418 | $869.22 | $14,670.53 | 77.27% |
| br2_late_confirm | neutral | 182 | $831.82 | $6,678.45 | 73.63% |
| br2_late_confirm | reversal_pressure | 102 | $256.85 | $3,659.04 | 69.61% |
| br2_late_favourite_load | expanded_chop | 6 | $7.76 | $22.26 | 100.00% |
| br2_late_favourite_load | expanded_continuation | 3 | $1.76 | $13.30 | 66.67% |
| br2_late_favourite_load | expanded_not_decisive | 478 | $684.78 | $15,634.83 | 79.71% |
| br2_late_favourite_load | neutral | 176 | $774.51 | $6,641.59 | 85.80% |
| br2_late_favourite_load | reversal_pressure | 98 | $154.82 | $3,282.89 | 78.57% |

## Post-Fill Path Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | held_side | 91 | $-13.62 | $158.83 | 9.89% |
| br2_high_skew_load | crossed_mid_after_fill | 157 | $-863.54 | $2,301.03 | 47.13% |
| br2_high_skew_load | held_side | 215 | $1,123.64 | $2,964.08 | 100.00% |
| br2_high_skew_load | moderate_adverse_excursion | 78 | $313.46 | $1,056.48 | 100.00% |
| br2_late_confirm | crossed_mid_after_fill | 295 | $-3,574.94 | $10,499.37 | 44.75% |
| br2_late_confirm | held_side | 344 | $5,181.28 | $12,558.50 | 98.55% |
| br2_late_confirm | large_adverse_then_soft_finish | 1 | $-26.07 | $26.07 | 0.00% |
| br2_late_confirm | moderate_adverse_excursion | 62 | $377.62 | $1,924.09 | 91.94% |
| br2_late_favourite_load | crossed_mid_after_fill | 264 | $-3,185.59 | $8,735.07 | 46.59% |
| br2_late_favourite_load | held_side | 357 | $3,478.95 | $12,142.66 | 100.00% |
| br2_late_favourite_load | moderate_adverse_excursion | 140 | $1,330.27 | $4,717.15 | 97.86% |

## Worst Markets

| Rank | Slug | PnL | Range | Late Fav | Late Confirm | High Skew | Tail | Tail/Fav Cost | Late Fav Wins |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | btc-updown-5m-1773363000 | $-220.00 | 0.925 | $-82.80 | $-120.64 | $-16.56 | $0.00 | 0.00% | 0/1 |
| 2 | btc-updown-5m-1773078900 | $-217.38 | 0.885 | $-196.81 | $0.00 | $-20.57 | $0.00 | 0.00% | 0/3 |
| 3 | btc-updown-5m-1773908100 | $-214.40 | 0.874 | $-214.40 | $0.00 | $0.00 | $0.00 | 0.00% | 0/2 |
| 4 | btc-updown-5m-1772642400 | $-212.45 | 0.785 | $-167.98 | $0.00 | $-44.47 | $0.00 | 0.00% | 0/5 |
| 5 | btc-updown-5m-1773273600 | $-186.45 | 0.755 | $0.00 | $-186.45 | $0.00 | $0.00 | 0.00% | 0/0 |
| 6 | btc-updown-5m-1773678300 | $-182.02 | 0.789 | $-28.30 | $-153.71 | $0.00 | $0.00 | 0.00% | 0/1 |
| 7 | btc-updown-5m-1773352500 | $-155.74 | 0.885 | $-112.10 | $0.00 | $-43.64 | $0.00 | 0.00% | 0/2 |
| 8 | btc-updown-5m-1773745800 | $-153.72 | 0.644 | $0.00 | $-153.72 | $0.00 | $0.00 | 0.00% | 0/0 |
| 9 | btc-updown-5m-1772775000 | $-146.79 | 0.800 | $-111.99 | $0.00 | $-34.80 | $0.00 | 0.00% | 0/3 |
| 10 | btc-updown-5m-1773688800 | $-136.28 | 0.605 | $0.00 | $-136.28 | $0.00 | $0.00 | 0.00% | 0/0 |
| 11 | btc-updown-5m-1773924900 | $-135.52 | 0.825 | $0.00 | $-135.52 | $0.00 | $0.00 | 0.00% | 0/0 |
| 12 | btc-updown-5m-1772758800 | $-132.87 | 0.745 | $-107.25 | $0.00 | $-25.62 | $0.00 | 0.00% | 0/4 |

