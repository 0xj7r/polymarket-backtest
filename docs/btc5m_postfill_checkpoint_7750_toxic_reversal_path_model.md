# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1680` before `2026-03-16T12:35:00+00:00`
Test fills: `567` in final `10` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 338 | 20.12% | 0.6574 | 0.2323 | 0.6525 | $2,682.26 |
| test | 121 | 21.34% | 0.7914 | 0.2852 | 0.5668 | $2,671.68 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 114 | 0.3833 | $-4.66 | $6,872.16 | 21.05% | 28.95% | 0.2160 | 0.7763 |
| 2 | 114 | 0.4714 | $861.18 | $5,793.51 | 10.53% | 25.44% | 0.1821 | 0.8694 |
| 3 | 113 | 0.5308 | $84.33 | $6,034.00 | 21.24% | 45.13% | 0.2411 | 0.7928 |
| 4 | 113 | 0.5946 | $268.83 | $5,039.03 | 26.55% | 53.10% | 0.2850 | 0.7325 |
| 5 | 113 | 0.7305 | $1,462.00 | $5,930.26 | 27.43% | 44.25% | 0.2152 | 0.7098 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5373 | all | 504 | $667.97 | 272 | $1,787.82 | $883.86 | 26.47% | 48.53% |
| 0.70 | 0.5373 | br2_high_skew_load | 80 | $-72.54 | 83 | $216.27 | $10.05 | 26.51% | 38.55% |
| 0.70 | 0.5373 | br2_late_confirm | 270 | $701.73 | 99 | $1,148.03 | $172.27 | 31.31% | 52.53% |
| 0.70 | 0.5373 | br2_late_favourite_load | 154 | $38.78 | 90 | $423.52 | $701.53 | 21.11% | 53.33% |
| 0.80 | 0.5837 | all | 336 | $507.08 | 188 | $1,417.98 | $1,253.70 | 29.26% | 49.47% |
| 0.80 | 0.5837 | br2_high_skew_load | 37 | $-2.75 | 46 | $291.33 | $-65.01 | 26.09% | 32.61% |
| 0.80 | 0.5837 | br2_late_confirm | 213 | $524.66 | 85 | $1,077.96 | $242.35 | 32.94% | 55.29% |
| 0.80 | 0.5837 | br2_late_favourite_load | 86 | $-14.82 | 57 | $48.69 | $1,076.36 | 26.32% | 54.39% |
| 0.90 | 0.6380 | all | 168 | $124.87 | 109 | $1,550.07 | $1,121.61 | 26.61% | 44.04% |
| 0.90 | 0.6380 | br2_high_skew_load | 14 | $1.04 | 24 | $300.70 | $-74.38 | 20.83% | 16.67% |
| 0.90 | 0.6380 | br2_late_confirm | 124 | $101.88 | 65 | $1,216.28 | $104.03 | 29.23% | 50.77% |
| 0.90 | 0.6380 | br2_late_favourite_load | 30 | $21.95 | 20 | $33.10 | $1,091.96 | 25.00% | 55.00% |
| 0.95 | 0.6827 | all | 84 | $-164.48 | 62 | $1,383.69 | $1,287.99 | 24.19% | 45.16% |
| 0.95 | 0.6827 | br2_high_skew_load | 5 | $-5.09 | 15 | $365.46 | $-139.14 | 6.67% | 6.67% |
| 0.95 | 0.6827 | br2_late_confirm | 67 | $-122.72 | 41 | $952.66 | $367.65 | 29.27% | 53.66% |
| 0.95 | 0.6827 | br2_late_favourite_load | 12 | $-36.67 | 6 | $65.58 | $1,059.48 | 33.33% | 83.33% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 179 | $226.32 | $4,990.24 | 22.91% | 37.99% | 0.2428 | 0.5372 |
| br2_late_confirm | 178 | $1,320.31 | $12,059.62 | 26.40% | 41.01% | 0.2087 | 0.5704 |
| br2_late_favourite_load | 210 | $1,125.05 | $12,619.10 | 15.71% | 39.05% | 0.2311 | 0.5213 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior_market_range_3d | -7.0544 |
| prior7d_minus_1d | -5.7619 |
| prior_market_range_7d | -5.4139 |
| side_edge_vs_fill | 3.4791 |
| prior_market_range_1d | -3.3425 |
| edge_x_confidence | 2.7579 |
| price | -1.7212 |
| regime_sign_flip_rate | 1.0634 |
| confidence_score | -0.9836 |
| regime_reversal_pressure | 0.9265 |
| prior1d_x_range | 0.9088 |
| price_x_model_p | -0.8266 |
| market_yes_range_so_far | 0.8255 |
| side_model_p | -0.8078 |
| risk_x_range | 0.7551 |
| whipsaw_x_low_efficiency | -0.6619 |
| range_x_reversal | -0.6390 |
| buy_yes | -0.4581 |
| risk_score | 0.4508 |
| regime_whipsaw_score | -0.3317 |
| whipsaw_x_reversal | -0.2164 |
| tag:br2_late_confirm | 0.1607 |
| regime_realized_vol_180s_bps | -0.1539 |
| tag:br2_high_skew_load | -0.0954 |
| tag:br2_late_favourite_load | -0.0844 |
| vol_x_reversal | -0.0664 |
| regime_path_efficiency | -0.0300 |
| seconds_to_close | 0.0066 |
