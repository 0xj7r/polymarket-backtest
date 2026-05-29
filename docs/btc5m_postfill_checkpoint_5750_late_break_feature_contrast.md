# BTC5m Late-Break Feature Contrast

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Fills: `1463` late-confirm/favourite fills
Calendar: `2026-02-27T15:45:00+00:00` to `2026-03-19T14:20:00+00:00`
PnL: `$3,581.52`
Toxic fills: `305` (`20.85%`)

This diagnostic contrasts failed late breaks against profitable late breaks using fill-time features only. Post-fill labels are used only to define the offline target.

## By Lane

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| br2_late_confirm | 702 | $1,957.89 | $25,008.02 | 75.21% | 23.36% | 42.02% |
| br2_late_favourite_load | 761 | $1,623.63 | $25,594.88 | 81.08% | 18.53% | 34.69% |

## By Post-Fill Path

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| crossed_mid_after_fill | 559 | $-6,760.52 | $19,234.44 | 45.62% | 54.38% | 100.00% |
| held_side | 701 | $8,660.23 | $24,701.16 | 99.29% | 0.00% | 0.00% |
| moderate_adverse_no_cross | 203 | $1,681.82 | $6,667.30 | 95.57% | 0.49% | 0.00% |

## Feature Contrast: Toxic vs Profitable Non-Toxic Late Breaks

| Feature | Toxic Mean | Profitable Non-Toxic Mean | Difference | Std Diff | Toxic N | Profitable N |
|---|---:|---:|---:|---:|---:|---:|
| price | 0.7033 | 0.7376 | -0.0343 | -0.388 | 305 | 1145 |
| side_model_p | 0.8087 | 0.8363 | -0.0275 | -0.338 | 305 | 1145 |
| risk_score | 0.4235 | 0.4096 | 0.0139 | 0.172 | 305 | 1145 |
| prior_market_range_7d | 0.6769 | 0.6778 | -0.0010 | -0.145 | 305 | 1145 |
| prior_market_range_3d | 0.6769 | 0.6781 | -0.0012 | -0.140 | 305 | 1145 |
| side_edge_vs_fill | 0.1054 | 0.0986 | 0.0068 | 0.138 | 305 | 1145 |
| regime_realized_vol_180s_bps | 2.0177 | 2.1251 | -0.1074 | -0.114 | 305 | 1145 |
| regime_whipsaw_score | 0.2900 | 0.2973 | -0.0073 | -0.079 | 305 | 1145 |
| market_yes_range_so_far | 0.4437 | 0.4497 | -0.0060 | -0.078 | 305 | 1145 |
| confidence_score | 0.8712 | 0.8755 | -0.0043 | -0.068 | 305 | 1145 |
| regime_reversal_pressure | 0.3304 | 0.3264 | 0.0040 | 0.035 | 305 | 1145 |
| seconds_to_close | 65.0801 | 65.9222 | -0.8421 | -0.028 | 305 | 1145 |
| prior_market_range_1d | 0.6778 | 0.6781 | -0.0003 | -0.025 | 305 | 1145 |
| regime_sign_flip_rate | 0.4256 | 0.4240 | 0.0016 | 0.017 | 305 | 1145 |
| regime_path_efficiency | 0.1421 | 0.1414 | 0.0007 | 0.007 | 305 | 1145 |

## Quartiles: price

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| price:q1 | 354 | $1,233.13 | $12,229.91 | 66.38% | 32.49% | 59.04% |
| price:q2 | 340 | $1,048.89 | $8,805.48 | 80.29% | 18.82% | 35.29% |
| price:q3 | 402 | $685.78 | $13,730.68 | 80.60% | 18.91% | 37.31% |
| price:q4 | 367 | $613.72 | $15,836.82 | 85.29% | 13.62% | 21.80% |

