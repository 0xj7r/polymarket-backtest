# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Target: `crossed_mid_after_fill`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1639` before `2026-03-14T14:25:00+00:00`
Test fills: `274` in final `5` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 612 | 37.34% | 0.6407 | 0.2250 | 0.6784 | $2,624.12 |
| test | 104 | 37.96% | 0.8068 | 0.2763 | 0.6418 | $1,530.97 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 55 | 0.3825 | $171.20 | $2,802.94 | 14.55% | 14.55% | 0.1462 | 0.8746 |
| 2 | 55 | 0.4855 | $239.83 | $1,998.95 | 30.91% | 30.91% | 0.1918 | 0.8604 |
| 3 | 55 | 0.5641 | $3.76 | $2,379.21 | 43.64% | 43.64% | 0.2545 | 0.7460 |
| 4 | 55 | 0.6643 | $57.75 | $2,069.71 | 58.18% | 58.18% | 0.2903 | 0.6793 |
| 5 | 54 | 0.8315 | $1,058.43 | $2,300.68 | 42.59% | 42.59% | 0.1931 | 0.6721 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5652 | all | 492 | $716.50 | 136 | $1,133.22 | $397.75 | 47.79% | 47.79% |
| 0.70 | 0.5652 | br2_high_skew_load | 80 | $-26.61 | 35 | $312.98 | $19.09 | 34.29% | 34.29% |
| 0.70 | 0.5652 | br2_late_confirm | 283 | $659.59 | 75 | $720.01 | $113.44 | 53.33% | 53.33% |
| 0.70 | 0.5652 | br2_late_favourite_load | 129 | $83.52 | 26 | $100.23 | $265.22 | 50.00% | 50.00% |
| 0.80 | 0.6193 | all | 328 | $385.42 | 98 | $1,162.06 | $368.91 | 50.00% | 50.00% |
| 0.80 | 0.6193 | br2_high_skew_load | 38 | $-59.47 | 23 | $330.33 | $1.74 | 30.43% | 30.43% |
| 0.80 | 0.6193 | br2_late_confirm | 228 | $621.05 | 60 | $778.19 | $55.25 | 56.67% | 56.67% |
| 0.80 | 0.6193 | br2_late_favourite_load | 62 | $-176.16 | 15 | $53.53 | $311.91 | 53.33% | 53.33% |
| 0.90 | 0.6871 | all | 164 | $319.10 | 72 | $1,232.73 | $298.24 | 45.83% | 45.83% |
| 0.90 | 0.6871 | br2_high_skew_load | 16 | $-5.53 | 17 | $303.90 | $28.18 | 29.41% | 29.41% |
| 0.90 | 0.6871 | br2_late_confirm | 131 | $311.08 | 48 | $907.02 | $-73.57 | 47.92% | 47.92% |
| 0.90 | 0.6871 | br2_late_favourite_load | 17 | $13.56 | 7 | $21.81 | $343.63 | 71.43% | 71.43% |
| 0.95 | 0.7421 | all | 82 | $42.67 | 50 | $1,216.47 | $314.50 | 40.00% | 40.00% |
| 0.95 | 0.7421 | br2_high_skew_load | 5 | $0.95 | 12 | $326.06 | $6.02 | 8.33% | 8.33% |
| 0.95 | 0.7421 | br2_late_confirm | 69 | $64.17 | 34 | $876.72 | $-43.27 | 47.06% | 47.06% |
| 0.95 | 0.7421 | br2_late_favourite_load | 8 | $-22.44 | 4 | $13.69 | $351.75 | 75.00% | 75.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 73 | $332.07 | $1,531.39 | 32.88% | 32.88% | 0.2129 | 0.6000 |
| br2_late_confirm | 110 | $833.45 | $5,752.02 | 42.73% | 42.73% | 0.2082 | 0.6283 |
| br2_late_favourite_load | 91 | $365.45 | $4,268.07 | 36.26% | 36.26% | 0.2257 | 0.5197 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | 7.3155 |
| prior_market_range_3d | -7.2690 |
| side_edge_vs_fill | 3.0214 |
| price | -2.8247 |
| side_model_p | -2.0900 |
| edge_x_confidence | 1.9261 |
| risk_x_range | 1.6553 |
| price_x_model_p | -1.5993 |
| prior1d_x_range | 1.0786 |
| risk_score | 0.9749 |
| whipsaw_x_low_efficiency | -0.9637 |
| prior_market_range_1d | -0.9354 |
| range_x_sign_flip | -0.9128 |
| regime_reversal_pressure | 0.8592 |
| market_yes_range_so_far | 0.7559 |
| regime_whipsaw_score | -0.6727 |
| range_x_reversal | 0.6549 |
| whipsaw_x_reversal | 0.6321 |
| regime_path_efficiency | -0.5822 |
| confidence_score | 0.5345 |
| regime_sign_flip_rate | 0.3960 |
| prior_market_range_7d | 0.3344 |
| buy_yes | -0.1584 |
| regime_realized_vol_180s_bps | -0.1286 |
| tag:br2_late_confirm | 0.0717 |
| tag:br2_late_favourite_load | -0.0431 |
| tag:br2_high_skew_load | -0.0346 |
| vol_x_reversal | -0.0228 |

