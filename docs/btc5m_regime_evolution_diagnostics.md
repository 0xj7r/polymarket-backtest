# BTC5m Regime Evolution Diagnostics

Source: `s3://pm-research-backtest-prod/results/20260528T225810Z-portfolio-grid-52322/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8/markets.jsonl`
Range: `2026-02-27T15:40:00+00:00` to `2026-05-20T23:55:00+00:00`

Core post-hoc regime: `range_078_093_midwide` means final resolved-market YES-mid range is at least `0.78` and below `0.93`.
This is not directly tradable; it is the label we are trying to explain with live-safe features.

## Window Regime Evolution

| Window | Markets | Active | Active Rate | Fills | PnL | Return | Mid-Wide Markets | Mid-Wide Rate | Mid-Wide Active | Mid-Wide PnL |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| first_third | 7901 | 1191 | 15.07% | 2374 | $5,072.67 | 507.27% | 1425 | 18.04% | 357 | $-4,398.56 |
| middle_third | 7902 | 569 | 7.20% | 943 | $4,144.62 | 68.25% | 935 | 11.83% | 110 | $-1,983.86 |
| last_third | 7902 | 256 | 3.24% | 406 | $-227.08 | -2.22% | 949 | 12.01% | 69 | $-2,650.65 |
| first_30d | 8640 | 1214 | 14.05% | 2413 | $5,394.12 | 539.41% | 1512 | 17.50% | 360 | $-4,380.98 |
| last_30d | 8633 | 335 | 3.88% | 549 | $20.80 | 0.21% | 1039 | 12.04% | 87 | $-3,006.12 |
| last_14d | 4025 | 100 | 2.48% | 147 | $-200.77 | -1.97% | 502 | 12.47% | 30 | $-850.46 |
| last_7d | 2013 | 61 | 3.03% | 87 | $-24.27 | -0.24% | 250 | 12.42% | 17 | $-251.01 |

## Weekly Drift

| Week | Markets | Active | Active Rate | PnL | Mid-Wide Markets | Mid-Wide Rate | Mid-Wide PnL |
|---|---:|---:|---:|---:|---:|---:|---:|
| 2026-02-23 | 676 | 95 | 14.05% | $158.27 | 138 | 20.41% | $-271.67 |
| 2026-03-02 | 2016 | 434 | 21.53% | $1,626.72 | 386 | 19.15% | $-883.05 |
| 2026-03-09 | 2016 | 319 | 15.82% | $758.13 | 361 | 17.91% | $-1,689.53 |
| 2026-03-16 | 2016 | 210 | 10.42% | $2,207.27 | 373 | 18.50% | $-52.39 |
| 2026-03-23 | 2016 | 158 | 7.84% | $751.49 | 265 | 13.14% | $-1,484.33 |
| 2026-03-30 | 2016 | 131 | 6.50% | $862.32 | 242 | 12.00% | $-406.46 |
| 2026-04-06 | 2016 | 143 | 7.09% | $1,043.49 | 261 | 12.95% | $-791.16 |
| 2026-04-13 | 2012 | 166 | 8.25% | $1,861.69 | 205 | 10.19% | $-467.91 |
| 2026-04-20 | 2016 | 152 | 7.54% | $305.34 | 251 | 12.45% | $-881.49 |
| 2026-04-27 | 2016 | 57 | 2.83% | $-429.63 | 231 | 11.46% | $-592.79 |
| 2026-05-04 | 2012 | 75 | 3.73% | $31.03 | 233 | 11.58% | $-1,103.54 |
| 2026-05-11 | 2013 | 39 | 1.94% | $-589.11 | 250 | 12.42% | $-244.30 |
| 2026-05-18 | 864 | 37 | 4.28% | $403.19 | 113 | 13.08% | $-164.44 |

### Weekly Mid-Wide Lane PnL

