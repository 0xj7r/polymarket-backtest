# BTC5m Post-Fill Regime Evolution

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Markets: `2000`
Calendar: `2026-02-27T15:40:00+00:00` to `2026-03-06T14:15:00+00:00`
Missing post-fill paths on tracked lane fills: `0`

This report uses post-fill labels only as diagnostics/training targets. Candidate gates still need to use replay-safe fill-time features.

## Market Periods

| Bucket | Markets | Active | PnL | Active Rate |
|---|---:|---:|---:|---:|
| first_third | 667 | 94 | $132.36 | 14.09% |
| last_third | 666 | 184 | $650.71 | 27.63% |
| middle_third | 667 | 175 | $364.75 | 26.24% |

## Recent Split (3d)

| Bucket | Markets | Active | PnL | Active Rate |
|---|---:|---:|---:|---:|
| last_3d | 864 | 251 | $954.16 | 29.05% |
| pre_last_3d | 1136 | 202 | $193.65 | 17.78% |

## Daily PnL (Last 45 Calendar Rows In Artifact)

| Bucket | Markets | Active | PnL | Active Rate |
|---|---:|---:|---:|---:|
| 2026-02-27 | 100 | 15 | $31.19 | 15.00% |
| 2026-02-28 | 288 | 27 | $114.17 | 9.38% |
| 2026-03-01 | 288 | 53 | $12.91 | 18.40% |
| 2026-03-02 | 288 | 69 | $-75.95 | 23.96% |
| 2026-03-03 | 288 | 73 | $307.92 | 25.35% |
| 2026-03-04 | 288 | 99 | $437.38 | 34.38% |
| 2026-03-05 | 288 | 73 | $523.26 | 25.35% |
| 2026-03-06 | 172 | 44 | $-203.06 | 25.58% |

## By Lane

| Bucket | Fills | PnL | Cost | Win Rate | Cross-Mid Rate | Avg Adverse | Log Loss | Brier |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_convex_tail | 39 | $9.93 | $56.23 | 10.26% | 0.00% | 0.0437 | 0.3534 | 0.0953 |
| br2_high_skew_load | 205 | $64.09 | $1,685.93 | 80.98% | 32.20% | 0.2311 | 0.5009 | 0.1571 |
| br2_late_confirm | 336 | $656.99 | $7,408.22 | 76.49% | 42.26% | 0.2198 | 0.5293 | 0.1743 |
| br2_late_favourite_load | 377 | $416.80 | $8,410.17 | 79.31% | 31.83% | 0.2324 | 0.5396 | 0.1713 |

## By Period And Lane

| Bucket | Fills | PnL | Cost | Win Rate | Cross-Mid Rate | Avg Adverse | Log Loss | Brier |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| first_third:br2_convex_tail | 6 | $1.42 | $6.47 | 16.67% | 0.00% | 0.0400 | 0.5292 | 0.1516 |
| first_third:br2_high_skew_load | 43 | $12.76 | $264.60 | 81.40% | 27.91% | 0.2000 | 0.4703 | 0.1489 |
| first_third:br2_late_confirm | 63 | $51.30 | $984.45 | 73.02% | 44.44% | 0.2170 | 0.5729 | 0.1936 |
| first_third:br2_late_favourite_load | 76 | $66.88 | $1,104.14 | 77.63% | 30.26% | 0.2226 | 0.5624 | 0.1815 |
| last_third:br2_convex_tail | 19 | $-19.94 | $35.69 | 5.26% | 0.00% | 0.0426 | 0.2138 | 0.0509 |
| last_third:br2_high_skew_load | 84 | $4.22 | $896.03 | 79.76% | 32.14% | 0.2382 | 0.5167 | 0.1644 |
| last_third:br2_late_confirm | 146 | $291.28 | $4,123.12 | 76.03% | 44.52% | 0.2238 | 0.5148 | 0.1701 |
| last_third:br2_late_favourite_load | 156 | $375.14 | $4,666.88 | 82.69% | 31.41% | 0.2141 | 0.4662 | 0.1446 |
| middle_third:br2_convex_tail | 14 | $28.45 | $14.07 | 14.29% | 0.00% | 0.0468 | 0.4675 | 0.1313 |
| middle_third:br2_high_skew_load | 78 | $47.11 | $525.29 | 82.05% | 34.62% | 0.2406 | 0.5009 | 0.1537 |
| middle_third:br2_late_confirm | 127 | $314.41 | $2,300.65 | 78.74% | 38.58% | 0.2167 | 0.5244 | 0.1697 |
| middle_third:br2_late_favourite_load | 145 | $-25.22 | $2,639.14 | 76.55% | 33.10% | 0.2572 | 0.6067 | 0.1946 |

