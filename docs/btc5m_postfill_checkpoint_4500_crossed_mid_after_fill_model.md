# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `crossed_mid_after_fill`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1221` before `2026-03-10T03:10:00+00:00`
Test fills: `419` in final `5` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 439 | 35.95% | 0.6250 | 0.2178 | 0.7023 | $2,226.69 |
| test | 174 | 41.53% | 0.6930 | 0.2488 | 0.5900 | $365.87 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 84 | 0.2833 | $188.51 | $3,744.12 | 29.76% | 29.76% | 0.2038 | 0.8527 |
| 2 | 84 | 0.4056 | $211.46 | $3,378.89 | 38.10% | 38.10% | 0.2598 | 0.8277 |
| 3 | 84 | 0.4882 | $-168.23 | $3,595.19 | 44.05% | 44.05% | 0.2800 | 0.7325 |
| 4 | 84 | 0.5775 | $-402.46 | $2,955.86 | 46.43% | 46.43% | 0.2636 | 0.7114 |
| 5 | 83 | 0.7193 | $536.59 | $3,622.53 | 49.40% | 49.40% | 0.2401 | 0.7193 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5768 | all | 367 | $90.43 | 125 | $331.44 | $34.44 | 51.20% | 51.20% |
| 0.70 | 0.5768 | br2_high_skew_load | 50 | $-69.85 | 17 | $48.77 | $10.23 | 47.06% | 47.06% |
| 0.70 | 0.5768 | br2_late_confirm | 225 | $330.05 | 77 | $147.80 | $-58.59 | 57.14% | 57.14% |
| 0.70 | 0.5768 | br2_late_favourite_load | 92 | $-169.77 | 31 | $134.86 | $82.80 | 38.71% | 38.71% |
| 0.80 | 0.6342 | all | 245 | $198.86 | 83 | $536.59 | $-170.71 | 49.40% | 49.40% |
| 0.80 | 0.6342 | br2_high_skew_load | 25 | $-52.71 | 12 | $13.67 | $45.33 | 50.00% | 50.00% |
| 0.80 | 0.6342 | br2_late_confirm | 177 | $296.73 | 58 | $385.10 | $-295.89 | 55.17% | 55.17% |
| 0.80 | 0.6342 | br2_late_favourite_load | 43 | $-45.16 | 13 | $137.82 | $79.85 | 23.08% | 23.08% |
| 0.90 | 0.7147 | all | 123 | $-122.65 | 42 | $358.47 | $7.40 | 54.76% | 54.76% |
| 0.90 | 0.7147 | br2_high_skew_load | 7 | $-24.95 | 5 | $11.79 | $47.21 | 60.00% | 60.00% |
| 0.90 | 0.7147 | br2_late_confirm | 102 | $-56.75 | 35 | $326.77 | $-237.56 | 57.14% | 57.14% |
| 0.90 | 0.7147 | br2_late_favourite_load | 14 | $-40.95 | 2 | $19.91 | $197.75 | 0.00% | 0.00% |
| 0.95 | 0.7711 | all | 62 | $-238.58 | 13 | $94.56 | $271.31 | 61.54% | 61.54% |
| 0.95 | 0.7711 | br2_late_confirm | 52 | $-215.71 | 12 | $91.95 | $-2.74 | 66.67% | 66.67% |
| 0.95 | 0.7711 | br2_late_favourite_load | 7 | $-25.05 | 1 | $2.61 | $215.05 | 0.00% | 0.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 104 | $59.00 | $2,017.26 | 42.31% | 42.31% | 0.2746 | 0.4631 |
| br2_late_confirm | 151 | $89.21 | $7,715.82 | 43.05% | 43.05% | 0.2326 | 0.5302 |
| br2_late_favourite_load | 164 | $217.66 | $7,563.51 | 39.63% | 39.63% | 0.2491 | 0.4809 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | 8.9622 |
| prior_market_range_3d | -6.2910 |
| price | -3.2353 |
| side_edge_vs_fill | 3.1862 |
| side_model_p | -2.4847 |
| price_x_model_p | -2.0294 |
| risk_x_range | 1.9149 |
| whipsaw_x_low_efficiency | -1.3242 |
| range_x_sign_flip | -1.2768 |
| prior1d_x_range | 1.0917 |
| edge_x_confidence | 1.0887 |
| whipsaw_x_reversal | 1.0096 |
| regime_whipsaw_score | -0.9259 |
| prior_market_range_7d | 0.8429 |
| risk_score | 0.8088 |
| market_yes_range_so_far | 0.7457 |
| prior_market_range_1d | -0.5990 |
| regime_reversal_pressure | 0.5988 |
| range_x_reversal | 0.2322 |
| regime_sign_flip_rate | 0.2280 |
| regime_realized_vol_180s_bps | -0.1364 |
| vol_x_reversal | 0.0942 |
| confidence_score | -0.0775 |
| tag:br2_high_skew_load | -0.0511 |
| regime_path_efficiency | 0.0489 |
| buy_yes | -0.0343 |
| tag:br2_late_favourite_load | 0.0225 |
| tag:br2_late_confirm | 0.0148 |

