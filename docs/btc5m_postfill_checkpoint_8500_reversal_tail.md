# Reversal And Tail Diagnostics

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Window: last `8500` markets.

Definitions:

- `final_range_078_093_mid_wide` is a post-hoc bucket using full resolved-market YES-mid range.
- `expanded_not_decisive` is live-safe: observed range `0.40..0.65` plus sign flips or low path efficiency.

Summary:

- Active markets: `1213`
- Total PnL: `$5,496.16`
- Late-favourite losing markets: `122`
- Late-favourite losing-market cost: `$6,416.03`
- Tail premium in late-favourite losing markets: `$5.09`
- Tail premium / late-favourite losing cost: `0.08%`
- Total tail premium: `$201.75`
- Total tail PnL: `$-26.77`

## Final Range Buckets

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | final_range_078_093_mid_wide | 23 | $-45.92 | $45.92 | 0.00% |
| br2_convex_tail | final_range_093_097 | 3 | $22.94 | $1.73 | 100.00% |
| br2_convex_tail | final_range_ge_097 | 7 | $141.64 | $8.66 | 100.00% |
| br2_convex_tail | final_range_lt_078 | 76 | $-145.43 | $145.43 | 0.00% |
| br2_high_skew_load | final_range_078_093_mid_wide | 204 | $-896.51 | $3,824.14 | 60.78% |
| br2_high_skew_load | final_range_093_097 | 21 | $-126.68 | $452.31 | 47.62% |
| br2_high_skew_load | final_range_ge_097 | 17 | $-38.33 | $527.47 | 64.71% |
| br2_high_skew_load | final_range_lt_078 | 335 | $1,677.87 | $5,571.51 | 95.22% |
| br2_late_confirm | final_range_078_093_mid_wide | 190 | $-97.15 | $7,895.33 | 61.05% |
| br2_late_confirm | final_range_093_097 | 17 | $-519.48 | $827.15 | 29.41% |
| br2_late_confirm | final_range_ge_097 | 34 | $-518.83 | $2,204.66 | 52.94% |
| br2_late_confirm | final_range_lt_078 | 568 | $3,623.21 | $23,057.51 | 82.75% |
| br2_late_favourite_load | final_range_078_093_mid_wide | 241 | $-3,341.40 | $8,544.73 | 54.36% |
| br2_late_favourite_load | final_range_093_097 | 24 | $-297.62 | $1,079.17 | 50.00% |
| br2_late_favourite_load | final_range_ge_097 | 27 | $54.89 | $1,563.44 | 77.78% |
| br2_late_favourite_load | final_range_lt_078 | 625 | $6,002.95 | $25,148.35 | 93.28% |

## Live-Safe Regime Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | expanded_chop | 13 | $2.79 | $18.71 | 15.38% |
| br2_convex_tail | expanded_continuation | 25 | $-20.24 | $42.13 | 8.00% |
| br2_convex_tail | expanded_not_decisive | 49 | $7.12 | $89.75 | 6.12% |
| br2_convex_tail | neutral | 12 | $16.88 | $17.83 | 25.00% |
| br2_convex_tail | reversal_pressure | 10 | $-33.32 | $33.32 | 0.00% |
| br2_high_skew_load | expanded_chop | 51 | $-164.70 | $930.81 | 60.78% |
| br2_high_skew_load | expanded_continuation | 42 | $-28.72 | $764.13 | 73.81% |
| br2_high_skew_load | expanded_not_decisive | 312 | $277.03 | $5,569.29 | 81.73% |
| br2_high_skew_load | neutral | 120 | $398.23 | $2,286.98 | 84.17% |
| br2_high_skew_load | reversal_pressure | 52 | $134.51 | $824.22 | 88.46% |
| br2_late_confirm | expanded_not_decisive | 481 | $891.24 | $19,984.88 | 76.72% |
| br2_late_confirm | neutral | 207 | $941.13 | $8,786.86 | 73.91% |
| br2_late_confirm | reversal_pressure | 121 | $655.39 | $5,212.91 | 71.90% |
| br2_late_favourite_load | expanded_chop | 7 | $14.77 | $43.41 | 100.00% |
| br2_late_favourite_load | expanded_continuation | 3 | $1.76 | $13.30 | 66.67% |
| br2_late_favourite_load | expanded_not_decisive | 582 | $1,214.99 | $22,901.55 | 80.41% |
| br2_late_favourite_load | neutral | 210 | $775.98 | $8,857.97 | 84.29% |
| br2_late_favourite_load | reversal_pressure | 115 | $411.31 | $4,519.47 | 80.87% |