| Week | Lane | Fills | PnL | Cost | Win Rate |
|---|---|---:|---:|---:|---:|
| 2026-02-23 | br2_high_skew_load | 11 | $-28.36 | $68.68 | 45.45% |
| 2026-02-23 | br2_late_confirm | 18 | $-105.90 | $289.98 | 38.89% |
| 2026-02-23 | br2_late_favourite_load | 23 | $-137.41 | $245.10 | 47.83% |
| 2026-03-02 | br2_convex_tail | 6 | $-8.10 | $8.10 | 0.00% |
| 2026-03-02 | br2_high_skew_load | 63 | $-128.26 | $593.06 | 61.90% |
| 2026-03-02 | br2_late_confirm | 74 | $3.34 | $1,886.26 | 64.86% |
| 2026-03-02 | br2_late_favourite_load | 92 | $-750.03 | $2,070.50 | 55.43% |
| 2026-03-09 | br2_convex_tail | 6 | $-13.23 | $13.23 | 0.00% |
| 2026-03-09 | br2_high_skew_load | 54 | $-276.20 | $1,014.86 | 59.26% |
| 2026-03-09 | br2_late_confirm | 49 | $-161.19 | $2,471.04 | 59.18% |
| 2026-03-09 | br2_late_favourite_load | 57 | $-1,238.92 | $2,562.17 | 49.12% |
| 2026-03-16 | br2_convex_tail | 7 | $-14.44 | $14.44 | 0.00% |
| 2026-03-16 | br2_high_skew_load | 43 | $-12.07 | $1,034.94 | 76.74% |
| 2026-03-16 | br2_late_confirm | 34 | $98.90 | $1,918.16 | 64.71% |
| 2026-03-16 | br2_late_favourite_load | 40 | $-124.79 | $1,782.02 | 70.00% |
| 2026-03-23 | br2_convex_tail | 4 | $-10.16 | $10.16 | 0.00% |
| 2026-03-23 | br2_high_skew_load | 33 | $-451.62 | $1,112.59 | 45.45% |
| 2026-03-23 | br2_late_confirm | 15 | $67.71 | $1,329.89 | 66.67% |
| 2026-03-23 | br2_late_favourite_load | 29 | $-1,090.26 | $1,884.93 | 44.83% |
| 2026-03-30 | br2_convex_tail | 2 | $-4.50 | $4.50 | 0.00% |
| 2026-03-30 | br2_high_skew_load | 12 | $-60.58 | $452.75 | 66.67% |
| 2026-03-30 | br2_late_confirm | 14 | $-4.96 | $1,198.88 | 57.14% |
| 2026-03-30 | br2_late_favourite_load | 9 | $-336.42 | $549.09 | 55.56% |
| 2026-04-06 | br2_convex_tail | 1 | $-0.87 | $0.87 | 0.00% |
| 2026-04-06 | br2_high_skew_load | 14 | $-54.04 | $605.92 | 71.43% |
| 2026-04-06 | br2_late_confirm | 14 | $-463.76 | $1,350.40 | 35.71% |
| 2026-04-06 | br2_late_favourite_load | 16 | $-272.49 | $887.06 | 68.75% |
| 2026-04-13 | br2_convex_tail | 1 | $-1.52 | $1.52 | 0.00% |
| 2026-04-13 | br2_high_skew_load | 15 | $-53.73 | $768.39 | 73.33% |
| 2026-04-13 | br2_late_confirm | 13 | $-128.69 | $1,774.64 | 61.54% |
| 2026-04-13 | br2_late_favourite_load | 17 | $-283.97 | $1,171.52 | 58.82% |
| 2026-04-20 | br2_convex_tail | 3 | $-14.91 | $14.91 | 0.00% |
| 2026-04-20 | br2_high_skew_load | 24 | $-166.91 | $1,210.20 | 66.67% |
| 2026-04-20 | br2_late_confirm | 16 | $-359.84 | $1,867.34 | 50.00% |
| 2026-04-20 | br2_late_favourite_load | 22 | $-339.82 | $1,817.15 | 50.00% |
| 2026-04-27 | br2_convex_tail | 2 | $-2.62 | $2.62 | 0.00% |
| 2026-04-27 | br2_high_skew_load | 4 | $-141.56 | $213.14 | 25.00% |
| 2026-04-27 | br2_late_confirm | 7 | $-400.46 | $688.60 | 28.57% |
| 2026-04-27 | br2_late_favourite_load | 4 | $-48.14 | $307.26 | 50.00% |
| 2026-05-04 | br2_convex_tail | 1 | $-0.19 | $0.19 | 0.00% |
| 2026-05-04 | br2_high_skew_load | 12 | $-180.85 | $574.69 | 58.33% |
| 2026-05-04 | br2_late_confirm | 4 | $-286.45 | $544.43 | 25.00% |
| 2026-05-04 | br2_late_favourite_load | 17 | $-636.06 | $911.49 | 29.41% |
| 2026-05-11 | br2_convex_tail | 2 | $-2.04 | $2.04 | 0.00% |
| 2026-05-11 | br2_high_skew_load | 5 | $-89.29 | $217.64 | 60.00% |
| 2026-05-11 | br2_late_confirm | 3 | $-53.06 | $313.13 | 33.33% |
| 2026-05-11 | br2_late_favourite_load | 5 | $-99.90 | $263.26 | 60.00% |
| 2026-05-18 | br2_convex_tail | 2 | $-1.82 | $1.82 | 0.00% |
| 2026-05-18 | br2_high_skew_load | 6 | $-19.76 | $249.60 | 66.67% |
| 2026-05-18 | br2_late_confirm | 3 | $-0.07 | $237.06 | 33.33% |
| 2026-05-18 | br2_late_favourite_load | 7 | $-142.79 | $459.55 | 42.86% |

