# Reversal And Tail Diagnostics

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Window: last `7750` markets.

Definitions:

- `final_range_078_093_mid_wide` is a post-hoc bucket using full resolved-market YES-mid range.
- `expanded_not_decisive` is live-safe: observed range `0.40..0.65` plus sign flips or low path efficiency.

Summary:

- Active markets: `1177`
- Total PnL: `$5,336.70`
- Late-favourite losing markets: `118`
- Late-favourite losing-market cost: `$5,952.98`
- Tail premium in late-favourite losing markets: `$5.09`
- Tail premium / late-favourite losing cost: `0.09%`
- Total tail premium: `$192.21`
- Total tail PnL: `$-17.23`

## Final Range Buckets

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | final_range_078_093_mid_wide | 22 | $-41.36 | $41.36 | 0.00% |
| br2_convex_tail | final_range_093_097 | 3 | $22.94 | $1.73 | 100.00% |
| br2_convex_tail | final_range_ge_097 | 7 | $141.64 | $8.66 | 100.00% |
| br2_convex_tail | final_range_lt_078 | 75 | $-140.45 | $140.45 | 0.00% |
| br2_high_skew_load | final_range_078_093_mid_wide | 204 | $-896.51 | $3,824.14 | 60.78% |
| br2_high_skew_load | final_range_093_097 | 21 | $-126.68 | $452.31 | 47.62% |
| br2_high_skew_load | final_range_ge_097 | 11 | $-115.50 | $299.13 | 45.45% |
| br2_high_skew_load | final_range_lt_078 | 328 | $1,602.81 | $5,345.45 | 95.12% |
| br2_late_confirm | final_range_078_093_mid_wide | 186 | $-2.75 | $7,500.92 | 61.29% |
| br2_late_confirm | final_range_093_097 | 16 | $-430.23 | $737.90 | 31.25% |
| br2_late_confirm | final_range_ge_097 | 28 | $-582.12 | $1,710.48 | 46.43% |
| br2_late_confirm | final_range_lt_078 | 560 | $3,468.82 | $22,380.64 | 82.50% |
| br2_late_favourite_load | final_range_078_093_mid_wide | 236 | $-3,031.79 | $8,155.95 | 55.08% |
| br2_late_favourite_load | final_range_093_097 | 23 | $-323.64 | $970.74 | 47.83% |
| br2_late_favourite_load | final_range_ge_097 | 18 | $-94.10 | $778.72 | 72.22% |
| br2_late_favourite_load | final_range_lt_078 | 616 | $5,885.62 | $24,428.48 | 93.34% |

## Live-Safe Regime Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | expanded_chop | 12 | $7.35 | $14.16 | 16.67% |
| br2_convex_tail | expanded_continuation | 25 | $-20.24 | $42.13 | 8.00% |
| br2_convex_tail | expanded_not_decisive | 49 | $7.12 | $89.75 | 6.12% |
| br2_convex_tail | neutral | 12 | $16.88 | $17.83 | 25.00% |
| br2_convex_tail | reversal_pressure | 9 | $-28.33 | $28.33 | 0.00% |
| br2_high_skew_load | expanded_chop | 51 | $-164.70 | $930.81 | 60.78% |
| br2_high_skew_load | expanded_continuation | 41 | $-38.24 | $725.60 | 73.17% |
| br2_high_skew_load | expanded_not_decisive | 303 | $168.43 | $5,270.19 | 81.19% |
| br2_high_skew_load | neutral | 119 | $385.26 | $2,247.83 | 84.03% |
| br2_high_skew_load | reversal_pressure | 50 | $113.37 | $746.60 | 88.00% |
| br2_late_confirm | expanded_not_decisive | 467 | $965.51 | $18,721.44 | 76.87% |
| br2_late_confirm | neutral | 204 | $874.25 | $8,502.26 | 73.53% |
| br2_late_confirm | reversal_pressure | 119 | $613.96 | $5,106.25 | 71.43% |
| br2_late_favourite_load | expanded_chop | 7 | $14.77 | $43.41 | 100.00% |
| br2_late_favourite_load | expanded_continuation | 3 | $1.76 | $13.30 | 66.67% |
| br2_late_favourite_load | expanded_not_decisive | 564 | $1,408.56 | $21,447.92 | 80.85% |
| br2_late_favourite_load | neutral | 208 | $707.84 | $8,617.90 | 84.13% |
| br2_late_favourite_load | reversal_pressure | 111 | $303.16 | $4,211.37 | 80.18% |

