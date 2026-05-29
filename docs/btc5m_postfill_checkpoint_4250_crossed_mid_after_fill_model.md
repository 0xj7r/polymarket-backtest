# BTC5m Post-Fill Reversal Model

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Target: `crossed_mid_after_fill`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1086` before `2026-03-09T04:15:00+00:00`
Test fills: `551` in final `5` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 390 | 35.91% | 0.6286 | 0.2195 | 0.6970 | $1,918.35 |
| test | 222 | 40.29% | 0.6661 | 0.2364 | 0.6332 | $663.02 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 111 | 0.2935 | $296.26 | $5,096.27 | 25.23% | 25.23% | 0.1788 | 0.8693 |
| 2 | 110 | 0.3995 | $292.20 | $4,562.70 | 31.82% | 31.82% | 0.2279 | 0.8346 |
| 3 | 110 | 0.4756 | $-78.94 | $4,518.81 | 42.73% | 42.73% | 0.2767 | 0.7530 |
| 4 | 110 | 0.5633 | $-300.98 | $3,739.14 | 50.00% | 50.00% | 0.2680 | 0.7392 |
| 5 | 110 | 0.7031 | $454.47 | $4,617.57 | 51.82% | 51.82% | 0.2486 | 0.7041 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5779 | all | 326 | $282.84 | 144 | $51.03 | $611.99 | 55.56% | 55.56% |
| 0.70 | 0.5779 | br2_high_skew_load | 43 | $-61.51 | 19 | $35.39 | $112.06 | 47.37% | 47.37% |
| 0.70 | 0.5779 | br2_late_confirm | 212 | $445.15 | 89 | $-26.97 | $76.94 | 60.67% | 60.67% |
| 0.70 | 0.5779 | br2_late_favourite_load | 71 | $-100.80 | 36 | $42.61 | $422.99 | 47.22% | 47.22% |
| 0.80 | 0.6333 | all | 218 | $99.53 | 100 | $443.36 | $219.65 | 53.00% | 53.00% |
| 0.80 | 0.6333 | br2_high_skew_load | 24 | $-57.27 | 13 | $18.06 | $129.38 | 46.15% | 46.15% |
| 0.80 | 0.6333 | br2_late_confirm | 160 | $253.73 | 71 | $351.20 | $-301.23 | 59.15% | 59.15% |
| 0.80 | 0.6333 | br2_late_favourite_load | 34 | $-96.93 | 16 | $74.10 | $391.50 | 31.25% | 31.25% |
| 0.90 | 0.7092 | all | 109 | $128.18 | 49 | $307.59 | $355.43 | 57.14% | 57.14% |
| 0.90 | 0.7092 | br2_high_skew_load | 8 | $6.73 | 6 | $17.57 | $129.87 | 50.00% | 50.00% |
| 0.90 | 0.7092 | br2_late_confirm | 91 | $125.32 | 40 | $290.85 | $-240.88 | 60.00% | 60.00% |
| 0.90 | 0.7092 | br2_late_favourite_load | 10 | $-3.87 | 3 | $-0.84 | $466.44 | 33.33% | 33.33% |
| 0.95 | 0.7627 | all | 55 | $-49.98 | 18 | $11.60 | $651.42 | 66.67% | 66.67% |
| 0.95 | 0.7627 | br2_high_skew_load | 4 | $5.43 | 2 | $-14.28 | $161.72 | 50.00% | 50.00% |
| 0.95 | 0.7627 | br2_late_confirm | 47 | $-42.40 | 14 | $44.01 | $5.96 | 71.43% | 71.43% |
| 0.95 | 0.7627 | br2_late_favourite_load | 4 | $-13.01 | 2 | $-18.14 | $483.74 | 50.00% | 50.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 139 | $147.45 | $2,659.31 | 39.57% | 39.57% | 0.2492 | 0.4535 |
| br2_late_confirm | 184 | $49.97 | $9,267.37 | 42.39% | 42.39% | 0.2263 | 0.5267 |
| br2_late_favourite_load | 228 | $465.60 | $10,607.81 | 39.04% | 39.04% | 0.2452 | 0.4745 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior_market_range_3d | -7.3218 |
| prior7d_minus_1d | 6.2726 |
| price | -3.0374 |
| side_model_p | -2.5296 |
| side_edge_vs_fill | 2.2781 |
| price_x_model_p | -1.9243 |
| risk_x_range | 1.7510 |
| whipsaw_x_low_efficiency | -1.5438 |
| prior_market_range_7d | 1.3010 |
| prior1d_x_range | 1.1908 |
| whipsaw_x_reversal | 1.1722 |
| range_x_sign_flip | -1.0330 |
| risk_score | 1.0174 |
| edge_x_confidence | 0.9085 |
| regime_whipsaw_score | -0.8730 |
| market_yes_range_so_far | 0.7682 |
| regime_reversal_pressure | 0.7337 |
| confidence_score | 0.4156 |
| range_x_reversal | 0.3808 |
| regime_sign_flip_rate | 0.2822 |
| prior_market_range_1d | 0.2728 |
| regime_realized_vol_180s_bps | -0.1231 |
| vol_x_reversal | 0.1094 |
| regime_path_efficiency | 0.0946 |
| tag:br2_high_skew_load | -0.0673 |
| tag:br2_late_confirm | 0.0285 |
| tag:br2_late_favourite_load | 0.0201 |
| seconds_to_close | 0.0069 |

