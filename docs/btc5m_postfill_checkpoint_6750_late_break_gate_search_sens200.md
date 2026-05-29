# BTC5m Late-Break Walk-Forward Gate Search

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Late fills: `1548`
Late-fill PnL: `$3,913.94`
Toxic late fills: `322` (`20.80%`)
Min train fills: `600`
Test fills per fold: `200`
Step fills: `200`

Candidate thresholds are computed from each fold's training fills only. A candidate is admitted in a fold only when the same train-side rule removed at least the configured minimum fills and had negative train PnL.

## Candidate Outcomes

| Candidate | Folds | Active Folds | Helpful Folds | Harmful Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Worst Fold Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Toxic Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| side_model_p:q1&side_edge_vs_fill:q2 | 2 | 2 | 2 | 0 | 400 | 29 | $1,423.02 | $-461.94 | $-54.09 | $858.13 | $461.94 | $230.97 | 55.17% | 68.97% |
| price:q1&side_edge_vs_fill:q2 | 1 | 1 | 1 | 0 | 200 | 20 | $924.51 | $-407.85 | $-407.85 | $877.14 | $407.85 | $203.93 | 65.00% | 75.00% |
| side_edge_vs_fill:q2&prior_market_range_7d:q1 | 2 | 1 | 1 | 0 | 400 | 48 | $1,977.29 | $-320.44 | $-320.44 | $1,642.78 | $320.44 | $160.22 | 37.50% | 43.75% |
| side_edge_vs_fill:q1&prior_market_range_7d:q1 | 4 | 3 | 3 | 0 | 800 | 115 | $5,775.40 | $-299.02 | $-29.41 | $2,190.23 | $299.02 | $149.51 | 26.09% | 35.65% |
| risk_score:q3&regime_reversal_pressure:q2 | 2 | 2 | 2 | 0 | 400 | 22 | $936.00 | $-278.16 | $-134.31 | $674.35 | $278.16 | $139.08 | 45.45% | 54.55% |
| side_edge_vs_fill:q4&prior_market_range_3d:q2 | 2 | 1 | 1 | 0 | 400 | 17 | $719.66 | $-110.77 | $-110.77 | $506.95 | $110.77 | $55.38 | 41.18% | 76.47% |
| regime_reversal_pressure:q2&prior_market_range_7d:q2 | 2 | 1 | 1 | 0 | 400 | 4 | $198.86 | $-99.39 | $-99.39 | $495.58 | $99.39 | $49.70 | 50.00% | 75.00% |
| side_edge_vs_fill:q2&regime_reversal_pressure:q2 | 1 | 1 | 1 | 0 | 200 | 7 | $311.71 | $-98.63 | $-98.63 | $567.92 | $98.63 | $49.31 | 42.86% | 42.86% |
| regime_reversal_pressure:q2&prior_market_range_3d:q2 | 2 | 1 | 1 | 0 | 400 | 24 | $988.52 | $-64.61 | $-64.61 | $460.80 | $64.61 | $32.31 | 29.17% | 54.17% |
| side_edge_vs_fill:q4&regime_reversal_pressure:q2 | 1 | 1 | 1 | 0 | 200 | 9 | $406.20 | $-60.34 | $-60.34 | $529.63 | $60.34 | $30.17 | 44.44% | 77.78% |
| side_model_p:q4&regime_reversal_pressure:q2 | 2 | 2 | 2 | 0 | 400 | 9 | $609.05 | $-55.76 | $-21.09 | $451.95 | $55.76 | $27.88 | 33.33% | 44.44% |
| side_model_p:q3&side_edge_vs_fill:q4 | 3 | 3 | 3 | 0 | 600 | 44 | $1,576.74 | $-54.51 | $-3.72 | $2,018.82 | $54.51 | $27.26 | 27.27% | 54.55% |
| side_model_p:q3&regime_reversal_pressure:q4 | 1 | 1 | 1 | 0 | 200 | 10 | $477.50 | $-40.27 | $-40.27 | $-32.83 | $40.27 | $20.13 | 30.00% | 40.00% |
| regime_reversal_pressure:q2&prior_market_range_7d:q1 | 1 | 1 | 1 | 0 | 200 | 41 | $1,858.74 | $-39.65 | $-39.65 | $681.62 | $39.65 | $19.83 | 24.39% | 53.66% |
| side_edge_vs_fill:q4&prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 200 | 7 | $296.60 | $-38.19 | $-38.19 | $507.48 | $38.19 | $19.10 | 28.57% | 42.86% |
| side_model_p:q1&risk_score:q1 | 1 | 1 | 1 | 0 | 200 | 5 | $226.05 | $-26.62 | $-26.62 | $668.59 | $26.62 | $13.31 | 40.00% | 60.00% |
| late_confirm:prior_market_range_7d:q1 | 2 | 1 | 1 | 0 | 400 | 88 | $4,119.15 | $-26.54 | $-26.54 | $1,348.88 | $26.54 | $13.27 | 35.23% | 46.59% |
| risk_score:q4&regime_reversal_pressure:q1 | 1 | 1 | 1 | 0 | 200 | 25 | $1,089.13 | $-26.31 | $-26.31 | $495.60 | $26.31 | $13.16 | 28.00% | 40.00% |
| risk_score:q3&prior_market_range_3d:q1 | 1 | 1 | 1 | 0 | 200 | 29 | $1,183.31 | $-14.05 | $-14.05 | $483.34 | $14.05 | $7.03 | 27.59% | 37.93% |
| side_edge_vs_fill:q4&risk_score:q3 | 4 | 4 | 3 | 1 | 800 | 41 | $1,310.96 | $-172.09 | $60.17 | $2,063.30 | $172.09 | $86.04 | 39.02% | 51.22% |
| side_model_p:q4&side_edge_vs_fill:q3 | 3 | 3 | 2 | 1 | 600 | 35 | $2,311.67 | $-115.22 | $108.93 | $1,153.38 | $115.22 | $57.61 | 20.00% | 28.57% |
| regime_reversal_pressure:q2&prior_market_range_3d:q1 | 2 | 2 | 1 | 1 | 400 | 56 | $2,571.63 | $-111.41 | $154.82 | $1,222.67 | $111.41 | $55.71 | 26.79% | 53.57% |
| risk_score:q4&prior_market_range_7d:q1 | 3 | 2 | 1 | 1 | 600 | 86 | $3,612.61 | $-109.42 | $169.92 | $2,073.73 | $109.42 | $54.71 | 31.40% | 48.84% |
| risk_score:q4&prior_market_range_3d:q1 | 2 | 2 | 1 | 1 | 400 | 73 | $2,790.67 | $-78.66 | $131.85 | $1,401.01 | $78.66 | $39.33 | 28.77% | 47.95% |
| price:q3&side_model_p:q4 | 2 | 2 | 1 | 1 | 400 | 19 | $1,006.01 | $-43.73 | $94.51 | $439.92 | $43.73 | $21.87 | 26.32% | 36.84% |
| price:q4&risk_score:q3 | 2 | 2 | 1 | 1 | 400 | 22 | $910.52 | $-10.59 | $53.02 | $1,332.93 | $10.59 | $5.30 | 18.18% | 18.18% |
| side_edge_vs_fill:q4&prior_market_range_3d:q1 | 2 | 2 | 1 | 1 | 400 | 34 | $1,073.54 | $-1.71 | $20.10 | $1,496.73 | $1.71 | $0.86 | 29.41% | 52.94% |
| risk_score:q2&prior_market_range_3d:q2 | 1 | 1 | 0 | 1 | 200 | 24 | $1,079.63 | $2.18 | $2.18 | $467.11 | $-2.18 | $-1.09 | 25.00% | 58.33% |
| price:q4&prior_market_range_3d:q3 | 1 | 1 | 0 | 1 | 200 | 1 | $28.30 | $2.59 | $2.59 | $850.47 | $-2.59 | $-1.29 | 0.00% | 0.00% |
| price:q3&prior_market_range_7d:q2 | 3 | 2 | 1 | 1 | 600 | 41 | $1,460.97 | $4.09 | $114.00 | $1,245.15 | $-4.09 | $-2.05 | 21.95% | 36.59% |
| risk_score:q2&regime_reversal_pressure:q3 | 2 | 2 | 1 | 1 | 400 | 10 | $287.27 | $9.29 | $15.94 | $1,485.73 | $-9.29 | $-4.64 | 30.00% | 50.00% |
| price:q4&side_model_p:q2 | 1 | 1 | 0 | 1 | 200 | 1 | $47.11 | $11.69 | $11.69 | $630.28 | $-11.69 | $-5.84 | 0.00% | 0.00% |

## Folds

| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Toxic Rate |
|---:|---:|---:|---|---|---:|---:|
| 1 | 600 | 200 | 2026-03-05 | 2026-03-08 | $853.05 | 17.50% |
| 2 | 800 | 200 | 2026-03-08 | 2026-03-10 | $641.97 | 17.50% |
| 3 | 1000 | 200 | 2026-03-10 | 2026-03-13 | $-73.10 | 24.50% |
| 4 | 1200 | 200 | 2026-03-13 | 2026-03-17 | $469.29 | 25.50% |
