# BTC5m Late-Regime Action Report

Source: `s3://pm-research-backtest-prod/results/20260528T225810Z-portfolio-grid-52322/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8/markets.jsonl`
Range: `2026-02-27T15:40:00+00:00` to `2026-05-20T23:55:00+00:00`

The `range_78_93_midwide` label is post-hoc final market range. It is useful for diagnosis, not directly deployable as a live gate.

## Executive Read

- The later regime problem is not that mid-wide markets became more common; their rate fell from the first window, but their expectancy stayed sharply negative while participation collapsed.
- The strategy still offsets some mid-wide damage with non-midwide continuation wins, but the last window has far fewer active markets, so the offset is much weaker.
- Replay-safe slices do identify late-window pain, but several were profitable early. Treat them as inputs to a regime/gated model, not fixed hard-coded throttles.

## Window Drift

| Window | Markets | Active | Active Rate | Fills | PnL | Start Eq | End Eq | Return | Mid-Wide Markets | Mid-Wide Rate | Mid-Wide PnL | Non-Mid-Wide PnL |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| first_third | 7901 | 1191 | 15.07% | 2374 | $5,072.67 | $1,000.00 | $6,072.67 | 507.27% | 1425 | 18.04% | $-4,398.56 | $9,471.23 |
| middle_third | 7902 | 569 | 7.20% | 943 | $4,144.62 | $6,072.67 | $10,217.29 | 68.25% | 935 | 11.83% | $-1,983.86 | $6,128.48 |
| last_third | 7902 | 256 | 3.24% | 406 | $-227.08 | $10,217.29 | $9,990.21 | -2.22% | 949 | 12.01% | $-2,650.65 | $2,423.57 |
| first_30d | 8640 | 1214 | 14.05% | 2413 | $5,394.12 | $1,000.00 | $6,394.12 | 539.41% | 1512 | 17.50% | $-4,380.98 | $9,775.10 |
| last_30d | 8633 | 335 | 3.88% | 549 | $20.80 | $9,969.41 | $9,990.21 | 0.21% | 1039 | 12.04% | $-3,006.12 | $3,026.92 |

## Weekly Evolution

| Week | Markets | Active | Active Rate | Fills | PnL | Mid-Wide Markets | Mid-Wide PnL | Non-Mid-Wide PnL |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| 2026-02-23 | 676 | 95 | 14.05% | 191 | $158.27 | 138 | $-271.67 | $429.94 |
| 2026-03-02 | 2016 | 434 | 21.53% | 906 | $1,626.72 | 386 | $-883.05 | $2,509.77 |
| 2026-03-09 | 2016 | 319 | 15.82% | 629 | $758.13 | 361 | $-1,689.53 | $2,447.66 |
| 2026-03-16 | 2016 | 210 | 10.42% | 402 | $2,207.27 | 373 | $-52.39 | $2,259.66 |
| 2026-03-23 | 2016 | 158 | 7.84% | 290 | $751.49 | 265 | $-1,484.33 | $2,235.82 |
| 2026-03-30 | 2016 | 131 | 6.50% | 207 | $862.32 | 242 | $-406.46 | $1,268.78 |
| 2026-04-06 | 2016 | 143 | 7.09% | 253 | $1,043.49 | 261 | $-791.16 | $1,834.65 |
| 2026-04-13 | 2012 | 166 | 8.25% | 261 | $1,861.69 | 205 | $-467.91 | $2,329.61 |
| 2026-04-20 | 2016 | 152 | 7.54% | 274 | $305.34 | 251 | $-881.49 | $1,186.83 |
| 2026-04-27 | 2016 | 57 | 2.83% | 78 | $-429.63 | 231 | $-592.79 | $163.16 |
| 2026-05-04 | 2012 | 75 | 3.73% | 124 | $31.03 | 233 | $-1,103.54 | $1,134.58 |
| 2026-05-11 | 2013 | 39 | 1.94% | 52 | $-589.11 | 250 | $-244.30 | $-344.81 |
| 2026-05-18 | 864 | 37 | 4.28% | 56 | $403.19 | 113 | $-164.44 | $567.63 |

## Last 30d Daily Attribution

