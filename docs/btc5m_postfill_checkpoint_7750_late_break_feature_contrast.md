# BTC5m Late-Break Feature Contrast

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Fills: `1683` late-confirm/favourite fills
Calendar: `2026-02-27T15:45:00+00:00` to `2026-03-26T12:30:00+00:00`
PnL: `$4,889.81`
Toxic fills: `346` (`20.56%`)

This diagnostic contrasts failed late breaks against profitable late breaks using fill-time features only. Post-fill labels are used only to define the offline target.

## By Lane

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| br2_late_confirm | 790 | $2,453.72 | $32,329.94 | 75.19% | 23.54% | 42.03% |
| br2_late_favourite_load | 893 | $2,436.09 | $34,333.89 | 81.63% | 17.92% | 35.61% |

## By Post-Fill Path

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| crossed_mid_after_fill | 650 | $-8,468.63 | $25,906.53 | 47.08% | 52.92% | 100.00% |
| held_side | 805 | $11,310.09 | $32,391.59 | 99.25% | 0.00% | 0.00% |
| moderate_adverse_no_cross | 228 | $2,048.36 | $8,365.72 | 95.61% | 0.88% | 0.00% |

## Feature Contrast: Toxic vs Profitable Non-Toxic Late Breaks

| Feature | Toxic Mean | Profitable Non-Toxic Mean | Difference | Std Diff | Toxic N | Profitable N |
|---|---:|---:|---:|---:|---:|---:|
| price | 0.7028 | 0.7386 | -0.0358 | -0.410 | 346 | 1323 |
| side_model_p | 0.8081 | 0.8375 | -0.0294 | -0.363 | 346 | 1323 |
| risk_score | 0.4206 | 0.4055 | 0.0152 | 0.190 | 346 | 1323 |
| side_edge_vs_fill | 0.1053 | 0.0989 | 0.0064 | 0.133 | 346 | 1323 |
| regime_realized_vol_180s_bps | 1.9983 | 2.0954 | -0.0971 | -0.106 | 346 | 1323 |
| confidence_score | 0.8708 | 0.8766 | -0.0058 | -0.093 | 346 | 1323 |
| regime_whipsaw_score | 0.2881 | 0.2943 | -0.0062 | -0.069 | 346 | 1323 |
| prior_market_range_7d | 0.6765 | 0.6770 | -0.0005 | -0.066 | 346 | 1323 |
| market_yes_range_so_far | 0.4467 | 0.4516 | -0.0048 | -0.063 | 346 | 1323 |
| seconds_to_close | 65.1753 | 66.8498 | -1.6744 | -0.056 | 346 | 1323 |
| prior_market_range_1d | 0.6809 | 0.6798 | 0.0011 | 0.049 | 346 | 1323 |
| prior_market_range_3d | 0.6776 | 0.6780 | -0.0004 | -0.039 | 346 | 1323 |
| regime_reversal_pressure | 0.3274 | 0.3246 | 0.0028 | 0.025 | 346 | 1323 |
| regime_path_efficiency | 0.1409 | 0.1435 | -0.0026 | -0.024 | 346 | 1323 |
| regime_sign_flip_rate | 0.4256 | 0.4236 | 0.0020 | 0.022 | 346 | 1323 |

## Quartiles: price

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| price:q1 | 393 | $1,647.34 | $15,491.09 | 66.67% | 32.32% | 59.03% |
| price:q2 | 403 | $1,124.37 | $12,005.67 | 79.40% | 19.60% | 37.22% |
| price:q3 | 465 | $1,286.69 | $18,306.09 | 81.29% | 18.28% | 36.99% |
| price:q4 | 422 | $831.41 | $20,860.97 | 86.02% | 13.03% | 22.75% |

## Quartiles: side_model_p

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_model_p:q1 | 420 | $2,031.21 | $17,274.62 | 69.52% | 29.29% | 54.05% |
| side_model_p:q2 | 421 | $726.53 | $12,989.59 | 76.96% | 21.85% | 40.38% |
| side_model_p:q3 | 420 | $1,587.51 | $16,244.53 | 83.10% | 16.43% | 35.24% |
| side_model_p:q4 | 422 | $544.55 | $20,155.09 | 84.83% | 14.69% | 24.88% |

