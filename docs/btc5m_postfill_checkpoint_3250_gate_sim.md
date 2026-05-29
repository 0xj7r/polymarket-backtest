# BTC5m Post-Fill Gate Simulation

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `toxic_reversal_path`
Fills: `1312`
Min train fills: `700`
Test fills per fold: `250`
Step fills: `250`

This is an offline diagnostic. It does not prove live performance, but it is stricter than a single split because thresholds are fit only on earlier fills and applied to later fills.

## Fold Quality

| Folds | Test Fills | Test PnL | Target Rate | Log Loss | Brier | Mean Fold AUC |
|---:|---:|---:|---:|---:|---:|---:|
| 2 | 500 | $1,291.99 | 19.20% | 0.6635 | 0.2352 | 0.6573 |

## Candidate Gate Outcomes

Improvement assumes full removal of high-risk fills. `Half-Throttle Improvement` assumes high-risk fill size is cut by 50%, so PnL contribution is also halved.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_late_confirm:q0.95:lane_threshold | 2 | 204 | 10 | $343.01 | $-114.68 | $667.03 | $114.68 | $57.34 | 50.00% | 70.00% |
| br2_late_confirm:q0.70:lane_threshold | 2 | 204 | 63 | $2,132.70 | $-108.15 | $660.50 | $108.15 | $54.08 | 38.10% | 66.67% |
| all_lanes:q0.80:global_threshold | 2 | 500 | 99 | $2,982.30 | $-61.91 | $1,353.90 | $61.91 | $30.95 | 32.32% | 55.56% |
| all_lanes:q0.90:global_threshold | 2 | 500 | 47 | $1,408.79 | $-53.89 | $1,345.88 | $53.89 | $26.94 | 36.17% | 63.83% |
| br2_late_confirm:q0.80:lane_threshold | 2 | 204 | 39 | $1,307.53 | $-50.05 | $602.39 | $50.05 | $25.02 | 38.46% | 66.67% |
| br2_high_skew_load:q0.70:lane_threshold | 2 | 99 | 23 | $309.93 | $-43.89 | $100.71 | $43.89 | $21.94 | 39.13% | 52.17% |
| br2_late_favourite_load:q0.90:lane_threshold | 2 | 197 | 14 | $332.37 | $-42.09 | $724.91 | $42.09 | $21.05 | 35.71% | 50.00% |
| br2_high_skew_load:q0.80:lane_threshold | 2 | 99 | 14 | $197.37 | $-30.06 | $86.89 | $30.06 | $15.03 | 42.86% | 50.00% |
| all_lanes:q0.95:global_threshold | 2 | 500 | 29 | $914.69 | $-3.80 | $1,295.79 | $3.80 | $1.90 | 34.48% | 58.62% |
| all_lanes:q0.70:global_threshold | 2 | 500 | 161 | $4,809.44 | $-1.43 | $1,293.42 | $1.43 | $0.71 | 30.43% | 54.66% |
| br2_high_skew_load:q0.90:lane_threshold | 2 | 99 | 7 | $100.51 | $0.44 | $56.39 | $-0.44 | $-0.22 | 28.57% | 42.86% |
| br2_late_favourite_load:q0.95:lane_threshold | 2 | 197 | 6 | $148.47 | $6.79 | $676.03 | $-6.79 | $-3.39 | 33.33% | 66.67% |
| br2_high_skew_load:q0.95:lane_threshold | 2 | 99 | 4 | $58.97 | $21.26 | $35.56 | $-21.26 | $-10.63 | 0.00% | 25.00% |
| br2_late_confirm:q0.90:lane_threshold | 2 | 204 | 27 | $907.17 | $25.07 | $527.28 | $-25.07 | $-12.53 | 33.33% | 62.96% |
| br2_late_favourite_load:q0.80:lane_threshold | 2 | 197 | 34 | $933.13 | $34.26 | $648.56 | $-34.26 | $-17.13 | 23.53% | 38.24% |
| br2_late_favourite_load:q0.70:lane_threshold | 2 | 197 | 65 | $1,807.60 | $95.10 | $587.71 | $-95.10 | $-47.55 | 21.54% | 41.54% |

## Folds

| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Target Rate | Log Loss | AUC |
|---:|---:|---:|---|---|---:|---:|---:|---:|
| 1 | 700 | 250 | 2026-03-04 | 2026-03-06 | $566.03 | 20.80% | 0.6408 | 0.6723 |
| 2 | 950 | 250 | 2026-03-06 | 2026-03-10 | $725.97 | 17.60% | 0.6861 | 0.6423 |