| Date | Markets | Active | Fills | PnL | Mid-Wide PnL | Non-Mid-Wide PnL | Late Fav | Late Confirm | High Skew | Tail |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 2026-04-21 | 288 | 21 | 41 | $391.21 | $83.72 | $307.49 | $149.49 | $324.99 | $-67.45 | $-15.83 |
| 2026-04-22 | 288 | 40 | 72 | $144.22 | $-144.09 | $288.31 | $136.52 | $-9.61 | $29.56 | $-12.25 |
| 2026-04-23 | 288 | 41 | 74 | $-516.05 | $-845.21 | $329.16 | $-162.10 | $-385.61 | $36.60 | $-4.93 |
| 2026-04-24 | 288 | 20 | 39 | $294.15 | $-82.00 | $376.16 | $-6.94 | $122.85 | $86.54 | $91.70 |
| 2026-04-25 | 288 | 0 | 0 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 |
| 2026-04-26 | 288 | 5 | 13 | $291.79 | $86.53 | $205.25 | $92.06 | $169.37 | $40.05 | $-9.69 |
| 2026-04-27 | 288 | 18 | 23 | $-118.15 | $-174.72 | $56.57 | $-135.89 | $-9.96 | $30.45 | $-2.75 |
| 2026-04-28 | 288 | 8 | 9 | $134.46 | $0.00 | $134.46 | $30.24 | $104.22 | $0.00 | $0.00 |
| 2026-04-29 | 288 | 15 | 19 | $-138.72 | $-204.24 | $65.52 | $-24.74 | $-114.71 | $6.18 | $-5.45 |
| 2026-04-30 | 288 | 7 | 13 | $-310.29 | $-213.83 | $-96.47 | $-40.21 | $-123.77 | $-146.32 | $0.00 |
| 2026-05-01 | 288 | 8 | 13 | $1.82 | $0.00 | $1.82 | $30.03 | $-110.99 | $84.14 | $-1.37 |
| 2026-05-02 | 288 | 0 | 0 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 |
| 2026-05-03 | 288 | 1 | 1 | $1.25 | $0.00 | $1.25 | $0.00 | $0.00 | $1.25 | $0.00 |
| 2026-05-04 | 288 | 23 | 42 | $54.36 | $22.54 | $31.82 | $-2.20 | $-9.25 | $71.31 | $-5.50 |
| 2026-05-05 | 288 | 14 | 22 | $168.62 | $-223.24 | $391.85 | $181.88 | $28.89 | $-36.37 | $-5.78 |
| 2026-05-06 | 288 | 14 | 21 | $-177.08 | $-461.12 | $284.04 | $-103.98 | $-113.04 | $41.18 | $-1.24 |
| 2026-05-07 | 284 | 14 | 20 | $32.41 | $-271.55 | $303.97 | $-186.76 | $257.51 | $-33.16 | $-5.18 |
| 2026-05-08 | 288 | 7 | 10 | $-44.54 | $-156.95 | $112.41 | $-95.83 | $14.80 | $37.93 | $-1.45 |
| 2026-05-09 | 288 | 0 | 0 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 |
| 2026-05-10 | 288 | 3 | 9 | $-2.72 | $-13.21 | $10.49 | $-124.19 | $113.29 | $8.33 | $-0.16 |
| 2026-05-11 | 288 | 4 | 4 | $-51.95 | $-156.88 | $104.93 | $8.27 | $-60.22 | $0.00 | $0.00 |
| 2026-05-12 | 288 | 4 | 8 | $-115.16 | $48.78 | $-163.94 | $-11.35 | $-49.95 | $-53.85 | $0.00 |
| 2026-05-13 | 288 | 7 | 9 | $5.46 | $-49.63 | $55.09 | $15.27 | $28.72 | $-38.52 | $0.00 |
| 2026-05-14 | 288 | 10 | 13 | $89.93 | $59.89 | $30.04 | $163.32 | $-127.31 | $55.63 | $-1.71 |
| 2026-05-15 | 285 | 10 | 12 | $-420.81 | $-148.89 | $-271.93 | $-211.83 | $-155.19 | $-53.79 | $0.00 |
| 2026-05-16 | 288 | 2 | 2 | $37.80 | $0.00 | $37.80 | $37.80 | $0.00 | $0.00 | $0.00 |
| 2026-05-17 | 288 | 2 | 4 | $-134.38 | $2.43 | $-136.81 | $1.34 | $-136.81 | $1.41 | $-0.33 |
| 2026-05-18 | 288 | 27 | 41 | $178.65 | $-103.24 | $281.89 | $54.07 | $112.52 | $22.00 | $-9.93 |
| 2026-05-19 | 288 | 3 | 4 | $83.41 | $0.00 | $83.41 | $50.20 | $2.96 | $30.26 | $0.00 |
| 2026-05-20 | 288 | 7 | 11 | $141.13 | $-61.21 | $202.33 | $87.89 | $40.63 | $17.64 | $-5.04 |

