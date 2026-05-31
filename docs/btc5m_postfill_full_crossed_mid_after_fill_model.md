# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_watch_markets.jsonl`
Target: `crossed_mid_after_fill`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `3028` before `2026-04-20T17:20:00+00:00`
Test fills: `521` in final `30` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 1158 | 38.24% | 0.6524 | 0.2303 | 0.6601 | $9,315.08 |
| test | 232 | 44.53% | 0.6598 | 0.2341 | 0.6368 | $-231.04 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 105 | 0.3229 | $141.07 | $8,492.84 | 28.57% | 28.57% | 0.2230 | 0.8272 |
| 2 | 104 | 0.4206 | $-501.05 | $7,470.83 | 43.27% | 43.27% | 0.2928 | 0.7162 |
| 3 | 104 | 0.4877 | $3.74 | $6,843.49 | 35.58% | 35.58% | 0.2708 | 0.7499 |
| 4 | 104 | 0.5606 | $245.79 | $6,723.03 | 52.88% | 52.88% | 0.3183 | 0.7575 |
| 5 | 104 | 0.6782 | $-120.59 | $8,734.32 | 62.50% | 62.50% | 0.3127 | 0.6493 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5596 | all | 909 | $3,562.39 | 154 | $54.34 | $-285.38 | 59.09% | 59.09% |
| 0.70 | 0.5596 | br2_high_skew_load | 155 | $511.39 | 43 | $32.29 | $245.28 | 48.84% | 48.84% |
| 0.70 | 0.5596 | br2_late_confirm | 570 | $2,798.14 | 68 | $268.14 | $-487.20 | 73.53% | 73.53% |
| 0.70 | 0.5596 | br2_late_favourite_load | 184 | $252.86 | 43 | $-246.08 | $-43.46 | 46.51% | 46.51% |
| 0.80 | 0.6137 | all | 606 | $3,451.76 | 96 | $-315.71 | $84.67 | 65.62% | 65.62% |
| 0.80 | 0.6137 | br2_high_skew_load | 98 | $252.89 | 20 | $-109.54 | $387.11 | 65.00% | 65.00% |
| 0.80 | 0.6137 | br2_late_confirm | 434 | $3,328.43 | 57 | $-114.11 | $-104.95 | 71.93% | 71.93% |
| 0.80 | 0.6137 | br2_late_favourite_load | 74 | $-129.56 | 19 | $-92.06 | $-197.48 | 47.37% | 47.37% |
| 0.90 | 0.6745 | all | 303 | $2,313.97 | 54 | $-595.98 | $364.94 | 72.22% | 72.22% |
| 0.90 | 0.6745 | br2_high_skew_load | 43 | $240.63 | 12 | $-151.36 | $428.93 | 75.00% | 75.00% |
| 0.90 | 0.6745 | br2_late_confirm | 242 | $2,001.28 | 35 | $-393.80 | $174.73 | 74.29% | 74.29% |
| 0.90 | 0.6745 | br2_late_favourite_load | 18 | $72.06 | 7 | $-50.82 | $-238.72 | 57.14% | 57.14% |
| 0.95 | 0.7131 | all | 152 | $499.00 | 24 | $-149.46 | $-81.58 | 75.00% | 75.00% |
| 0.95 | 0.7131 | br2_high_skew_load | 19 | $-32.44 | 5 | $-59.24 | $336.81 | 80.00% | 80.00% |
| 0.95 | 0.7131 | br2_late_confirm | 129 | $483.97 | 18 | $-98.88 | $-120.18 | 77.78% | 77.78% |
| 0.95 | 0.7131 | br2_late_favourite_load | 4 | $47.47 | 1 | $8.66 | $-298.21 | 0.00% | 0.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 177 | $277.57 | $8,705.04 | 36.16% | 36.16% | 0.2622 | 0.4790 |
| br2_late_confirm | 133 | $-219.06 | $16,094.47 | 56.39% | 56.39% | 0.2894 | 0.5334 |
| br2_late_favourite_load | 211 | $-289.54 | $13,465.00 | 44.08% | 44.08% | 0.2974 | 0.4810 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| price | -2.3041 |
| side_model_p | -2.2559 |
| price_x_model_p | -2.0200 |
| market_yes_range_so_far | 1.9696 |
| edge_x_confidence | -1.6314 |
| side_edge_vs_fill | 1.4268 |
| prior1d_x_range | 1.3571 |
| regime_path_efficiency | -1.2369 |
| range_x_sign_flip | -1.0347 |
| prior_market_range_3d | -0.8489 |
| whipsaw_x_reversal | 0.8486 |
| regime_whipsaw_score | -0.6856 |
| regime_reversal_pressure | 0.6078 |
| whipsaw_x_low_efficiency | -0.5734 |
| regime_sign_flip_rate | 0.5422 |
| risk_score | 0.4857 |
| risk_x_range | 0.4386 |
| prior7d_minus_1d | 0.4043 |
| range_x_reversal | 0.3705 |
| confidence_score | 0.2533 |
| prior_market_range_1d | -0.2272 |
| prior_market_range_7d | -0.2096 |
| regime_realized_vol_180s_bps | -0.1369 |
| tag:br2_high_skew_load | -0.0463 |
| buy_yes | 0.0336 |
| tag:br2_late_favourite_load | 0.0245 |
| tag:br2_late_confirm | 0.0137 |
| vol_x_reversal | -0.0066 |

