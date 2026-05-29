# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1086` before `2026-03-09T04:15:00+00:00`
Test fills: `551` in final `5` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 212 | 19.52% | 0.6488 | 0.2286 | 0.6653 | $1,918.35 |
| test | 115 | 20.87% | 0.6736 | 0.2400 | 0.5972 | $663.02 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 111 | 0.3182 | $107.60 | $4,993.37 | 15.32% | 21.62% | 0.1697 | 0.8564 |
| 2 | 110 | 0.4175 | $243.00 | $4,499.25 | 16.36% | 37.27% | 0.2392 | 0.8225 |
| 3 | 110 | 0.4780 | $190.55 | $4,408.80 | 18.18% | 41.82% | 0.2592 | 0.7799 |
| 4 | 110 | 0.5443 | $32.85 | $4,125.65 | 23.64% | 48.18% | 0.2712 | 0.7555 |
| 5 | 110 | 0.6461 | $89.02 | $4,507.43 | 30.91% | 52.73% | 0.2608 | 0.6860 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5409 | all | 326 | $-88.57 | 167 | $539.30 | $123.72 | 26.35% | 50.90% |
| 0.70 | 0.5409 | br2_high_skew_load | 52 | $-82.95 | 28 | $53.70 | $93.74 | 17.86% | 39.29% |
| 0.70 | 0.5409 | br2_late_confirm | 162 | $243.46 | 67 | $395.01 | $-345.04 | 31.34% | 58.21% |
| 0.70 | 0.5409 | br2_late_favourite_load | 112 | $-249.09 | 72 | $90.59 | $375.02 | 25.00% | 48.61% |
| 0.80 | 0.5863 | all | 218 | $-89.68 | 103 | $222.17 | $440.84 | 30.10% | 53.40% |
| 0.80 | 0.5863 | br2_high_skew_load | 34 | $-49.71 | 14 | $-7.13 | $154.57 | 28.57% | 42.86% |
| 0.80 | 0.5863 | br2_late_confirm | 124 | $141.78 | 54 | $264.74 | $-214.77 | 33.33% | 61.11% |
| 0.80 | 0.5863 | br2_late_favourite_load | 60 | $-181.76 | 35 | $-35.44 | $501.04 | 25.71% | 45.71% |
| 0.90 | 0.6411 | all | 109 | $-99.88 | 46 | $138.56 | $524.45 | 34.78% | 58.70% |
| 0.90 | 0.6411 | br2_high_skew_load | 12 | $-1.34 | 5 | $15.39 | $132.05 | 20.00% | 40.00% |
| 0.90 | 0.6411 | br2_late_confirm | 74 | $-67.05 | 28 | $145.36 | $-95.39 | 35.71% | 60.71% |
| 0.90 | 0.6411 | br2_late_favourite_load | 23 | $-31.49 | 13 | $-22.19 | $487.79 | 38.46% | 61.54% |
| 0.95 | 0.6981 | all | 55 | $-169.07 | 16 | $106.12 | $556.90 | 37.50% | 56.25% |
| 0.95 | 0.6981 | br2_high_skew_load | 8 | $-2.28 | 1 | $-18.78 | $166.22 | 100.00% | 100.00% |
| 0.95 | 0.6981 | br2_late_confirm | 36 | $-146.38 | 13 | $142.90 | $-92.94 | 30.77% | 53.85% |
| 0.95 | 0.6981 | br2_late_favourite_load | 11 | $-20.41 | 2 | $-18.01 | $483.61 | 50.00% | 50.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 139 | $147.45 | $2,659.31 | 17.99% | 39.57% | 0.2492 | 0.4656 |
| br2_late_confirm | 184 | $49.97 | $9,267.37 | 25.54% | 42.39% | 0.2263 | 0.4781 |
| br2_late_favourite_load | 228 | $465.60 | $10,607.81 | 18.86% | 39.04% | 0.2452 | 0.4915 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | -8.2524 |
| prior_market_range_3d | -5.9797 |
| prior_market_range_7d | -4.4354 |
| side_edge_vs_fill | 3.3449 |
| prior_market_range_1d | -2.6544 |
| edge_x_confidence | 2.1098 |
| price | -2.0415 |
| regime_sign_flip_rate | 1.4365 |
| regime_reversal_pressure | 1.2436 |
| range_x_reversal | -1.2167 |
| side_model_p | -1.1737 |
| price_x_model_p | -1.1259 |
| regime_path_efficiency | 1.0297 |
| risk_x_range | 1.0291 |
| whipsaw_x_low_efficiency | -1.0052 |
| whipsaw_x_reversal | -0.5382 |
| range_x_sign_flip | 0.4330 |
| buy_yes | -0.3146 |
| market_yes_range_so_far | 0.3117 |
| prior1d_x_range | 0.2435 |
| risk_score | 0.1361 |
| confidence_score | 0.1042 |
| tag:br2_late_confirm | 0.0988 |
| regime_realized_vol_180s_bps | -0.0743 |
| tag:br2_high_skew_load | -0.0714 |
| vol_x_reversal | -0.0572 |
| tag:br2_late_favourite_load | -0.0452 |
| regime_whipsaw_score | 0.0363 |