## Worst Last-Window Days

| Rank | Date | Markets | Active | Fills | PnL | Mid-Wide PnL | Non-Mid-Wide PnL | Late Fav | Late Confirm | High Skew | Tail |
|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 1 | 2026-04-23 | 288 | 41 | 74 | $-516.05 | $-845.21 | $329.16 | $-162.10 | $-385.61 | $36.60 | $-4.93 |
| 2 | 2026-05-15 | 285 | 10 | 12 | $-420.81 | $-148.89 | $-271.93 | $-211.83 | $-155.19 | $-53.79 | $0.00 |
| 3 | 2026-04-30 | 288 | 7 | 13 | $-310.29 | $-213.83 | $-96.47 | $-40.21 | $-123.77 | $-146.32 | $0.00 |
| 4 | 2026-05-06 | 288 | 14 | 21 | $-177.08 | $-461.12 | $284.04 | $-103.98 | $-113.04 | $41.18 | $-1.24 |
| 5 | 2026-04-29 | 288 | 15 | 19 | $-138.72 | $-204.24 | $65.52 | $-24.74 | $-114.71 | $6.18 | $-5.45 |
| 6 | 2026-05-17 | 288 | 2 | 4 | $-134.38 | $2.43 | $-136.81 | $1.34 | $-136.81 | $1.41 | $-0.33 |
| 7 | 2026-04-27 | 288 | 18 | 23 | $-118.15 | $-174.72 | $56.57 | $-135.89 | $-9.96 | $30.45 | $-2.75 |
| 8 | 2026-05-12 | 288 | 4 | 8 | $-115.16 | $48.78 | $-163.94 | $-11.35 | $-49.95 | $-53.85 | $0.00 |
| 9 | 2026-05-11 | 288 | 4 | 4 | $-51.95 | $-156.88 | $104.93 | $8.27 | $-60.22 | $0.00 | $0.00 |
| 10 | 2026-05-08 | 288 | 7 | 10 | $-44.54 | $-156.95 | $112.41 | $-95.83 | $14.80 | $37.93 | $-1.45 |
| 11 | 2026-05-10 | 288 | 3 | 9 | $-2.72 | $-13.21 | $10.49 | $-124.19 | $113.29 | $8.33 | $-0.16 |
| 12 | 2026-04-25 | 288 | 0 | 0 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 | $0.00 |

## Last-Window Lane x Final-Range Attribution

| Lane + Final Range | Fills | PnL | Cost | Win Rate | PnL/Fill |
|---|---:|---:|---:|---:|---:|
| br2_late_confirm:range_78_93_midwide | 30 | $-1,300.89 | $3,300.07 | 33.33% | $-43.36 |
| br2_late_favourite_load:range_78_93_midwide | 52 | $-1,067.18 | $3,348.14 | 44.23% | $-20.52 |
| br2_high_skew_load:range_78_93_midwide | 50 | $-616.47 | $2,407.64 | 60.00% | $-12.33 |
| br2_late_confirm:range_93_97 | 17 | $-163.77 | $2,111.76 | 70.59% | $-9.63 |
| br2_late_confirm:range_lt50 | 3 | $-56.99 | $427.83 | 66.67% | $-19.00 |
| br2_convex_tail:range_ge97 | 11 | $-28.35 | $28.35 | 0.00% | $-2.58 |
| br2_convex_tail:range_78_93_midwide | 10 | $-21.58 | $21.58 | 0.00% | $-2.16 |
| br2_convex_tail:range_93_97 | 7 | $-20.75 | $20.75 | 0.00% | $-2.96 |
| br2_convex_tail:range_lt50 | 2 | $-1.46 | $1.46 | 0.00% | $-0.73 |
| br2_high_skew_load:range_lt50 | 4 | $60.12 | $224.61 | 100.00% | $15.03 |
| br2_late_favourite_load:range_lt50 | 3 | $61.75 | $170.83 | 100.00% | $20.58 |
| br2_convex_tail:range_50_78 | 10 | $75.24 | $38.61 | 10.00% | $7.52 |
| br2_high_skew_load:range_93_97 | 15 | $111.82 | $668.31 | 93.33% | $7.45 |
| br2_late_favourite_load:range_93_97 | 34 | $123.56 | $1,886.46 | 79.41% | $3.63 |
| br2_high_skew_load:range_50_78 | 38 | $212.52 | $1,919.11 | 84.21% | $5.59 |
| br2_late_favourite_load:range_50_78 | 49 | $295.36 | $3,254.99 | 79.59% | $6.03 |
| br2_high_skew_load:range_ge97 | 64 | $403.02 | $3,111.51 | 89.06% | $6.30 |
| br2_late_favourite_load:range_ge97 | 69 | $518.87 | $4,498.06 | 85.51% | $7.52 |
| br2_late_confirm:range_ge97 | 39 | $688.70 | $4,733.18 | 79.49% | $17.66 |
| br2_late_confirm:range_50_78 | 42 | $747.29 | $5,217.20 | 83.33% | $17.79 |

