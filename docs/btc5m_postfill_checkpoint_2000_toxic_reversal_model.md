# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `624` before `2026-03-04T14:10:00+00:00`
Test fills: `294` in final `2` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 117 | 18.75% | 0.6436 | 0.2261 | 0.6830 | $921.25 |
| test | 70 | 23.81% | 0.5770 | 0.1949 | 0.6191 | $216.64 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 59 | 0.2173 | $97.63 | $1,693.55 | 15.25% | 20.34% | 0.1679 | 0.8355 |
| 2 | 59 | 0.2986 | $70.34 | $1,374.84 | 18.64% | 28.81% | 0.2184 | 0.7867 |
| 3 | 59 | 0.3658 | $176.48 | $1,474.67 | 20.34% | 37.29% | 0.2251 | 0.7893 |
| 4 | 59 | 0.4271 | $110.46 | $1,532.01 | 23.73% | 50.85% | 0.2726 | 0.7564 |
| 5 | 58 | 0.5461 | $-238.27 | $1,609.36 | 41.38% | 68.97% | 0.3683 | 0.5316 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5422 | all | 187 | $30.65 | 26 | $-10.17 | $226.81 | 34.62% | 69.23% |
| 0.70 | 0.5422 | br2_late_confirm | 77 | $96.54 | 23 | $31.39 | $146.74 | 30.43% | 65.22% |
| 0.70 | 0.5422 | br2_late_favourite_load | 75 | $-88.11 | 3 | $-41.56 | $125.22 | 66.67% | 100.00% |
| 0.80 | 0.5974 | all | 125 | $41.96 | 11 | $-39.98 | $256.61 | 36.36% | 81.82% |
| 0.80 | 0.5974 | br2_late_confirm | 53 | $100.82 | 10 | $-46.62 | $224.76 | 40.00% | 80.00% |
| 0.80 | 0.5974 | br2_late_favourite_load | 49 | $-80.69 | 1 | $6.65 | $77.01 | 0.00% | 100.00% |
| 0.90 | 0.6479 | all | 63 | $27.88 | 4 | $-56.64 | $273.27 | 50.00% | 50.00% |
| 0.90 | 0.6479 | br2_late_confirm | 29 | $50.69 | 4 | $-56.64 | $234.77 | 50.00% | 50.00% |
| 0.95 | 0.6882 | all | 32 | $70.43 | 3 | $-79.76 | $296.40 | 66.67% | 66.67% |
| 0.95 | 0.6882 | br2_late_confirm | 17 | $57.88 | 3 | $-79.76 | $257.89 | 66.67% | 66.67% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 56 | $-45.15 | $625.89 | 26.79% | 37.50% | 0.2838 | 0.3143 |
| br2_late_confirm | 121 | $178.13 | $3,471.36 | 23.97% | 46.28% | 0.2362 | 0.4112 |
| br2_late_favourite_load | 117 | $83.65 | $3,587.19 | 22.22% | 37.61% | 0.2483 | 0.3551 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | -32.6422 |
| prior_market_range_3d | -15.9996 |
| prior_market_range_7d | -7.4375 |
| prior_market_range_1d | -5.3789 |
| side_edge_vs_fill | 2.0332 |
| regime_path_efficiency | 1.9891 |
| range_x_sign_flip | 1.9388 |
| risk_score | 1.6154 |
| regime_sign_flip_rate | -1.5678 |
| price | -1.5573 |
| range_x_reversal | -1.5290 |
| side_model_p | -0.9310 |
| whipsaw_x_low_efficiency | -0.8696 |
| price_x_model_p | -0.7557 |
| risk_x_range | 0.7447 |
| whipsaw_x_reversal | 0.7355 |
| regime_whipsaw_score | -0.6469 |
| prior1d_x_range | 0.5420 |
| market_yes_range_so_far | 0.5344 |
| buy_yes | -0.4467 |
| confidence_score | 0.4273 |
| edge_x_confidence | 0.3959 |
| tag:br2_high_skew_load | -0.2461 |
| tag:br2_late_confirm | 0.1753 |
| vol_x_reversal | 0.1407 |
| regime_realized_vol_180s_bps | -0.0752 |
| tag:br2_late_favourite_load | 0.0211 |
| seconds_to_close | 0.0086 |

