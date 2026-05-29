# BTC5m Post-Fill Gate Simulation

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Target: `toxic_reversal_path`
Fills: `2247`
Min train fills: `600`
Test fills per fold: `200`
Step fills: `200`

This is an offline diagnostic. It does not prove live performance, but it is stricter than a single split because thresholds are fit only on earlier fills and applied to later fills.

## Fold Quality

| Folds | Test Fills | Test PnL | Target Rate | Log Loss | Brier | Mean Fold AUC |
|---:|---:|---:|---:|---:|---:|---:|
| 8 | 1600 | $4,471.09 | 20.94% | 0.7031 | 0.2511 | 0.5911 |

## Candidate Gate Outcomes

Improvement assumes full removal of high-risk fills. `Half-Throttle Improvement` assumes high-risk fill size is cut by 50%, so PnL contribution is also halved.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_late_favourite_load:q0.90:lane_threshold | 8 | 625 | 88 | $3,202.45 | $10.82 | $2,116.41 | $-10.82 | $-5.41 | 26.14% | 48.86% |
| br2_late_favourite_load:q0.95:lane_threshold | 8 | 625 | 41 | $1,378.70 | $21.95 | $2,105.29 | $-21.95 | $-10.97 | 31.71% | 51.22% |
| br2_high_skew_load:q0.90:lane_threshold | 8 | 406 | 61 | $1,455.09 | $316.99 | $-3.46 | $-316.99 | $-158.50 | 26.23% | 34.43% |
| br2_high_skew_load:q0.70:lane_threshold | 8 | 406 | 145 | $3,468.13 | $331.63 | $-18.10 | $-331.63 | $-165.82 | 22.76% | 37.93% |
| br2_high_skew_load:q0.95:lane_threshold | 8 | 406 | 44 | $1,070.36 | $336.77 | $-23.24 | $-336.77 | $-168.39 | 25.00% | 31.82% |
| br2_late_favourite_load:q0.80:lane_threshold | 8 | 625 | 156 | $6,176.55 | $401.55 | $1,725.68 | $-401.55 | $-200.78 | 23.08% | 48.72% |
| br2_high_skew_load:q0.80:lane_threshold | 8 | 406 | 102 | $2,394.74 | $477.03 | $-163.50 | $-477.03 | $-238.52 | 19.61% | 36.27% |
| br2_late_confirm:q0.95:lane_threshold | 8 | 569 | 46 | $2,618.92 | $633.49 | $1,396.84 | $-633.49 | $-316.75 | 32.61% | 56.52% |
| br2_late_favourite_load:q0.70:lane_threshold | 8 | 625 | 238 | $10,277.73 | $772.43 | $1,354.80 | $-772.43 | $-386.21 | 19.33% | 44.96% |
| br2_late_confirm:q0.70:lane_threshold | 8 | 569 | 179 | $9,489.74 | $867.84 | $1,162.49 | $-867.84 | $-433.92 | 35.20% | 59.22% |
| br2_late_confirm:q0.90:lane_threshold | 8 | 569 | 83 | $4,494.00 | $896.79 | $1,133.54 | $-896.79 | $-448.40 | 33.73% | 57.83% |
| br2_late_confirm:q0.80:lane_threshold | 8 | 569 | 133 | $7,209.24 | $1,174.47 | $855.85 | $-1,174.47 | $-587.24 | 33.08% | 57.89% |
| all_lanes:q0.95:global_threshold | 8 | 1600 | 113 | $5,213.52 | $1,279.92 | $3,191.17 | $-1,279.92 | $-639.96 | 30.09% | 51.33% |
| all_lanes:q0.80:global_threshold | 8 | 1600 | 363 | $15,542.46 | $1,285.32 | $3,185.77 | $-1,285.32 | $-642.66 | 30.58% | 51.52% |
| all_lanes:q0.90:global_threshold | 8 | 1600 | 189 | $8,622.13 | $1,578.74 | $2,892.35 | $-1,578.74 | $-789.37 | 30.69% | 52.91% |
| all_lanes:q0.70:global_threshold | 8 | 1600 | 518 | $21,719.20 | $2,582.24 | $1,888.85 | $-2,582.24 | $-1,291.12 | 25.48% | 48.46% |

