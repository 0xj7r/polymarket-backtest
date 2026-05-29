# Reversal And Tail Diagnostics

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Window: last `4250` markets.

Definitions:

- `final_range_078_093_mid_wide` is a post-hoc bucket using full resolved-market YES-mid range.
- `expanded_not_decisive` is live-safe: observed range `0.40..0.65` plus sign flips or low path efficiency.

Summary:

- Active markets: `836`
- Total PnL: `$2,589.16`
- Late-favourite losing markets: `88`
- Late-favourite losing-market cost: `$3,881.21`
- Tail premium in late-favourite losing markets: `$5.09`
- Tail premium / late-favourite losing cost: `0.13%`
- Total tail premium: `$119.89`
- Total tail PnL: `$7.80`

## Final Range Buckets

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | final_range_078_093_mid_wide | 12 | $-21.33 | $21.33 | 0.00% |
| br2_convex_tail | final_range_093_097 | 3 | $22.94 | $1.73 | 100.00% |
| br2_convex_tail | final_range_ge_097 | 5 | $97.19 | $5.82 | 100.00% |
| br2_convex_tail | final_range_lt_078 | 51 | $-91.01 | $91.01 | 0.00% |
| br2_high_skew_load | final_range_078_093_mid_wide | 126 | $-442.94 | $1,642.51 | 58.73% |
| br2_high_skew_load | final_range_093_097 | 9 | $-51.39 | $102.18 | 33.33% |
| br2_high_skew_load | final_range_ge_097 | 2 | $14.69 | $33.64 | 100.00% |
| br2_high_skew_load | final_range_lt_078 | 240 | $721.14 | $3,011.87 | 95.00% |
| br2_late_confirm | final_range_078_093_mid_wide | 136 | $-163.09 | $4,433.68 | 60.29% |
| br2_late_confirm | final_range_093_097 | 12 | $-332.22 | $479.23 | 33.33% |
| br2_late_confirm | final_range_ge_097 | 12 | $-121.38 | $307.37 | 41.67% |
| br2_late_confirm | final_range_lt_078 | 430 | $1,698.39 | $13,965.54 | 83.02% |
| br2_late_favourite_load | final_range_078_093_mid_wide | 170 | $-2,153.53 | $4,781.36 | 51.76% |
| br2_late_favourite_load | final_range_093_097 | 14 | $-359.63 | $375.70 | 21.43% |
| br2_late_favourite_load | final_range_ge_097 | 11 | $-40.59 | $201.08 | 72.73% |
| br2_late_favourite_load | final_range_lt_078 | 475 | $3,811.93 | $15,968.67 | 93.26% |

## Live-Safe Regime Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | expanded_chop | 8 | $-5.25 | $9.24 | 12.50% |
| br2_convex_tail | expanded_continuation | 10 | $10.55 | $11.34 | 20.00% |
| br2_convex_tail | expanded_not_decisive | 39 | $-3.54 | $70.64 | 5.13% |
| br2_convex_tail | neutral | 9 | $23.18 | $11.53 | 33.33% |
| br2_convex_tail | reversal_pressure | 5 | $-17.14 | $17.14 | 0.00% |
| br2_high_skew_load | expanded_chop | 32 | $-95.72 | $389.72 | 59.38% |
| br2_high_skew_load | expanded_continuation | 25 | $-5.91 | $294.54 | 76.00% |
| br2_high_skew_load | expanded_not_decisive | 211 | $145.77 | $2,721.98 | 82.46% |
| br2_high_skew_load | neutral | 76 | $153.87 | $999.27 | 88.16% |
| br2_high_skew_load | reversal_pressure | 33 | $43.47 | $384.69 | 84.85% |
| br2_late_confirm | expanded_not_decisive | 362 | $863.75 | $11,695.65 | 78.73% |
| br2_late_confirm | neutral | 139 | $107.87 | $4,563.19 | 73.38% |
| br2_late_confirm | reversal_pressure | 89 | $110.08 | $2,926.98 | 68.54% |
| br2_late_favourite_load | expanded_chop | 6 | $7.76 | $22.26 | 100.00% |
| br2_late_favourite_load | expanded_continuation | 2 | $4.63 | $10.43 | 100.00% |
| br2_late_favourite_load | expanded_not_decisive | 428 | $471.41 | $13,047.79 | 79.21% |
| br2_late_favourite_load | neutral | 158 | $706.11 | $5,900.05 | 85.44% |
| br2_late_favourite_load | reversal_pressure | 76 | $68.27 | $2,346.27 | 78.95% |

