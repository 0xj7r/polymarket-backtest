# Reversal And Tail Diagnostics

Source: `/tmp/btc5m_postfill_markets_062901.jsonl`
Window: last `10500` markets.

Definitions:

- `final_range_078_093_mid_wide` is a post-hoc bucket using full resolved-market YES-mid range.
- `expanded_not_decisive` is live-safe: observed range `0.40..0.65` plus sign flips or low path efficiency.

Summary:

- Active markets: `1341`
- Total PnL: `$6,309.67`
- Late-favourite losing markets: `130`
- Late-favourite losing-market cost: `$6,990.43`
- Tail premium in late-favourite losing markets: `$5.09`
- Tail premium / late-favourite losing cost: `0.07%`
- Total tail premium: `$221.69`
- Total tail PnL: `$-46.71`

## Final Range Buckets

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | final_range_078_093_mid_wide | 25 | $-50.42 | $50.42 | 0.00% |
| br2_convex_tail | final_range_093_097 | 6 | $10.22 | $14.45 | 50.00% |
| br2_convex_tail | final_range_ge_097 | 10 | $138.92 | $11.38 | 70.00% |
| br2_convex_tail | final_range_lt_078 | 76 | $-145.43 | $145.43 | 0.00% |
| br2_high_skew_load | final_range_078_093_mid_wide | 216 | $-957.09 | $4,276.89 | 61.11% |
| br2_high_skew_load | final_range_093_097 | 36 | $-91.51 | $1,060.61 | 61.11% |
| br2_high_skew_load | final_range_ge_097 | 37 | $-76.37 | $1,272.07 | 70.27% |
| br2_high_skew_load | final_range_lt_078 | 345 | $1,784.61 | $5,903.00 | 95.36% |
| br2_late_confirm | final_range_078_093_mid_wide | 203 | $-176.97 | $8,982.80 | 60.59% |
| br2_late_confirm | final_range_093_097 | 28 | $-792.67 | $1,826.26 | 39.29% |
| br2_late_confirm | final_range_ge_097 | 48 | $-567.18 | $3,588.39 | 56.25% |
| br2_late_confirm | final_range_lt_078 | 596 | $3,887.16 | $25,415.37 | 82.72% |
| br2_late_favourite_load | final_range_078_093_mid_wide | 248 | $-3,572.14 | $8,988.13 | 54.84% |
| br2_late_favourite_load | final_range_093_097 | 46 | $-10.28 | $2,877.29 | 69.57% |
| br2_late_favourite_load | final_range_ge_097 | 49 | $386.31 | $3,076.40 | 81.63% |
| br2_late_favourite_load | final_range_lt_078 | 646 | $6,542.51 | $27,105.71 | 93.34% |

## Live-Safe Regime Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | expanded_chop | 14 | $1.61 | $19.90 | 14.29% |
| br2_convex_tail | expanded_continuation | 31 | $-34.39 | $56.28 | 6.45% |
| br2_convex_tail | expanded_not_decisive | 49 | $7.12 | $89.75 | 6.12% |
| br2_convex_tail | neutral | 13 | $12.26 | $22.45 | 23.08% |
| br2_convex_tail | reversal_pressure | 10 | $-33.32 | $33.32 | 0.00% |
| br2_high_skew_load | expanded_chop | 58 | $-283.40 | $1,193.10 | 58.62% |
| br2_high_skew_load | expanded_continuation | 54 | $6.87 | $1,190.80 | 75.93% |
| br2_high_skew_load | expanded_not_decisive | 336 | $310.51 | $6,446.55 | 81.55% |
| br2_high_skew_load | neutral | 129 | $494.50 | $2,664.13 | 85.27% |
| br2_high_skew_load | reversal_pressure | 57 | $131.15 | $1,017.98 | 87.72% |
| br2_late_confirm | expanded_not_decisive | 526 | $1,281.56 | $23,983.75 | 76.62% |
| br2_late_confirm | neutral | 217 | $636.05 | $9,593.84 | 72.81% |
| br2_late_confirm | reversal_pressure | 132 | $432.73 | $6,235.23 | 70.45% |
| br2_late_favourite_load | expanded_chop | 7 | $14.77 | $43.41 | 100.00% |
| br2_late_favourite_load | expanded_continuation | 6 | $6.47 | $45.32 | 66.67% |
| br2_late_favourite_load | expanded_not_decisive | 629 | $1,939.63 | $26,974.14 | 81.24% |
| br2_late_favourite_load | neutral | 226 | $1,035.34 | $10,060.58 | 84.51% |
| br2_late_favourite_load | reversal_pressure | 121 | $350.20 | $4,924.08 | 80.99% |