## Last-Window Final Range Buckets

| Lane | Bucket | Fills | PnL | Cost | Win Rate | PnL/Fill |
|---|---|---:|---:|---:|---:|---:|
| br2_convex_tail | range_ge_097 | 11 | $-28.35 | $28.35 | 0.00% | $-2.58 |
| br2_convex_tail | range_078_093_midwide | 10 | $-21.58 | $21.58 | 0.00% | $-2.16 |
| br2_convex_tail | range_050_078 | 10 | $75.24 | $38.61 | 10.00% | $7.52 |
| br2_high_skew_load | range_078_093_midwide | 50 | $-616.47 | $2,407.64 | 60.00% | $-12.33 |
| br2_high_skew_load | range_093_097 | 15 | $111.82 | $668.31 | 93.33% | $7.45 |
| br2_high_skew_load | range_050_078 | 38 | $212.52 | $1,919.11 | 84.21% | $5.59 |
| br2_high_skew_load | range_ge_097 | 64 | $403.02 | $3,111.51 | 89.06% | $6.30 |
| br2_late_confirm | range_078_093_midwide | 30 | $-1,300.89 | $3,300.07 | 33.33% | $-43.36 |
| br2_late_confirm | range_093_097 | 17 | $-163.77 | $2,111.76 | 70.59% | $-9.63 |
| br2_late_confirm | range_ge_097 | 39 | $688.70 | $4,733.18 | 79.49% | $17.66 |
| br2_late_confirm | range_050_078 | 42 | $747.29 | $5,217.20 | 83.33% | $17.79 |
| br2_late_favourite_load | range_078_093_midwide | 52 | $-1,067.18 | $3,348.14 | 44.23% | $-20.52 |
| br2_late_favourite_load | range_093_097 | 34 | $123.56 | $1,886.46 | 79.41% | $3.63 |
| br2_late_favourite_load | range_050_078 | 49 | $295.36 | $3,254.99 | 79.59% | $6.03 |
| br2_late_favourite_load | range_ge_097 | 69 | $518.87 | $4,498.06 | 85.51% | $7.52 |

## Last-Window Observed Range Buckets

| Lane | Bucket | Fills | PnL | Cost | Win Rate | PnL/Fill |
|---|---|---:|---:|---:|---:|---:|
| br2_convex_tail | obs_ge_065 | 27 | $-67.91 | $67.91 | 0.00% | $-2.52 |
| br2_convex_tail | obs_050_065 | 8 | $86.82 | $27.04 | 12.50% | $10.85 |
| br2_high_skew_load | obs_050_065 | 66 | $-242.85 | $3,117.75 | 74.24% | $-3.68 |
| br2_high_skew_load | obs_lt_040 | 32 | $36.30 | $1,570.37 | 78.12% | $1.13 |
| br2_high_skew_load | obs_040_050 | 33 | $108.78 | $1,671.26 | 81.82% | $3.30 |
| br2_high_skew_load | obs_ge_065 | 40 | $268.78 | $1,971.81 | 90.00% | $6.72 |
| br2_late_confirm | obs_lt_040 | 35 | $-234.21 | $3,663.34 | 65.71% | $-6.69 |
| br2_late_confirm | obs_040_050 | 92 | $110.44 | $11,634.03 | 69.57% | $1.20 |
| br2_late_favourite_load | obs_040_050 | 57 | $-660.43 | $3,141.41 | 64.91% | $-11.59 |
| br2_late_favourite_load | obs_lt_040 | 51 | $6.39 | $3,941.59 | 68.63% | $0.13 |
| br2_late_favourite_load | obs_ge_065 | 9 | $18.59 | $84.40 | 77.78% | $2.07 |
| br2_late_favourite_load | obs_050_065 | 90 | $567.80 | $5,991.07 | 80.00% | $6.31 |

## Last-Window Live Regime Labels