## Candidate Guardrail Stability

Replay-safe rules are marked `yes`. Negative removed PnL means removing/throttling that slice would have helped that window. Positive early removed PnL means the rule would have damaged the strong early regime.

| Rule | Lane | Replay Safe | Early Removed Fills | Early Removed PnL | Late Removed Fills | Late Removed PnL | Late Removed Win Rate | Late Removed Cost |
|---|---|---|---:|---:|---:|---:|---:|---:|
| posthoc:final_midwide | all_loading | no | 635 | $-4,335.06 | 132 | $-2,984.54 | 47.73% | $9,055.85 |
| late_confirm_sign_flip_040_0457 | late_confirm | yes | 213 | $-400.85 | 29 | $-306.19 | 58.62% | $3,417.74 |
| late_confirm_reversal_024_034 | late_confirm | yes | 386 | $828.37 | 63 | $-1,063.99 | 60.32% | $7,211.34 |
| late_confirm_low_conf_lt_081 | late_confirm | yes | 211 | $303.78 | 33 | $-605.36 | 60.61% | $3,762.21 |
| late_fav_obs_040_050 | late_favourite | yes | 337 | $832.54 | 57 | $-660.43 | 64.91% | $3,141.41 |
| late_fav_risk_036_039 | late_favourite | yes | 185 | $937.10 | 50 | $-509.27 | 62.00% | $3,986.50 |
| late_fav_model_q4 | late_favourite | yes | 219 | $417.96 | 52 | $-382.96 | 71.15% | $3,453.37 |

## Feature Drift By Lane