## Quartiles: risk_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| risk_score:q1 | 420 | $2,475.32 | $20,458.13 | 82.62% | 16.90% | 30.95% |
| risk_score:q2 | 421 | $1,562.69 | $16,441.85 | 81.24% | 18.05% | 39.90% |
| risk_score:q3 | 420 | $277.72 | $14,144.56 | 76.90% | 22.62% | 39.05% |
| risk_score:q4 | 422 | $574.08 | $15,619.28 | 73.70% | 24.64% | 44.55% |

## Quartiles: side_edge_vs_fill

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_edge_vs_fill:q1 | 420 | $259.48 | $17,380.73 | 79.05% | 20.00% | 33.10% |
| side_edge_vs_fill:q2 | 421 | $1,664.29 | $18,116.93 | 80.29% | 18.53% | 37.77% |
| side_edge_vs_fill:q3 | 420 | $1,922.76 | $17,901.56 | 81.67% | 17.62% | 36.67% |
| side_edge_vs_fill:q4 | 422 | $1,043.28 | $13,264.62 | 73.46% | 26.07% | 46.92% |

## Quartiles: regime_realized_vol_180s_bps

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| regime_realized_vol_180s_bps:q1 | 420 | $1,434.16 | $18,019.76 | 78.10% | 20.71% | 39.76% |
| regime_realized_vol_180s_bps:q2 | 421 | $511.77 | $17,230.42 | 74.82% | 24.47% | 42.04% |
| regime_realized_vol_180s_bps:q3 | 420 | $570.58 | $16,601.35 | 78.10% | 20.71% | 40.48% |
| regime_realized_vol_180s_bps:q4 | 422 | $2,373.30 | $14,812.30 | 83.41% | 16.35% | 32.23% |

## Quartiles: confidence_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confidence_score:q1 | 420 | $1,254.24 | $16,190.99 | 76.67% | 21.90% | 40.00% |
| confidence_score:q2 | 421 | $942.01 | $16,393.95 | 79.10% | 20.43% | 36.58% |
| confidence_score:q3 | 420 | $1,137.94 | $16,442.01 | 80.48% | 19.05% | 37.14% |
| confidence_score:q4 | 422 | $1,555.62 | $17,636.88 | 78.20% | 20.85% | 40.76% |

## Single-Feature Removal Scan

Positive removed PnL means a gate would remove profitable fills. Negative removed PnL is the interesting direction.

