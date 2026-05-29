# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `213` before `2026-03-02T02:40:00+00:00`
Test fills: `131` in final `1` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 49 | 23.00% | 0.6074 | 0.2097 | 0.7320 | $124.02 |
| test | 33 | 25.19% | 0.7158 | 0.2585 | 0.5365 | $-7.78 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 27 | 0.2384 | $-12.34 | $376.82 | 18.52% | 37.04% | 0.2652 | 0.7793 |
| 2 | 26 | 0.3967 | $-2.70 | $325.40 | 26.92% | 46.15% | 0.3062 | 0.7690 |
| 3 | 26 | 0.4718 | $38.51 | $389.72 | 23.08% | 53.85% | 0.3223 | 0.7215 |
| 4 | 26 | 0.5601 | $-13.39 | $344.51 | 30.77% | 46.15% | 0.2912 | 0.6910 |
| 5 | 26 | 0.7156 | $-17.86 | $306.45 | 26.92% | 73.08% | 0.3988 | 0.7096 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5551 | all | 64 | $-95.84 | 42 | $22.43 | $-30.21 | 23.81% | 59.52% |
| 0.70 | 0.5551 | br2_high_skew_load | 6 | $5.61 | 8 | $13.54 | $-16.78 | 12.50% | 87.50% |
| 0.70 | 0.5551 | br2_late_confirm | 32 | $-41.23 | 13 | $54.10 | $28.87 | 23.08% | 69.23% |
| 0.70 | 0.5551 | br2_late_favourite_load | 26 | $-60.22 | 21 | $-45.21 | $-42.31 | 28.57% | 42.86% |
| 0.80 | 0.6068 | all | 43 | $-135.48 | 28 | $-2.01 | $-5.77 | 25.00% | 67.86% |
| 0.80 | 0.6068 | br2_high_skew_load | 3 | $-1.10 | 7 | $10.24 | $-13.48 | 14.29% | 85.71% |
| 0.80 | 0.6068 | br2_late_confirm | 25 | $-77.79 | 8 | $34.37 | $48.60 | 25.00% | 75.00% |
| 0.80 | 0.6068 | br2_late_favourite_load | 15 | $-56.59 | 13 | $-46.62 | $-40.89 | 30.77% | 53.85% |
| 0.90 | 0.7278 | all | 22 | $-79.01 | 12 | $12.39 | $-20.17 | 16.67% | 58.33% |
| 0.90 | 0.7278 | br2_high_skew_load | 2 | $-2.87 | 3 | $9.00 | $-12.24 | 0.00% | 100.00% |
| 0.90 | 0.7278 | br2_late_confirm | 13 | $-41.58 | 3 | $3.40 | $79.57 | 33.33% | 66.67% |
| 0.90 | 0.7278 | br2_late_favourite_load | 7 | $-34.56 | 6 | $-0.01 | $-87.50 | 16.67% | 33.33% |
| 0.95 | 0.7610 | all | 11 | $-89.90 | 6 | $6.27 | $-14.05 | 16.67% | 66.67% |
| 0.95 | 0.7610 | br2_high_skew_load | 1 | $-4.88 | 1 | $3.39 | $-6.63 | 0.00% | 100.00% |
| 0.95 | 0.7610 | br2_late_confirm | 6 | $-60.42 | 1 | $9.99 | $72.98 | 0.00% | 100.00% |
| 0.95 | 0.7610 | br2_late_favourite_load | 4 | $-24.59 | 4 | $-7.11 | $-80.41 | 25.00% | 50.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 34 | $-3.24 | $209.05 | 26.47% | 61.76% | 0.3563 | 0.4459 |
| br2_late_confirm | 36 | $82.97 | $585.39 | 19.44% | 50.00% | 0.2614 | 0.4478 |
| br2_late_favourite_load | 61 | $-87.51 | $948.45 | 27.87% | 45.90% | 0.3265 | 0.5066 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | -6.7691 |
| whipsaw_x_low_efficiency | -5.8279 |
| regime_path_efficiency | 3.2531 |
| prior_market_range_3d | -3.2010 |
| prior_market_range_7d | -3.2010 |
| range_x_sign_flip | 3.0283 |
| prior_market_range_1d | -2.8505 |
| side_model_p | -2.5480 |
| regime_sign_flip_rate | -2.5415 |
| price_x_model_p | -2.2394 |
| confidence_score | 2.2293 |
| edge_x_confidence | 2.1805 |
| price | -2.0781 |
| risk_score | 1.9543 |
| risk_x_range | -1.8860 |
| whipsaw_x_reversal | 1.6234 |
| side_edge_vs_fill | -1.1601 |
| regime_reversal_pressure | 0.7151 |
| range_x_reversal | -0.6524 |
| vol_x_reversal | 0.5843 |
| regime_realized_vol_180s_bps | 0.4065 |
| prior1d_x_range | -0.3564 |
| tag:br2_high_skew_load | -0.3204 |
| tag:br2_late_favourite_load | 0.2058 |
| regime_whipsaw_score | -0.1436 |
| buy_yes | 0.1182 |
| market_yes_range_so_far | 0.0617 |
| tag:br2_late_confirm | 0.0241 |