| Lane | Feature | Early Median | Late Median | Delta | Early P25..P75 | Late P25..P75 |
|---|---|---:|---:|---:|---:|---:|
| br2_late_favourite_load | market_yes_range_so_far | 0.4800 | 0.4800 | 0.0000 | 0.4150..0.5350 | 0.4000..0.5500 |
| br2_late_favourite_load | side_model_p | 0.8878 | 0.8867 | -0.0011 | 0.8723..0.8978 | 0.8600..0.8984 |
| br2_late_favourite_load | side_edge_vs_fill | 0.1123 | 0.1099 | -0.0024 | 0.0993..0.1330 | 0.0986..0.1334 |
| br2_late_favourite_load | confidence_score | 0.8940 | 0.8984 | 0.0044 | 0.8700..0.9146 | 0.8690..0.9205 |
| br2_late_favourite_load | risk_score | 0.3743 | 0.3911 | 0.0168 | 0.3348..0.4184 | 0.3600..0.4372 |
| br2_late_favourite_load | regime_whipsaw_score | 0.2829 | 0.2437 | -0.0392 | 0.2371..0.3638 | 0.2178..0.2779 |
| br2_late_favourite_load | regime_path_efficiency | 0.1368 | 0.1322 | -0.0046 | 0.0768..0.2111 | 0.0861..0.1988 |
| br2_late_favourite_load | regime_reversal_pressure | 0.3000 | 0.2800 | -0.0200 | 0.2600..0.3600 | 0.2400..0.3400 |
| br2_late_favourite_load | regime_sign_flip_rate | 0.4286 | 0.4000 | -0.0286 | 0.3714..0.4857 | 0.3429..0.4571 |
| br2_late_favourite_load | regime_realized_vol_180s_bps | 1.9921 | 1.5861 | -0.4060 | 1.5322..2.7708 | 1.3824..1.8590 |
| br2_late_confirm | market_yes_range_so_far | 0.4350 | 0.4400 | 0.0050 | 0.3900..0.4700 | 0.3950..0.4700 |
| br2_late_confirm | side_model_p | 0.7546 | 0.7593 | 0.0047 | 0.7174..0.8316 | 0.6957..0.8782 |
| br2_late_confirm | side_edge_vs_fill | 0.0649 | 0.0628 | -0.0021 | 0.0386..0.0989 | 0.0370..0.0871 |
| br2_late_confirm | confidence_score | 0.8880 | 0.8845 | -0.0035 | 0.8052..0.9141 | 0.8093..0.9106 |
| br2_late_confirm | risk_score | 0.4434 | 0.4698 | 0.0264 | 0.3808..0.5021 | 0.4153..0.5289 |
| br2_late_confirm | regime_whipsaw_score | 0.2624 | 0.2461 | -0.0163 | 0.2274..0.3040 | 0.2164..0.2808 |
| br2_late_confirm | regime_path_efficiency | 0.1020 | 0.1157 | 0.0137 | 0.0503..0.1859 | 0.0610..0.1917 |
| br2_late_confirm | regime_reversal_pressure | 0.3000 | 0.2800 | -0.0200 | 0.2600..0.3600 | 0.2400..0.3400 |
| br2_late_confirm | regime_sign_flip_rate | 0.4286 | 0.4000 | -0.0286 | 0.3429..0.4857 | 0.3429..0.4571 |
| br2_late_confirm | regime_realized_vol_180s_bps | 1.6370 | 1.5033 | -0.1338 | 1.3895..1.9832 | 1.3368..1.8083 |
| br2_high_skew_load | market_yes_range_so_far | 0.5300 | 0.5600 | 0.0300 | 0.4600..0.6100 | 0.4400..0.6400 |
| br2_high_skew_load | side_model_p | 0.8909 | 0.8941 | 0.0031 | 0.8729..0.9019 | 0.8828..0.9022 |
| br2_high_skew_load | side_edge_vs_fill | 0.1079 | 0.1005 | -0.0073 | 0.0985..0.1226 | 0.0955..0.1142 |
| br2_high_skew_load | confidence_score | 0.8990 | 0.9029 | 0.0039 | 0.8787..0.9189 | 0.8767..0.9238 |
| br2_high_skew_load | risk_score | 0.3552 | 0.3778 | 0.0227 | 0.3173..0.3979 | 0.3366..0.4241 |
| br2_high_skew_load | regime_whipsaw_score | 0.2709 | 0.2463 | -0.0246 | 0.2294..0.3403 | 0.2150..0.2959 |
| br2_high_skew_load | regime_path_efficiency | 0.1610 | 0.1521 | -0.0089 | 0.0914..0.2474 | 0.0952..0.2020 |
| br2_high_skew_load | regime_reversal_pressure | 0.3000 | 0.2800 | -0.0200 | 0.2600..0.3600 | 0.2400..0.3400 |
| br2_high_skew_load | regime_sign_flip_rate | 0.4000 | 0.4000 | 0.0000 | 0.3429..0.4857 | 0.3429..0.4571 |
| br2_high_skew_load | regime_realized_vol_180s_bps | 1.9084 | 1.6390 | -0.2694 | 1.5253..2.6364 | 1.3835..1.9717 |

## Current Actionable Interpretation

- Do not use final mid-wide range directly; it is a resolved-market diagnostic.
- The next live-safe improvement should be a regime/gated sizing model trained to reduce late-confirm and late-favourite exposure only when replay-time features imply a late break is likely to fail.
- Fixed hard gates are risky because the same slices that lose in the last window often made money in the early window.
- The post-fill rerun should confirm whether the last-window mid-wide losses are the same `crossed_mid_after_fill` path seen in the early checkpoint.
