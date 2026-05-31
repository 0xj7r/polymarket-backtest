# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_markets_062901.jsonl`
Target: `crossed_mid_after_fill`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `799` before `2026-03-05T15:25:00+00:00`
Test fills: `1699` in final `30` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 276 | 34.54% | 0.6398 | 0.2248 | 0.6753 | $1,021.42 |
| test | 669 | 39.38% | 0.6799 | 0.2387 | 0.6368 | $5,334.96 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 340 | 0.2504 | $1,274.44 | $20,589.02 | 23.53% | 23.53% | 0.1883 | 0.8476 |
| 2 | 340 | 0.3908 | $1,111.10 | $16,153.68 | 29.71% | 29.71% | 0.1969 | 0.8542 |
| 3 | 340 | 0.4753 | $501.20 | $14,852.44 | 41.76% | 41.76% | 0.2546 | 0.7754 |
| 4 | 340 | 0.5651 | $90.53 | $13,756.17 | 46.18% | 46.18% | 0.2678 | 0.7299 |
| 5 | 339 | 0.7176 | $2,357.70 | $14,940.56 | 55.75% | 55.75% | 0.2551 | 0.6724 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5562 | all | 240 | $348.99 | 528 | $2,211.28 | $3,123.68 | 53.03% | 53.03% |
| 0.70 | 0.5562 | br2_high_skew_load | 31 | $-13.35 | 80 | $203.92 | $338.99 | 45.00% | 45.00% |
| 0.70 | 0.5562 | br2_late_confirm | 149 | $310.74 | 311 | $1,812.84 | $13.63 | 55.31% | 55.31% |
| 0.70 | 0.5562 | br2_late_favourite_load | 60 | $51.60 | 137 | $194.52 | $2,771.06 | 52.55% | 52.55% |
| 0.80 | 0.6118 | all | 160 | $245.88 | 370 | $2,261.73 | $3,073.24 | 55.14% | 55.14% |
| 0.80 | 0.6118 | br2_high_skew_load | 17 | $-1.68 | 46 | $167.28 | $375.63 | 50.00% | 50.00% |
| 0.80 | 0.6118 | br2_late_confirm | 120 | $237.21 | 259 | $1,891.05 | $-64.57 | 57.92% | 57.92% |
| 0.80 | 0.6118 | br2_late_favourite_load | 23 | $10.35 | 65 | $203.40 | $2,762.17 | 47.69% | 47.69% |
| 0.90 | 0.6883 | all | 80 | $-78.72 | 193 | $2,004.27 | $3,330.69 | 54.92% | 54.92% |
| 0.90 | 0.6883 | br2_high_skew_load | 4 | $5.43 | 16 | $368.83 | $174.08 | 12.50% | 12.50% |
| 0.90 | 0.6883 | br2_late_confirm | 68 | $-85.30 | 164 | $1,632.78 | $193.70 | 59.15% | 59.15% |
| 0.90 | 0.6883 | br2_late_favourite_load | 8 | $1.14 | 13 | $2.66 | $2,962.91 | 53.85% | 53.85% |
| 0.95 | 0.7350 | all | 40 | $-117.03 | 111 | $1,237.99 | $4,096.97 | 54.05% | 54.05% |
| 0.95 | 0.7350 | br2_high_skew_load | 3 | $1.75 | 10 | $342.14 | $200.78 | 0.00% | 0.00% |
| 0.95 | 0.7350 | br2_late_confirm | 35 | $-110.79 | 97 | $904.32 | $922.16 | 59.79% | 59.79% |
| 0.95 | 0.7350 | br2_late_favourite_load | 2 | $-8.00 | 4 | $-8.47 | $2,974.04 | 50.00% | 50.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 450 | $542.91 | $11,082.29 | 38.67% | 38.67% | 0.2507 | 0.4396 |
| br2_late_confirm | 592 | $1,826.48 | $34,053.65 | 43.24% | 43.24% | 0.2248 | 0.5357 |
| br2_late_favourite_load | 657 | $2,965.58 | $35,155.92 | 36.38% | 36.38% | 0.2271 | 0.4566 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior_market_range_3d | -8.3754 |
| price | -2.7692 |
| side_edge_vs_fill | 2.7581 |
| prior7d_minus_1d | 2.7445 |
| range_x_sign_flip | -2.3898 |
| whipsaw_x_reversal | 2.3341 |
| side_model_p | -2.0159 |
| prior1d_x_range | 1.6754 |
| price_x_model_p | -1.5511 |
| confidence_score | -1.3371 |
| market_yes_range_so_far | 1.0212 |
| regime_whipsaw_score | -1.0066 |
| range_x_reversal | -0.7330 |
| whipsaw_x_low_efficiency | -0.7219 |
| edge_x_confidence | 0.7022 |
| regime_path_efficiency | 0.5628 |
| regime_sign_flip_rate | 0.5427 |
| regime_reversal_pressure | 0.4938 |
| risk_x_range | 0.4918 |
| prior_market_range_7d | 0.2005 |
| regime_realized_vol_180s_bps | -0.1709 |
| vol_x_reversal | 0.1426 |
| tag:br2_high_skew_load | -0.1314 |
| tag:br2_late_confirm | 0.1107 |
| risk_score | -0.0692 |
| prior_market_range_1d | -0.0141 |
| tag:br2_late_favourite_load | -0.0084 |
| seconds_to_close | 0.0052 |