## Quartiles: side_model_p

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_model_p:q1 | 365 | $1,391.27 | $13,139.81 | 68.77% | 29.86% | 55.07% |
| side_model_p:q2 | 366 | $988.76 | $9,911.15 | 78.42% | 20.49% | 38.25% |
| side_model_p:q3 | 365 | $830.60 | $12,174.60 | 81.64% | 17.81% | 35.07% |
| side_model_p:q4 | 367 | $370.90 | $15,377.35 | 84.20% | 15.26% | 24.52% |

## Quartiles: risk_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| risk_score:q1 | 365 | $2,033.02 | $14,471.09 | 81.92% | 17.53% | 31.78% |
| risk_score:q2 | 366 | $720.59 | $12,623.21 | 79.51% | 19.67% | 38.52% |
| risk_score:q3 | 365 | $554.24 | $11,112.40 | 78.63% | 20.82% | 38.08% |
| risk_score:q4 | 367 | $273.67 | $12,396.20 | 73.02% | 25.34% | 44.41% |

## Quartiles: prior_market_range_7d

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| prior_market_range_7d:q1 | 365 | $2,071.58 | $16,271.60 | 79.45% | 20.00% | 35.34% |
| prior_market_range_7d:q2 | 366 | $-449.20 | $13,187.53 | 71.58% | 27.87% | 45.90% |
| prior_market_range_7d:q3 | 365 | $1,020.58 | $12,098.76 | 80.27% | 19.18% | 40.55% |
| prior_market_range_7d:q4 | 367 | $938.57 | $9,045.01 | 81.74% | 16.35% | 31.06% |

## Quartiles: prior_market_range_3d

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| prior_market_range_3d:q1 | 365 | $1,994.26 | $16,434.53 | 79.45% | 19.73% | 34.25% |
| prior_market_range_3d:q2 | 365 | $-40.69 | $14,657.19 | 74.25% | 25.48% | 45.75% |
| prior_market_range_3d:q3 | 366 | $657.94 | $10,579.03 | 77.32% | 21.86% | 42.08% |
| prior_market_range_3d:q4 | 367 | $970.02 | $8,932.14 | 82.02% | 16.35% | 30.79% |

## Quartiles: side_edge_vs_fill

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_edge_vs_fill:q1 | 365 | $528.13 | $13,290.44 | 80.00% | 18.90% | 32.05% |
| side_edge_vs_fill:q2 | 366 | $1,012.43 | $13,881.96 | 78.96% | 19.95% | 39.07% |
| side_edge_vs_fill:q3 | 365 | $1,069.51 | $13,345.38 | 80.82% | 18.36% | 35.89% |
| side_edge_vs_fill:q4 | 367 | $971.46 | $10,085.12 | 73.30% | 26.16% | 45.78% |

## Single-Feature Removal Scan

Positive removed PnL means a gate would remove profitable fills. Negative removed PnL is the interesting direction.

