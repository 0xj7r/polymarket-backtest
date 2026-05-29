# BTC5m Recency And Regime Report

Generated from `s3://pm-research-backtest-prod/results/20260528T225810Z-portfolio-grid-52322/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8/markets.jsonl`.
Artifact range: `2026-02-27T15:40:00+00:00` to `2026-05-20T23:55:00+00:00`.

## Window Summary

| Window | Markets | Active | Fills | PnL | Start | End | Return | Daily | Max DD | Active Rate |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| first_third | 7901 | 1191 | 2374 | $5072.67 | $1000.00 | $6072.67 | 507.27% | 6.796% | 23.70% | 15.07% |
| middle_third | 7902 | 569 | 943 | $4144.62 | $6072.67 | $10217.29 | 68.25% | 1.914% | 12.14% | 7.20% |
| last_third | 7902 | 256 | 406 | $-227.08 | $10217.29 | $9990.21 | -2.22% | -0.082% | 11.05% | 3.24% |
| first_30d | 8640 | 1214 | 2413 | $5394.12 | $1000.00 | $6394.12 | 539.41% | 6.380% | 23.70% | 14.05% |
| last_30d | 8633 | 335 | 549 | $20.80 | $9969.41 | $9990.21 | 0.21% | 0.007% | 12.81% | 3.88% |
| last_14d | 4025 | 100 | 147 | $-200.77 | $10190.98 | $9990.21 | -1.97% | -0.142% | 6.97% | 2.48% |
| last_7d | 2013 | 61 | 87 | $-24.27 | $10014.48 | $9990.21 | -0.24% | -0.035% | 5.15% | 3.03% |

## Fill Tag Drift

| Window | Tag | Fills | PnL | Cost | Wins | Win Rate |
|---|---|---:|---:|---:|---:|---:|
| first_30d | br2_convex_tail | 109 | $-26.77 | $201.75 | 10 | 9.17% |
| first_30d | br2_high_skew_load | 577 | $616.36 | $10375.43 | 464 | 80.42% |
| first_30d | br2_late_confirm | 810 | $2385.73 | $34086.68 | 609 | 75.19% |
| first_30d | br2_late_favourite_load | 917 | $2418.81 | $36335.69 | 747 | 81.46% |
| last_30d | br2_convex_tail | 40 | $3.10 | $110.76 | 1 | 2.50% |
| last_30d | br2_high_skew_load | 171 | $171.01 | $8331.19 | 137 | 80.12% |
| last_30d | br2_late_confirm | 131 | $-85.67 | $15790.04 | 90 | 68.70% |
| last_30d | br2_late_favourite_load | 207 | $-67.65 | $13158.47 | 151 | 72.95% |
| last_14d | br2_convex_tail | 12 | $-23.80 | $23.80 | 0 | 0.00% |
| last_14d | br2_high_skew_load | 41 | $-6.12 | $1847.81 | 32 | 78.05% |
| last_14d | br2_late_confirm | 34 | $40.95 | $3836.99 | 23 | 67.65% |
| last_14d | br2_late_favourite_load | 60 | $-211.80 | $4074.85 | 41 | 68.33% |
| last_7d | br2_convex_tail | 8 | $-17.02 | $17.02 | 0 | 0.00% |
| last_7d | br2_high_skew_load | 23 | $73.15 | $1046.34 | 19 | 82.61% |
| last_7d | br2_late_confirm | 17 | $-263.20 | $1739.88 | 10 | 58.82% |
| last_7d | br2_late_favourite_load | 39 | $182.79 | $3221.33 | 31 | 79.49% |

## Recent Regime Bins

Recent bins use fill-level PnL in the last window. They are diagnostic, not a promotion rule.

### Deterministic Labels

| Tag | Regime | Fills | PnL | Cost | Wins | Win Rate |
|---|---|---:|---:|---:|---:|---:|
| br2_convex_tail | wide_range | 30 | $34.39 | $79.47 | 1 | 3.33% |
| br2_high_skew_load | wide_range | 93 | $-101.03 | $4413.53 | 73 | 78.49% |
| br2_high_skew_load | wide_range_chop | 11 | $99.28 | $567.92 | 10 | 90.91% |
| br2_high_skew_load | neutral | 63 | $102.77 | $3120.72 | 50 | 79.37% |
| br2_late_confirm | neutral | 119 | $247.71 | $14331.20 | 83 | 69.75% |
| br2_late_favourite_load | neutral | 106 | $-720.82 | $6892.77 | 70 | 66.04% |
| br2_late_favourite_load | wide_range | 90 | $548.51 | $5561.85 | 72 | 80.00% |

### Quantile Feature Bins

