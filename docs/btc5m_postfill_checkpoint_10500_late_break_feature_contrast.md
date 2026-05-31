# BTC5m Late-Break Feature Contrast

Source: `/tmp/btc5m_postfill_markets_062901.jsonl`
Fills: `1864` late-confirm/favourite fills
Calendar: `2026-02-27T15:45:00+00:00` to `2026-04-04T15:20:00+00:00`
PnL: `$5,696.75`
Toxic fills: `384` (`20.60%`)

This diagnostic contrasts failed late breaks against profitable late breaks using fill-time features only. Post-fill labels are used only to define the offline target.

## By Lane

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| br2_late_confirm | 875 | $2,350.34 | $39,812.82 | 74.74% | 24.00% | 42.40% |
| br2_late_favourite_load | 989 | $3,346.40 | $42,047.54 | 82.00% | 17.59% | 34.68% |

## By Post-Fill Path

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| crossed_mid_after_fill | 714 | $-10,543.62 | $31,421.08 | 46.50% | 53.50% | 100.00% |
| held_side | 895 | $13,521.65 | $39,480.27 | 99.22% | 0.00% | 0.00% |
| moderate_adverse_no_cross | 255 | $2,718.71 | $10,959.01 | 96.08% | 0.78% | 0.00% |

## Feature Contrast: Toxic vs Profitable Non-Toxic Late Breaks

| Feature | Toxic Mean | Profitable Non-Toxic Mean | Difference | Std Diff | Toxic N | Profitable N |
|---|---:|---:|---:|---:|---:|---:|
| price | 0.7035 | 0.7396 | -0.0361 | -0.417 | 384 | 1465 |
| side_model_p | 0.8078 | 0.8382 | -0.0305 | -0.379 | 384 | 1465 |
| risk_score | 0.4204 | 0.4049 | 0.0156 | 0.194 | 384 | 1465 |
| side_edge_vs_fill | 0.1042 | 0.0986 | 0.0056 | 0.117 | 384 | 1465 |
| regime_realized_vol_180s_bps | 1.9630 | 2.0660 | -0.1030 | -0.115 | 384 | 1465 |
| confidence_score | 0.8710 | 0.8773 | -0.0063 | -0.102 | 384 | 1465 |
| market_yes_range_so_far | 0.4473 | 0.4535 | -0.0062 | -0.081 | 384 | 1465 |
| seconds_to_close | 65.0054 | 67.1341 | -2.1288 | -0.071 | 384 | 1465 |
| regime_whipsaw_score | 0.2856 | 0.2919 | -0.0063 | -0.071 | 384 | 1465 |
| regime_sign_flip_rate | 0.4268 | 0.4228 | 0.0040 | 0.043 | 384 | 1465 |
| regime_reversal_pressure | 0.3277 | 0.3236 | 0.0041 | 0.037 | 384 | 1465 |
| prior_market_range_1d | 0.6945 | 0.6932 | 0.0013 | 0.028 | 384 | 1465 |
| regime_path_efficiency | 0.1401 | 0.1416 | -0.0015 | -0.014 | 384 | 1465 |
| prior_market_range_7d | 0.6872 | 0.6876 | -0.0005 | -0.014 | 384 | 1465 |
| prior_market_range_3d | 0.6908 | 0.6910 | -0.0002 | -0.005 | 384 | 1465 |

## Quartiles: price

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| price:q1 | 465 | $1,872.75 | $20,042.46 | 67.53% | 31.18% | 57.85% |
| price:q2 | 409 | $1,181.98 | $13,270.80 | 78.73% | 20.54% | 37.65% |
| price:q3 | 522 | $1,395.81 | $23,120.95 | 81.23% | 18.39% | 36.40% |
| price:q4 | 468 | $1,246.20 | $25,426.15 | 86.54% | 12.61% | 21.58% |

## Quartiles: side_model_p

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_model_p:q1 | 465 | $2,035.69 | $20,922.19 | 69.03% | 29.68% | 54.62% |
| side_model_p:q2 | 466 | $939.50 | $16,163.12 | 77.04% | 21.89% | 40.13% |
| side_model_p:q3 | 466 | $1,972.11 | $20,261.31 | 83.26% | 16.31% | 34.55% |
| side_model_p:q4 | 467 | $749.44 | $24,513.73 | 85.01% | 14.56% | 23.98% |

