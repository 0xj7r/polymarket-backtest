# Recent Regime Logistic Gate Report

Source: `s3://pm-research-backtest-prod/results/20260528T225810Z-portfolio-grid-52322/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8/markets.jsonl`
Train fills: `3162` before `2026-04-20T17:20:00+00:00`
Test fills: `561` in final `30` days

## Probability Quality

| Split | Model | Log Loss | Brier |
|---|---|---:|---:|
| train | regime logistic | 0.4859 | 0.1556 |
| test | regime logistic | 0.5475 | 0.1811 |
| test | existing side_model_p | 0.5669 | 0.1856 |

## PnL Gate On Final Window

All final-window fills: `561`, PnL `$-227.94`.
The gate excludes final full-market `volatility_range`; only replay-time fields are used.

| Gate | Fills | PnL | Cost | Wins | Win Rate |
|---|---:|---:|---:|---:|---:|
| existing_model_edge | 474 | $-502.12 | $32390.20 | 348 | 73.42% |
| regime_logistic_edge | 269 | $-255.92 | $18950.76 | 185 | 68.77% |

### Logistic Gate Selected Fills By Tag

| Tag | Fills | PnL | Cost | Wins | Win Rate |
|---|---:|---:|---:|---:|---:|
| br2_convex_tail | 4 | $-8.79 | $8.79 | 0 | 0.00% |
| br2_high_skew_load | 79 | $50.28 | $3970.62 | 60 | 75.95% |
| br2_late_confirm | 54 | $-38.89 | $6224.70 | 33 | 61.11% |
| br2_late_favourite_load | 132 | $-258.52 | $8746.64 | 92 | 69.70% |

## Test Metrics By Fill Tag

| Tag | Fills | PnL | Logistic LL | Existing LL | Logistic Brier | Existing Brier |
|---|---:|---:|---:|---:|---:|---:|
| br2_convex_tail | 40 | $3.10 | 0.1410 | 0.1391 | 0.0274 | 0.0271 |
| br2_high_skew_load | 177 | $277.57 | 0.4966 | 0.5081 | 0.1569 | 0.1592 |
| br2_late_confirm | 133 | $-219.06 | 0.6152 | 0.6106 | 0.2130 | 0.2122 |
| br2_late_favourite_load | 211 | $-289.54 | 0.6246 | 0.6697 | 0.2103 | 0.2210 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| market_yes_range_so_far | -2.1094 |
| side_edge_vs_fill | -1.5509 |
| risk_score | -1.4391 |
| price_x_model_p | 1.3857 |
| price | 1.2874 |
| regime_sign_flip_rate | -1.2195 |
| edge_x_confidence | 1.2167 |
| prior_market_range_1d | -1.1914 |
| prior_market_range_3d | 1.0992 |
| side_model_p | 0.9497 |
| prior_market_range_7d | 0.8270 |
| tag:br2_convex_tail | -0.6995 |
| whipsaw_x_low_efficiency | 0.5995 |
| regime_path_efficiency | 0.5048 |
| regime_whipsaw_score | 0.2627 |
| confidence_score | 0.2067 |
| whipsaw_x_reversal | -0.2041 |
| buy_yes | 0.1510 |
| tag:br2_high_skew_load | 0.1038 |
| regime_realized_vol_180s_bps | 0.0907 |

