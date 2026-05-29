# Reversal And Tail Diagnostics

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Window: last `1000` markets.

Definitions:

- `final_range_078_093_mid_wide` is a post-hoc bucket using full resolved-market YES-mid range.
- `expanded_not_decisive` is live-safe: observed range `0.40..0.65` plus sign flips or low path efficiency.

Summary:

- Active markets: `175`
- Total PnL: `$153.55`
- Late-favourite losing markets: `23`
- Late-favourite losing-market cost: `$563.88`
- Tail premium in late-favourite losing markets: `$2.25`
- Tail premium / late-favourite losing cost: `0.40%`
- Total tail premium: `$13.10`
- Total tail PnL: `$37.31`

## Final Range Buckets

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | final_range_078_093_mid_wide | 3 | $-3.54 | $3.54 | 0.00% |
| br2_convex_tail | final_range_093_097 | 1 | $5.71 | $0.43 | 100.00% |
| br2_convex_tail | final_range_ge_097 | 2 | $42.06 | $2.22 | 100.00% |
| br2_convex_tail | final_range_lt_078 | 7 | $-6.92 | $6.92 | 0.00% |
| br2_high_skew_load | final_range_078_093_mid_wide | 30 | $-37.84 | $191.16 | 60.00% |
| br2_high_skew_load | final_range_093_097 | 3 | $-8.57 | $17.91 | 33.33% |
| br2_high_skew_load | final_range_lt_078 | 49 | $67.20 | $297.67 | 93.88% |
| br2_late_confirm | final_range_078_093_mid_wide | 31 | $-62.98 | $503.37 | 54.84% |
| br2_late_confirm | final_range_093_097 | 2 | $11.28 | $34.13 | 100.00% |
| br2_late_confirm | final_range_ge_097 | 5 | $-64.33 | $85.90 | 20.00% |
| br2_late_confirm | final_range_lt_078 | 71 | $233.88 | $1,124.43 | 85.92% |
| br2_late_favourite_load | final_range_078_093_mid_wide | 51 | $-247.41 | $663.18 | 56.86% |
| br2_late_favourite_load | final_range_093_097 | 2 | $-41.97 | $41.97 | 0.00% |
| br2_late_favourite_load | final_range_ge_097 | 3 | $-17.86 | $27.86 | 33.33% |
| br2_late_favourite_load | final_range_lt_078 | 97 | $284.86 | $1,587.59 | 86.60% |

## Live-Safe Regime Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | expanded_chop | 1 | $-0.29 | $0.29 | 0.00% |
| br2_convex_tail | expanded_continuation | 3 | $2.38 | $3.76 | 33.33% |
| br2_convex_tail | expanded_not_decisive | 7 | $31.11 | $5.27 | 14.29% |
| br2_convex_tail | neutral | 2 | $4.11 | $3.78 | 50.00% |
| br2_high_skew_load | expanded_chop | 10 | $-12.27 | $56.01 | 60.00% |
| br2_high_skew_load | expanded_continuation | 5 | $10.66 | $32.60 | 100.00% |
| br2_high_skew_load | expanded_not_decisive | 43 | $5.17 | $261.68 | 79.07% |
| br2_high_skew_load | neutral | 16 | $-3.61 | $108.40 | 75.00% |
| br2_high_skew_load | reversal_pressure | 8 | $20.85 | $48.03 | 100.00% |
| br2_late_confirm | expanded_not_decisive | 65 | $98.08 | $1,045.22 | 78.46% |
| br2_late_confirm | neutral | 30 | $77.61 | $486.64 | 76.67% |
| br2_late_confirm | reversal_pressure | 14 | $-57.85 | $215.96 | 50.00% |
| br2_late_favourite_load | expanded_chop | 3 | $1.92 | $4.78 | 100.00% |
| br2_late_favourite_load | expanded_not_decisive | 106 | $13.37 | $1,630.78 | 75.47% |
| br2_late_favourite_load | neutral | 32 | $-61.73 | $522.71 | 65.62% |
| br2_late_favourite_load | reversal_pressure | 12 | $24.05 | $162.33 | 83.33% |

## Post-Fill Path Labels

| Tag | Bucket | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | held_side | 13 | $37.31 | $13.10 | 23.08% |
| br2_high_skew_load | crossed_mid_after_fill | 33 | $-69.48 | $211.02 | 48.48% |
| br2_high_skew_load | held_side | 38 | $69.82 | $227.68 | 100.00% |
| br2_high_skew_load | moderate_adverse_excursion | 11 | $20.45 | $68.03 | 100.00% |
| br2_late_confirm | crossed_mid_after_fill | 51 | $-242.81 | $810.17 | 47.06% |
| br2_late_confirm | held_side | 51 | $343.04 | $826.24 | 100.00% |
| br2_late_confirm | moderate_adverse_excursion | 7 | $17.60 | $111.41 | 85.71% |
| br2_late_favourite_load | crossed_mid_after_fill | 57 | $-465.44 | $855.17 | 33.33% |
| br2_late_favourite_load | held_side | 64 | $299.62 | $1,021.11 | 100.00% |
| br2_late_favourite_load | moderate_adverse_excursion | 32 | $143.43 | $444.32 | 96.88% |

## Worst Markets

| Rank | Slug | PnL | Range | Late Fav | Late Confirm | High Skew | Tail | Tail/Fav Cost | Late Fav Wins |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | btc-updown-5m-1772444100 | $-100.34 | 0.815 | $-71.56 | $0.00 | $-28.78 | $0.00 | 0.00% | 0/4 |
| 2 | btc-updown-5m-1772404500 | $-64.32 | 0.825 | $-64.32 | $0.00 | $0.00 | $0.00 | 0.00% | 0/4 |
| 3 | btc-updown-5m-1772464500 | $-57.77 | 0.765 | $-52.32 | $0.00 | $-5.45 | $0.00 | 0.00% | 0/4 |
| 4 | btc-updown-5m-1772333400 | $-48.95 | 0.895 | $0.00 | $-48.95 | $0.00 | $0.00 | 0.00% | 0/0 |
| 5 | btc-updown-5m-1772381400 | $-44.55 | 0.810 | $-38.82 | $0.00 | $-5.73 | $0.00 | 0.00% | 0/2 |
| 6 | btc-updown-5m-1772418300 | $-44.02 | 0.730 | $-25.32 | $-18.70 | $0.00 | $0.00 | 0.00% | 0/2 |
| 7 | btc-updown-5m-1772410200 | $-39.88 | 0.975 | $-20.25 | $-54.19 | $0.00 | $34.56 | 8.99% | 0/2 |
| 8 | btc-updown-5m-1772389200 | $-37.35 | 0.835 | $-20.02 | $-17.33 | $0.00 | $0.00 | 0.00% | 0/2 |
| 9 | btc-updown-5m-1772436600 | $-35.00 | 0.885 | $0.00 | $-35.00 | $0.00 | $0.00 | 0.00% | 0/0 |
| 10 | btc-updown-5m-1772483700 | $-31.86 | 0.925 | $-31.86 | $0.00 | $0.00 | $0.00 | 0.00% | 0/2 |
| 11 | btc-updown-5m-1772480100 | $-29.68 | 0.935 | $-29.68 | $0.00 | $0.00 | $0.00 | 0.00% | 0/1 |
| 12 | btc-updown-5m-1772455800 | $-27.11 | 0.850 | $-20.86 | $0.00 | $-6.26 | $0.00 | 0.00% | 0/1 |
