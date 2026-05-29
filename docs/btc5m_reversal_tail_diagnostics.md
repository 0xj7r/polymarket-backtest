# Reversal And Tail Diagnostics

Source: `/tmp/btc5m_tail08_markets.jsonl`
Window: last `8633` markets.

Definitions:

- `final_range_078_093_mid_wide` is a post-hoc bucket using full resolved-market YES-mid range.
- `expanded_not_decisive` is live-safe: observed range `0.40..0.65` plus sign flips or low path efficiency.

Summary:

- Active markets: `335`
- Total PnL: `$20.80`
- Late-favourite losing markets: `40`
- Late-favourite losing-market cost: `$3,130.67`
- Tail premium in late-favourite losing markets: `$7.64`
- Tail premium / late-favourite losing cost: `0.24%`
- Total tail premium: `$110.76`
- Total tail PnL: `$3.10`

## Final Range Buckets

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | final_range_078_093_mid_wide | 10 | $-21.58 | $21.58 | 0.00% |
| br2_convex_tail | final_range_093_097 | 7 | $-20.75 | $20.75 | 0.00% |
| br2_convex_tail | final_range_ge_097 | 11 | $-28.35 | $28.35 | 0.00% |
| br2_convex_tail | final_range_lt_078 | 12 | $73.78 | $40.08 | 8.33% |
| br2_high_skew_load | final_range_078_093_mid_wide | 50 | $-616.47 | $2,407.64 | 60.00% |
| br2_high_skew_load | final_range_093_097 | 15 | $111.82 | $668.31 | 93.33% |
| br2_high_skew_load | final_range_ge_097 | 64 | $403.02 | $3,111.51 | 89.06% |
| br2_high_skew_load | final_range_lt_078 | 42 | $272.64 | $2,143.73 | 85.71% |
| br2_late_confirm | final_range_078_093_mid_wide | 30 | $-1,300.89 | $3,300.07 | 33.33% |
| br2_late_confirm | final_range_093_097 | 17 | $-163.77 | $2,111.76 | 70.59% |
| br2_late_confirm | final_range_ge_097 | 39 | $688.70 | $4,733.18 | 79.49% |
| br2_late_confirm | final_range_lt_078 | 45 | $690.29 | $5,645.03 | 82.22% |
| br2_late_favourite_load | final_range_078_093_mid_wide | 52 | $-1,067.18 | $3,348.14 | 44.23% |
| br2_late_favourite_load | final_range_093_097 | 34 | $123.56 | $1,886.46 | 79.41% |
| br2_late_favourite_load | final_range_ge_097 | 69 | $518.87 | $4,498.06 | 85.51% |
| br2_late_favourite_load | final_range_lt_078 | 52 | $357.10 | $3,425.82 | 80.77% |

## Live-Safe Regime Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | expanded_chop | 4 | $-6.82 | $6.82 | 0.00% |
| br2_convex_tail | expanded_continuation | 20 | $-54.09 | $54.09 | 0.00% |
| br2_convex_tail | expanded_not_decisive | 7 | $-27.93 | $27.93 | 0.00% |
| br2_convex_tail | neutral | 6 | $98.94 | $14.92 | 16.67% |
| br2_convex_tail | reversal_pressure | 3 | $-7.00 | $7.00 | 0.00% |
| br2_high_skew_load | expanded_chop | 11 | $88.04 | $550.95 | 90.91% |
| br2_high_skew_load | expanded_continuation | 26 | $143.97 | $1,282.69 | 88.46% |
| br2_high_skew_load | expanded_not_decisive | 72 | $96.33 | $3,400.80 | 80.56% |
| br2_high_skew_load | neutral | 45 | $-166.43 | $2,256.00 | 71.11% |
| br2_high_skew_load | reversal_pressure | 17 | $9.10 | $840.74 | 82.35% |
| br2_late_confirm | expanded_not_decisive | 83 | $456.11 | $10,503.81 | 72.29% |
| br2_late_confirm | neutral | 24 | $-450.68 | $2,717.66 | 62.50% |
| br2_late_confirm | reversal_pressure | 24 | $-91.09 | $2,568.57 | 62.50% |
| br2_late_favourite_load | expanded_chop | 1 | $1.64 | $3.36 | 100.00% |
| br2_late_favourite_load | expanded_continuation | 8 | $16.95 | $81.05 | 75.00% |
| br2_late_favourite_load | expanded_not_decisive | 119 | $-298.69 | $7,364.61 | 70.59% |
| br2_late_favourite_load | neutral | 56 | $21.54 | $4,363.25 | 75.00% |
| br2_late_favourite_load | reversal_pressure | 23 | $190.91 | $1,346.21 | 78.26% |

## Post-Fill Path Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | post_fill_path_unavailable | 40 | $3.10 | $110.76 | 2.50% |
| br2_high_skew_load | post_fill_path_unavailable | 171 | $171.01 | $8,331.19 | 80.12% |
| br2_late_confirm | post_fill_path_unavailable | 131 | $-85.67 | $15,790.04 | 68.70% |
| br2_late_favourite_load | post_fill_path_unavailable | 207 | $-67.65 | $13,158.47 | 72.95% |

## Worst Markets

| Rank | Slug | PnL | Range | Late Fav | Late Confirm | High Skew | Tail | Tail/Fav Cost | Late Fav Wins |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | btc-updown-5m-1778120100 | $-247.02 | 0.825 | $-190.94 | $0.00 | $-56.08 | $0.00 | 0.00% | 0/2 |
| 2 | btc-updown-5m-1776816000 | $-233.62 | 0.824 | $0.00 | $-181.54 | $-52.08 | $0.00 | 0.00% | 0/0 |
| 3 | btc-updown-5m-1776875700 | $-232.83 | 0.745 | $-183.37 | $0.00 | $-49.46 | $0.00 | 0.00% | 0/2 |
| 4 | btc-updown-5m-1778069100 | $-217.37 | 0.805 | $-183.93 | $0.00 | $-33.44 | $0.00 | 0.00% | 0/3 |
| 5 | btc-updown-5m-1776872700 | $-216.25 | 0.970 | $-216.25 | $0.00 | $0.00 | $0.00 | 0.00% | 0/3 |
| 6 | btc-updown-5m-1777547400 | $-213.33 | 0.795 | $-105.80 | $0.00 | $-107.53 | $0.00 | 0.00% | 0/1 |
| 7 | btc-updown-5m-1777916100 | $-211.81 | 0.965 | $-46.16 | $-165.65 | $0.00 | $0.00 | 0.00% | 0/2 |
| 8 | btc-updown-5m-1776967800 | $-210.33 | 0.980 | $0.00 | $-210.33 | $0.00 | $0.00 | 0.00% | 0/0 |
| 9 | btc-updown-5m-1776955500 | $-202.27 | 0.805 | $-96.95 | $0.00 | $-105.33 | $0.00 | 0.00% | 0/2 |
| 10 | btc-updown-5m-1777257300 | $-201.49 | 0.970 | $-201.49 | $0.00 | $0.00 | $0.00 | 0.00% | 0/1 |
| 11 | btc-updown-5m-1777380900 | $-190.88 | 0.959 | $0.00 | $-190.88 | $0.00 | $0.00 | 0.00% | 0/0 |
| 12 | btc-updown-5m-1776808800 | $-187.23 | 0.970 | $0.00 | $0.00 | $-187.23 | $0.00 | 0.00% | 0/0 |