## By Post-Fill Path

| Bucket | Fills | PnL | Cost | Win Rate | Cross-Mid Rate | Avg Adverse | Log Loss | Brier |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_convex_tail:held_side | 39 | $9.93 | $56.23 | 10.26% | 0.00% | 0.0437 | 0.3534 | 0.0953 |
| br2_high_skew_load:crossed_mid_after_fill | 66 | $-265.80 | $548.32 | 40.91% | 100.00% | 0.5833 | 1.3046 | 0.4591 |
| br2_high_skew_load:held_side | 101 | $232.97 | $812.06 | 100.00% | 0.00% | 0.0269 | 0.1189 | 0.0136 |
| br2_high_skew_load:moderate_adverse_excursion | 38 | $96.92 | $325.55 | 100.00% | 0.00% | 0.1620 | 0.1206 | 0.0138 |
| br2_late_confirm:crossed_mid_after_fill | 142 | $-790.85 | $3,175.81 | 48.59% | 100.00% | 0.4544 | 0.8750 | 0.3216 |
| br2_late_confirm:held_side | 158 | $1,344.27 | $3,499.27 | 98.73% | 0.00% | 0.0225 | 0.2537 | 0.0567 |
| br2_late_confirm:moderate_adverse_excursion | 36 | $103.57 | $733.14 | 88.89% | 0.00% | 0.1606 | 0.3757 | 0.1100 |
| br2_late_favourite_load:crossed_mid_after_fill | 120 | $-1,216.94 | $2,530.75 | 37.50% | 100.00% | 0.5913 | 1.3817 | 0.4888 |
| br2_late_favourite_load:held_side | 183 | $1,194.07 | $4,160.44 | 100.00% | 0.00% | 0.0264 | 0.1228 | 0.0141 |
| br2_late_favourite_load:moderate_adverse_excursion | 74 | $439.66 | $1,718.98 | 95.95% | 0.00% | 0.1599 | 0.2049 | 0.0451 |

## By Final Range

| Bucket | Fills | PnL | Cost | Win Rate | Cross-Mid Rate | Avg Adverse | Log Loss | Brier |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_convex_tail:range_050_078 | 29 | $-44.28 | $44.28 | 0.00% | 0.00% | 0.0424 | 0.0702 | 0.0048 |
| br2_convex_tail:range_078_093_midwide | 6 | $-8.10 | $8.10 | 0.00% | 0.00% | 0.0542 | 0.0823 | 0.0069 |
| br2_convex_tail:range_093_097 | 1 | $5.71 | $0.43 | 100.00% | 0.00% | 0.0300 | 2.8134 | 0.8836 |
| br2_convex_tail:range_ge_097_extreme | 3 | $56.59 | $3.43 | 100.00% | 0.00% | 0.0400 | 2.8134 | 0.8836 |
| br2_high_skew_load:range_050_078 | 135 | $208.13 | $1,107.17 | 92.59% | 18.52% | 0.1400 | 0.2514 | 0.0654 |
| br2_high_skew_load:range_078_093_midwide | 64 | $-132.81 | $527.99 | 59.38% | 57.81% | 0.4052 | 0.9706 | 0.3290 |
| br2_high_skew_load:range_093_097 | 5 | $-15.51 | $39.50 | 40.00% | 80.00% | 0.5050 | 1.2880 | 0.4571 |
| br2_high_skew_load:range_lt_050 | 1 | $4.29 | $11.27 | 100.00% | 0.00% | 0.0100 | 0.1908 | 0.0302 |
| br2_late_confirm:range_050_078 | 240 | $746.65 | $5,371.78 | 82.92% | 32.92% | 0.1712 | 0.4076 | 0.1268 |
| br2_late_confirm:range_078_093_midwide | 78 | $-82.00 | $1,664.47 | 58.97% | 64.10% | 0.3083 | 0.8770 | 0.3083 |
| br2_late_confirm:range_093_097 | 4 | $10.26 | $81.82 | 75.00% | 100.00% | 0.6575 | 0.6058 | 0.2053 |
| br2_late_confirm:range_ge_097_extreme | 10 | $-33.11 | $219.09 | 50.00% | 90.00% | 0.5665 | 0.8668 | 0.3196 |
| br2_late_confirm:range_lt_050 | 4 | $15.19 | $71.06 | 100.00% | 0.00% | 0.1113 | 0.1339 | 0.0183 |
| br2_late_favourite_load:range_050_078 | 258 | $1,336.42 | $6,187.55 | 91.09% | 19.77% | 0.1511 | 0.2905 | 0.0791 |
| br2_late_favourite_load:range_078_093_midwide | 103 | $-838.30 | $1,959.04 | 52.43% | 60.19% | 0.4195 | 1.1125 | 0.3830 |
| br2_late_favourite_load:range_093_097 | 7 | $-98.57 | $114.63 | 42.86% | 57.14% | 0.4393 | 1.3107 | 0.4514 |
| br2_late_favourite_load:range_ge_097_extreme | 5 | $-4.42 | $81.37 | 60.00% | 60.00% | 0.4300 | 0.8421 | 0.2981 |
| br2_late_favourite_load:range_lt_050 | 4 | $21.67 | $67.58 | 100.00% | 0.00% | 0.0525 | 0.1292 | 0.0155 |

