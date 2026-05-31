# BTC5m Late-Break Walk-Forward Gate Search

Source: `/tmp/btc5m_postfill_markets_062901.jsonl`
Late fills: `1864`
Late-fill PnL: `$5,696.75`
Toxic late fills: `384` (`20.60%`)
Min train fills: `900`
Test fills per fold: `300`
Step fills: `300`

Candidate thresholds are computed from each fold's training fills only. A candidate is admitted in a fold only when the same train-side rule removed at least the configured minimum fills and had negative train PnL.

## Candidate Outcomes

| Candidate | Folds | Active Folds | Helpful Folds | Harmful Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Worst Fold Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Toxic Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| side_model_p:q4&regime_reversal_pressure:q2 | 3 | 3 | 3 | 0 | 900 | 48 | $3,353.69 | $-372.18 | $-9.86 | $3,606.58 | $372.18 | $186.09 | 20.83% | 35.42% |
| late_confirm:regime_sign_flip_rate:q2 | 1 | 1 | 1 | 0 | 300 | 37 | $3,163.83 | $-367.79 | $-367.79 | $2,031.08 | $367.79 | $183.90 | 35.14% | 43.24% |
| side_edge_vs_fill:q2&regime_reversal_pressure:q2 | 2 | 2 | 2 | 0 | 600 | 33 | $2,327.96 | $-359.02 | $-39.59 | $3,595.73 | $359.02 | $179.51 | 30.30% | 42.42% |
| late_confirm:regime_reversal_pressure:q2 | 1 | 1 | 1 | 0 | 300 | 33 | $2,829.39 | $-339.20 | $-339.20 | $2,002.49 | $339.20 | $169.60 | 36.36% | 42.42% |
| risk_score:q4&prior_market_range_7d:q1 | 2 | 2 | 2 | 0 | 600 | 171 | $8,057.96 | $-236.11 | $-105.86 | $1,807.22 | $236.11 | $118.05 | 29.82% | 45.03% |
| side_edge_vs_fill:q1&risk_score:q4 | 1 | 1 | 1 | 0 | 300 | 27 | $2,211.42 | $-230.48 | $-230.48 | $1,893.77 | $230.48 | $115.24 | 22.22% | 29.63% |
| regime_reversal_pressure:q2&prior_market_range_3d:q2 | 2 | 2 | 2 | 0 | 600 | 33 | $1,691.36 | $-139.85 | $-64.61 | $3,376.57 | $139.85 | $69.93 | 27.27% | 51.52% |
| side_edge_vs_fill:q4&prior_market_range_3d:q2 | 2 | 2 | 2 | 0 | 600 | 30 | $1,087.10 | $-127.46 | $-16.69 | $1,698.57 | $127.46 | $63.73 | 36.67% | 73.33% |
| price:q3&prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 300 | 6 | $231.24 | $-109.91 | $-109.91 | $1,683.33 | $109.91 | $54.95 | 50.00% | 50.00% |
| price:q4&risk_score:q3 | 2 | 2 | 2 | 0 | 600 | 22 | $1,196.75 | $-108.57 | $-9.23 | $1,679.68 | $108.57 | $54.28 | 22.73% | 36.36% |
| late_fav:prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 300 | 6 | $246.79 | $-108.56 | $-108.56 | $1,681.98 | $108.56 | $54.28 | 50.00% | 50.00% |
| regime_reversal_pressure:q2&prior_market_range_7d:q3 | 1 | 1 | 1 | 0 | 300 | 1 | $90.33 | $-90.33 | $-90.33 | $1,753.62 | $90.33 | $45.17 | 100.00% | 100.00% |
| price:q3&prior_market_range_7d:q3 | 1 | 1 | 1 | 0 | 300 | 3 | $128.99 | $-78.13 | $-78.13 | $1,741.42 | $78.13 | $39.06 | 33.33% | 66.67% |
| prior_market_range_3d:q1&prior_market_range_7d:q1 | 1 | 1 | 1 | 0 | 300 | 258 | $13,064.31 | $-63.23 | $-63.23 | $60.92 | $63.23 | $31.62 | 24.03% | 40.31% |
| price:q1&regime_reversal_pressure:q2 | 1 | 1 | 1 | 0 | 300 | 16 | $1,408.29 | $-62.09 | $-62.09 | $1,725.38 | $62.09 | $31.05 | 37.50% | 50.00% |
| side_model_p:q1&prior_market_range_3d:q3 | 1 | 1 | 1 | 0 | 300 | 4 | $355.29 | $-61.15 | $-61.15 | $1,724.44 | $61.15 | $30.57 | 50.00% | 75.00% |
| price:q4&regime_reversal_pressure:q2 | 2 | 2 | 2 | 0 | 600 | 23 | $2,008.78 | $-55.60 | $-5.85 | $3,292.31 | $55.60 | $27.80 | 13.04% | 17.39% |
| side_edge_vs_fill:q4&risk_score:q3 | 1 | 1 | 1 | 0 | 300 | 9 | $360.71 | $-42.63 | $-42.63 | $1,616.05 | $42.63 | $21.31 | 55.56% | 66.67% |
| risk_score:q4&prior_market_range_3d:q1 | 1 | 1 | 1 | 0 | 300 | 62 | $3,066.50 | $-37.04 | $-37.04 | $1,610.46 | $37.04 | $18.52 | 30.65% | 41.94% |
| late_confirm:side_model_p:q3 | 1 | 1 | 1 | 0 | 300 | 4 | $388.35 | $-29.76 | $-29.76 | $1,693.06 | $29.76 | $14.88 | 25.00% | 25.00% |
| regime_reversal_pressure:q4&prior_market_range_7d:q2 | 1 | 1 | 1 | 0 | 300 | 1 | $23.40 | $-23.40 | $-23.40 | $1,686.69 | $23.40 | $11.70 | 100.00% | 100.00% |
| regime_reversal_pressure:q2&prior_market_range_3d:q1 | 1 | 1 | 1 | 0 | 300 | 50 | $2,848.98 | $-12.60 | $-12.60 | $1,586.02 | $12.60 | $6.30 | 28.00% | 46.00% |
| prior_market_range_7d:q1 | 1 | 1 | 1 | 0 | 300 | 300 | $14,872.02 | $-2.31 | $-2.31 | $0.00 | $2.31 | $1.16 | 23.33% | 40.67% |
| risk_score:q3&regime_reversal_pressure:q2 | 2 | 2 | 1 | 1 | 600 | 41 | $2,455.55 | $-229.87 | $15.08 | $3,466.58 | $229.87 | $114.93 | 31.71% | 36.59% |
| price:q4&risk_score:q4 | 2 | 2 | 1 | 1 | 600 | 40 | $2,679.44 | $-181.43 | $34.98 | $3,418.15 | $181.43 | $90.72 | 15.00% | 17.50% |
| side_model_p:q4&risk_score:q4 | 3 | 3 | 2 | 1 | 900 | 56 | $3,357.90 | $-175.47 | $88.41 | $3,409.87 | $175.47 | $87.74 | 16.07% | 19.64% |
| regime_reversal_pressure:q2&prior_market_range_7d:q2 | 2 | 2 | 1 | 1 | 600 | 7 | $287.68 | $-67.30 | $32.09 | $3,304.01 | $67.30 | $33.65 | 28.57% | 42.86% |
| side_edge_vs_fill:q4&regime_reversal_pressure:q2 | 2 | 2 | 1 | 1 | 600 | 28 | $1,570.07 | $-11.30 | $52.60 | $3,248.02 | $11.30 | $5.65 | 32.14% | 53.57% |
| price:q2&prior_market_range_7d:q2 | 1 | 1 | 0 | 1 | 300 | 3 | $87.31 | $1.18 | $1.18 | $1,662.11 | $-1.18 | $-0.59 | 33.33% | 33.33% |
| side_edge_vs_fill:q4&prior_market_range_3d:q3 | 1 | 1 | 0 | 1 | 300 | 2 | $149.24 | $4.89 | $4.89 | $1,658.40 | $-4.89 | $-2.44 | 50.00% | 100.00% |
| late_fav:prior_market_range_7d:q1 | 1 | 1 | 0 | 1 | 300 | 158 | $7,636.30 | $12.41 | $12.41 | $-14.72 | $-12.41 | $-6.20 | 20.89% | 39.24% |
| side_edge_vs_fill:q4&regime_reversal_pressure:q3 | 1 | 1 | 0 | 1 | 300 | 9 | $309.09 | $14.82 | $14.82 | $-17.13 | $-14.82 | $-7.41 | 22.22% | 33.33% |
| risk_score:q2&regime_reversal_pressure:q3 | 1 | 1 | 0 | 1 | 300 | 15 | $649.26 | $17.02 | $17.02 | $-19.33 | $-17.02 | $-8.51 | 20.00% | 20.00% |
| price:q4&prior_market_range_7d:q2 | 1 | 1 | 0 | 1 | 300 | 1 | $96.70 | $21.05 | $21.05 | $1,642.24 | $-21.05 | $-10.53 | 0.00% | 0.00% |
| side_model_p:q1&side_edge_vs_fill:q2 | 2 | 2 | 1 | 1 | 600 | 48 | $3,303.67 | $21.73 | $271.69 | $3,214.99 | $-21.73 | $-10.86 | 41.67% | 66.67% |
| price:q4&regime_reversal_pressure:q3 | 1 | 1 | 0 | 1 | 300 | 9 | $466.96 | $23.11 | $23.11 | $-25.43 | $-23.11 | $-11.56 | 11.11% | 11.11% |
| price:q3&regime_reversal_pressure:q4 | 3 | 3 | 2 | 1 | 900 | 38 | $2,399.42 | $29.15 | $127.35 | $3,205.25 | $-29.15 | $-14.58 | 28.95% | 50.00% |
| side_model_p:q4&prior_market_range_7d:q2 | 1 | 1 | 0 | 1 | 300 | 5 | $203.67 | $34.16 | $34.16 | $1,539.26 | $-34.16 | $-17.08 | 0.00% | 0.00% |
| side_model_p:q4&side_edge_vs_fill:q1 | 1 | 1 | 0 | 1 | 300 | 17 | $991.37 | $39.50 | $39.50 | $1,533.92 | $-39.50 | $-19.75 | 11.76% | 17.65% |
| side_edge_vs_fill:q4&prior_market_range_7d:q2 | 2 | 2 | 1 | 1 | 600 | 11 | $551.90 | $41.38 | $79.58 | $3,195.33 | $-41.38 | $-20.69 | 18.18% | 27.27% |
| side_model_p:q2&regime_reversal_pressure:q3 | 2 | 2 | 1 | 1 | 600 | 34 | $1,393.99 | $50.00 | $109.53 | $1,521.10 | $-50.00 | $-25.00 | 23.53% | 41.18% |
| side_model_p:q4&prior_market_range_7d:q1 | 2 | 2 | 1 | 1 | 600 | 131 | $7,737.86 | $54.91 | $265.51 | $1,516.19 | $-54.91 | $-27.46 | 16.79% | 29.01% |
| price:q3&risk_score:q4 | 1 | 1 | 0 | 1 | 300 | 12 | $486.05 | $61.32 | $61.32 | $-63.64 | $-61.32 | $-30.66 | 16.67% | 33.33% |
| regime_reversal_pressure:q4&prior_market_range_3d:q2 | 1 | 1 | 0 | 1 | 300 | 5 | $173.36 | $64.45 | $64.45 | $-66.77 | $-64.45 | $-32.23 | 0.00% | 60.00% |
| side_model_p:q2&side_edge_vs_fill:q3 | 1 | 1 | 0 | 1 | 300 | 14 | $510.06 | $65.58 | $65.58 | $-67.89 | $-65.58 | $-32.79 | 21.43% | 42.86% |
| side_model_p:q2&risk_score:q3 | 1 | 1 | 0 | 1 | 300 | 18 | $599.86 | $69.00 | $69.00 | $1,504.43 | $-69.00 | $-34.50 | 22.22% | 33.33% |
| side_edge_vs_fill:q1&regime_reversal_pressure:q3 | 2 | 2 | 1 | 1 | 600 | 18 | $1,037.80 | $70.37 | $99.90 | $1,500.73 | $-70.37 | $-35.19 | 22.22% | 38.89% |
| late_confirm:side_model_p:q4 | 1 | 1 | 0 | 1 | 300 | 20 | $1,169.49 | $76.29 | $76.29 | $1,497.14 | $-76.29 | $-38.14 | 10.00% | 15.00% |
| regime_reversal_pressure:q2&prior_market_range_3d:q3 | 1 | 1 | 0 | 1 | 300 | 2 | $220.72 | $85.04 | $85.04 | $1,578.25 | $-85.04 | $-42.52 | 0.00% | 0.00% |
| price:q4&side_model_p:q2 | 1 | 1 | 0 | 1 | 300 | 5 | $353.09 | $86.23 | $86.23 | $1,577.07 | $-86.23 | $-43.11 | 0.00% | 0.00% |

## Folds

| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Toxic Rate |
|---:|---:|---:|---|---|---:|---:|
| 1 | 900 | 300 | 2026-03-09 | 2026-03-13 | $-2.31 | 23.33% |
| 2 | 1200 | 300 | 2026-03-13 | 2026-03-20 | $1,573.42 | 22.33% |
| 3 | 1500 | 300 | 2026-03-20 | 2026-04-01 | $1,663.29 | 19.67% |
