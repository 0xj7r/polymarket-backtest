# BTC5m Mid-Wide Regime Model

Source: `s3://pm-research-backtest-prod/results/20260528T225810Z-portfolio-grid-52322/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8/markets.jsonl`
Target: `midwide` using replay-safe fill-time features only.
Train fills: `3028` before `2026-04-20T17:20:00+00:00`
Test fills: `521` in final `30` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 764 | 25.23% | 0.6030 | 0.2070 | 0.7406 | $9,315.08 |
| test | 134 | 25.72% | 0.5873 | 0.1987 | 0.6587 | $-231.04 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Mid-Wide Rate | Toxic Rate | Toxic PnL |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 105 | 0.1338 | $-50.57 | $8,377.14 | 18.10% | 13.33% | $-805.81 |
| 2 | 104 | 0.2283 | $219.32 | $8,239.09 | 10.58% | 8.65% | $-861.25 |
| 3 | 104 | 0.3330 | $-1,274.96 | $8,854.15 | 23.08% | 20.19% | $-1,973.76 |
| 4 | 104 | 0.4576 | $432.22 | $7,630.52 | 34.62% | 17.31% | $-888.87 |
| 5 | 104 | 0.6587 | $442.96 | $5,163.61 | 42.31% | 8.65% | $-533.10 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and then applied to the final test window. Removing a negative-PnL bucket is only a diagnostic; it still needs a clean backtest implementation.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Toxic Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5678 | all | 909 | $783.05 | 82 | $136.65 | $-367.69 | 9.76% |
| 0.70 | 0.5678 | br2_high_skew_load | 334 | $-340.81 | 58 | $161.32 | $116.25 | 6.90% |
| 0.70 | 0.5678 | br2_late_confirm | 238 | $821.68 | 2 | $63.94 | $-283.01 | 0.00% |
| 0.70 | 0.5678 | br2_late_favourite_load | 337 | $302.18 | 22 | $-88.61 | $-200.94 | 18.18% |
| 0.80 | 0.6282 | all | 606 | $422.61 | 59 | $212.48 | $-443.52 | 6.78% |
| 0.80 | 0.6282 | br2_high_skew_load | 271 | $-290.00 | 45 | $121.04 | $156.53 | 6.67% |
| 0.80 | 0.6282 | br2_late_favourite_load | 202 | $139.96 | 14 | $91.44 | $-380.99 | 7.14% |
| 0.90 | 0.7024 | all | 303 | $95.72 | 27 | $119.77 | $-350.81 | 7.41% |
| 0.90 | 0.7024 | br2_high_skew_load | 178 | $-284.82 | 25 | $117.54 | $160.03 | 8.00% |
| 0.90 | 0.7024 | br2_late_favourite_load | 96 | $249.46 | 2 | $2.23 | $-291.77 | 0.00% |
| 0.95 | 0.7642 | all | 152 | $-200.34 | 16 | $89.33 | $-320.38 | 6.25% |
| 0.95 | 0.7642 | br2_high_skew_load | 116 | $-255.69 | 16 | $89.33 | $188.23 | 6.25% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Avg Risk |
|---|---:|---:|---:|---:|---:|
| br2_high_skew_load | 177 | $277.57 | $8,705.04 | 28.25% | 0.4286 |
| br2_late_confirm | 133 | $-219.06 | $16,094.47 | 22.56% | 0.3075 |
| br2_late_favourite_load | 211 | $-289.54 | $13,465.00 | 25.59% | 0.3401 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| market_yes_range_so_far | 4.2051 |
| range_x_sign_flip | 3.6944 |
| prior1d_x_range | 2.8960 |
| prior_market_range_3d | -2.6455 |
| prior_market_range_7d | -2.3806 |
| whipsaw_x_reversal | -2.1201 |
| range_x_reversal | 1.9926 |
| prior_market_range_1d | -1.9915 |
| side_edge_vs_fill | 1.5500 |
| price | -1.5275 |
| side_model_p | -1.2607 |
| whipsaw_x_low_efficiency | -1.2450 |
| price_x_model_p | -1.1717 |
| prior7d_minus_1d | -0.8971 |
| confidence_score | -0.8813 |
| risk_score | -0.4564 |
| regime_reversal_pressure | -0.4298 |
| edge_x_confidence | -0.4049 |
| regime_whipsaw_score | -0.2160 |
| regime_sign_flip_rate | -0.2096 |
| tag:br2_high_skew_load | -0.1052 |
| buy_yes | 0.0845 |
| tag:br2_late_confirm | 0.0454 |
| tag:br2_late_favourite_load | 0.0422 |

