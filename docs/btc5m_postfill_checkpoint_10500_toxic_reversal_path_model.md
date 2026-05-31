# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_markets_062901.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `799` before `2026-03-05T15:25:00+00:00`
Test fills: `1699` in final `30` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 156 | 19.52% | 0.6538 | 0.2309 | 0.6588 | $1,021.42 |
| test | 353 | 20.78% | 0.7013 | 0.2478 | 0.5734 | $5,334.96 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 340 | 0.1828 | $1,470.97 | $20,334.13 | 15.29% | 27.94% | 0.1987 | 0.8349 |
| 2 | 340 | 0.4007 | $453.36 | $13,388.06 | 18.24% | 35.59% | 0.2292 | 0.8013 |
| 3 | 340 | 0.4772 | $148.25 | $14,228.85 | 21.18% | 41.47% | 0.2574 | 0.7640 |
| 4 | 340 | 0.5497 | $1,681.08 | $15,267.96 | 19.12% | 38.53% | 0.2234 | 0.7808 |
| 5 | 339 | 0.6809 | $1,581.31 | $17,072.86 | 30.09% | 53.39% | 0.2539 | 0.6985 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5363 | all | 240 | $-36.14 | 564 | $2,597.35 | $2,737.61 | 25.71% | 47.34% |
| 0.70 | 0.5363 | br2_high_skew_load | 31 | $-16.06 | 88 | $409.66 | $133.25 | 20.45% | 36.36% |
| 0.70 | 0.5363 | br2_late_confirm | 124 | $153.67 | 265 | $1,334.36 | $492.12 | 32.45% | 54.72% |
| 0.70 | 0.5363 | br2_late_favourite_load | 85 | $-173.75 | 211 | $853.33 | $2,112.24 | 19.43% | 42.65% |
| 0.80 | 0.5793 | all | 160 | $59.41 | 387 | $1,564.62 | $3,770.34 | 29.72% | 52.20% |
| 0.80 | 0.5793 | br2_high_skew_load | 14 | $9.36 | 57 | $411.61 | $131.30 | 19.30% | 40.35% |
| 0.80 | 0.5793 | br2_late_confirm | 94 | $156.68 | 212 | $1,103.29 | $723.19 | 34.91% | 58.02% |
| 0.80 | 0.5793 | br2_late_favourite_load | 52 | $-106.64 | 118 | $49.72 | $2,915.85 | 25.42% | 47.46% |
| 0.90 | 0.6362 | all | 80 | $-51.19 | 208 | $1,331.19 | $4,003.78 | 29.33% | 51.92% |
| 0.90 | 0.6362 | br2_high_skew_load | 6 | $-2.07 | 35 | $441.46 | $101.45 | 11.43% | 34.29% |
| 0.90 | 0.6362 | br2_late_confirm | 55 | $-40.83 | 122 | $886.19 | $940.29 | 34.43% | 59.84% |
| 0.90 | 0.6362 | br2_late_favourite_load | 19 | $-8.30 | 51 | $3.53 | $2,962.04 | 29.41% | 45.10% |
| 0.95 | 0.6740 | all | 40 | $-95.94 | 131 | $1,547.89 | $3,787.07 | 25.19% | 46.56% |
| 0.95 | 0.6740 | br2_high_skew_load | 2 | $-1.75 | 24 | $418.10 | $124.81 | 8.33% | 33.33% |
| 0.95 | 0.6740 | br2_late_confirm | 28 | $-99.15 | 78 | $1,073.29 | $753.19 | 32.05% | 53.85% |
| 0.95 | 0.6740 | br2_late_favourite_load | 10 | $4.96 | 29 | $56.51 | $2,909.07 | 20.69% | 37.93% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 450 | $542.91 | $11,082.29 | 20.89% | 38.67% | 0.2507 | 0.4215 |
| br2_late_confirm | 592 | $1,826.48 | $34,053.65 | 25.17% | 43.24% | 0.2248 | 0.4818 |
| br2_late_favourite_load | 657 | $2,965.58 | $35,155.92 | 16.74% | 36.38% | 0.2271 | 0.4619 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | -17.1317 |
| prior_market_range_3d | -10.8842 |
| prior_market_range_7d | -4.5586 |
| range_x_reversal | -3.4728 |
| side_edge_vs_fill | 3.0514 |
| prior_market_range_1d | -2.8473 |
| price | -1.8307 |
| regime_sign_flip_rate | 1.4880 |
| confidence_score | -1.3641 |
| regime_path_efficiency | 1.1741 |
| whipsaw_x_reversal | 1.1466 |
| edge_x_confidence | 1.1142 |
| range_x_sign_flip | -1.1070 |
| side_model_p | -0.9443 |
| prior1d_x_range | 0.9239 |
| price_x_model_p | -0.8441 |
| market_yes_range_so_far | 0.7254 |
| regime_reversal_pressure | 0.6789 |
| risk_score | -0.4919 |
| risk_x_range | 0.4875 |
| regime_whipsaw_score | -0.3786 |
| whipsaw_x_low_efficiency | -0.3371 |
| buy_yes | -0.2308 |
| tag:br2_high_skew_load | -0.1836 |
| tag:br2_late_confirm | 0.1495 |
| regime_realized_vol_180s_bps | -0.1135 |
| vol_x_reversal | 0.0909 |
| tag:br2_late_favourite_load | -0.0068 |

