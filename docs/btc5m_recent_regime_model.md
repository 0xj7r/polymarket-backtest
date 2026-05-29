# Recent Regime Logistic Gate Report

Source: `s3://pm-research-backtest-prod/results/20260528T225810Z-portfolio-grid-52322/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_mem128_cf8/markets.jsonl`
Train fills: `3162` before `2026-04-20T17:20:00+00:00`
Test fills: `561` in final `30` days

## Probability Quality

| Split | Model | Log Loss | Brier |
|---|---|---:|---:|
| train | regime logistic | 0.4487 | 0.1466 |
| test | regime logistic | 0.5648 | 0.1928 |
| test | existing side_model_p | 0.5669 | 0.1856 |

## PnL Gate On Final Window

All final-window fills: `561`, PnL `$-227.94`.

| Gate | Fills | PnL | Cost | Wins | Win Rate |
|---|---:|---:|---:|---:|---:|
| existing_model_edge | 474 | $-502.12 | $32390.20 | 348 | 73.42% |
| regime_logistic_edge | 159 | $1202.95 | $11681.47 | 123 | 77.36% |

## Test Metrics By Fill Tag

| Tag | Fills | PnL | Logistic LL | Existing LL | Logistic Brier | Existing Brier |
|---|---:|---:|---:|---:|---:|---:|
| br2_convex_tail | 40 | $3.10 | 0.1199 | 0.1391 | 0.0268 | 0.0271 |
| br2_high_skew_load | 177 | $277.57 | 0.5458 | 0.5081 | 0.1803 | 0.1592 |
| br2_late_confirm | 133 | $-219.06 | 0.6220 | 0.6106 | 0.2193 | 0.2122 |
| br2_late_favourite_load | 211 | $-289.54 | 0.6289 | 0.6697 | 0.2179 | 0.2210 |

## Largest Coefficients

| Feature | Coefficient |
|---|---:|
| volatility_range | -5.2955 |
| side_edge_vs_fill | -1.6327 |
| tag:br2_convex_tail | -1.5971 |
| risk_score | -1.5505 |
| price | 1.3365 |
| range_x_reversal | 1.2172 |
| regime_sign_flip_rate | -1.0153 |
| whipsaw_x_reversal | -1.0088 |
| side_model_p | 0.9840 |
| price_x_model_p | 0.7599 |
| edge_x_confidence | 0.6141 |
| regime_reversal_pressure | -0.4125 |
| whipsaw_x_low_efficiency | 0.2715 |
| buy_yes | 0.2196 |
| confidence_score | 0.1604 |
| tag:br2_late_favourite_load | 0.1568 |
| tag:br2_high_skew_load | 0.0980 |
| market_yes_range_so_far | 0.0967 |
| regime_path_efficiency | 0.0962 |
| regime_realized_vol_180s_bps | 0.0698 |

