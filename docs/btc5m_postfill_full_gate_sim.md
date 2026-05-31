# BTC5m Post-Fill Gate Simulation

Source: `/tmp/btc5m_postfill_watch_markets.jsonl`
Target: `toxic_reversal_path`
Fills: `3549`
Min train fills: `1000`
Test fills per fold: `500`
Step fills: `500`

This is an offline diagnostic. It does not prove live performance, but it is stricter than a single split because thresholds are fit only on earlier fills and applied to later fills.

## Fold Quality

| Folds | Test Fills | Test PnL | Target Rate | Log Loss | Brier | Mean Fold AUC |
|---:|---:|---:|---:|---:|---:|---:|
| 5 | 2500 | $7,178.91 | 21.28% | 0.6909 | 0.2451 | 0.5739 |

## Candidate Gate Outcomes

Improvement assumes full removal of high-risk fills. `Half-Throttle Improvement` assumes high-risk fill size is cut by 50%, so PnL contribution is also halved.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_late_favourite_load:q0.90:lane_threshold | 5 | 959 | 125 | $5,714.58 | $-167.55 | $4,144.32 | $167.55 | $83.78 | 25.60% | 44.00% |
| br2_late_favourite_load:q0.95:lane_threshold | 5 | 959 | 79 | $3,740.64 | $7.95 | $3,968.82 | $-7.95 | $-3.97 | 27.85% | 46.84% |
| br2_late_confirm:q0.95:lane_threshold | 5 | 807 | 60 | $4,602.33 | $267.42 | $1,092.67 | $-267.42 | $-133.71 | 36.67% | 60.00% |
| br2_high_skew_load:q0.95:lane_threshold | 5 | 734 | 74 | $2,217.64 | $363.93 | $1,478.13 | $-363.93 | $-181.97 | 24.32% | 35.14% |
| br2_late_favourite_load:q0.80:lane_threshold | 5 | 959 | 230 | $10,626.38 | $367.38 | $3,609.39 | $-367.38 | $-183.69 | 23.48% | 43.48% |
| br2_late_favourite_load:q0.70:lane_threshold | 5 | 959 | 323 | $14,986.83 | $374.58 | $3,602.19 | $-374.58 | $-187.29 | 22.60% | 45.51% |
| br2_late_confirm:q0.90:lane_threshold | 5 | 807 | 98 | $7,176.53 | $576.77 | $783.31 | $-576.77 | $-288.39 | 34.69% | 60.20% |
| br2_high_skew_load:q0.90:lane_threshold | 5 | 734 | 102 | $3,086.05 | $653.80 | $1,188.25 | $-653.80 | $-326.90 | 22.55% | 34.31% |
| br2_high_skew_load:q0.80:lane_threshold | 5 | 734 | 168 | $5,339.23 | $773.05 | $1,069.01 | $-773.05 | $-386.52 | 20.24% | 37.50% |
| br2_late_confirm:q0.70:lane_threshold | 5 | 807 | 233 | $18,188.96 | $811.25 | $548.83 | $-811.25 | $-405.63 | 35.19% | 61.80% |
| br2_high_skew_load:q0.70:lane_threshold | 5 | 734 | 233 | $7,452.82 | $812.36 | $1,029.70 | $-812.36 | $-406.18 | 21.46% | 38.63% |
| all_lanes:q0.95:global_threshold | 5 | 2500 | 164 | $10,348.39 | $1,043.42 | $6,135.50 | $-1,043.42 | $-521.71 | 31.10% | 54.27% |
| br2_late_confirm:q0.80:lane_threshold | 5 | 807 | 159 | $12,216.09 | $1,151.54 | $208.55 | $-1,151.54 | $-575.77 | 33.96% | 61.01% |
| all_lanes:q0.90:global_threshold | 5 | 2500 | 276 | $17,581.61 | $1,382.08 | $5,796.83 | $-1,382.08 | $-691.04 | 30.07% | 52.90% |
| all_lanes:q0.80:global_threshold | 5 | 2500 | 528 | $33,311.63 | $2,448.75 | $4,730.16 | $-2,448.75 | $-1,224.38 | 28.98% | 50.38% |
| all_lanes:q0.70:global_threshold | 5 | 2500 | 730 | $44,048.38 | $3,340.24 | $3,838.68 | $-3,340.24 | $-1,670.12 | 26.85% | 49.04% |

## Replay-Safe Hard-Regime Gate Diagnostics

These rules use only fill-time features. They are not automatically fitted per fold, so treat them as diagnostics for candidate regime throttles rather than validated live gates.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| hard:late_confirm:edge_le08_reversal | 5 | 2500 | 292 | $23,981.47 | $-506.43 | $7,685.35 | $506.43 | $253.22 | 25.68% | 40.41% |
| hard:late_fav:price_ge78_edge_le10_choppy | 5 | 2500 | 175 | $12,322.22 | $518.27 | $6,660.64 | $-518.27 | $-259.14 | 15.43% | 28.57% |
| hard:late_loads:signflip40_eff20 | 5 | 2500 | 870 | $61,085.64 | $917.79 | $6,261.13 | $-917.79 | $-458.89 | 24.60% | 43.68% |
| hard:late_confirm:expanded_not_decisive | 5 | 2500 | 497 | $42,389.31 | $1,013.99 | $6,164.92 | $-1,013.99 | $-507.00 | 25.75% | 44.67% |
| hard:late_loads:obs50_65_low_eff | 5 | 2500 | 294 | $18,756.23 | $1,662.16 | $5,516.76 | $-1,662.16 | $-831.08 | 17.69% | 37.41% |
| hard:late_fav:price75_90_obs40_65_signflip35 | 5 | 2500 | 360 | $25,682.83 | $1,707.28 | $5,471.64 | $-1,707.28 | $-853.64 | 15.83% | 33.89% |
| hard:late_fav:expanded_not_decisive | 5 | 2500 | 603 | $36,644.04 | $1,931.45 | $5,247.46 | $-1,931.45 | $-965.73 | 19.57% | 39.14% |
| hard:loading_lanes:obs_ge50_signflip35 | 5 | 2500 | 638 | $31,349.16 | $2,038.12 | $5,140.79 | $-2,038.12 | $-1,019.06 | 19.44% | 38.09% |
| hard:late_loads:obs40_65_reversal34 | 5 | 2500 | 419 | $30,571.07 | $2,699.10 | $4,479.82 | $-2,699.10 | $-1,349.55 | 21.00% | 38.66% |
| hard:late_loads:expanded_not_decisive | 5 | 2500 | 1100 | $79,033.35 | $2,945.44 | $4,233.47 | $-2,945.44 | $-1,472.72 | 22.36% | 41.64% |

## Folds

| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Target Rate | Log Loss | AUC |
|---:|---:|---:|---|---|---:|---:|---:|---:|
| 1 | 1000 | 500 | 2026-03-08 | 2026-03-12 | $1,654.69 | 17.40% | 0.6791 | 0.5968 |
| 2 | 1500 | 500 | 2026-03-12 | 2026-03-20 | $1,487.11 | 23.00% | 0.7843 | 0.5757 |
| 3 | 2000 | 500 | 2026-03-20 | 2026-04-05 | $1,721.74 | 21.40% | 0.6881 | 0.5621 |
| 4 | 2500 | 500 | 2026-04-05 | 2026-04-19 | $3,242.22 | 19.00% | 0.6418 | 0.5727 |
| 5 | 3000 | 500 | 2026-04-19 | 2026-05-18 | $-926.85 | 25.60% | 0.6613 | 0.5619 |