| Feature | Direction | Threshold | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| regime_sign_flip_rate | ge | 0.6000 | 65 | $2,520.20 | $40.23 | $-40.23 | 27.69% | 38.46% |
| regime_reversal_pressure | ge | 0.5600 | 119 | $4,105.17 | $72.11 | $-72.11 | 24.37% | 47.90% |
| regime_reversal_pressure | ge | 0.6200 | 67 | $2,242.97 | $142.43 | $-142.43 | 17.91% | 46.27% |
| risk_score | ge | 0.4738 | 339 | $12,618.71 | $171.28 | $-171.28 | 26.55% | 45.13% |
| regime_reversal_pressure | ge | 0.5000 | 150 | $5,230.30 | $203.17 | $-203.17 | 22.00% | 43.33% |
| regime_path_efficiency | le | 0.0519 | 337 | $12,610.32 | $326.25 | $-326.25 | 22.26% | 40.36% |
| side_edge_vs_fill | le | 0.0826 | 507 | $20,988.98 | $349.95 | $-349.95 | 21.30% | 35.70% |
| side_model_p | ge | 0.8973 | 350 | $16,585.18 | $350.84 | $-350.84 | 14.86% | 24.57% |
| regime_path_efficiency | le | 0.0736 | 504 | $19,234.34 | $484.98 | $-484.98 | 22.62% | 41.07% |
| regime_reversal_pressure | ge | 0.4400 | 190 | $6,864.70 | $485.35 | $-485.35 | 20.53% | 41.05% |
| market_yes_range_so_far | le | 0.3250 | 92 | $3,236.42 | $617.01 | $-617.01 | 29.35% | 42.39% |
| seconds_to_close | le | 40.4060 | 334 | $11,856.92 | $688.16 | $-688.16 | 21.56% | 36.53% |
| regime_sign_flip_rate | le | 0.2571 | 90 | $3,406.89 | $723.46 | $-723.46 | 12.22% | 41.11% |
| prior_market_range_1d | ge | 0.6875 | 377 | $11,620.13 | $745.03 | $-745.03 | 19.10% | 31.83% |
| prior_market_range_7d | ge | 0.6823 | 369 | $10,855.00 | $753.20 | $-753.20 | 18.16% | 32.52% |
| side_model_p | ge | 0.8909 | 518 | $24,331.19 | $764.72 | $-764.72 | 15.44% | 26.06% |
| prior_market_range_1d | ge | 0.6842 | 549 | $16,326.98 | $769.24 | $-769.24 | 21.31% | 36.07% |
| market_yes_range_so_far | ge | 0.5400 | 208 | $6,597.13 | $783.40 | $-783.40 | 16.83% | 37.02% |
| prior_market_range_3d | ge | 0.6859 | 370 | $11,685.41 | $815.27 | $-815.27 | 17.30% | 30.54% |
| confidence_score | le | 0.8439 | 336 | $13,042.55 | $821.82 | $-821.82 | 23.51% | 42.56% |
| price | ge | 0.7961 | 343 | $16,888.16 | $831.91 | $-831.91 | 11.95% | 19.53% |
| risk_score | ge | 0.3994 | 842 | $29,763.84 | $851.80 | $-851.80 | 23.63% | 41.81% |
| side_edge_vs_fill | le | 0.0940 | 674 | $28,042.75 | $871.68 | $-871.68 | 21.07% | 36.65% |
| regime_whipsaw_score | le | 0.2242 | 337 | $13,762.94 | $881.26 | $-881.26 | 20.77% | 40.65% |
| risk_score | ge | 0.4448 | 506 | $18,212.02 | $891.85 | $-891.85 | 23.12% | 43.28% |
| side_edge_vs_fill | ge | 0.1295 | 337 | $9,871.89 | $911.79 | $-911.79 | 26.71% | 47.48% |
| side_edge_vs_fill | le | 0.0569 | 340 | $13,834.72 | $912.90 | $-912.90 | 16.18% | 29.12% |
| market_yes_range_so_far | ge | 0.4950 | 435 | $15,813.46 | $913.47 | $-913.47 | 19.54% | 38.62% |
| regime_path_efficiency | le | 0.0958 | 672 | $25,471.41 | $919.35 | $-919.35 | 21.43% | 38.54% |
| regime_reversal_pressure | le | 0.2400 | 373 | $14,789.81 | $939.29 | $-939.29 | 20.38% | 38.87% |

## Two-Feature Candidate Scan

| Candidate | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confirm_low_edge_reversal | 294 | $11,720.43 | $352.34 | $-352.34 | 20.41% | 36.39% |
| price_high_edge_low | 351 | $16,189.61 | $849.27 | $-849.27 | 12.54% | 22.51% |
| fav_high_price_chop | 302 | $15,452.27 | $1,068.26 | $-1,068.26 | 13.58% | 27.48% |
| price_high_reversal | 311 | $14,298.17 | $1,142.39 | $-1,142.39 | 12.54% | 24.76% |
| obs_mid_high_signflip | 924 | $36,250.67 | $2,571.54 | $-2,571.54 | 19.91% | 38.74% |
| high_reversal_low_eff | 900 | $35,203.22 | $3,269.42 | $-3,269.42 | 19.33% | 38.33% |
| high_signflip_low_eff | 1123 | $44,459.94 | $3,588.59 | $-3,588.59 | 20.21% | 38.82% |