## By Live Regime Label

| Bucket | Fills | PnL | Cost | Win Rate | Cross-Mid Rate | Avg Adverse | Log Loss | Brier |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_convex_tail:expanded_chop | 3 | $-2.39 | $2.39 | 0.00% | 0.00% | 0.0450 | 0.1027 | 0.0101 |
| br2_convex_tail:expanded_continuation | 6 | $14.56 | $7.32 | 33.33% | 0.00% | 0.0417 | 0.9791 | 0.2969 |
| br2_convex_tail:expanded_not_decisive | 22 | $10.69 | $25.69 | 4.55% | 0.00% | 0.0434 | 0.1934 | 0.0445 |
| br2_convex_tail:neutral | 4 | $2.46 | $5.43 | 25.00% | 0.00% | 0.0400 | 0.7657 | 0.2260 |
| br2_convex_tail:reversal_pressure | 4 | $-15.40 | $15.40 | 0.00% | 0.00% | 0.0513 | 0.0708 | 0.0048 |
| br2_high_skew_load:expanded_chop | 15 | $-22.39 | $99.88 | 60.00% | 60.00% | 0.3773 | 0.8746 | 0.3030 |
| br2_high_skew_load:expanded_continuation | 17 | $8.49 | $151.90 | 82.35% | 23.53% | 0.1753 | 0.4690 | 0.1451 |
| br2_high_skew_load:expanded_not_decisive | 111 | $52.17 | $902.20 | 82.88% | 29.73% | 0.2091 | 0.4537 | 0.1403 |
| br2_high_skew_load:neutral | 40 | $37.32 | $347.31 | 85.00% | 25.00% | 0.2208 | 0.4425 | 0.1321 |
| br2_high_skew_load:reversal_pressure | 22 | $-11.50 | $184.64 | 77.27% | 45.45% | 0.3043 | 0.6156 | 0.1968 |
| br2_late_confirm:expanded_not_decisive | 212 | $523.81 | $4,651.11 | 79.72% | 41.04% | 0.2081 | 0.4960 | 0.1587 |
| br2_late_confirm:neutral | 74 | $155.76 | $1,644.44 | 75.68% | 39.19% | 0.2159 | 0.5144 | 0.1704 |
| br2_late_confirm:reversal_pressure | 50 | $-22.57 | $1,112.67 | 64.00% | 52.00% | 0.2754 | 0.6927 | 0.2467 |
| br2_late_favourite_load:expanded_chop | 6 | $7.76 | $22.26 | 100.00% | 0.00% | 0.0800 | 0.1013 | 0.0095 |
| br2_late_favourite_load:expanded_not_decisive | 252 | $357.72 | $5,549.47 | 79.76% | 31.75% | 0.2262 | 0.5291 | 0.1675 |
| br2_late_favourite_load:neutral | 76 | $95.79 | $1,803.05 | 78.95% | 30.26% | 0.2418 | 0.5575 | 0.1766 |
| br2_late_favourite_load:reversal_pressure | 43 | $-44.47 | $1,035.40 | 74.42% | 39.53% | 0.2737 | 0.6310 | 0.2067 |

