# BTC5m Post-Fill Gate Simulation

Source: `/tmp/btc5m_postfill_diagnostics_markets.jsonl`
Target: `toxic_reversal_path`
Fills: `1640`
Min train fills: `1000`
Test fills per fold: `500`
Step fills: `500`

This is an offline diagnostic. It does not prove live performance, but it is stricter than a single split because thresholds are fit only on earlier fills and applied to later fills.

## Fold Quality

| Folds | Test Fills | Test PnL | Target Rate | Log Loss | Brier | Mean Fold AUC |
|---:|---:|---:|---:|---:|---:|---:|
| 1 | 500 | $1,654.69 | 17.40% | 0.6791 | 0.2426 | 0.5968 |

## Candidate Gate Outcomes

Improvement assumes full removal of high-risk fills. `Half-Throttle Improvement` assumes high-risk fill size is cut by 50%, so PnL contribution is also halved.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| br2_late_confirm:q0.95:lane_threshold | 1 | 176 | 9 | $428.76 | $-35.29 | $543.23 | $35.29 | $17.64 | 44.44% | 55.56% |
| br2_late_favourite_load:q0.90:lane_threshold | 1 | 206 | 16 | $459.71 | $-14.86 | $872.87 | $14.86 | $7.43 | 25.00% | 37.50% |
| br2_high_skew_load:q0.90:lane_threshold | 1 | 118 | 7 | $142.55 | $2.52 | $286.21 | $-2.52 | $-1.26 | 28.57% | 42.86% |
| br2_high_skew_load:q0.95:lane_threshold | 1 | 118 | 5 | $106.53 | $11.35 | $277.38 | $-11.35 | $-5.67 | 20.00% | 40.00% |
| br2_late_favourite_load:q0.70:lane_threshold | 1 | 206 | 64 | $2,346.70 | $27.68 | $830.33 | $-27.68 | $-13.84 | 23.44% | 48.44% |
| br2_late_favourite_load:q0.95:lane_threshold | 1 | 206 | 10 | $278.63 | $52.19 | $805.82 | $-52.19 | $-26.10 | 20.00% | 40.00% |
| br2_high_skew_load:q0.80:lane_threshold | 1 | 118 | 19 | $377.83 | $53.75 | $234.98 | $-53.75 | $-26.87 | 15.79% | 36.84% |
| br2_late_favourite_load:q0.80:lane_threshold | 1 | 206 | 49 | $1,824.30 | $78.18 | $779.84 | $-78.18 | $-39.09 | 22.45% | 40.82% |
| br2_high_skew_load:q0.70:lane_threshold | 1 | 118 | 27 | $522.82 | $91.90 | $196.83 | $-91.90 | $-45.95 | 11.11% | 33.33% |
| all_lanes:q0.95:global_threshold | 1 | 500 | 26 | $1,119.46 | $181.20 | $1,473.49 | $-181.20 | $-90.60 | 26.92% | 46.15% |
| br2_late_confirm:q0.80:lane_threshold | 1 | 176 | 35 | $1,673.99 | $214.86 | $293.08 | $-214.86 | $-107.43 | 31.43% | 54.29% |
| all_lanes:q0.90:global_threshold | 1 | 500 | 48 | $1,992.63 | $233.24 | $1,421.45 | $-233.24 | $-116.62 | 27.08% | 50.00% |
| br2_late_confirm:q0.90:lane_threshold | 1 | 176 | 25 | $1,191.98 | $238.00 | $269.95 | $-238.00 | $-119.00 | 28.00% | 56.00% |
| br2_late_confirm:q0.70:lane_threshold | 1 | 176 | 57 | $2,772.76 | $339.32 | $168.63 | $-339.32 | $-169.66 | 29.82% | 56.14% |
| all_lanes:q0.80:global_threshold | 1 | 500 | 100 | $4,106.92 | $485.83 | $1,168.86 | $-485.83 | $-242.91 | 24.00% | 44.00% |
| all_lanes:q0.70:global_threshold | 1 | 500 | 141 | $5,505.97 | $526.65 | $1,128.04 | $-526.65 | $-263.32 | 24.82% | 50.35% |

## Replay-Safe Hard-Regime Gate Diagnostics

These rules use only fill-time features. They are not automatically fitted per fold, so treat them as diagnostics for candidate regime throttles rather than validated live gates.

| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| hard:late_fav:price_ge78_edge_le10_choppy | 1 | 500 | 31 | $1,781.89 | $91.11 | $1,563.58 | $-91.11 | $-45.56 | 9.68% | 25.81% |
| hard:late_confirm:edge_le08_reversal | 1 | 500 | 73 | $3,354.13 | $170.33 | $1,484.36 | $-170.33 | $-85.16 | 19.18% | 35.62% |
| hard:loading_lanes:obs_ge50_signflip35 | 1 | 500 | 100 | $2,962.73 | $284.88 | $1,369.81 | $-284.88 | $-142.44 | 17.00% | 40.00% |
| hard:late_fav:price75_90_obs40_65_signflip35 | 1 | 500 | 78 | $4,263.39 | $288.74 | $1,365.95 | $-288.74 | $-144.37 | 16.67% | 33.33% |
| hard:late_loads:obs50_65_low_eff | 1 | 500 | 48 | $1,972.91 | $340.94 | $1,313.75 | $-340.94 | $-170.47 | 12.50% | 27.08% |
| hard:late_fav:expanded_not_decisive | 1 | 500 | 125 | $5,480.90 | $377.97 | $1,276.72 | $-377.97 | $-188.98 | 17.60% | 38.40% |
| hard:late_confirm:expanded_not_decisive | 1 | 500 | 108 | $5,122.66 | $467.38 | $1,187.31 | $-467.38 | $-233.69 | 19.44% | 42.59% |
| hard:late_loads:obs40_65_reversal34 | 1 | 500 | 91 | $4,406.64 | $630.90 | $1,023.79 | $-630.90 | $-315.45 | 15.38% | 27.47% |
| hard:late_loads:signflip40_eff20 | 1 | 500 | 189 | $8,389.45 | $729.57 | $925.12 | $-729.57 | $-364.79 | 18.52% | 39.15% |
| hard:late_loads:expanded_not_decisive | 1 | 500 | 233 | $10,603.56 | $845.35 | $809.34 | $-845.35 | $-422.68 | 18.45% | 40.34% |

## Folds

| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Target Rate | Log Loss | AUC |
|---:|---:|---:|---|---|---:|---:|---:|---:|
| 1 | 1000 | 500 | 2026-03-08 | 2026-03-12 | $1,654.69 | 17.40% | 0.6791 | 0.5968 |