| Tag | Feature | Range | Fills | PnL | PnL/Fill | Win Rate |
|---|---|---:|---:|---:|---:|---:|
| br2_convex_tail | price | -inf..0.0601 | 11 | $-25.97 | $-2.36 | 0.00% |
| br2_convex_tail | price | 0.0601..0.0701 | 15 | $69.54 | $4.64 | 6.67% |
| br2_convex_tail | price | 0.0701..0.0701 | 17 | $-40.25 | $-2.37 | 0.00% |
| br2_convex_tail | price | 0.0701..inf | 16 | $-45.90 | $-2.87 | 0.00% |
| br2_convex_tail | side_model_p | -inf..0.0600 | 30 | $24.27 | $0.81 | 3.33% |
| br2_convex_tail | side_model_p | 0.0600..0.0600 | 30 | $24.27 | $0.81 | 3.33% |
| br2_convex_tail | side_model_p | 0.0600..0.0600 | 30 | $24.27 | $0.81 | 3.33% |
| br2_convex_tail | side_model_p | 0.0600..inf | 40 | $3.10 | $0.08 | 2.50% |
| br2_convex_tail | side_edge_vs_fill | -inf..-0.0101 | 14 | $-42.15 | $-3.01 | 0.00% |
| br2_convex_tail | side_edge_vs_fill | -0.0101..-0.0001 | 20 | $54.02 | $2.70 | 5.00% |
| br2_convex_tail | side_edge_vs_fill | -0.0001..0.0100 | 11 | $-27.27 | $-2.48 | 0.00% |
| br2_convex_tail | side_edge_vs_fill | 0.0100..inf | 12 | $-28.82 | $-2.40 | 0.00% |
| br2_convex_tail | confidence_score | -inf..0.8746 | 11 | $-25.74 | $-2.34 | 0.00% |
| br2_convex_tail | confidence_score | 0.8746..0.8920 | 11 | $74.51 | $6.77 | 9.09% |
| br2_convex_tail | confidence_score | 0.8920..0.9110 | 10 | $-29.42 | $-2.94 | 0.00% |
| br2_convex_tail | confidence_score | 0.9110..inf | 11 | $-23.01 | $-2.09 | 0.00% |
| br2_convex_tail | risk_score | -inf..0.5809 | 11 | $-36.39 | $-3.31 | 0.00% |
| br2_convex_tail | risk_score | 0.5809..0.6172 | 11 | $-27.72 | $-2.52 | 0.00% |
| br2_convex_tail | risk_score | 0.6172..0.6572 | 10 | $90.89 | $9.09 | 10.00% |
| br2_convex_tail | risk_score | 0.6572..inf | 11 | $-26.45 | $-2.40 | 0.00% |
| br2_convex_tail | market_yes_range_so_far | -inf..0.5800 | 11 | $78.04 | $7.09 | 9.09% |
| br2_convex_tail | market_yes_range_so_far | 0.5800..0.6900 | 11 | $-37.19 | $-3.38 | 0.00% |
| br2_convex_tail | market_yes_range_so_far | 0.6900..0.8100 | 10 | $-16.18 | $-1.62 | 0.00% |
| br2_convex_tail | market_yes_range_so_far | 0.8100..inf | 11 | $-26.06 | $-2.37 | 0.00% |
| br2_convex_tail | seconds_to_close | -inf..25.5580 | 11 | $-25.76 | $-2.34 | 0.00% |
| br2_convex_tail | seconds_to_close | 25.5580..38.1500 | 11 | $-24.44 | $-2.22 | 0.00% |
| br2_convex_tail | seconds_to_close | 38.1500..59.5670 | 10 | $-39.47 | $-3.95 | 0.00% |
| br2_convex_tail | seconds_to_close | 59.5670..inf | 11 | $82.56 | $7.51 | 9.09% |
| br2_convex_tail | regime_whipsaw_score | -inf..0.1965 | 11 | $-33.35 | $-3.03 | 0.00% |
| br2_convex_tail | regime_whipsaw_score | 0.1965..0.2184 | 11 | $-22.04 | $-2.00 | 0.00% |
| br2_convex_tail | regime_whipsaw_score | 0.2184..0.2574 | 10 | $83.67 | $8.37 | 10.00% |
| br2_convex_tail | regime_whipsaw_score | 0.2574..inf | 11 | $-32.97 | $-3.00 | 0.00% |
| br2_convex_tail | regime_path_efficiency | -inf..0.1678 | 11 | $-40.10 | $-3.65 | 0.00% |
| br2_convex_tail | regime_path_efficiency | 0.1678..0.2986 | 11 | $80.95 | $7.36 | 9.09% |
| br2_convex_tail | regime_path_efficiency | 0.2986..0.3400 | 10 | $-22.09 | $-2.21 | 0.00% |
| br2_convex_tail | regime_path_efficiency | 0.3400..inf | 11 | $-25.38 | $-2.31 | 0.00% |
| br2_convex_tail | regime_reversal_pressure | -inf..0.2600 | 19 | $65.03 | $3.42 | 5.26% |
| br2_convex_tail | regime_reversal_pressure | 0.2600..0.2800 | 14 | $76.32 | $5.45 | 7.14% |
| br2_convex_tail | regime_reversal_pressure | 0.2800..0.3000 | 13 | $-38.99 | $-3.00 | 0.00% |
| br2_convex_tail | regime_reversal_pressure | 0.3000..inf | 16 | $-45.29 | $-2.83 | 0.00% |
| br2_convex_tail | regime_sign_flip_rate | -inf..0.3429 | 11 | $-28.91 | $-2.63 | 0.00% |
| br2_convex_tail | regime_sign_flip_rate | 0.3429..0.4000 | 17 | $66.21 | $3.89 | 5.88% |
| br2_convex_tail | regime_sign_flip_rate | 0.4000..0.4286 | 13 | $-38.99 | $-3.00 | 0.00% |
| br2_convex_tail | regime_sign_flip_rate | 0.4286..inf | 15 | $-44.31 | $-2.95 | 0.00% |
| br2_convex_tail | regime_realized_vol_180s_bps | -inf..1.3183 | 11 | $-35.20 | $-3.20 | 0.00% |
| br2_convex_tail | regime_realized_vol_180s_bps | 1.3183..1.5262 | 11 | $83.15 | $7.56 | 9.09% |
| br2_convex_tail | regime_realized_vol_180s_bps | 1.5262..1.9689 | 10 | $-24.27 | $-2.43 | 0.00% |
| br2_convex_tail | regime_realized_vol_180s_bps | 1.9689..inf | 11 | $-26.09 | $-2.37 | 0.00% |
| br2_convex_tail | volatility_range | -inf..0.6940 | 11 | $78.31 | $7.12 | 9.09% |
| br2_convex_tail | volatility_range | 0.6940..0.9200 | 13 | $-33.85 | $-2.60 | 0.00% |
| br2_convex_tail | volatility_range | 0.9200..0.9700 | 13 | $-39.29 | $-3.02 | 0.00% |
| br2_convex_tail | volatility_range | 0.9700..inf | 11 | $-28.35 | $-2.58 | 0.00% |
| br2_high_skew_load | price | -inf..0.7574 | 43 | $-8.59 | $-0.20 | 69.77% |
| br2_high_skew_load | price | 0.7574..0.7912 | 63 | $266.08 | $4.22 | 85.71% |
| br2_high_skew_load | price | 0.7912..0.8012 | 53 | $-305.85 | $-5.77 | 73.58% |
| br2_high_skew_load | price | 0.8012..inf | 58 | $29.07 | $0.50 | 84.48% |
| br2_high_skew_load | side_model_p | -inf..0.8828 | 43 | $9.73 | $0.23 | 72.09% |
| br2_high_skew_load | side_model_p | 0.8828..0.8941 | 44 | $199.56 | $4.54 | 86.36% |
| br2_high_skew_load | side_model_p | 0.8941..0.9022 | 44 | $76.20 | $1.73 | 81.82% |
| br2_high_skew_load | side_model_p | 0.9022..inf | 43 | $-69.28 | $-1.61 | 81.40% |
| br2_high_skew_load | side_edge_vs_fill | -inf..0.0955 | 43 | $19.84 | $0.46 | 83.72% |
| br2_high_skew_load | side_edge_vs_fill | 0.0955..0.1005 | 44 | $17.30 | $0.39 | 79.55% |
| br2_high_skew_load | side_edge_vs_fill | 0.1005..0.1142 | 44 | $161.38 | $3.67 | 81.82% |
| br2_high_skew_load | side_edge_vs_fill | 0.1142..inf | 43 | $-47.43 | $-1.10 | 74.42% |
| br2_high_skew_load | confidence_score | -inf..0.8767 | 43 | $211.57 | $4.92 | 86.05% |
| br2_high_skew_load | confidence_score | 0.8767..0.9029 | 44 | $156.05 | $3.55 | 84.09% |
| br2_high_skew_load | confidence_score | 0.9029..0.9238 | 44 | $26.38 | $0.60 | 77.27% |
| br2_high_skew_load | confidence_score | 0.9238..inf | 43 | $-247.54 | $-5.76 | 72.09% |
| br2_high_skew_load | risk_score | -inf..0.3366 | 43 | $-4.11 | $-0.10 | 79.07% |
| br2_high_skew_load | risk_score | 0.3366..0.3778 | 44 | $-37.21 | $-0.85 | 77.27% |
| br2_high_skew_load | risk_score | 0.3778..0.4241 | 44 | $-41.41 | $-0.94 | 77.27% |
| br2_high_skew_load | risk_score | 0.4241..inf | 43 | $224.87 | $5.23 | 86.05% |
| br2_high_skew_load | market_yes_range_so_far | -inf..0.4400 | 47 | $232.32 | $4.94 | 82.98% |
| br2_high_skew_load | market_yes_range_so_far | 0.4400..0.5600 | 46 | $-48.09 | $-1.05 | 78.26% |
| br2_high_skew_load | market_yes_range_so_far | 0.5600..0.6400 | 46 | $-237.93 | $-5.17 | 71.74% |
| br2_high_skew_load | market_yes_range_so_far | 0.6400..inf | 44 | $231.23 | $5.26 | 88.64% |
| br2_high_skew_load | seconds_to_close | -inf..69.6360 | 43 | $268.79 | $6.25 | 86.05% |
| br2_high_skew_load | seconds_to_close | 69.6360..92.4790 | 44 | $-37.86 | $-0.86 | 77.27% |
| br2_high_skew_load | seconds_to_close | 92.4790..110.5950 | 44 | $22.22 | $0.50 | 81.82% |
| br2_high_skew_load | seconds_to_close | 110.5950..inf | 43 | $-40.99 | $-0.95 | 76.74% |
| br2_high_skew_load | regime_whipsaw_score | -inf..0.2150 | 43 | $-151.58 | $-3.53 | 74.42% |
| br2_high_skew_load | regime_whipsaw_score | 0.2150..0.2463 | 44 | $-143.65 | $-3.26 | 72.73% |
| br2_high_skew_load | regime_whipsaw_score | 0.2463..0.2959 | 44 | $140.18 | $3.19 | 84.09% |
| br2_high_skew_load | regime_whipsaw_score | 0.2959..inf | 43 | $283.57 | $6.59 | 88.37% |
| br2_high_skew_load | regime_path_efficiency | -inf..0.0952 | 43 | $-22.23 | $-0.52 | 76.74% |
| br2_high_skew_load | regime_path_efficiency | 0.0952..0.1521 | 44 | $131.49 | $2.99 | 84.09% |
| br2_high_skew_load | regime_path_efficiency | 0.1521..0.2020 | 44 | $46.74 | $1.06 | 79.55% |
| br2_high_skew_load | regime_path_efficiency | 0.2020..inf | 43 | $-12.85 | $-0.30 | 79.07% |
| br2_high_skew_load | regime_reversal_pressure | -inf..0.2400 | 53 | $-299.89 | $-5.66 | 71.70% |
| br2_high_skew_load | regime_reversal_pressure | 0.2400..0.2800 | 55 | $66.18 | $1.20 | 78.18% |
| br2_high_skew_load | regime_reversal_pressure | 0.2800..0.3400 | 67 | $398.98 | $5.95 | 86.57% |
| br2_high_skew_load | regime_reversal_pressure | 0.3400..inf | 47 | $157.05 | $3.34 | 85.11% |
| br2_high_skew_load | regime_sign_flip_rate | -inf..0.3429 | 58 | $-378.02 | $-6.52 | 70.69% |
| br2_high_skew_load | regime_sign_flip_rate | 0.3429..0.4000 | 60 | $52.25 | $0.87 | 78.33% |
| br2_high_skew_load | regime_sign_flip_rate | 0.4000..0.4571 | 63 | $237.21 | $3.77 | 82.54% |
| br2_high_skew_load | regime_sign_flip_rate | 0.4571..inf | 49 | $386.30 | $7.88 | 89.80% |
| br2_high_skew_load | regime_realized_vol_180s_bps | -inf..1.3835 | 43 | $88.20 | $2.05 | 79.07% |
| br2_high_skew_load | regime_realized_vol_180s_bps | 1.3835..1.6390 | 44 | $-102.18 | $-2.32 | 77.27% |
| br2_high_skew_load | regime_realized_vol_180s_bps | 1.6390..1.9717 | 44 | $-150.91 | $-3.43 | 75.00% |
| br2_high_skew_load | regime_realized_vol_180s_bps | 1.9717..inf | 43 | $291.98 | $6.79 | 88.37% |
| br2_high_skew_load | volatility_range | -inf..0.7850 | 43 | $285.89 | $6.65 | 86.05% |
| br2_high_skew_load | volatility_range | 0.7850..0.9050 | 44 | $-497.59 | $-11.31 | 61.36% |
| br2_high_skew_load | volatility_range | 0.9050..0.9700 | 60 | $396.69 | $6.61 | 90.00% |
| br2_high_skew_load | volatility_range | 0.9700..inf | 64 | $403.02 | $6.30 | 89.06% |
| br2_late_confirm | price | -inf..0.6208 | 33 | $-152.67 | $-4.63 | 57.58% |
| br2_late_confirm | price | 0.6208..0.6791 | 34 | $613.82 | $18.05 | 67.65% |
| br2_late_confirm | price | 0.6791..0.7912 | 34 | $-539.47 | $-15.87 | 64.71% |
| br2_late_confirm | price | 0.7912..inf | 33 | $-18.05 | $-0.55 | 84.85% |
| br2_late_confirm | side_model_p | -inf..0.6957 | 33 | $-345.47 | $-10.47 | 57.58% |
| br2_late_confirm | side_model_p | 0.6957..0.7593 | 34 | $559.46 | $16.45 | 67.65% |
| br2_late_confirm | side_model_p | 0.7593..0.8782 | 34 | $-404.39 | $-11.89 | 61.76% |
| br2_late_confirm | side_model_p | 0.8782..inf | 33 | $-118.72 | $-3.60 | 84.85% |
| br2_late_confirm | side_edge_vs_fill | -inf..0.0370 | 33 | $-48.08 | $-1.46 | 78.79% |
| br2_late_confirm | side_edge_vs_fill | 0.0370..0.0628 | 34 | $128.87 | $3.79 | 73.53% |
| br2_late_confirm | side_edge_vs_fill | 0.0628..0.0871 | 34 | $-224.28 | $-6.60 | 58.82% |
| br2_late_confirm | side_edge_vs_fill | 0.0871..inf | 33 | $-46.67 | $-1.41 | 63.64% |
| br2_late_confirm | confidence_score | -inf..0.8093 | 33 | $-605.36 | $-18.34 | 60.61% |
| br2_late_confirm | confidence_score | 0.8093..0.8845 | 34 | $334.73 | $9.85 | 70.59% |
| br2_late_confirm | confidence_score | 0.8845..0.9106 | 34 | $16.57 | $0.49 | 67.65% |
| br2_late_confirm | confidence_score | 0.9106..inf | 33 | $136.80 | $4.15 | 75.76% |
| br2_late_confirm | risk_score | -inf..0.4153 | 33 | $-28.57 | $-0.87 | 66.67% |
| br2_late_confirm | risk_score | 0.4153..0.4698 | 34 | $-49.18 | $-1.45 | 61.76% |
| br2_late_confirm | risk_score | 0.4698..0.5289 | 34 | $-102.50 | $-3.01 | 70.59% |
| br2_late_confirm | risk_score | 0.5289..inf | 33 | $158.16 | $4.79 | 75.76% |
| br2_late_confirm | market_yes_range_so_far | -inf..0.3950 | 33 | $-416.45 | $-12.62 | 63.64% |
| br2_late_confirm | market_yes_range_so_far | 0.3950..0.4400 | 35 | $-62.61 | $-1.79 | 68.57% |
| br2_late_confirm | market_yes_range_so_far | 0.4400..0.4700 | 35 | $495.55 | $14.16 | 80.00% |
| br2_late_confirm | market_yes_range_so_far | 0.4700..inf | 35 | $42.93 | $1.23 | 65.71% |
| br2_late_confirm | seconds_to_close | -inf..41.5180 | 33 | $236.22 | $7.16 | 75.76% |
| br2_late_confirm | seconds_to_close | 41.5180..54.1570 | 34 | $118.88 | $3.50 | 64.71% |
| br2_late_confirm | seconds_to_close | 54.1570..57.8200 | 34 | $-476.65 | $-14.02 | 64.71% |
| br2_late_confirm | seconds_to_close | 57.8200..inf | 33 | $159.02 | $4.82 | 72.73% |
| br2_late_confirm | regime_whipsaw_score | -inf..0.2164 | 33 | $-119.86 | $-3.63 | 72.73% |
| br2_late_confirm | regime_whipsaw_score | 0.2164..0.2461 | 34 | $-451.25 | $-13.27 | 61.76% |
| br2_late_confirm | regime_whipsaw_score | 0.2461..0.2808 | 34 | $689.70 | $20.29 | 76.47% |
| br2_late_confirm | regime_whipsaw_score | 0.2808..inf | 33 | $-65.25 | $-1.98 | 66.67% |
| br2_late_confirm | regime_path_efficiency | -inf..0.0610 | 33 | $-125.96 | $-3.82 | 69.70% |
| br2_late_confirm | regime_path_efficiency | 0.0610..0.1157 | 34 | $893.23 | $26.27 | 79.41% |
| br2_late_confirm | regime_path_efficiency | 0.1157..0.1917 | 34 | $-109.27 | $-3.21 | 64.71% |
| br2_late_confirm | regime_path_efficiency | 0.1917..inf | 33 | $-399.65 | $-12.11 | 63.64% |
| br2_late_confirm | regime_reversal_pressure | -inf..0.2400 | 36 | $527.14 | $14.64 | 80.56% |
| br2_late_confirm | regime_reversal_pressure | 0.2400..0.2800 | 45 | $-595.11 | $-13.22 | 62.22% |
| br2_late_confirm | regime_reversal_pressure | 0.2800..0.3400 | 50 | $-608.40 | $-12.17 | 60.00% |
| br2_late_confirm | regime_reversal_pressure | 0.3400..inf | 43 | $699.90 | $16.28 | 76.74% |
| br2_late_confirm | regime_sign_flip_rate | -inf..0.3429 | 41 | $447.79 | $10.92 | 78.05% |
| br2_late_confirm | regime_sign_flip_rate | 0.3429..0.4000 | 48 | $-396.56 | $-8.26 | 64.58% |
| br2_late_confirm | regime_sign_flip_rate | 0.4000..0.4571 | 42 | $-1091.84 | $-26.00 | 52.38% |
| br2_late_confirm | regime_sign_flip_rate | 0.4571..inf | 45 | $-165.46 | $-3.68 | 66.67% |
| br2_late_confirm | regime_realized_vol_180s_bps | -inf..1.3368 | 33 | $-425.43 | $-12.89 | 63.64% |
| br2_late_confirm | regime_realized_vol_180s_bps | 1.3368..1.5033 | 34 | $311.02 | $9.15 | 76.47% |
| br2_late_confirm | regime_realized_vol_180s_bps | 1.5033..1.8083 | 34 | $495.51 | $14.57 | 76.47% |
| br2_late_confirm | regime_realized_vol_180s_bps | 1.8083..inf | 33 | $-440.99 | $-13.36 | 57.58% |
| br2_late_confirm | volatility_range | -inf..0.7100 | 33 | $474.63 | $14.38 | 84.85% |
| br2_late_confirm | volatility_range | 0.7100..0.9200 | 36 | $-548.21 | $-15.23 | 52.78% |
| br2_late_confirm | volatility_range | 0.9200..0.9700 | 43 | $-549.51 | $-12.78 | 62.79% |
| br2_late_confirm | volatility_range | 0.9700..inf | 39 | $688.70 | $17.66 | 79.49% |
| br2_late_favourite_load | price | -inf..0.7311 | 68 | $-233.12 | $-3.43 | 66.18% |
| br2_late_favourite_load | price | 0.7311..0.7611 | 60 | $-415.91 | $-6.93 | 65.00% |
| br2_late_favourite_load | price | 0.7611..0.7912 | 65 | $609.85 | $9.38 | 84.62% |
| br2_late_favourite_load | price | 0.7912..inf | 56 | $-425.76 | $-7.60 | 69.64% |
| br2_late_favourite_load | side_model_p | -inf..0.8600 | 53 | $-118.71 | $-2.24 | 66.04% |
| br2_late_favourite_load | side_model_p | 0.8600..0.8867 | 52 | $12.08 | $0.23 | 71.15% |
| br2_late_favourite_load | side_model_p | 0.8867..0.8984 | 52 | $418.08 | $8.04 | 82.69% |
| br2_late_favourite_load | side_model_p | 0.8984..inf | 53 | $-325.36 | $-6.14 | 71.70% |
| br2_late_favourite_load | side_edge_vs_fill | -inf..0.0986 | 53 | $171.92 | $3.24 | 81.13% |
| br2_late_favourite_load | side_edge_vs_fill | 0.0986..0.1099 | 52 | $-288.85 | $-5.55 | 67.31% |
| br2_late_favourite_load | side_edge_vs_fill | 0.1099..0.1334 | 52 | $390.29 | $7.51 | 73.08% |
| br2_late_favourite_load | side_edge_vs_fill | 0.1334..inf | 53 | $-318.26 | $-6.00 | 69.81% |
| br2_late_favourite_load | confidence_score | -inf..0.8690 | 53 | $-250.25 | $-4.72 | 71.70% |
| br2_late_favourite_load | confidence_score | 0.8690..0.8984 | 52 | $-188.61 | $-3.63 | 71.15% |
| br2_late_favourite_load | confidence_score | 0.8984..0.9205 | 52 | $218.69 | $4.21 | 73.08% |
| br2_late_favourite_load | confidence_score | 0.9205..inf | 53 | $-18.21 | $-0.34 | 75.47% |
| br2_late_favourite_load | risk_score | -inf..0.3600 | 53 | $637.15 | $12.02 | 83.02% |
| br2_late_favourite_load | risk_score | 0.3600..0.3911 | 52 | $-448.81 | $-8.63 | 63.46% |
| br2_late_favourite_load | risk_score | 0.3911..0.4372 | 52 | $43.22 | $0.83 | 75.00% |
| br2_late_favourite_load | risk_score | 0.4372..inf | 53 | $-234.12 | $-4.42 | 71.70% |
| br2_late_favourite_load | market_yes_range_so_far | -inf..0.4000 | 54 | $-6.38 | $-0.12 | 66.67% |
| br2_late_favourite_load | market_yes_range_so_far | 0.4000..0.4800 | 54 | $-587.75 | $-10.88 | 64.81% |
| br2_late_favourite_load | market_yes_range_so_far | 0.4800..0.5500 | 54 | $-45.07 | $-0.83 | 75.93% |
| br2_late_favourite_load | market_yes_range_so_far | 0.5500..inf | 55 | $400.02 | $7.27 | 80.00% |
| br2_late_favourite_load | seconds_to_close | -inf..77.5500 | 53 | $-217.35 | $-4.10 | 71.70% |
| br2_late_favourite_load | seconds_to_close | 77.5500..91.3280 | 52 | $-34.77 | $-0.67 | 71.15% |
| br2_late_favourite_load | seconds_to_close | 91.3280..108.5840 | 52 | $232.25 | $4.47 | 75.00% |
| br2_late_favourite_load | seconds_to_close | 108.5840..inf | 53 | $-116.98 | $-2.21 | 73.58% |
| br2_late_favourite_load | regime_whipsaw_score | -inf..0.2178 | 53 | $-415.87 | $-7.85 | 71.70% |
| br2_late_favourite_load | regime_whipsaw_score | 0.2178..0.2437 | 52 | $-12.46 | $-0.24 | 65.38% |
| br2_late_favourite_load | regime_whipsaw_score | 0.2437..0.2779 | 52 | $184.53 | $3.55 | 76.92% |
| br2_late_favourite_load | regime_whipsaw_score | 0.2779..inf | 53 | $220.45 | $4.16 | 77.36% |
| br2_late_favourite_load | regime_path_efficiency | -inf..0.0861 | 53 | $-157.57 | $-2.97 | 69.81% |
| br2_late_favourite_load | regime_path_efficiency | 0.0861..0.1322 | 52 | $183.75 | $3.53 | 75.00% |
| br2_late_favourite_load | regime_path_efficiency | 0.1322..0.1988 | 52 | $-101.71 | $-1.96 | 67.31% |
| br2_late_favourite_load | regime_path_efficiency | 0.1988..inf | 53 | $97.80 | $1.85 | 79.25% |
| br2_late_favourite_load | regime_reversal_pressure | -inf..0.2400 | 64 | $-151.05 | $-2.36 | 73.44% |
| br2_late_favourite_load | regime_reversal_pressure | 0.2400..0.2800 | 60 | $-65.22 | $-1.09 | 68.33% |
| br2_late_favourite_load | regime_reversal_pressure | 0.2800..0.3400 | 84 | $12.49 | $0.15 | 72.62% |
| br2_late_favourite_load | regime_reversal_pressure | 0.3400..inf | 57 | $164.62 | $2.89 | 71.93% |
| br2_late_favourite_load | regime_sign_flip_rate | -inf..0.3429 | 67 | $-134.53 | $-2.01 | 73.13% |
| br2_late_favourite_load | regime_sign_flip_rate | 0.3429..0.4000 | 67 | $27.89 | $0.42 | 68.66% |
| br2_late_favourite_load | regime_sign_flip_rate | 0.4000..0.4571 | 73 | $-104.37 | $-1.43 | 69.86% |
| br2_late_favourite_load | regime_sign_flip_rate | 0.4571..inf | 61 | $129.83 | $2.13 | 73.77% |
| br2_late_favourite_load | regime_realized_vol_180s_bps | -inf..1.3824 | 53 | $-128.44 | $-2.42 | 67.92% |
| br2_late_favourite_load | regime_realized_vol_180s_bps | 1.3824..1.5861 | 52 | $-435.58 | $-8.38 | 65.38% |
| br2_late_favourite_load | regime_realized_vol_180s_bps | 1.5861..1.8590 | 52 | $26.57 | $0.51 | 73.08% |
| br2_late_favourite_load | regime_realized_vol_180s_bps | 1.8590..inf | 53 | $607.30 | $11.46 | 86.79% |
| br2_late_favourite_load | volatility_range | -inf..0.7800 | 55 | $477.12 | $8.67 | 81.82% |
| br2_late_favourite_load | volatility_range | 0.7800..0.9300 | 52 | $-1067.18 | $-20.52 | 44.23% |
| br2_late_favourite_load | volatility_range | 0.9300..0.9700 | 68 | $591.20 | $8.69 | 86.76% |
| br2_late_favourite_load | volatility_range | 0.9700..inf | 69 | $518.87 | $7.52 | 85.51% |

