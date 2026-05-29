# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `toxic_reversal_path`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1874` before `2026-03-19T01:20:00+00:00`
Test fills: `429` in final `10` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 377 | 20.12% | 0.6614 | 0.2345 | 0.6402 | $4,013.42 |
| test | 91 | 21.21% | 0.7471 | 0.2746 | 0.5994 | $1,509.51 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 86 | 0.3743 | $712.73 | $5,812.06 | 11.63% | 23.26% | 0.1690 | 0.8527 |
| 2 | 86 | 0.4731 | $228.69 | $5,380.01 | 19.77% | 45.35% | 0.2692 | 0.7966 |
| 3 | 86 | 0.5339 | $188.86 | $4,855.21 | 19.77% | 41.86% | 0.2503 | 0.7930 |
| 4 | 86 | 0.6087 | $104.39 | $4,500.90 | 25.58% | 41.86% | 0.2422 | 0.7463 |
| 5 | 85 | 0.7108 | $274.84 | $5,858.25 | 29.41% | 58.82% | 0.2907 | 0.7000 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5395 | all | 562 | $1,280.81 | 205 | $684.06 | $825.45 | 24.88% | 46.34% |
| 0.70 | 0.5395 | br2_high_skew_load | 75 | $228.69 | 57 | $100.40 | $-165.42 | 22.81% | 38.60% |
| 0.70 | 0.5395 | br2_late_confirm | 340 | $906.98 | 70 | $413.62 | $421.29 | 31.43% | 55.71% |
| 0.70 | 0.5395 | br2_late_favourite_load | 147 | $145.14 | 78 | $170.04 | $569.57 | 20.51% | 43.59% |
| 0.80 | 0.5847 | all | 375 | $936.82 | 151 | $161.42 | $1,348.09 | 28.48% | 50.99% |
| 0.80 | 0.5847 | br2_high_skew_load | 41 | $126.64 | 34 | $39.73 | $-104.75 | 26.47% | 47.06% |
| 0.80 | 0.5847 | br2_late_confirm | 262 | $845.68 | 62 | $107.71 | $727.20 | 35.48% | 59.68% |
| 0.80 | 0.5847 | br2_late_favourite_load | 72 | $-35.49 | 55 | $13.98 | $725.64 | 21.82% | 43.64% |
| 0.90 | 0.6358 | all | 188 | $533.05 | 100 | $346.17 | $1,163.34 | 29.00% | 56.00% |
| 0.90 | 0.6358 | br2_high_skew_load | 15 | $159.68 | 20 | $-5.32 | $-59.69 | 25.00% | 55.00% |
| 0.90 | 0.6358 | br2_late_confirm | 146 | $310.99 | 54 | $318.13 | $516.78 | 33.33% | 61.11% |
| 0.90 | 0.6358 | br2_late_favourite_load | 27 | $62.38 | 26 | $33.36 | $706.25 | 23.08% | 46.15% |
| 0.95 | 0.6733 | all | 94 | $-190.70 | 69 | $82.97 | $1,426.54 | 31.88% | 59.42% |
| 0.95 | 0.6733 | br2_high_skew_load | 5 | $-47.50 | 14 | $31.93 | $-96.94 | 21.43% | 64.29% |
| 0.95 | 0.6733 | br2_late_confirm | 83 | $-157.29 | 40 | $78.19 | $756.72 | 37.50% | 65.00% |
| 0.95 | 0.6733 | br2_late_favourite_load | 6 | $14.08 | 15 | $-27.15 | $766.76 | 26.67% | 40.00% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 137 | $-65.01 | $4,352.36 | 25.55% | 44.53% | 0.2756 | 0.5154 |
| br2_late_confirm | 119 | $834.91 | $9,844.05 | 22.69% | 41.18% | 0.2000 | 0.5770 |
| br2_late_favourite_load | 173 | $739.61 | $12,210.02 | 16.76% | 41.04% | 0.2497 | 0.5334 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior7d_minus_1d | -11.7376 |
| prior_market_range_3d | -6.1434 |
| prior_market_range_7d | -4.6423 |
| side_edge_vs_fill | 1.9862 |
| prior1d_x_range | 1.7368 |
| price | -1.6665 |
| risk_score | 1.5850 |
| market_yes_range_so_far | 1.4927 |
| prior_market_range_1d | -1.3746 |
| regime_sign_flip_rate | 1.3232 |
| side_model_p | -1.2906 |
| price_x_model_p | -1.1967 |
| edge_x_confidence | -0.9042 |
| range_x_reversal | -0.8151 |
| regime_reversal_pressure | 0.7894 |
| whipsaw_x_low_efficiency | -0.7233 |
| regime_path_efficiency | -0.6020 |
| risk_x_range | 0.3668 |
| whipsaw_x_reversal | -0.3632 |
| buy_yes | -0.2019 |
| range_x_sign_flip | -0.1763 |
| tag:br2_high_skew_load | -0.1389 |
| regime_realized_vol_180s_bps | -0.1362 |
| regime_whipsaw_score | -0.1133 |
| tag:br2_late_confirm | 0.0861 |
| vol_x_reversal | -0.0775 |
| confidence_score | -0.0583 |
| tag:br2_late_favourite_load | 0.0206 |