## Quartiles: risk_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| risk_score:q1 | 465 | $2,905.17 | $25,127.90 | 83.01% | 16.56% | 30.97% |
| risk_score:q2 | 466 | $1,777.18 | $19,872.49 | 80.90% | 18.45% | 39.27% |
| risk_score:q3 | 466 | $510.97 | $17,442.47 | 77.04% | 22.53% | 38.63% |
| risk_score:q4 | 467 | $503.43 | $19,417.50 | 73.45% | 24.84% | 44.33% |

## Quartiles: side_edge_vs_fill

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_edge_vs_fill:q1 | 465 | $108.96 | $21,152.00 | 78.71% | 20.22% | 32.90% |
| side_edge_vs_fill:q2 | 466 | $2,011.46 | $22,625.89 | 80.26% | 18.67% | 37.55% |
| side_edge_vs_fill:q3 | 466 | $2,061.18 | $21,707.39 | 81.12% | 18.24% | 36.70% |
| side_edge_vs_fill:q4 | 467 | $1,515.15 | $16,375.08 | 74.30% | 25.27% | 46.04% |

## Quartiles: regime_realized_vol_180s_bps

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| regime_realized_vol_180s_bps:q1 | 465 | $1,385.26 | $22,566.73 | 77.20% | 21.72% | 38.71% |
| regime_realized_vol_180s_bps:q2 | 466 | $267.08 | $21,962.50 | 74.68% | 24.46% | 43.35% |
| regime_realized_vol_180s_bps:q3 | 466 | $1,408.02 | $19,716.58 | 79.18% | 19.74% | 38.41% |
| regime_realized_vol_180s_bps:q4 | 467 | $2,636.37 | $17,614.55 | 83.30% | 16.49% | 32.76% |

## Quartiles: confidence_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confidence_score:q1 | 465 | $1,129.22 | $19,689.44 | 76.34% | 22.15% | 40.00% |
| confidence_score:q2 | 466 | $1,381.41 | $19,502.98 | 79.61% | 19.96% | 35.62% |
| confidence_score:q3 | 466 | $1,126.82 | $20,632.44 | 79.61% | 19.96% | 37.55% |
| confidence_score:q4 | 467 | $2,059.28 | $22,035.50 | 78.80% | 20.34% | 40.04% |

## Single-Feature Removal Scan

Positive removed PnL means a gate would remove profitable fills. Negative removed PnL is the interesting direction.

