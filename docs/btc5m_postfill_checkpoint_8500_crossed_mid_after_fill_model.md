# BTC5m Post-Fill Reversal Model

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `crossed_mid_after_fill`. Features are fill-time/replay-safe only; the target uses post-fill path labels for offline diagnosis.
Train fills: `1874` before `2026-03-19T01:20:00+00:00`
Test fills: `429` in final `10` days

## Model Quality

| Split | Positives | Base Rate | Log Loss | Brier | AUC | PnL |
|---|---:|---:|---:|---:|---:|---:|
| train | 694 | 37.03% | 0.6437 | 0.2263 | 0.6749 | $4,013.42 |
| test | 181 | 42.19% | 0.6655 | 0.2366 | 0.6242 | $1,509.51 |

## Test Risk Buckets

| Risk Bucket | Fills | Avg Risk | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Final Side Mid |
|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 86 | 0.2433 | $289.19 | $5,597.47 | 25.58% | 25.58% | 0.1916 | 0.8366 |
| 2 | 86 | 0.3695 | $416.19 | $5,919.58 | 33.72% | 33.72% | 0.1992 | 0.8391 |
| 3 | 86 | 0.4613 | $-50.54 | $5,266.45 | 50.00% | 50.00% | 0.2887 | 0.7445 |
| 4 | 86 | 0.5444 | $73.04 | $4,952.38 | 53.49% | 53.49% | 0.2924 | 0.7541 |
| 5 | 85 | 0.6874 | $781.63 | $4,670.54 | 48.24% | 48.24% | 0.2489 | 0.7144 |

## Candidate Removal Diagnostics

Thresholds are fitted from train risk quantiles and applied to the final test window. Positive removed PnL means the gate would have removed good trades, so only negative removed PnL is interesting.

| Train Quantile | Threshold | Lane | Train Removed | Train Removed PnL | Test Removed | Test Removed PnL | Test Kept PnL | Removed Target Rate | Removed Cross Rate |
|---:|---:|---|---:|---:|---:|---:|---:|---:|---:|
| 0.70 | 0.5670 | all | 562 | $567.22 | 108 | $742.16 | $767.35 | 50.00% | 50.00% |
| 0.70 | 0.5670 | br2_high_skew_load | 84 | $154.90 | 25 | $-128.57 | $63.55 | 56.00% | 56.00% |
| 0.70 | 0.5670 | br2_late_confirm | 344 | $440.34 | 45 | $661.47 | $173.44 | 51.11% | 51.11% |
| 0.70 | 0.5670 | br2_late_favourite_load | 134 | $-28.02 | 38 | $209.25 | $530.36 | 44.74% | 44.74% |
| 0.80 | 0.6241 | all | 375 | $211.83 | 69 | $634.44 | $875.07 | 52.17% | 52.17% |
| 0.80 | 0.6241 | br2_high_skew_load | 51 | $48.15 | 13 | $-116.47 | $51.45 | 61.54% | 61.54% |
| 0.80 | 0.6241 | br2_late_confirm | 258 | $161.86 | 36 | $686.00 | $148.91 | 52.78% | 52.78% |
| 0.80 | 0.6241 | br2_late_favourite_load | 66 | $1.82 | 20 | $64.91 | $674.71 | 45.00% | 45.00% |
| 0.90 | 0.6811 | all | 188 | $684.50 | 43 | $419.91 | $1,089.59 | 58.14% | 58.14% |
| 0.90 | 0.6811 | br2_high_skew_load | 20 | $-58.80 | 6 | $17.66 | $-82.68 | 50.00% | 50.00% |
| 0.90 | 0.6811 | br2_late_confirm | 148 | $748.61 | 30 | $365.04 | $469.87 | 60.00% | 60.00% |
| 0.90 | 0.6811 | br2_late_favourite_load | 20 | $-5.32 | 7 | $37.21 | $702.41 | 57.14% | 57.14% |
| 0.95 | 0.7272 | all | 94 | $418.16 | 25 | $127.87 | $1,381.64 | 68.00% | 68.00% |
| 0.95 | 0.7272 | br2_high_skew_load | 11 | $-38.22 | 1 | $15.77 | $-80.79 | 0.00% | 0.00% |
| 0.95 | 0.7272 | br2_late_confirm | 74 | $421.38 | 21 | $78.25 | $756.66 | 71.43% | 71.43% |
| 0.95 | 0.7272 | br2_late_favourite_load | 9 | $35.00 | 3 | $33.85 | $705.77 | 66.67% | 66.67% |

## Test By Lane

| Lane | Fills | PnL | Cost | Target Rate | Cross-Mid Rate | Avg Adverse | Avg Risk |
|---|---:|---:|---:|---:|---:|---:|---:|
| br2_high_skew_load | 137 | $-65.01 | $4,352.36 | 44.53% | 44.53% | 0.2756 | 0.4466 |
| br2_late_confirm | 119 | $834.91 | $9,844.05 | 41.18% | 41.18% | 0.2000 | 0.4891 |
| br2_late_favourite_load | 173 | $739.61 | $12,210.02 | 41.04% | 41.04% | 0.2497 | 0.4523 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| prior_market_range_3d | -5.2910 |
| prior7d_minus_1d | 4.8340 |
| side_model_p | -2.6266 |
| price | -2.4621 |
| prior1d_x_range | 2.2128 |
| edge_x_confidence | -1.9857 |
| price_x_model_p | -1.9672 |
| market_yes_range_so_far | 1.7426 |
| risk_score | 1.4087 |
| regime_path_efficiency | -1.4063 |
| side_edge_vs_fill | 1.2278 |
| range_x_sign_flip | -1.1989 |
| whipsaw_x_low_efficiency | -0.9528 |
| risk_x_range | 0.9158 |
| regime_reversal_pressure | 0.8169 |
| confidence_score | 0.7496 |
| prior_market_range_7d | 0.7405 |
| regime_sign_flip_rate | 0.7206 |
| whipsaw_x_reversal | 0.5644 |
| regime_whipsaw_score | -0.4860 |
| prior_market_range_1d | -0.3293 |
| range_x_reversal | 0.1404 |
| regime_realized_vol_180s_bps | -0.1257 |
| buy_yes | 0.0863 |
| tag:br2_high_skew_load | -0.0828 |
| tag:br2_late_favourite_load | 0.0737 |
| tag:br2_late_confirm | -0.0119 |
| vol_x_reversal | -0.0103 |