## Replay-Safe Hard-Regime Gate Diagnostics

These rules use only fill-time features. They are not automatically fitted per fold, so treat them as diagnostics for candidate regime throttles rather than validated live gates.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| hard:late_confirm:edge_le08_reversal | 8 | 1600 | 204 | $9,674.14 | $320.76 | $4,150.33 | $-320.76 | $-160.38 | 21.57% | 38.24% |
| hard:loading_lanes:obs_ge50_signflip35 | 8 | 1600 | 375 | $12,808.39 | $467.18 | $4,003.92 | $-467.18 | $-233.59 | 22.13% | 41.07% |
| hard:late_fav:price_ge78_edge_le10_choppy | 8 | 1600 | 108 | $6,204.20 | $640.76 | $3,830.33 | $-640.76 | $-320.38 | 9.26% | 23.15% |
| hard:late_loads:obs50_65_low_eff | 8 | 1600 | 178 | $8,019.98 | $710.00 | $3,761.09 | $-710.00 | $-355.00 | 17.42% | 37.08% |
| hard:late_confirm:expanded_not_decisive | 8 | 1600 | 327 | $15,809.73 | $763.28 | $3,707.81 | $-763.28 | $-381.64 | 23.55% | 42.81% |
| hard:late_fav:price75_90_obs40_65_signflip35 | 8 | 1600 | 239 | $13,283.93 | $1,010.16 | $3,460.93 | $-1,010.16 | $-505.08 | 15.48% | 34.31% |
| hard:late_fav:expanded_not_decisive | 8 | 1600 | 381 | $17,566.62 | $1,069.87 | $3,401.22 | $-1,069.87 | $-534.94 | 19.42% | 40.42% |
| hard:late_loads:obs40_65_reversal34 | 8 | 1600 | 305 | $14,427.35 | $1,405.68 | $3,065.41 | $-1,405.68 | $-702.84 | 19.34% | 38.36% |
| hard:late_loads:signflip40_eff20 | 8 | 1600 | 604 | $28,105.00 | $1,792.88 | $2,678.21 | $-1,792.88 | $-896.44 | 22.19% | 41.72% |
| hard:late_loads:expanded_not_decisive | 8 | 1600 | 708 | $33,376.35 | $1,833.16 | $2,637.94 | $-1,833.16 | $-916.58 | 21.33% | 41.53% |

## Folds

| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Target Rate | Log Loss | AUC |
|---:|---:|---:|---|---|---:|---:|---:|---:|
| 1 | 600 | 200 | 2026-03-04 | 2026-03-05 | $187.33 | 21.50% | 0.6052 | 0.5214 |
| 2 | 800 | 200 | 2026-03-05 | 2026-03-08 | $464.15 | 22.50% | 0.6362 | 0.6450 |
| 3 | 1000 | 200 | 2026-03-08 | 2026-03-10 | $526.98 | 17.50% | 0.6554 | 0.6362 |
| 4 | 1200 | 200 | 2026-03-10 | 2026-03-11 | $751.76 | 16.50% | 0.6694 | 0.6516 |
| 5 | 1400 | 200 | 2026-03-11 | 2026-03-13 | $-256.25 | 25.00% | 0.7068 | 0.5207 |
| 6 | 1600 | 200 | 2026-03-13 | 2026-03-17 | $-30.52 | 25.50% | 0.7157 | 0.6190 |
| 7 | 1800 | 200 | 2026-03-17 | 2026-03-20 | $2,149.84 | 16.50% | 0.8845 | 0.5785 |
| 8 | 2000 | 200 | 2026-03-20 | 2026-03-25 | $677.81 | 22.50% | 0.7511 | 0.5566 |