| Feature | Direction | Threshold | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| regime_reversal_pressure | ge | 0.5600 | 127 | $4,855.97 | $17.59 | $-17.59 | 24.41% | 47.24% |
| regime_sign_flip_rate | ge | 0.6000 | 70 | $2,995.93 | $48.39 | $-48.39 | 27.14% | 38.57% |
| side_edge_vs_fill | le | 0.0830 | 561 | $25,575.21 | $100.76 | $-100.76 | 21.57% | 35.47% |
| regime_reversal_pressure | ge | 0.6200 | 72 | $2,755.34 | $160.49 | $-160.49 | 18.06% | 45.83% |
| regime_reversal_pressure | ge | 0.5000 | 163 | $6,363.90 | $243.95 | $-243.95 | 22.09% | 42.33% |
| risk_score | ge | 0.4737 | 376 | $15,591.43 | $273.24 | $-273.24 | 26.33% | 44.68% |
| side_edge_vs_fill | le | 0.0567 | 376 | $16,854.12 | $446.53 | $-446.53 | 17.29% | 29.26% |
| confidence_score | le | 0.8445 | 372 | $15,906.11 | $599.45 | $-599.45 | 23.66% | 42.20% |
| regime_reversal_pressure | ge | 0.4400 | 206 | $8,276.88 | $602.99 | $-602.99 | 20.39% | 40.29% |
| market_yes_range_so_far | le | 0.3250 | 94 | $3,305.37 | $651.70 | $-651.70 | 28.72% | 41.49% |
| market_yes_range_so_far | ge | 0.5450 | 213 | $7,346.81 | $770.81 | $-770.81 | 17.84% | 35.68% |
| price | ge | 0.7975 | 383 | $20,953.07 | $782.38 | $-782.38 | 12.53% | 19.32% |
| side_edge_vs_fill | le | 0.0939 | 746 | $34,503.01 | $808.22 | $-808.22 | 21.18% | 36.60% |
| side_model_p | ge | 0.8975 | 387 | $20,285.18 | $874.56 | $-874.56 | 13.95% | 23.00% |
| side_model_p | ge | 0.8910 | 573 | $29,891.07 | $876.07 | $-876.07 | 15.18% | 25.13% |
| regime_sign_flip_rate | le | 0.2571 | 103 | $4,284.57 | $908.77 | $-908.77 | 11.65% | 39.81% |
| regime_whipsaw_score | le | 0.2386 | 561 | $25,980.02 | $973.64 | $-973.64 | 22.28% | 39.75% |
| risk_score | ge | 0.4447 | 561 | $22,731.01 | $994.37 | $-994.37 | 23.35% | 42.96% |
| regime_realized_vol_180s_bps | le | 1.6128 | 748 | $35,624.51 | $1,003.70 | $-1,003.70 | 23.26% | 40.91% |
| risk_score | ge | 0.3987 | 933 | $36,859.97 | $1,014.40 | $-1,014.40 | 23.69% | 41.48% |
| prior_market_range_1d | ge | 0.6951 | 350 | $21,626.86 | $1,106.62 | $-1,106.62 | 20.29% | 34.29% |
| prior_market_range_7d | ge | 0.6837 | 366 | $21,922.91 | $1,121.76 | $-1,121.76 | 19.95% | 32.79% |
| regime_sign_flip_rate | ge | 0.5429 | 242 | $10,406.36 | $1,134.64 | $-1,134.64 | 21.07% | 39.26% |
| regime_realized_vol_180s_bps | le | 1.3884 | 374 | $19,040.19 | $1,154.05 | $-1,154.05 | 21.93% | 38.77% |
| regime_whipsaw_score | le | 0.2528 | 748 | $34,979.90 | $1,173.79 | $-1,173.79 | 22.59% | 40.24% |
| regime_path_efficiency | le | 0.0509 | 374 | $16,678.03 | $1,188.64 | $-1,188.64 | 21.39% | 39.57% |
| side_edge_vs_fill | ge | 0.1293 | 373 | $12,309.75 | $1,225.21 | $-1,225.21 | 26.27% | 46.92% |
| market_yes_range_so_far | le | 0.3650 | 226 | $10,051.14 | $1,227.66 | $-1,227.66 | 23.89% | 41.59% |
| regime_path_efficiency | le | 0.0725 | 559 | $24,525.26 | $1,230.09 | $-1,230.09 | 22.00% | 40.43% |
| regime_whipsaw_score | le | 0.2244 | 373 | $16,406.72 | $1,250.24 | $-1,250.24 | 20.38% | 38.87% |

## Two-Feature Candidate Scan

| Candidate | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confirm_low_edge_reversal | 326 | $14,400.21 | $24.47 | $-24.47 | 21.47% | 36.81% |
| price_high_edge_low | 401 | $20,875.07 | $974.36 | $-974.36 | 12.72% | 21.70% |
| price_high_reversal | 348 | $18,105.43 | $1,162.55 | $-1,162.55 | 12.93% | 24.14% |
| fav_high_price_chop | 340 | $19,307.78 | $1,377.42 | $-1,377.42 | 13.53% | 26.47% |
| obs_mid_high_signflip | 1040 | $46,281.50 | $3,119.13 | $-3,119.13 | 20.10% | 38.27% |
| high_reversal_low_eff | 992 | $43,251.22 | $3,261.74 | $-3,261.74 | 19.86% | 38.21% |
| high_signflip_low_eff | 1257 | $55,992.22 | $3,787.74 | $-3,787.74 | 20.60% | 38.82% |