| Lane | Bucket | Fills | PnL | Cost | Win Rate | PnL/Fill |
|---|---|---:|---:|---:|---:|---:|
| br2_convex_tail | expanded_continuation | 20 | $-54.09 | $54.09 | 0.00% | $-2.70 |
| br2_high_skew_load | neutral | 45 | $-166.43 | $2,256.00 | 71.11% | $-3.70 |
| br2_high_skew_load | reversal_pressure | 17 | $9.10 | $840.74 | 82.35% | $0.54 |
| br2_high_skew_load | expanded_chop | 11 | $88.04 | $550.95 | 90.91% | $8.00 |
| br2_high_skew_load | expanded_not_decisive | 72 | $96.33 | $3,400.80 | 80.56% | $1.34 |
| br2_high_skew_load | expanded_continuation | 26 | $143.97 | $1,282.69 | 88.46% | $5.54 |
| br2_late_confirm | neutral | 24 | $-450.68 | $2,717.66 | 62.50% | $-18.78 |
| br2_late_confirm | reversal_pressure | 24 | $-91.09 | $2,568.57 | 62.50% | $-3.80 |
| br2_late_confirm | expanded_not_decisive | 83 | $456.11 | $10,503.81 | 72.29% | $5.50 |
| br2_late_favourite_load | expanded_not_decisive | 119 | $-298.69 | $7,364.61 | 70.59% | $-2.51 |
| br2_late_favourite_load | expanded_continuation | 8 | $16.95 | $81.05 | 75.00% | $2.12 |
| br2_late_favourite_load | neutral | 56 | $21.54 | $4,363.25 | 75.00% | $0.38 |
| br2_late_favourite_load | reversal_pressure | 23 | $190.91 | $1,346.21 | 78.26% | $8.30 |

## Feature Drift

| Lane | Feature | Early Median | Late Median | Delta | Early P25..P75 | Late P25..P75 |
|---|---|---:|---:|---:|---:|---:|
| br2_late_favourite_load | market_yes_range_so_far | 0.4800 | 0.4800 | 0.0000 | 0.4150..0.5350 | 0.4000..0.5500 |
| br2_late_favourite_load | regime_whipsaw_score | 0.2829 | 0.2437 | -0.0392 | 0.2371..0.3638 | 0.2178..0.2779 |
| br2_late_favourite_load | regime_path_efficiency | 0.1368 | 0.1322 | -0.0046 | 0.0768..0.2111 | 0.0861..0.1988 |
| br2_late_favourite_load | regime_reversal_pressure | 0.3000 | 0.2800 | -0.0200 | 0.2600..0.3600 | 0.2400..0.3400 |
| br2_late_favourite_load | regime_sign_flip_rate | 0.4286 | 0.4000 | -0.0286 | 0.3714..0.4857 | 0.3429..0.4571 |
| br2_late_favourite_load | regime_realized_vol_180s_bps | 1.9921 | 1.5861 | -0.4060 | 1.5322..2.7708 | 1.3824..1.8590 |
| br2_late_favourite_load | side_edge_vs_fill | 0.1123 | 0.1099 | -0.0024 | 0.0993..0.1330 | 0.0986..0.1334 |
| br2_late_favourite_load | confidence_score | 0.8940 | 0.8984 | 0.0044 | 0.8700..0.9146 | 0.8690..0.9205 |
| br2_late_confirm | market_yes_range_so_far | 0.4350 | 0.4400 | 0.0050 | 0.3900..0.4700 | 0.3950..0.4700 |
| br2_late_confirm | regime_whipsaw_score | 0.2624 | 0.2461 | -0.0163 | 0.2274..0.3040 | 0.2164..0.2808 |
| br2_late_confirm | regime_path_efficiency | 0.1020 | 0.1157 | 0.0137 | 0.0503..0.1859 | 0.0610..0.1917 |
| br2_late_confirm | regime_reversal_pressure | 0.3000 | 0.2800 | -0.0200 | 0.2600..0.3600 | 0.2400..0.3400 |
| br2_late_confirm | regime_sign_flip_rate | 0.4286 | 0.4000 | -0.0286 | 0.3429..0.4857 | 0.3429..0.4571 |
| br2_late_confirm | regime_realized_vol_180s_bps | 1.6370 | 1.5033 | -0.1338 | 1.3895..1.9832 | 1.3368..1.8083 |
| br2_late_confirm | side_edge_vs_fill | 0.0649 | 0.0628 | -0.0021 | 0.0386..0.0989 | 0.0370..0.0871 |
| br2_late_confirm | confidence_score | 0.8880 | 0.8845 | -0.0035 | 0.8052..0.9141 | 0.8093..0.9106 |
| br2_high_skew_load | market_yes_range_so_far | 0.5300 | 0.5600 | 0.0300 | 0.4600..0.6100 | 0.4400..0.6400 |
| br2_high_skew_load | regime_whipsaw_score | 0.2709 | 0.2463 | -0.0246 | 0.2294..0.3403 | 0.2150..0.2959 |
| br2_high_skew_load | regime_path_efficiency | 0.1610 | 0.1521 | -0.0089 | 0.0914..0.2474 | 0.0952..0.2020 |
| br2_high_skew_load | regime_reversal_pressure | 0.3000 | 0.2800 | -0.0200 | 0.2600..0.3600 | 0.2400..0.3400 |
| br2_high_skew_load | regime_sign_flip_rate | 0.4000 | 0.4000 | 0.0000 | 0.3429..0.4857 | 0.3429..0.4571 |
| br2_high_skew_load | regime_realized_vol_180s_bps | 1.9084 | 1.6390 | -0.2694 | 1.5253..2.6364 | 1.3835..1.9717 |
| br2_high_skew_load | side_edge_vs_fill | 0.1079 | 0.1005 | -0.0073 | 0.0985..0.1226 | 0.0955..0.1142 |
| br2_high_skew_load | confidence_score | 0.8990 | 0.9029 | 0.0039 | 0.8787..0.9189 | 0.8767..0.9238 |
| br2_convex_tail | market_yes_range_so_far | 0.6300 | 0.6900 | 0.0600 | 0.5500..0.7200 | 0.5800..0.8100 |
| br2_convex_tail | regime_whipsaw_score | 0.2559 | 0.2184 | -0.0375 | 0.2171..0.3121 | 0.1965..0.2574 |
| br2_convex_tail | regime_path_efficiency | 0.2154 | 0.2986 | 0.0832 | 0.1209..0.3340 | 0.1678..0.3400 |
| br2_convex_tail | regime_reversal_pressure | 0.3000 | 0.2800 | -0.0200 | 0.2400..0.3400 | 0.2600..0.3000 |
| br2_convex_tail | regime_sign_flip_rate | 0.4286 | 0.4000 | -0.0286 | 0.3429..0.4857 | 0.3429..0.4286 |
| br2_convex_tail | regime_realized_vol_180s_bps | 1.8451 | 1.5262 | -0.3188 | 1.4356..2.3271 | 1.3183..1.9689 |
| br2_convex_tail | side_edge_vs_fill | -0.0001 | -0.0001 | 0.0000 | -0.0101..0.0100 | -0.0101..0.0100 |
| br2_convex_tail | confidence_score | 0.8951 | 0.8920 | -0.0031 | 0.8742..0.9151 | 0.8746..0.9110 |

