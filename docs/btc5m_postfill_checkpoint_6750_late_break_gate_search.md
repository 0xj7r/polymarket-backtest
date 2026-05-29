# BTC5m Late-Break Walk-Forward Gate Search

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Late fills: `1548`
Late-fill PnL: `$3,913.94`
Toxic late fills: `322` (`20.80%`)
Min train fills: `900`
Test fills per fold: `300`
Step fills: `300`

Candidate thresholds are computed from each fold's training fills only. A candidate is admitted in a fold only when the same train-side rule removed at least the configured minimum fills and had negative train PnL.

## Candidate Outcomes

| Candidate | Folds | Active Folds | Helpful Folds | Harmful Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Worst Fold Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Toxic Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| side_model_p:q1&side_edge_vs_fill:q2 | 1 | 1 | 1 | 0 | 300 | 29 | $1,555.21 | $-249.96 | $-249.96 | $1,823.38 | $249.96 | $124.98 | 51.72% | 62.07% |
| risk_score:q4&prior_market_range_7d:q1 | 2 | 2 | 2 | 0 | 600 | 171 | $8,057.96 | $-236.11 | $-105.86 | $1,807.22 | $236.11 | $118.05 | 29.82% | 45.03% |
| price:q1&side_edge_vs_fill:q2 | 1 | 1 | 1 | 0 | 300 | 28 | $1,479.35 | $-174.10 | $-174.10 | $1,747.52 | $174.10 | $87.05 | 50.00% | 60.71% |
| price:q3&side_edge_vs_fill:q4 | 1 | 1 | 1 | 0 | 300 | 15 | $806.16 | $-144.00 | $-144.00 | $1,717.42 | $144.00 | $72.00 | 20.00% | 33.33% |
| side_edge_vs_fill:q4&prior_market_range_3d:q2 | 2 | 2 | 2 | 0 | 600 | 30 | $1,087.10 | $-127.46 | $-16.69 | $1,698.57 | $127.46 | $63.73 | 36.67% | 73.33% |
| side_edge_vs_fill:q4&regime_reversal_pressure:q4 | 2 | 2 | 2 | 0 | 600 | 23 | $1,056.09 | $-122.26 | $-44.60 | $1,693.36 | $122.26 | $61.13 | 34.78% | 56.52% |
| price:q3&prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 300 | 6 | $231.24 | $-109.91 | $-109.91 | $1,683.33 | $109.91 | $54.95 | 50.00% | 50.00% |
| price:q4&risk_score:q3 | 2 | 2 | 2 | 0 | 600 | 22 | $1,196.75 | $-108.57 | $-9.23 | $1,679.68 | $108.57 | $54.28 | 22.73% | 36.36% |
| late_fav:prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 300 | 6 | $246.79 | $-108.56 | $-108.56 | $1,681.98 | $108.56 | $54.28 | 50.00% | 50.00% |
| regime_reversal_pressure:q2&prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 300 | 4 | $198.86 | $-99.39 | $-99.39 | $1,672.82 | $99.39 | $49.70 | 50.00% | 75.00% |
| price:q3&regime_reversal_pressure:q4 | 2 | 2 | 2 | 0 | 600 | 25 | $1,322.69 | $-98.20 | $-6.30 | $1,669.31 | $98.20 | $49.10 | 32.00% | 52.00% |
| side_model_p:q3&side_edge_vs_fill:q4 | 1 | 1 | 1 | 0 | 300 | 20 | $862.46 | $-93.63 | $-93.63 | $1,667.06 | $93.63 | $46.82 | 30.00% | 50.00% |
| regime_reversal_pressure:q2&prior_market_range_3d:q2 | 1 | 1 | 1 | 0 | 300 | 24 | $988.52 | $-64.61 | $-64.61 | $1,638.04 | $64.61 | $32.31 | 29.17% | 54.17% |
| prior_market_range_3d:q1&prior_market_range_7d:q1 | 1 | 1 | 1 | 0 | 300 | 258 | $13,064.31 | $-63.23 | $-63.23 | $60.92 | $63.23 | $31.62 | 24.03% | 40.31% |
| side_edge_vs_fill:q4&risk_score:q3 | 1 | 1 | 1 | 0 | 300 | 9 | $360.71 | $-42.63 | $-42.63 | $1,616.05 | $42.63 | $21.31 | 55.56% | 66.67% |
| side_model_p:q4&regime_reversal_pressure:q2 | 2 | 2 | 2 | 0 | 600 | 32 | $2,023.34 | $-42.30 | $-9.86 | $1,613.41 | $42.30 | $21.15 | 18.75% | 40.62% |
| side_edge_vs_fill:q2&regime_reversal_pressure:q2 | 1 | 1 | 1 | 0 | 300 | 14 | $720.18 | $-39.59 | $-39.59 | $1,613.01 | $39.59 | $19.80 | 28.57% | 42.86% |
| side_edge_vs_fill:q4&prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 300 | 7 | $296.60 | $-38.19 | $-38.19 | $1,611.62 | $38.19 | $19.10 | 28.57% | 42.86% |
| risk_score:q4&prior_market_range_3d:q1 | 1 | 1 | 1 | 0 | 300 | 62 | $3,066.50 | $-37.04 | $-37.04 | $1,610.46 | $37.04 | $18.52 | 30.65% | 41.94% |
| regime_reversal_pressure:q2&prior_market_range_3d:q1 | 1 | 1 | 1 | 0 | 300 | 50 | $2,848.98 | $-12.60 | $-12.60 | $1,586.02 | $12.60 | $6.30 | 28.00% | 46.00% |
| risk_score:q4&regime_reversal_pressure:q1 | 1 | 1 | 1 | 0 | 300 | 27 | $1,193.48 | $-7.07 | $-7.07 | $1,580.49 | $7.07 | $3.53 | 25.93% | 40.74% |
| price:q4&regime_reversal_pressure:q2 | 1 | 1 | 1 | 0 | 300 | 5 | $311.31 | $-5.85 | $-5.85 | $1,579.28 | $5.85 | $2.93 | 20.00% | 20.00% |
| price:q3&side_model_p:q4 | 1 | 1 | 1 | 0 | 300 | 10 | $556.73 | $-2.45 | $-2.45 | $1,575.88 | $2.45 | $1.23 | 10.00% | 20.00% |
| prior_market_range_7d:q1 | 1 | 1 | 1 | 0 | 300 | 300 | $14,872.02 | $-2.31 | $-2.31 | $0.00 | $2.31 | $1.16 | 23.33% | 40.67% |
| side_model_p:q4&risk_score:q4 | 2 | 2 | 1 | 1 | 600 | 38 | $1,839.04 | $-123.42 | $88.41 | $1,694.53 | $123.42 | $61.71 | 18.42% | 21.05% |
| side_model_p:q4&side_edge_vs_fill:q3 | 2 | 2 | 1 | 1 | 600 | 29 | $1,983.68 | $-107.17 | $154.45 | $1,678.28 | $107.17 | $53.59 | 20.69% | 34.48% |
| side_edge_vs_fill:q3&risk_score:q3 | 2 | 2 | 1 | 1 | 600 | 39 | $1,596.34 | $-40.53 | $7.48 | $1,611.64 | $40.53 | $20.27 | 23.08% | 41.03% |
| risk_score:q2&prior_market_range_3d:q2 | 1 | 1 | 0 | 1 | 300 | 24 | $1,079.63 | $2.18 | $2.18 | $1,571.25 | $-2.18 | $-1.09 | 25.00% | 58.33% |
| late_fav:prior_market_range_7d:q1 | 1 | 1 | 0 | 1 | 300 | 158 | $7,636.30 | $12.41 | $12.41 | $-14.72 | $-12.41 | $-6.20 | 20.89% | 39.24% |
| side_edge_vs_fill:q4&regime_reversal_pressure:q3 | 1 | 1 | 0 | 1 | 300 | 9 | $309.09 | $14.82 | $14.82 | $-17.13 | $-14.82 | $-7.41 | 22.22% | 33.33% |
| risk_score:q3&regime_reversal_pressure:q2 | 1 | 1 | 0 | 1 | 300 | 21 | $964.02 | $15.08 | $15.08 | $1,558.34 | $-15.08 | $-7.54 | 28.57% | 38.10% |
| risk_score:q2&regime_reversal_pressure:q3 | 1 | 1 | 0 | 1 | 300 | 15 | $649.26 | $17.02 | $17.02 | $-19.33 | $-17.02 | $-8.51 | 20.00% | 20.00% |
| risk_score:q3&prior_market_range_3d:q1 | 1 | 1 | 0 | 1 | 300 | 47 | $2,321.46 | $19.54 | $19.54 | $1,553.88 | $-19.54 | $-9.77 | 25.53% | 42.55% |
| side_edge_vs_fill:q1&prior_market_range_3d:q2 | 1 | 1 | 0 | 1 | 300 | 18 | $838.85 | $23.10 | $23.10 | $1,550.33 | $-23.10 | $-11.55 | 22.22% | 38.89% |
| price:q4&regime_reversal_pressure:q3 | 1 | 1 | 0 | 1 | 300 | 9 | $466.96 | $23.11 | $23.11 | $-25.43 | $-23.11 | $-11.56 | 11.11% | 11.11% |
| side_model_p:q4&prior_market_range_7d:q2 | 1 | 1 | 0 | 1 | 300 | 5 | $203.67 | $34.16 | $34.16 | $1,539.26 | $-34.16 | $-17.08 | 0.00% | 0.00% |
| price:q4&risk_score:q4 | 1 | 1 | 0 | 1 | 300 | 18 | $870.45 | $34.98 | $34.98 | $1,538.44 | $-34.98 | $-17.49 | 11.11% | 11.11% |
| side_model_p:q4&side_edge_vs_fill:q1 | 1 | 1 | 0 | 1 | 300 | 17 | $991.37 | $39.50 | $39.50 | $1,533.92 | $-39.50 | $-19.75 | 11.76% | 17.65% |
| price:q4&side_edge_vs_fill:q1 | 1 | 1 | 0 | 1 | 300 | 24 | $1,359.51 | $42.43 | $42.43 | $1,530.99 | $-42.43 | $-21.21 | 12.50% | 16.67% |
| late_fav:prior_market_range_1d:q2 | 1 | 1 | 0 | 1 | 300 | 26 | $1,130.38 | $42.82 | $42.82 | $1,530.60 | $-42.82 | $-21.41 | 23.08% | 30.77% |
| side_model_p:q2&regime_reversal_pressure:q3 | 2 | 2 | 1 | 1 | 600 | 34 | $1,393.99 | $50.00 | $109.53 | $1,521.10 | $-50.00 | $-25.00 | 23.53% | 41.18% |
| side_edge_vs_fill:q4&regime_reversal_pressure:q2 | 1 | 1 | 0 | 1 | 300 | 12 | $635.70 | $52.60 | $52.60 | $1,520.82 | $-52.60 | $-26.30 | 33.33% | 58.33% |
| side_model_p:q4&prior_market_range_7d:q1 | 2 | 2 | 1 | 1 | 600 | 131 | $7,737.86 | $54.91 | $265.51 | $1,516.19 | $-54.91 | $-27.46 | 16.79% | 29.01% |
| risk_score:q4&prior_market_range_3d:q2 | 1 | 1 | 0 | 1 | 300 | 9 | $369.49 | $57.90 | $57.90 | $-60.21 | $-57.90 | $-28.95 | 22.22% | 55.56% |
| price:q3&risk_score:q4 | 1 | 1 | 0 | 1 | 300 | 12 | $486.05 | $61.32 | $61.32 | $-63.64 | $-61.32 | $-30.66 | 16.67% | 33.33% |
| regime_reversal_pressure:q4&prior_market_range_3d:q2 | 1 | 1 | 0 | 1 | 300 | 5 | $173.36 | $64.45 | $64.45 | $-66.77 | $-64.45 | $-32.23 | 0.00% | 60.00% |
| side_model_p:q2&side_edge_vs_fill:q3 | 1 | 1 | 0 | 1 | 300 | 14 | $510.06 | $65.58 | $65.58 | $-67.89 | $-65.58 | $-32.79 | 21.43% | 42.86% |
| side_model_p:q2&risk_score:q3 | 1 | 1 | 0 | 1 | 300 | 18 | $599.86 | $69.00 | $69.00 | $1,504.43 | $-69.00 | $-34.50 | 22.22% | 33.33% |
| side_edge_vs_fill:q1&regime_reversal_pressure:q3 | 2 | 2 | 1 | 1 | 600 | 18 | $1,037.80 | $70.37 | $99.90 | $1,500.73 | $-70.37 | $-35.19 | 22.22% | 38.89% |
| late_confirm:side_model_p:q4 | 1 | 1 | 0 | 1 | 300 | 20 | $1,169.49 | $76.29 | $76.29 | $1,497.14 | $-76.29 | $-38.14 | 10.00% | 15.00% |
| price:q2&prior_market_range_3d:q2 | 1 | 1 | 0 | 1 | 300 | 18 | $499.33 | $88.72 | $88.72 | $1,484.71 | $-88.72 | $-44.36 | 22.22% | 38.89% |
| late_confirm:price:q4 | 1 | 1 | 0 | 1 | 300 | 28 | $1,593.18 | $92.14 | $92.14 | $1,481.28 | $-92.14 | $-46.07 | 10.71% | 14.29% |
| risk_score:q2&regime_reversal_pressure:q4 | 1 | 1 | 0 | 1 | 300 | 9 | $477.91 | $96.47 | $96.47 | $-98.79 | $-96.47 | $-48.24 | 11.11% | 44.44% |
| risk_score:q3&prior_market_range_7d:q1 | 1 | 1 | 0 | 1 | 300 | 61 | $2,805.02 | $100.62 | $100.62 | $1,472.80 | $-100.62 | $-50.31 | 24.59% | 42.62% |

## Folds

| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Toxic Rate |
|---:|---:|---:|---|---|---:|---:|
| 1 | 900 | 300 | 2026-03-09 | 2026-03-13 | $-2.31 | 23.33% |
| 2 | 1200 | 300 | 2026-03-13 | 2026-03-20 | $1,573.42 | 22.33% |