## Post-Fill Path Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | held_side | 107 | $-17.23 | $192.21 | 9.35% |
| br2_high_skew_load | crossed_mid_after_fill | 206 | $-1,467.54 | $3,823.43 | 46.60% |
| br2_high_skew_load | held_side | 267 | $1,627.06 | $4,634.41 | 100.00% |
| br2_high_skew_load | large_adverse_then_soft_finish | 3 | $-94.69 | $94.69 | 0.00% |
| br2_high_skew_load | moderate_adverse_excursion | 88 | $399.29 | $1,368.49 | 100.00% |
| br2_late_confirm | crossed_mid_after_fill | 332 | $-4,701.10 | $13,602.38 | 44.28% |
| br2_late_confirm | held_side | 393 | $6,772.89 | $16,604.16 | 98.73% |
| br2_late_confirm | large_adverse_then_soft_finish | 1 | $-26.07 | $26.07 | 0.00% |
| br2_late_confirm | moderate_adverse_excursion | 64 | $408.00 | $2,097.33 | 92.19% |
| br2_late_favourite_load | crossed_mid_after_fill | 318 | $-3,767.53 | $12,304.14 | 50.00% |
| br2_late_favourite_load | held_side | 412 | $4,537.20 | $15,787.43 | 99.76% |
| br2_late_favourite_load | large_adverse_then_soft_finish | 1 | $-102.96 | $102.96 | 0.00% |
| br2_late_favourite_load | moderate_adverse_excursion | 162 | $1,769.39 | $6,139.35 | 98.15% |

## Worst Markets

| Rank | Slug | PnL | Range | Late Fav | Late Confirm | High Skew | Tail | Tail/Fav Cost | Late Fav Wins |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | btc-updown-5m-1774260600 | $-248.30 | 0.855 | $-180.04 | $0.00 | $-68.25 | $0.00 | 0.00% | 0/1 |
| 2 | btc-updown-5m-1774289400 | $-243.73 | 0.925 | $-178.29 | $0.00 | $-65.44 | $0.00 | 0.00% | 0/1 |
| 3 | btc-updown-5m-1774227300 | $-230.56 | 0.784 | $-159.82 | $0.00 | $-70.74 | $0.00 | 0.00% | 0/2 |
| 4 | btc-updown-5m-1774229400 | $-223.60 | 0.844 | $-121.16 | $0.00 | $-102.44 | $0.00 | 0.00% | 0/2 |
| 5 | btc-updown-5m-1773363000 | $-220.00 | 0.925 | $-82.80 | $-120.64 | $-16.56 | $0.00 | 0.00% | 0/1 |
| 6 | btc-updown-5m-1773078900 | $-217.38 | 0.885 | $-196.81 | $0.00 | $-20.57 | $0.00 | 0.00% | 0/3 |
| 7 | btc-updown-5m-1773908100 | $-214.40 | 0.874 | $-214.40 | $0.00 | $0.00 | $0.00 | 0.00% | 0/2 |
| 8 | btc-updown-5m-1772642400 | $-212.45 | 0.785 | $-167.98 | $0.00 | $-44.47 | $0.00 | 0.00% | 0/5 |
| 9 | btc-updown-5m-1773932100 | $-197.65 | 0.964 | $-102.96 | $0.00 | $-94.69 | $0.00 | 0.00% | 0/1 |
| 10 | btc-updown-5m-1773273600 | $-186.45 | 0.755 | $0.00 | $-186.45 | $0.00 | $0.00 | 0.00% | 0/0 |
| 11 | btc-updown-5m-1773678300 | $-182.02 | 0.789 | $-28.30 | $-153.71 | $0.00 | $0.00 | 0.00% | 0/1 |
| 12 | btc-updown-5m-1774446300 | $-176.47 | 0.970 | $-135.75 | $0.00 | $-40.72 | $0.00 | 0.00% | 0/1 |