## Candidate Guardrail Diagnostics

Rows show what would happen if a rule removed those fills. Post-hoc rules are explicitly marked and are not deployable as-is.

| Rule | Lane | Early Removed Fills | Early Removed PnL | Late Removed Fills | Late Removed PnL | Late Kept PnL | Late PnL If Removed |
|---|---|---:|---:|---:|---:|---:|---:|
| `late_confirm_range_ge_050` | br2_late_confirm | 31 | $49.87 | 4 | $38.10 | $-123.77 | $-123.77 |
| `late_confirm_sign_flip_040_0457` | br2_late_confirm | 304 | $190.19 | 42 | $-1,091.84 | $1,006.17 | $1,006.17 |
| `late_confirm_reversal_024_034` | br2_late_confirm | 386 | $828.37 | 63 | $-1,063.99 | $978.33 | $978.33 |
| `late_confirm_low_conf_lt_081` | br2_late_confirm | 211 | $303.78 | 33 | $-605.36 | $519.69 | $519.69 |
| `late_fav_obs_040_050` | br2_late_favourite_load | 337 | $832.54 | 57 | $-660.43 | $592.78 | $592.78 |
| `late_fav_low_price_lt_076` | br2_late_favourite_load | 395 | $723.55 | 99 | $-431.86 | $364.21 | $364.21 |
| `late_fav_high_price_ge_079` | br2_late_favourite_load | 246 | $670.75 | 58 | $-400.72 | $333.07 | $333.07 |
| `high_skew_final_midwide_posthoc` | br2_high_skew_load | 204 | $-896.51 | 50 | $-616.47 | $787.48 | $787.48 |
| `late_confirm_final_midwide_posthoc` | br2_late_confirm | 190 | $-97.15 | 30 | $-1,300.89 | $1,215.22 | $1,215.22 |
| `late_fav_final_midwide_posthoc` | br2_late_favourite_load | 241 | $-3,341.40 | 52 | $-1,067.18 | $999.53 | $999.53 |

