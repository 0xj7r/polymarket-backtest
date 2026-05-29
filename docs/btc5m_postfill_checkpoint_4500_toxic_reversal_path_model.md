# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1221` before `2026-03-10T03:10:00+00:00`
Test fills: `419` in final `5` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 236 | 19.33% | 0.6477 | 0.2280 | 0.6637 | $2,226.69 |
| test | 92 | 21.96% | 0.6897 | 0.2474 | 0.5804 | $365.87 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 84 | 0.3232 | $40.46 | $3,616.10 | 15.48% | 25.00% | 0.1891 | 0.8587 |
| 2 | 84 | 0.4271 | $291.31 | $3,819.18 | 16.67% | 40.48% | 0.2611 | 0.8163 |
| 3 | 84 | 0.4864 | $-227.30 | $3,118.00 | 23.81% | 41.67% | 0.2569 | 0.7345 |
| 4 | 84 | 0.5509 | $-150.43 | $3,148.52 | 26.19% | 50.00% | 0.2876 | 0.7296 |
| 5 | 83 | 0.6559 | $411.84 | $3,594.78 | 27.71% | 50.60% | 0.2527 | 0.7043 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5336 | all | 367 | $87.75 | 147 | $537.97 | $-172.09 | 25.85% | 51.02% |
| 0.70 | 0.5336 | br2_high_skew_load | 54 | $-64.84 | 22 | $24.83 | $34.17 | 22.73% | 54.55% |
| 0.70 | 0.5336 | br2_late_confirm | 190 | $370.34 | 62 | $408.24 | $-319.03 | 29.03% | 53.23% |
| 0.70 | 0.5336 | br2_late_favourite_load | 123 | $-217.75 | 63 | $104.89 | $112.77 | 23.81% | 47.62% |
| 0.80 | 0.5812 | all | 245 | $8.29 | 90 | $461.53 | $-95.66 | 26.67% | 51.11% |
| 0.80 | 0.5812 | br2_high_skew_load | 27 | $-57.44 | 11 | $4.26 | $54.74 | 27.27% | 45.45% |
| 0.80 | 0.5812 | br2_late_confirm | 156 | $207.37 | 49 | $322.60 | $-233.39 | 30.61% | 57.14% |
| 0.80 | 0.5812 | br2_late_favourite_load | 62 | $-141.64 | 30 | $134.67 | $82.99 | 20.00% | 43.33% |
| 0.90 | 0.6510 | all | 123 | $-201.27 | 33 | $153.96 | $211.91 | 33.33% | 57.58% |
| 0.90 | 0.6510 | br2_high_skew_load | 12 | $0.31 | 3 | $-0.77 | $59.77 | 33.33% | 66.67% |
| 0.90 | 0.6510 | br2_late_confirm | 89 | $-148.61 | 23 | $123.80 | $-34.59 | 34.78% | 60.87% |
| 0.90 | 0.6510 | br2_late_favourite_load | 22 | $-52.97 | 7 | $30.92 | $186.74 | 28.57% | 42.86% |
| 0.95 | 0.6958 | all | 62 | $-244.85 | 17 | $190.04 | $175.83 | 23.53% | 41.18% |
| 0.95 | 0.6958 | br2_late_confirm | 49 | $-202.40 | 14 | $168.67 | $-79.46 | 28.57% | 50.00% |
| 0.95 | 0.6958 | br2_late_favourite_load | 9 | $-37.22 | 3 | $21.37 | $196.29 | 0.00% | 0.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 104 | $59.00 | $2,017.26 | 20.19% | 42.31% | 0.2746 | 0.4676 |
| br2_late_confirm | 151 | $89.21 | $7,715.82 | 25.17% | 43.05% | 0.2326 | 0.4901 |
| br2_late_favourite_load | 164 | $217.66 | $7,563.51 | 20.12% | 39.63% | 0.2491 | 0.4997 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior_market_range_3d | -5.8879 |
| prior7d_minus_1d | -5.8560 |
| prior_market_range_7d | -4.7852 |
| side_edge_vs_fill | 3.6034 |
| prior_market_range_1d | -3.1122 |
| edge_x_confidence | 2.1853 |
| price | -2.1831 |
| regime_sign_flip_rate | 1.6041 |
| range_x_reversal | -1.4432 |
| price_x_model_p | -1.2632 |
| side_model_p | -1.2520 |
| regime_reversal_pressure | 1.0933 |
| risk_x_range | 0.9969 |
| whipsaw_x_low_efficiency | -0.8952 |
| regime_path_efficiency | 0.5490 |
| market_yes_range_so_far | 0.3714 |
| buy_yes | -0.3486 |
| prior1d_x_range | 0.3052 |
| whipsaw_x_reversal | -0.2222 |
| regime_whipsaw_score | -0.1802 |
| regime_realized_vol_180s_bps | -0.1039 |
| range_x_sign_flip | -0.0831 |
| risk_score | 0.0714 |
| tag:br2_late_confirm | 0.0466 |
| tag:br2_high_skew_load | -0.0371 |
| tag:br2_late_favourite_load | -0.0179 |
| confidence_score | -0.0141 |
| vol_x_reversal | -0.0124 |

