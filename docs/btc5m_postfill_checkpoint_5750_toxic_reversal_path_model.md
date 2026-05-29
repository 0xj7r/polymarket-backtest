# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1639` before `2026-03-14T14:25:00+00:00`
Test fills: `274` in final `5` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 327 | 19.95% | 0.6572 | 0.2323 | 0.6509 | $2,624.12 |
| test | 61 | 22.26% | 0.8819 | 0.3122 | 0.5454 | $1,530.97 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 55 | 0.4088 | $6.22 | $2,632.52 | 21.82% | 27.27% | 0.2144 | 0.7925 |
| 2 | 55 | 0.4885 | $234.83 | $2,123.34 | 14.55% | 25.45% | 0.1822 | 0.8336 |
| 3 | 55 | 0.5446 | $191.31 | $2,368.33 | 18.18% | 43.64% | 0.2159 | 0.8050 |
| 4 | 55 | 0.6161 | $-18.06 | $2,230.61 | 32.73% | 50.91% | 0.2836 | 0.6800 |
| 5 | 54 | 0.7936 | $1,116.66 | $2,196.68 | 24.07% | 42.59% | 0.1795 | 0.7222 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5382 | all | 492 | $500.32 | 140 | $1,254.75 | $276.22 | 26.43% | 47.14% |
| 0.70 | 0.5382 | br2_high_skew_load | 80 | $-76.91 | 37 | $344.70 | $-12.62 | 16.22% | 32.43% |
| 0.70 | 0.5382 | br2_late_confirm | 253 | $608.80 | 70 | $762.38 | $71.07 | 32.86% | 51.43% |
| 0.70 | 0.5382 | br2_late_favourite_load | 159 | $-31.57 | 33 | $147.67 | $217.77 | 24.24% | 54.55% |
| 0.80 | 0.5805 | all | 328 | $407.44 | 102 | $1,018.47 | $512.50 | 28.43% | 46.08% |
| 0.80 | 0.5805 | br2_high_skew_load | 39 | $-37.37 | 24 | $354.46 | $-22.38 | 12.50% | 20.83% |
| 0.80 | 0.5805 | br2_late_confirm | 197 | $471.94 | 56 | $529.69 | $303.75 | 37.50% | 53.57% |
| 0.80 | 0.5805 | br2_late_favourite_load | 92 | $-27.13 | 22 | $134.32 | $231.13 | 22.73% | 54.55% |
| 0.90 | 0.6340 | all | 164 | $9.57 | 70 | $1,145.28 | $385.69 | 25.71% | 42.86% |
| 0.90 | 0.6340 | br2_high_skew_load | 13 | $-18.52 | 15 | $378.03 | $-45.96 | 0.00% | 6.67% |
| 0.90 | 0.6340 | br2_late_confirm | 118 | $6.72 | 46 | $716.64 | $116.81 | 32.61% | 50.00% |
| 0.90 | 0.6340 | br2_late_favourite_load | 33 | $21.38 | 9 | $50.61 | $314.84 | 33.33% | 66.67% |
| 0.95 | 0.6784 | all | 82 | $-133.34 | 45 | $1,100.74 | $430.23 | 24.44% | 42.22% |
| 0.95 | 0.6784 | br2_high_skew_load | 5 | $-5.09 | 11 | $352.33 | $-20.26 | 0.00% | 0.00% |
| 0.95 | 0.6784 | br2_late_confirm | 62 | $-103.57 | 30 | $743.86 | $89.58 | 30.00% | 50.00% |
| 0.95 | 0.6784 | br2_late_favourite_load | 15 | $-24.67 | 4 | $4.54 | $360.90 | 50.00% | 100.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 73 | $332.07 | $1,531.39 | 17.81% | 32.88% | 0.2129 | 0.5857 |
| br2_late_confirm | 110 | $833.45 | $5,752.02 | 29.09% | 42.73% | 0.2082 | 0.5988 |
| br2_late_favourite_load | 91 | $365.45 | $4,268.07 | 17.58% | 36.26% | 0.2257 | 0.5212 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior_market_range_3d | -6.8586 |
| prior7d_minus_1d | -6.1155 |
| prior_market_range_7d | -5.2696 |
| side_edge_vs_fill | 3.7521 |
| prior_market_range_1d | -3.2006 |
| edge_x_confidence | 2.7865 |
| price | -1.7313 |
| regime_sign_flip_rate | 0.9937 |
| risk_x_range | 0.9179 |
| regime_reversal_pressure | 0.8723 |
| price_x_model_p | -0.7495 |
| side_model_p | -0.7329 |
| whipsaw_x_low_efficiency | -0.6911 |
| prior1d_x_range | 0.6821 |
| market_yes_range_so_far | 0.6424 |
| risk_score | 0.5741 |
| buy_yes | -0.4584 |
| confidence_score | -0.4092 |
| range_x_reversal | -0.3867 |
| regime_whipsaw_score | -0.3747 |
| whipsaw_x_reversal | -0.2449 |
| tag:br2_late_confirm | 0.1690 |
| range_x_sign_flip | 0.1636 |
| regime_realized_vol_180s_bps | -0.1565 |
| tag:br2_high_skew_load | -0.1023 |
| tag:br2_late_favourite_load | -0.0864 |
| vol_x_reversal | -0.0577 |
| regime_path_efficiency | 0.0441 |