## Post-Fill Path Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | held_side | 109 | $-26.77 | $201.75 | 9.17% |
| br2_high_skew_load | crossed_mid_after_fill | 210 | $-1,410.49 | $3,976.01 | 47.62% |
| br2_high_skew_load | held_side | 275 | $1,710.65 | $4,897.08 | 100.00% |
| br2_high_skew_load | large_adverse_then_soft_finish | 3 | $-94.69 | $94.69 | 0.00% |
| br2_high_skew_load | moderate_adverse_excursion | 89 | $410.88 | $1,407.65 | 100.00% |
| br2_late_confirm | crossed_mid_after_fill | 339 | $-4,801.57 | $14,231.12 | 44.54% |
| br2_late_confirm | held_side | 402 | $6,859.16 | $17,383.36 | 98.51% |
| br2_late_confirm | large_adverse_then_soft_finish | 1 | $-26.07 | $26.07 | 0.00% |
| br2_late_confirm | moderate_adverse_excursion | 67 | $456.24 | $2,344.10 | 92.54% |
| br2_late_favourite_load | crossed_mid_after_fill | 326 | $-4,174.34 | $12,952.67 | 49.39% |
| br2_late_favourite_load | held_side | 425 | $4,837.50 | $16,784.36 | 99.76% |
| br2_late_favourite_load | large_adverse_then_soft_finish | 1 | $-102.96 | $102.96 | 0.00% |
| br2_late_favourite_load | moderate_adverse_excursion | 165 | $1,858.61 | $6,495.70 | 98.18% |

## Worst Markets

| Rank | Slug | PnL | Range | Late Fav | Late Confirm | High Skew | Tail | Tail/Fav Cost | Late Fav Wins |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | btc-updown-5m-1774260600 | $-248.30 | 0.855 | $-180.04 | $0.00 | $-68.25 | $0.00 | 0.00% | 0/1 |
| 2 | btc-updown-5m-1774289400 | $-243.73 | 0.925 | $-178.29 | $0.00 | $-65.44 | $0.00 | 0.00% | 0/1 |
| 3 | btc-updown-5m-1774227300 | $-230.56 | 0.784 | $-159.82 | $0.00 | $-70.74 | $0.00 | 0.00% | 0/2 |
| 4 | btc-updown-5m-1774229400 | $-223.60 | 0.844 | $-121.16 | $0.00 | $-102.44 | $0.00 | 0.00% | 0/2 |
| 5 | btc-updown-5m-1773363000 | $-220.00 | 0.925 | $-82.80 | $-120.64 | $-16.56 | $0.00 | 0.00% | 0/1 |
| 6 | btc-updown-5m-1773078900 | $-217.38 | 0.885 | $-196.81 | $0.00 | $-20.57 | $0.00 | 0.00% | 0/3 |
| 7 | btc-updown-5m-1774570200 | $-216.39 | 0.815 | $-121.94 | $-94.45 | $0.00 | $0.00 | 0.00% | 0/2 |
| 8 | btc-updown-5m-1773908100 | $-214.40 | 0.874 | $-214.40 | $0.00 | $0.00 | $0.00 | 0.00% | 0/2 |
| 9 | btc-updown-5m-1772642400 | $-212.45 | 0.785 | $-167.98 | $0.00 | $-44.47 | $0.00 | 0.00% | 0/5 |
| 10 | btc-updown-5m-1774556100 | $-209.75 | 0.875 | $-209.75 | $0.00 | $0.00 | $0.00 | 0.00% | 0/2 |
| 11 | btc-updown-5m-1773932100 | $-197.65 | 0.964 | $-102.96 | $0.00 | $-94.69 | $0.00 | 0.00% | 0/1 |
| 12 | btc-updown-5m-1773273600 | $-186.45 | 0.755 | $0.00 | $-186.45 | $0.00 | $0.00 | 0.00% | 0/0 |