## By Observed Range At Entry

| Bucket | Fills | PnL | Cost | Win Rate | Cross-Mid Rate | Avg Adverse | Log Loss | Brier |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_convex_tail:obs_040_050 | 6 | $40.76 | $3.51 | 33.33% | 0.00% | 0.0450 | 0.9898 | 0.2985 |
| br2_convex_tail:obs_050_065 | 20 | $-27.60 | $27.60 | 0.00% | 0.00% | 0.0422 | 0.0689 | 0.0046 |
| br2_convex_tail:obs_ge_065 | 13 | $-3.23 | $25.11 | 15.38% | 0.00% | 0.0454 | 0.4973 | 0.1409 |
| br2_high_skew_load:obs_030_040 | 31 | $0.09 | $270.47 | 80.65% | 25.81% | 0.2137 | 0.5289 | 0.1662 |
| br2_high_skew_load:obs_040_050 | 61 | $17.00 | $501.92 | 83.61% | 22.95% | 0.1830 | 0.4508 | 0.1379 |
| br2_high_skew_load:obs_050_065 | 80 | $57.65 | $654.58 | 82.50% | 37.50% | 0.2586 | 0.4684 | 0.1447 |
| br2_high_skew_load:obs_ge_065 | 33 | $-10.65 | $258.96 | 72.73% | 42.42% | 0.2697 | 0.6462 | 0.2138 |
| br2_late_confirm:obs_030_040 | 78 | $277.16 | $1,667.81 | 76.92% | 44.87% | 0.2279 | 0.5553 | 0.1842 |
| br2_late_confirm:obs_040_050 | 227 | $385.72 | $4,986.76 | 77.53% | 41.41% | 0.2146 | 0.5127 | 0.1669 |
| br2_late_confirm:obs_050_065 | 14 | $110.79 | $332.67 | 92.86% | 21.43% | 0.1157 | 0.3526 | 0.0999 |
| br2_late_confirm:obs_lt_030 | 17 | $-116.67 | $420.98 | 47.06% | 58.82% | 0.3382 | 0.7780 | 0.2896 |
| br2_late_favourite_load:obs_030_040 | 74 | $80.61 | $1,816.90 | 79.73% | 31.08% | 0.2280 | 0.5354 | 0.1693 |
| br2_late_favourite_load:obs_040_050 | 138 | $140.23 | $3,406.06 | 81.16% | 29.71% | 0.2192 | 0.5136 | 0.1597 |
| br2_late_favourite_load:obs_050_065 | 156 | $185.75 | $3,121.76 | 76.92% | 35.26% | 0.2520 | 0.5785 | 0.1872 |
| br2_late_favourite_load:obs_ge_065 | 6 | $7.76 | $22.26 | 100.00% | 0.00% | 0.0800 | 0.1013 | 0.0095 |
| br2_late_favourite_load:obs_lt_030 | 3 | $2.45 | $43.18 | 66.67% | 33.33% | 0.2367 | 0.6951 | 0.2441 |

## Worst Cross-Mid Fills