## Best Recent Thresholds

Top single-feature recent filters by PnL per fill with minimum fill count.

| Tag | Rule | Fills | PnL | PnL/Fill | Win Rate |
|---|---|---:|---:|---:|---:|
| br2_convex_tail | `volatility_range <= 0.8190` | 13 | $73.45 | $5.65 | 7.69% |
| br2_convex_tail | `market_yes_range_so_far <= 0.6050` | 13 | $71.01 | $5.46 | 7.69% |
| br2_convex_tail | `confidence_score <= 0.8772` | 14 | $74.53 | $5.32 | 7.14% |
| br2_convex_tail | `seconds_to_close >= 44.7710` | 14 | $67.20 | $4.80 | 7.14% |
| br2_convex_tail | `regime_path_efficiency <= 0.2184` | 14 | $61.44 | $4.39 | 7.14% |
| br2_convex_tail | `volatility_range <= 0.8740` | 18 | $67.44 | $3.75 | 5.56% |
| br2_convex_tail | `regime_reversal_pressure <= 0.2600` | 19 | $65.03 | $3.42 | 5.26% |
| br2_convex_tail | `risk_score >= 0.6172` | 20 | $66.17 | $3.31 | 5.00% |
| br2_convex_tail | `regime_sign_flip_rate <= 0.3714` | 20 | $64.05 | $3.20 | 5.00% |
| br2_convex_tail | `side_edge_vs_fill <= -0.0059` | 18 | $53.77 | $2.99 | 5.56% |
| br2_convex_tail | `regime_whipsaw_score >= 0.2184` | 20 | $50.85 | $2.54 | 5.00% |
| br2_convex_tail | `seconds_to_close >= 38.1500` | 20 | $50.29 | $2.51 | 5.00% |
| br2_high_skew_load | `regime_realized_vol_180s_bps >= 2.0706` | 35 | $382.60 | $10.93 | 94.29% |
| br2_high_skew_load | `regime_sign_flip_rate >= 0.4857` | 37 | $308.94 | $8.35 | 91.89% |
| br2_high_skew_load | `seconds_to_close <= 64.4150` | 35 | $278.54 | $7.96 | 88.57% |
| br2_high_skew_load | `volatility_range <= 0.6800` | 27 | $193.45 | $7.16 | 88.89% |
| br2_high_skew_load | `confidence_score <= 0.8740` | 35 | $245.22 | $7.01 | 88.57% |
| br2_high_skew_load | `seconds_to_close <= 76.6430` | 57 | $392.60 | $6.89 | 87.72% |
| br2_high_skew_load | `volatility_range >= 0.9300` | 79 | $514.84 | $6.52 | 89.87% |
| br2_high_skew_load | `volatility_range <= 0.7640` | 39 | $243.96 | $6.26 | 87.18% |
| br2_high_skew_load | `regime_whipsaw_score >= 0.2760` | 57 | $320.77 | $5.63 | 87.72% |
| br2_high_skew_load | `market_yes_range_so_far >= 0.6600` | 35 | $193.90 | $5.54 | 88.57% |
| br2_high_skew_load | `regime_sign_flip_rate >= 0.4000` | 100 | $546.15 | $5.46 | 86.00% |
| br2_high_skew_load | `regime_whipsaw_score >= 0.2261` | 115 | $602.24 | $5.24 | 86.09% |
| br2_late_confirm | `regime_sign_flip_rate >= 0.4857` | 32 | $620.19 | $19.38 | 78.12% |
| br2_late_confirm | `volatility_range <= 0.6940` | 29 | $468.08 | $16.14 | 86.21% |
| br2_late_confirm | `regime_sign_flip_rate >= 0.5143` | 18 | $272.80 | $15.16 | 72.22% |
| br2_late_confirm | `regime_reversal_pressure >= 0.4200` | 23 | $339.16 | $14.75 | 78.26% |
| br2_late_confirm | `risk_score >= 0.5435` | 27 | $371.58 | $13.76 | 81.48% |
| br2_late_confirm | `volatility_range <= 0.7850` | 46 | $573.09 | $12.46 | 80.43% |
| br2_late_confirm | `regime_sign_flip_rate <= 0.3429` | 41 | $447.79 | $10.92 | 78.05% |
| br2_late_confirm | `volatility_range >= 0.9500` | 52 | $565.16 | $10.87 | 76.92% |
| br2_late_confirm | `regime_path_efficiency <= 0.1157` | 66 | $669.01 | $10.14 | 74.24% |
| br2_late_confirm | `volatility_range <= 0.6050` | 18 | $181.59 | $10.09 | 88.89% |
| br2_late_confirm | `seconds_to_close <= 40.2960` | 27 | $269.43 | $9.98 | 77.78% |
| br2_late_confirm | `regime_whipsaw_score >= 0.2461` | 66 | $605.65 | $9.18 | 71.21% |
| br2_late_favourite_load | `volatility_range <= 0.7350` | 41 | $607.29 | $14.81 | 90.24% |
| br2_late_favourite_load | `volatility_range <= 0.6750` | 26 | $343.59 | $13.21 | 88.46% |
| br2_late_favourite_load | `risk_score <= 0.3489` | 42 | $409.91 | $9.76 | 80.95% |
| br2_late_favourite_load | `regime_realized_vol_180s_bps >= 1.9252` | 44 | $396.30 | $9.01 | 84.09% |
| br2_late_favourite_load | `regime_realized_vol_180s_bps >= 1.7512` | 70 | $549.80 | $7.85 | 82.86% |
| br2_late_favourite_load | `side_edge_vs_fill <= 0.0973` | 42 | $299.04 | $7.12 | 85.71% |
| br2_late_favourite_load | `risk_score <= 0.3703` | 69 | $455.12 | $6.60 | 76.81% |
| br2_late_favourite_load | `volatility_range >= 0.9450` | 93 | $612.33 | $6.58 | 83.87% |
| br2_late_favourite_load | `regime_reversal_pressure >= 0.4800` | 17 | $104.13 | $6.13 | 64.71% |
| br2_late_favourite_load | `market_yes_range_so_far >= 0.5150` | 87 | $530.11 | $6.09 | 79.31% |
| br2_late_favourite_load | `market_yes_range_so_far >= 0.5750` | 38 | $225.77 | $5.94 | 81.58% |
| br2_late_favourite_load | `regime_realized_vol_180s_bps >= 1.5861` | 104 | $597.22 | $5.74 | 79.81% |
