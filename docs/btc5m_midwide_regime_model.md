# BTC5m Mid-Wide Regime Model

Source: `s3://pm-research-backtest-prod/results/20260528T225810Z-portfolio-grid-52322/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8/markets.jsonl`
Target: `toxic_midwide` using replay-safe fill-time features only.
Train fills: `3028` before `2026-04-20T17:20:00+00:00`
Test fills: `521` in final `30` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 312 | 10.30% | 0.6701 | 0.2391 | 0.6226 | $9,315.08 |
| test | 71 | 13.63% | 0.5597 | 0.1853 | 0.4412 | $-231.04 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Mid-Wide Rate | Toxic Rate | Toxic PnL |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 105 | 0.2846 | $-531.17 | $9,598.93 | 25.71% | 18.10% | $-1,700.19 |
| 2 | 104 | 0.3371 | $475.81 | $7,488.82 | 22.12% | 13.46% | $-869.97 |
| 3 | 104 | 0.3711 | $-499.07 | $7,898.07 | 23.08% | 13.46% | $-928.47 |
| 4 | 104 | 0.4174 | $-331.63 | $7,005.64 | 27.88% | 13.46% | $-1,040.47 |
| 5 | 104 | 0.4861 | $655.02 | $6,273.05 | 29.81% | 9.62% | $-523.70 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and then applied to the final test window. Removing a negative-PnL bucket is only a diagnostic; it still needs a clean backtest implementation.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Toxic Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5395 | all | 909 | $925.00 | 9 | $55.12 | $-286.16 | 11.11% |
| 0.70 | 0.5395 | br2_high_skew_load | 334 | $-125.66 | 9 | $55.12 | $222.44 | 11.11% |
| 0.80 | 0.5684 | all | 606 | $-202.02 | 3 | $37.73 | $-268.77 | 0.00% |
| 0.80 | 0.5684 | br2_high_skew_load | 251 | $-136.34 | 3 | $37.73 | $239.83 | 0.00% |
| 0.90 | 0.6025 | all | 303 | $-246.20 | 1 | $8.33 | $-239.37 | 0.00% |
| 0.90 | 0.6025 | br2_high_skew_load | 143 | $-279.50 | 1 | $8.33 | $269.24 | 0.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Avg Risk |
|---|---:|---:|---:|---:|---:|
| br2_high_skew_load | 177 | $277.57 | $8,705.04 | 11.30% | 0.4096 |
| br2_late_confirm | 133 | $-219.06 | $16,094.47 | 15.04% | 0.3376 |
| br2_late_favourite_load | 211 | $-289.54 | $13,465.00 | 14.69% | 0.3796 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | -5.3557 |
| prior_market_range_3d | -2.2381 |
| prior_market_range_7d | -1.7195 |
| whipsaw_x_low_efficiency | -1.5762 |
| range_x_reversal | 1.5147 |
| side_model_p | 1.4898 |
| market_yes_range_so_far | 1.4614 |
| side_edge_vs_fill | 1.4190 |
| confidence_score | -1.3990 |
| prior_market_range_1d | -0.9333 |
| price | 0.7693 |
| range_x_sign_flip | 0.7212 |
| regime_reversal_pressure | 0.6753 |
| regime_path_efficiency | -0.6253 |
| whipsaw_x_reversal | -0.4748 |
| regime_whipsaw_score | -0.4658 |
| regime_sign_flip_rate | 0.4101 |
| buy_yes | -0.2649 |
| edge_x_confidence | -0.2238 |
| price_x_model_p | -0.2233 |
| risk_score | -0.1848 |
| regime_realized_vol_180s_bps | -0.0642 |
| prior1d_x_range | -0.0629 |
| tag:br2_high_skew_load | -0.0164 |