| Rank | Date | Slug | Lane | PnL | Cost | Price | Observed Range | Final Range | Model P | Edge |
|---:|---|---|---|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2026-03-04 | btc-updown-5m-1772585700 | br2_late_favourite_load | $-47.80 | $47.80 | 0.8012 | 0.510 | 0.955 | 0.903 | 0.102 |
| 2 | 2026-03-04 | btc-updown-5m-1772645700 | br2_late_favourite_load | $-46.93 | $46.93 | 0.8012 | 0.445 | 0.920 | 0.901 | 0.100 |
| 3 | 2026-03-06 | btc-updown-5m-1772758800 | br2_late_favourite_load | $-44.97 | $44.97 | 0.7511 | 0.440 | 0.745 | 0.890 | 0.139 |
| 4 | 2026-03-06 | btc-updown-5m-1772775000 | br2_late_favourite_load | $-41.11 | $41.11 | 0.7812 | 0.470 | 0.800 | 0.900 | 0.119 |
| 5 | 2026-03-04 | btc-updown-5m-1772642400 | br2_late_favourite_load | $-40.86 | $40.86 | 0.7912 | 0.340 | 0.785 | 0.898 | 0.107 |
| 6 | 2026-03-06 | btc-updown-5m-1772775300 | br2_late_favourite_load | $-39.17 | $39.17 | 0.7876 | 0.450 | 0.915 | 0.900 | 0.112 |
| 7 | 2026-03-06 | btc-updown-5m-1772768100 | br2_late_confirm | $-36.90 | $36.90 | 0.7046 | 0.420 | 0.845 | 0.750 | 0.046 |
| 8 | 2026-03-06 | btc-updown-5m-1772775000 | br2_late_favourite_load | $-36.85 | $36.85 | 0.7901 | 0.470 | 0.800 | 0.897 | 0.107 |
| 9 | 2026-03-05 | btc-updown-5m-1772691300 | br2_late_favourite_load | $-36.84 | $36.84 | 0.7703 | 0.350 | 0.805 | 0.868 | 0.097 |
| 10 | 2026-03-04 | btc-updown-5m-1772642400 | br2_late_favourite_load | $-36.61 | $36.61 | 0.7559 | 0.340 | 0.785 | 0.892 | 0.136 |
| 11 | 2026-03-04 | btc-updown-5m-1772642400 | br2_late_favourite_load | $-36.58 | $36.58 | 0.7912 | 0.340 | 0.785 | 0.884 | 0.093 |
| 12 | 2026-03-03 | btc-updown-5m-1772509800 | br2_late_favourite_load | $-36.21 | $36.21 | 0.8112 | 0.535 | 0.855 | 0.902 | 0.091 |
| 13 | 2026-03-04 | btc-updown-5m-1772642400 | br2_late_favourite_load | $-36.06 | $36.06 | 0.7712 | 0.310 | 0.785 | 0.884 | 0.113 |
| 14 | 2026-03-06 | btc-updown-5m-1772775000 | br2_late_favourite_load | $-34.03 | $34.03 | 0.7912 | 0.470 | 0.800 | 0.898 | 0.107 |
| 15 | 2026-03-06 | btc-updown-5m-1772765100 | br2_late_confirm | $-33.84 | $33.84 | 0.5849 | 0.310 | 0.615 | 0.685 | 0.100 |
| 16 | 2026-03-06 | btc-updown-5m-1772769900 | br2_late_confirm | $-33.58 | $33.58 | 0.6163 | 0.480 | 0.715 | 0.769 | 0.152 |
| 17 | 2026-03-04 | btc-updown-5m-1772640000 | br2_late_favourite_load | $-33.19 | $33.19 | 0.7611 | 0.490 | 0.785 | 0.876 | 0.115 |
| 18 | 2026-03-06 | btc-updown-5m-1772770500 | br2_late_confirm | $-32.79 | $32.79 | 0.5909 | 0.490 | 0.730 | 0.665 | 0.074 |
| 19 | 2026-03-04 | btc-updown-5m-1772640000 | br2_late_favourite_load | $-32.65 | $32.65 | 0.7111 | 0.530 | 0.785 | 0.874 | 0.163 |
| 20 | 2026-03-06 | btc-updown-5m-1772754900 | br2_late_confirm | $-32.16 | $32.16 | 0.5770 | 0.280 | 0.640 | 0.642 | 0.065 |
| 21 | 2026-03-06 | btc-updown-5m-1772802000 | br2_late_confirm | $-32.05 | $32.05 | 0.6510 | 0.430 | 0.740 | 0.688 | 0.037 |
| 22 | 2026-03-06 | btc-updown-5m-1772802000 | br2_late_confirm | $-32.05 | $32.05 | 0.6410 | 0.480 | 0.740 | 0.722 | 0.081 |
| 23 | 2026-03-06 | btc-updown-5m-1772773200 | br2_late_confirm | $-31.89 | $31.89 | 0.6721 | 0.440 | 0.805 | 0.769 | 0.097 |
| 24 | 2026-03-04 | btc-updown-5m-1772653200 | br2_late_favourite_load | $-31.73 | $31.73 | 0.7111 | 0.475 | 0.745 | 0.883 | 0.172 |

