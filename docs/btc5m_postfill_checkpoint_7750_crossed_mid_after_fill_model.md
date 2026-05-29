# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `crossed_mid_after_fill`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1680` before `2026-03-16T12:35:00+00:00`
Test fills: `567` in final `10` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 633 | 37.68% | 0.6385 | 0.2240 | 0.6825 | $2,682.26 |
| test | 223 | 39.33% | 0.7476 | 0.2616 | 0.5901 | $2,671.68 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 114 | 0.2673 | $-366.09 | $7,474.50 | 28.07% | 28.07% | 0.2024 | 0.7917 |
| 2 | 114 | 0.4177 | $981.67 | $5,968.87 | 28.07% | 28.07% | 0.1861 | 0.8705 |
| 3 | 113 | 0.5086 | $402.32 | $5,522.95 | 51.33% | 51.33% | 0.2860 | 0.7758 |
| 4 | 113 | 0.5953 | $382.66 | $5,395.03 | 38.94% | 38.94% | 0.2188 | 0.7589 |
| 5 | 113 | 0.7691 | $1,271.12 | $5,307.61 | 50.44% | 50.44% | 0.2461 | 0.6839 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5694 | all | 504 | $610.36 | 195 | $1,565.80 | $1,105.87 | 45.64% | 45.64% |
| 0.70 | 0.5694 | br2_high_skew_load | 84 | $-30.88 | 55 | $214.18 | $12.14 | 34.55% | 34.55% |
| 0.70 | 0.5694 | br2_late_confirm | 291 | $563.15 | 88 | $990.18 | $330.13 | 52.27% | 52.27% |
| 0.70 | 0.5694 | br2_late_favourite_load | 129 | $78.09 | 52 | $361.45 | $763.60 | 46.15% | 46.15% |
| 0.80 | 0.6237 | all | 336 | $368.44 | 137 | $1,305.89 | $1,365.79 | 49.64% | 49.64% |
| 0.80 | 0.6237 | br2_high_skew_load | 42 | $-58.35 | 35 | $184.02 | $42.29 | 40.00% | 40.00% |
| 0.80 | 0.6237 | br2_late_confirm | 231 | $509.72 | 72 | $992.70 | $327.61 | 55.56% | 55.56% |
| 0.80 | 0.6237 | br2_late_favourite_load | 63 | $-82.93 | 30 | $129.17 | $995.89 | 46.67% | 46.67% |
| 0.90 | 0.6940 | all | 168 | $417.03 | 87 | $1,340.32 | $1,331.36 | 45.98% | 45.98% |
| 0.90 | 0.6940 | br2_high_skew_load | 19 | $-8.13 | 19 | $281.95 | $-55.64 | 26.32% | 26.32% |
| 0.90 | 0.6940 | br2_late_confirm | 131 | $401.56 | 58 | $975.50 | $344.81 | 50.00% | 50.00% |
| 0.90 | 0.6940 | br2_late_favourite_load | 18 | $23.60 | 10 | $82.86 | $1,042.19 | 60.00% | 60.00% |
| 0.95 | 0.7460 | all | 84 | $49.45 | 56 | $1,110.94 | $1,560.74 | 41.07% | 41.07% |
| 0.95 | 0.7460 | br2_high_skew_load | 5 | $0.95 | 15 | $323.18 | $-96.87 | 13.33% | 13.33% |
| 0.95 | 0.7460 | br2_late_confirm | 72 | $76.05 | 36 | $754.37 | $565.94 | 50.00% | 50.00% |
| 0.95 | 0.7460 | br2_late_favourite_load | 7 | $-27.55 | 5 | $33.39 | $1,091.66 | 60.00% | 60.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 179 | $226.32 | $4,990.24 | 37.99% | 37.99% | 0.2428 | 0.5069 |
| br2_late_confirm | 178 | $1,320.31 | $12,059.62 | 41.01% | 41.01% | 0.2087 | 0.5512 |
| br2_late_favourite_load | 210 | $1,125.05 | $12,619.10 | 39.05% | 39.05% | 0.2311 | 0.4805 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | 7.4988 |
| prior_market_range_3d | -7.2119 |
| side_edge_vs_fill | 3.0390 |
| price | -2.8115 |
| edge_x_confidence | 2.2716 |
| side_model_p | -2.0803 |
| risk_x_range | 1.8083 |
| price_x_model_p | -1.6395 |
| range_x_sign_flip | -1.2209 |
| prior1d_x_range | 1.1467 |
| prior_market_range_1d | -0.9955 |
| whipsaw_x_low_efficiency | -0.9515 |
| regime_reversal_pressure | 0.9429 |
| risk_score | 0.9117 |
| market_yes_range_so_far | 0.8230 |
| range_x_reversal | 0.8036 |
| regime_path_efficiency | -0.6786 |
| regime_whipsaw_score | -0.6342 |
| whipsaw_x_reversal | 0.5382 |
| confidence_score | 0.4304 |
| regime_sign_flip_rate | 0.3910 |
| prior_market_range_7d | 0.3085 |
| buy_yes | -0.1540 |
| regime_realized_vol_180s_bps | -0.1236 |
| tag:br2_late_confirm | 0.0873 |
| tag:br2_late_favourite_load | -0.0583 |
| tag:br2_high_skew_load | -0.0349 |
| vol_x_reversal | -0.0345 |
