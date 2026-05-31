# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_watch_markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `3028` before `2026-04-20T17:20:00+00:00`
Test fills: `521` in final `30` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 615 | 20.31% | 0.6680 | 0.2373 | 0.6294 | $9,315.08 |
| test | 129 | 24.76% | 0.6706 | 0.2388 | 0.5427 | $-231.04 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 105 | 0.3494 | $-530.56 | $8,231.62 | 22.86% | 37.14% | 0.2751 | 0.7483 |
| 2 | 104 | 0.4110 | $90.55 | $7,393.25 | 26.92% | 38.46% | 0.2743 | 0.7439 |
| 3 | 104 | 0.4552 | $-99.54 | $7,044.77 | 20.19% | 39.42% | 0.2803 | 0.7638 |
| 4 | 104 | 0.5192 | $214.36 | $6,357.41 | 22.12% | 43.27% | 0.2723 | 0.7697 |
| 5 | 104 | 0.6238 | $94.14 | $9,237.46 | 31.73% | 64.42% | 0.3150 | 0.6752 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5320 | all | 909 | $2,547.73 | 134 | $-153.05 | $-77.99 | 32.09% | 62.69% |
| 0.70 | 0.5320 | br2_high_skew_load | 145 | $334.92 | 38 | $68.14 | $209.43 | 21.05% | 52.63% |
| 0.70 | 0.5320 | br2_late_confirm | 628 | $2,384.66 | 71 | $-270.56 | $51.49 | 39.44% | 73.24% |
| 0.70 | 0.5320 | br2_late_favourite_load | 136 | $-171.85 | 25 | $49.37 | $-338.91 | 28.00% | 48.00% |
| 0.80 | 0.5764 | all | 606 | $2,997.02 | 81 | $-258.62 | $27.58 | 37.04% | 70.37% |
| 0.80 | 0.5764 | br2_high_skew_load | 73 | $63.82 | 14 | $-119.82 | $397.39 | 35.71% | 57.14% |
| 0.80 | 0.5764 | br2_late_confirm | 491 | $2,829.14 | 57 | $-12.84 | $-206.23 | 36.84% | 75.44% |
| 0.80 | 0.5764 | br2_late_favourite_load | 42 | $104.06 | 10 | $-125.96 | $-163.58 | 40.00% | 60.00% |
| 0.90 | 0.6301 | all | 303 | $1,363.73 | 39 | $-399.62 | $168.58 | 41.03% | 71.79% |
| 0.90 | 0.6301 | br2_high_skew_load | 18 | $-147.77 | 3 | $-62.51 | $340.08 | 66.67% | 66.67% |
| 0.90 | 0.6301 | br2_late_confirm | 281 | $1,486.54 | 35 | $-338.15 | $119.09 | 40.00% | 74.29% |
| 0.90 | 0.6301 | br2_late_favourite_load | 4 | $24.95 | 1 | $1.04 | $-290.59 | 0.00% | 0.00% |
| 0.95 | 0.6713 | all | 152 | $416.59 | 17 | $283.43 | $-514.48 | 35.29% | 88.24% |
| 0.95 | 0.6713 | br2_high_skew_load | 9 | $-69.05 | 1 | $-42.19 | $319.75 | 100.00% | 100.00% |
| 0.95 | 0.6713 | br2_late_confirm | 142 | $483.03 | 16 | $325.62 | $-544.69 | 31.25% | 87.50% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 177 | $277.57 | $8,705.04 | 18.08% | 36.16% | 0.2622 | 0.4554 |
| br2_late_confirm | 133 | $-219.06 | $16,094.47 | 30.83% | 56.39% | 0.2894 | 0.5344 |
| br2_late_favourite_load | 211 | $-289.54 | $13,465.00 | 26.54% | 44.08% | 0.2974 | 0.4453 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| side_edge_vs_fill | 1.8056 |
| prior7d_minus_1d | -1.6650 |
| prior_market_range_3d | -1.4292 |
| market_yes_range_so_far | 1.4259 |
| price | -1.4172 |
| risk_x_range | 1.3594 |
| price_x_model_p | -1.3167 |
| edge_x_confidence | -1.1309 |
| regime_sign_flip_rate | 1.1193 |
| side_model_p | -1.0268 |
| regime_path_efficiency | -0.8170 |
| whipsaw_x_low_efficiency | -0.7822 |
| range_x_reversal | -0.7107 |
| risk_score | 0.6718 |
| prior1d_x_range | 0.5646 |
| confidence_score | -0.4751 |
| range_x_sign_flip | 0.3854 |
| regime_whipsaw_score | -0.3590 |
| whipsaw_x_reversal | 0.2414 |
| regime_reversal_pressure | 0.2351 |
| prior_market_range_1d | 0.2306 |
| buy_yes | -0.1617 |
| tag:br2_late_confirm | 0.1224 |
| regime_realized_vol_180s_bps | -0.1061 |
| vol_x_reversal | 0.0849 |
| tag:br2_high_skew_load | -0.0663 |
| tag:br2_late_favourite_load | -0.0629 |
| prior_market_range_7d | 0.0592 |

