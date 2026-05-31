# BTC5m Post-Fill Gate Simulation

Source: `/tmp/btc5m_postfill_markets_062901.jsonl`
Target: `toxic_reversal_path`
Fills: `2498`
Min train fills: `600`
Test fills per fold: `200`
Step fills: `200`

This is an offline diagnostic. It does not prove live performance, but it is stricter than a single split because thresholds are fit only on earlier fills and applied to later fills.

## Fold Quality

| Folds | Test Fills | Test PnL | Target Rate | Log Loss | Brier | Mean Fold AUC |
|---:|---:|---:|---:|---:|---:|---:|
| 9 | 1800 | $5,174.24 | 20.72% | 0.6868 | 0.2441 | 0.5965 |

## Candidate Gate Outcomes

Improvement assumes full removal of high-risk fills. `Half-Throttle Improvement` assumes high-risk fill size is cut by 50%, so PnL contribution is also halved.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_late_favourite_load:q0.95:lane_threshold | 9 | 695 | 43 | $1,506.39 | $61.37 | $2,731.68 | $-61.37 | $-30.68 | 30.23% | 48.84% |
| br2_late_favourite_load:q0.90:lane_threshold | 9 | 695 | 92 | $3,489.96 | $106.58 | $2,686.47 | $-106.58 | $-53.29 | 25.00% | 48.91% |
| br2_high_skew_load:q0.70:lane_threshold | 9 | 464 | 158 | $3,932.48 | $341.39 | $291.99 | $-341.39 | $-170.69 | 22.78% | 39.24% |
| br2_high_skew_load:q0.90:lane_threshold | 9 | 464 | 69 | $1,731.86 | $367.62 | $265.76 | $-367.62 | $-183.81 | 24.64% | 37.68% |
| br2_high_skew_load:q0.95:lane_threshold | 9 | 464 | 47 | $1,173.74 | $384.48 | $248.90 | $-384.48 | $-192.24 | 23.40% | 34.04% |
| br2_late_favourite_load:q0.80:lane_threshold | 9 | 695 | 167 | $7,036.28 | $396.96 | $2,396.08 | $-396.96 | $-198.48 | 22.75% | 47.90% |
| br2_high_skew_load:q0.80:lane_threshold | 9 | 464 | 113 | $2,779.64 | $462.95 | $170.43 | $-462.95 | $-231.48 | 20.35% | 38.94% |
| br2_late_favourite_load:q0.70:lane_threshold | 9 | 695 | 251 | $11,260.79 | $727.39 | $2,065.66 | $-727.39 | $-363.69 | 19.52% | 44.62% |
| br2_late_confirm:q0.95:lane_threshold | 9 | 641 | 48 | $2,799.85 | $772.07 | $975.75 | $-772.07 | $-386.03 | 31.25% | 58.33% |
| br2_late_confirm:q0.70:lane_threshold | 9 | 641 | 189 | $10,421.30 | $846.75 | $901.07 | $-846.75 | $-423.38 | 35.45% | 60.32% |
| br2_late_confirm:q0.90:lane_threshold | 9 | 641 | 86 | $4,765.16 | $1,088.62 | $659.20 | $-1,088.62 | $-544.31 | 32.56% | 58.14% |
| br2_late_confirm:q0.80:lane_threshold | 9 | 641 | 138 | $7,664.47 | $1,334.46 | $413.36 | $-1,334.46 | $-667.23 | 32.61% | 58.70% |
| all_lanes:q0.80:global_threshold | 9 | 1800 | 386 | $17,133.68 | $1,453.28 | $3,720.96 | $-1,453.28 | $-726.64 | 30.05% | 52.33% |
| all_lanes:q0.95:global_threshold | 9 | 1800 | 118 | $5,550.27 | $1,505.56 | $3,668.68 | $-1,505.56 | $-752.78 | 28.81% | 52.54% |
| all_lanes:q0.90:global_threshold | 9 | 1800 | 201 | $9,418.56 | $1,838.38 | $3,335.86 | $-1,838.38 | $-919.19 | 29.85% | 53.73% |
| all_lanes:q0.70:global_threshold | 9 | 1800 | 555 | $24,309.07 | $2,509.52 | $2,664.73 | $-2,509.52 | $-1,254.76 | 25.77% | 49.01% |

## Replay-Safe Hard-Regime Gate Diagnostics

These rules use only fill-time features. They are not automatically fitted per fold, so treat them as diagnostics for candidate regime throttles rather than validated live gates.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| hard:late_confirm:edge_le08_reversal | 9 | 1800 | 233 | $12,045.72 | $-53.41 | $5,227.65 | $53.41 | $26.70 | 22.75% | 37.77% |
| hard:loading_lanes:obs_ge50_signflip35 | 9 | 1800 | 425 | $15,635.01 | $486.78 | $4,687.46 | $-486.78 | $-243.39 | 21.88% | 40.71% |
| hard:late_fav:price_ge78_edge_le10_choppy | 9 | 1800 | 126 | $7,942.54 | $634.15 | $4,540.09 | $-634.15 | $-317.07 | 10.32% | 23.02% |
| hard:late_loads:obs50_65_low_eff | 9 | 1800 | 198 | $9,799.64 | $780.64 | $4,393.60 | $-780.64 | $-390.32 | 17.17% | 36.36% |
| hard:late_confirm:expanded_not_decisive | 9 | 1800 | 372 | $19,605.68 | $966.32 | $4,207.92 | $-966.32 | $-483.16 | 23.12% | 41.67% |
| hard:late_fav:price75_90_obs40_65_signflip35 | 9 | 1800 | 274 | $16,473.34 | $1,216.33 | $3,957.91 | $-1,216.33 | $-608.16 | 15.33% | 33.21% |
| hard:late_fav:expanded_not_decisive | 9 | 1800 | 422 | $21,062.57 | $1,305.43 | $3,868.81 | $-1,305.43 | $-652.71 | 19.19% | 39.10% |
| hard:late_loads:signflip40_eff20 | 9 | 1800 | 684 | $34,959.95 | $1,453.34 | $3,720.90 | $-1,453.34 | $-726.67 | 22.81% | 41.37% |
| hard:late_loads:obs40_65_reversal34 | 9 | 1800 | 336 | $17,230.13 | $1,613.41 | $3,560.83 | $-1,613.41 | $-806.71 | 19.35% | 37.20% |
| hard:late_loads:expanded_not_decisive | 9 | 1800 | 794 | $40,668.26 | $2,271.75 | $2,902.49 | $-2,271.75 | $-1,135.87 | 21.03% | 40.30% |

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
| 9 | 2200 | 200 | 2026-03-25 | 2026-04-01 | $703.15 | 19.00% | 0.5565 | 0.6395 |