## Post-Fill Path Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | held_side | 117 | $-46.71 | $221.69 | 8.55% |
| br2_high_skew_load | crossed_mid_after_fill | 231 | $-1,752.51 | $4,787.35 | 47.19% |
| br2_high_skew_load | held_side | 303 | $2,005.81 | $5,911.72 | 100.00% |
| br2_high_skew_load | large_adverse_then_soft_finish | 3 | $-94.69 | $94.69 | 0.00% |
| br2_high_skew_load | moderate_adverse_excursion | 97 | $501.03 | $1,718.81 | 100.00% |
| br2_late_confirm | crossed_mid_after_fill | 371 | $-6,004.82 | $17,236.27 | 43.67% |
| br2_late_confirm | held_side | 429 | $7,747.12 | $19,569.85 | 98.60% |
| br2_late_confirm | large_adverse_then_soft_finish | 1 | $-26.07 | $26.07 | 0.00% |
| br2_late_confirm | moderate_adverse_excursion | 74 | $634.11 | $2,980.63 | 93.24% |
| br2_late_favourite_load | crossed_mid_after_fill | 343 | $-4,538.80 | $14,184.81 | 49.56% |
| br2_late_favourite_load | held_side | 466 | $5,774.54 | $19,910.42 | 99.79% |
| br2_late_favourite_load | large_adverse_then_soft_finish | 1 | $-102.96 | $102.96 | 0.00% |
| br2_late_favourite_load | moderate_adverse_excursion | 179 | $2,213.63 | $7,849.34 | 98.32% |

## Worst Markets

| Rank | Slug | PnL | Range | Late Fav | Late Confirm | High Skew | Tail | Tail/Fav Cost | Late Fav Wins |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | btc-updown-5m-1774260600 | $-248.30 | 0.855 | $-180.04 | $0.00 | $-68.25 | $0.00 | 0.00% | 0/1 |
| 2 | btc-updown-5m-1774289400 | $-243.73 | 0.925 | $-178.29 | $0.00 | $-65.44 | $0.00 | 0.00% | 0/1 |
| 3 | btc-updown-5m-1774884000 | $-232.24 | 0.875 | $0.00 | $-199.83 | $-32.42 | $0.00 | 0.00% | 0/0 |
| 4 | btc-updown-5m-1774227300 | $-230.56 | 0.784 | $-159.82 | $0.00 | $-70.74 | $0.00 | 0.00% | 0/2 |
| 5 | btc-updown-5m-1774229400 | $-223.60 | 0.844 | $-121.16 | $0.00 | $-102.44 | $0.00 | 0.00% | 0/2 |
| 6 | btc-updown-5m-1773363000 | $-220.00 | 0.925 | $-82.80 | $-120.64 | $-16.56 | $0.00 | 0.00% | 0/1 |
| 7 | btc-updown-5m-1773078900 | $-217.38 | 0.885 | $-196.81 | $0.00 | $-20.57 | $0.00 | 0.00% | 0/3 |
| 8 | btc-updown-5m-1774570200 | $-216.39 | 0.815 | $-121.94 | $-94.45 | $0.00 | $0.00 | 0.00% | 0/2 |
| 9 | btc-updown-5m-1773908100 | $-214.40 | 0.874 | $-214.40 | $0.00 | $0.00 | $0.00 | 0.00% | 0/2 |
| 10 | btc-updown-5m-1775055300 | $-213.82 | 0.704 | $0.00 | $-213.82 | $0.00 | $0.00 | 0.00% | 0/0 |
| 11 | btc-updown-5m-1772642400 | $-212.45 | 0.785 | $-167.98 | $0.00 | $-44.47 | $0.00 | 0.00% | 0/5 |
| 12 | btc-updown-5m-1774556100 | $-209.75 | 0.875 | $-209.75 | $0.00 | $0.00 | $0.00 | 0.00% | 0/2 |