## Post-Fill Path Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | held_side | 71 | $7.80 | $119.89 | 11.27% |
| br2_high_skew_load | crossed_mid_after_fill | 133 | $-637.31 | $1,742.04 | 47.37% |
| br2_high_skew_load | held_side | 179 | $630.84 | $2,218.46 | 100.00% |
| br2_high_skew_load | moderate_adverse_excursion | 65 | $247.96 | $829.69 | 100.00% |
| br2_late_confirm | crossed_mid_after_fill | 248 | $-2,543.20 | $8,031.16 | 46.77% |
| br2_late_confirm | held_side | 284 | $3,303.96 | $9,413.15 | 98.24% |
| br2_late_confirm | moderate_adverse_excursion | 58 | $320.94 | $1,741.51 | 91.38% |
| br2_late_favourite_load | crossed_mid_after_fill | 231 | $-2,739.40 | $7,282.25 | 45.89% |
| br2_late_favourite_load | held_side | 311 | $2,858.32 | $10,034.18 | 100.00% |
| br2_late_favourite_load | moderate_adverse_excursion | 128 | $1,139.26 | $4,010.38 | 97.66% |

## Worst Markets

| Rank | Slug | PnL | Range | Late Fav | Late Confirm | High Skew | Tail | Tail/Fav Cost | Late Fav Wins |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | btc-updown-5m-1773363000 | $-220.00 | 0.925 | $-82.80 | $-120.64 | $-16.56 | $0.00 | 0.00% | 0/1 |
| 2 | btc-updown-5m-1773078900 | $-217.38 | 0.885 | $-196.81 | $0.00 | $-20.57 | $0.00 | 0.00% | 0/3 |
| 3 | btc-updown-5m-1772642400 | $-212.45 | 0.785 | $-167.98 | $0.00 | $-44.47 | $0.00 | 0.00% | 0/5 |
| 4 | btc-updown-5m-1773273600 | $-186.45 | 0.755 | $0.00 | $-186.45 | $0.00 | $0.00 | 0.00% | 0/0 |
| 5 | btc-updown-5m-1773352500 | $-155.74 | 0.885 | $-112.10 | $0.00 | $-43.64 | $0.00 | 0.00% | 0/2 |
| 6 | btc-updown-5m-1772775000 | $-146.79 | 0.800 | $-111.99 | $0.00 | $-34.80 | $0.00 | 0.00% | 0/3 |
| 7 | btc-updown-5m-1772758800 | $-132.87 | 0.745 | $-107.25 | $0.00 | $-25.62 | $0.00 | 0.00% | 0/4 |
| 8 | btc-updown-5m-1773370800 | $-131.49 | 0.725 | $0.00 | $-112.61 | $-18.88 | $0.00 | 0.00% | 0/0 |
| 9 | btc-updown-5m-1773096300 | $-127.00 | 0.685 | $-20.75 | $-106.25 | $0.00 | $0.00 | 0.00% | 0/1 |
| 10 | btc-updown-5m-1772902500 | $-126.49 | 0.845 | $-88.94 | $0.00 | $-37.55 | $0.00 | 0.00% | 0/3 |
| 11 | btc-updown-5m-1773173400 | $-125.63 | 0.865 | $-70.79 | $-54.84 | $0.00 | $0.00 | 0.00% | 0/1 |
| 12 | btc-updown-5m-1773015000 | $-119.43 | 0.650 | $0.00 | $-119.43 | $0.00 | $0.00 | 0.00% | 0/0 |