| Feature | Direction | Threshold | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| regime_reversal_pressure | ge | 0.5600 | 110 | $3,566.69 | $-75.90 | $75.90 | 25.45% | 47.27% |
| risk_score | ge | 0.4770 | 295 | $10,074.77 | $-23.52 | $23.52 | 27.12% | 45.42% |
| regime_reversal_pressure | ge | 0.6200 | 64 | $2,028.05 | $66.47 | $-66.47 | 18.75% | 45.31% |
| regime_reversal_pressure | ge | 0.5000 | 138 | $4,387.00 | $68.03 | $-68.03 | 22.46% | 42.75% |
| regime_sign_flip_rate | le | 0.2286 | 37 | $1,295.97 | $136.47 | $-136.47 | 18.92% | 40.54% |
| regime_sign_flip_rate | ge | 0.5714 | 112 | $3,653.03 | $212.46 | $-212.46 | 23.21% | 40.18% |
| regime_reversal_pressure | ge | 0.4400 | 173 | $5,674.06 | $257.65 | $-257.65 | 21.39% | 40.46% |
| regime_path_efficiency | le | 0.0517 | 292 | $9,463.63 | $338.44 | $-338.44 | 22.26% | 40.07% |
| side_model_p | ge | 0.8976 | 304 | $12,709.73 | $346.68 | $-346.68 | 15.46% | 24.01% |
| seconds_to_close | le | 39.4930 | 292 | $9,227.54 | $369.68 | $-369.68 | 22.26% | 38.01% |
| regime_realized_vol_180s_bps | le | 1.4094 | 294 | $10,943.45 | $390.30 | $-390.30 | 22.45% | 38.78% |
| side_model_p | ge | 0.8909 | 451 | $18,428.49 | $413.34 | $-413.34 | 16.19% | 25.72% |
| seconds_to_close | ge | 98.9740 | 295 | $10,729.01 | $461.37 | $-461.37 | 23.05% | 39.66% |
| regime_path_efficiency | le | 0.0732 | 437 | $14,476.97 | $486.16 | $-486.16 | 22.43% | 40.73% |
| side_edge_vs_fill | le | 0.0552 | 296 | $10,750.89 | $506.98 | $-506.98 | 15.88% | 28.38% |
| regime_whipsaw_score | le | 0.2247 | 293 | $10,527.97 | $516.01 | $-516.01 | 21.50% | 39.25% |
| side_edge_vs_fill | le | 0.0799 | 441 | $15,958.98 | $534.15 | $-534.15 | 20.41% | 35.15% |
| market_yes_range_so_far | le | 0.3250 | 86 | $2,917.85 | $534.65 | $-534.65 | 30.23% | 41.86% |
| market_yes_range_so_far | ge | 0.5450 | 155 | $3,804.91 | $555.07 | $-555.07 | 17.42% | 36.13% |
| prior_market_range_1d | ge | 0.6869 | 341 | $7,440.62 | $567.65 | $-567.65 | 19.06% | 31.67% |
| regime_reversal_pressure | le | 0.2400 | 326 | $11,447.05 | $593.76 | $-593.76 | 20.55% | 37.73% |
| seconds_to_close | ge | 85.5950 | 439 | $15,828.44 | $603.52 | $-603.52 | 21.87% | 37.81% |
| regime_realized_vol_180s_bps | le | 1.7962 | 735 | $26,616.44 | $610.05 | $-610.05 | 24.22% | 41.36% |
| prior_market_range_1d | ge | 0.6842 | 485 | $11,389.67 | $655.10 | $-655.10 | 21.03% | 35.26% |
| price | ge | 0.7965 | 300 | $13,273.05 | $656.45 | $-656.45 | 12.67% | 19.00% |
| regime_sign_flip_rate | le | 0.2857 | 135 | $4,387.55 | $660.59 | $-660.59 | 14.81% | 37.78% |
| risk_score | ge | 0.3837 | 879 | $28,303.48 | $688.69 | $-688.69 | 23.21% | 41.52% |
| prior_market_range_3d | ge | 0.6855 | 331 | $8,064.06 | $713.40 | $-713.40 | 16.92% | 30.82% |
| prior_market_range_7d | ge | 0.6824 | 333 | $8,056.20 | $729.04 | $-729.04 | 17.72% | 32.13% |
| regime_whipsaw_score | le | 0.2407 | 440 | $16,021.56 | $729.58 | $-729.58 | 22.50% | 39.32% |

## Two-Feature Candidate Scan

| Candidate | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confirm_low_edge_reversal | 266 | $9,388.82 | $582.85 | $-582.85 | 19.17% | 36.09% |
| price_high_edge_low | 309 | $12,682.47 | $637.90 | $-637.90 | 13.27% | 22.33% |
| price_high_reversal | 279 | $11,466.20 | $784.46 | $-784.46 | 13.26% | 24.01% |
| fav_high_price_chop | 263 | $11,787.76 | $910.61 | $-910.61 | 14.07% | 25.48% |
| obs_mid_high_signflip | 797 | $27,190.26 | $1,919.25 | $-1,919.25 | 20.20% | 38.27% |
| high_reversal_low_eff | 782 | $26,405.11 | $2,496.27 | $-2,496.27 | 19.31% | 37.72% |
| high_signflip_low_eff | 977 | $33,689.21 | $2,910.27 | $-2,910.27 | 20.16% | 38.49% |
