# BTC5m Late-Break Feature Contrast

Source: `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
Fills: `1726` late-confirm/favourite fills
Calendar: `2026-02-27T15:45:00+00:00` to `2026-03-29T01:15:00+00:00`
PnL: `$4,906.57`
Toxic fills: `355` (`20.57%`)

This diagnostic contrasts failed late breaks against profitable late breaks using fill-time features only. Post-fill labels are used only to define the offline target.

## By Lane

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| br2_late_confirm | 809 | $2,487.76 | $33,984.65 | 75.28% | 23.36% | 41.90% |
| br2_late_favourite_load | 917 | $2,418.81 | $36,335.69 | 81.46% | 18.10% | 35.55% |

## By Post-Fill Path

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| crossed_mid_after_fill | 665 | $-8,975.91 | $27,183.80 | 46.92% | 53.08% | 100.00% |
| held_side | 827 | $11,696.66 | $34,167.72 | 99.15% | 0.00% | 0.00% |
| moderate_adverse_no_cross | 234 | $2,185.82 | $8,968.83 | 95.73% | 0.85% | 0.00% |

## Feature Contrast: Toxic vs Profitable Non-Toxic Late Breaks

| Feature | Toxic Mean | Profitable Non-Toxic Mean | Difference | Std Diff | Toxic N | Profitable N |
|---|---:|---:|---:|---:|---:|---:|
| price | 0.7033 | 0.7392 | -0.0359 | -0.413 | 355 | 1356 |
| side_model_p | 0.8084 | 0.8378 | -0.0294 | -0.365 | 355 | 1356 |
| risk_score | 0.4209 | 0.4051 | 0.0158 | 0.197 | 355 | 1356 |
| side_edge_vs_fill | 0.1051 | 0.0986 | 0.0065 | 0.135 | 355 | 1356 |
| confidence_score | 0.8702 | 0.8768 | -0.0066 | -0.106 | 355 | 1356 |
| regime_realized_vol_180s_bps | 1.9991 | 2.0932 | -0.0941 | -0.103 | 355 | 1356 |
| regime_whipsaw_score | 0.2884 | 0.2943 | -0.0059 | -0.065 | 355 | 1356 |
| seconds_to_close | 65.2237 | 66.9852 | -1.7615 | -0.059 | 355 | 1356 |
| prior_market_range_7d | 0.6776 | 0.6782 | -0.0006 | -0.054 | 355 | 1356 |
| market_yes_range_so_far | 0.4479 | 0.4515 | -0.0036 | -0.047 | 355 | 1356 |
| prior_market_range_1d | 0.6842 | 0.6829 | 0.0012 | 0.041 | 355 | 1356 |
| regime_reversal_pressure | 0.3283 | 0.3243 | 0.0040 | 0.037 | 355 | 1356 |
| regime_path_efficiency | 0.1397 | 0.1423 | -0.0027 | -0.025 | 355 | 1356 |
| prior_market_range_3d | 0.6804 | 0.6809 | -0.0004 | -0.021 | 355 | 1356 |
| regime_sign_flip_rate | 0.4255 | 0.4241 | 0.0015 | 0.016 | 355 | 1356 |

## Quartiles: price

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| price:q1 | 400 | $1,561.04 | $16,177.34 | 66.50% | 32.25% | 59.25% |
| price:q2 | 414 | $1,139.37 | $12,629.03 | 79.23% | 19.81% | 37.20% |
| price:q3 | 479 | $1,147.11 | $19,537.25 | 81.00% | 18.58% | 37.16% |
| price:q4 | 433 | $1,059.05 | $21,976.72 | 86.37% | 12.70% | 22.17% |

## Quartiles: side_model_p

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_model_p:q1 | 431 | $1,857.22 | $18,080.71 | 69.37% | 29.23% | 54.29% |
| side_model_p:q2 | 431 | $816.99 | $13,703.18 | 76.80% | 22.04% | 40.14% |
| side_model_p:q3 | 431 | $1,617.92 | $17,272.46 | 83.06% | 16.47% | 35.27% |
| side_model_p:q4 | 433 | $614.45 | $21,264.00 | 84.99% | 14.55% | 24.48% |

## Quartiles: risk_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| risk_score:q1 | 431 | $2,675.48 | $21,372.39 | 82.83% | 16.71% | 30.63% |
| risk_score:q2 | 431 | $1,759.84 | $17,386.03 | 81.44% | 17.87% | 39.68% |
| risk_score:q3 | 431 | $62.16 | $14,986.37 | 76.57% | 22.97% | 39.21% |
| risk_score:q4 | 433 | $409.10 | $16,575.56 | 73.44% | 24.71% | 44.57% |

## Quartiles: side_edge_vs_fill

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_edge_vs_fill:q1 | 431 | $181.86 | $18,199.22 | 78.89% | 19.95% | 32.95% |
| side_edge_vs_fill:q2 | 431 | $1,649.21 | $19,523.35 | 80.05% | 18.79% | 37.82% |
| side_edge_vs_fill:q3 | 431 | $1,891.94 | $18,606.14 | 81.44% | 17.87% | 36.66% |
| side_edge_vs_fill:q4 | 433 | $1,183.57 | $13,991.63 | 73.90% | 25.64% | 46.65% |

## Quartiles: confidence_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confidence_score:q1 | 431 | $1,129.28 | $17,189.67 | 76.57% | 22.04% | 40.60% |
| confidence_score:q2 | 431 | $936.78 | $16,924.94 | 79.12% | 20.19% | 35.96% |
| confidence_score:q3 | 431 | $898.53 | $17,291.21 | 79.58% | 19.95% | 37.59% |
| confidence_score:q4 | 433 | $1,941.99 | $18,914.53 | 78.98% | 20.09% | 39.95% |

## Quartiles: regime_realized_vol_180s_bps

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| regime_realized_vol_180s_bps:q1 | 431 | $1,618.08 | $18,972.41 | 78.42% | 20.42% | 38.98% |
| regime_realized_vol_180s_bps:q2 | 431 | $155.69 | $18,327.88 | 74.48% | 24.59% | 42.23% |
| regime_realized_vol_180s_bps:q3 | 431 | $737.95 | $17,248.87 | 77.96% | 20.88% | 40.84% |
| regime_realized_vol_180s_bps:q4 | 433 | $2,394.85 | $15,771.18 | 83.37% | 16.40% | 32.10% |

## Single-Feature Removal Scan

Positive removed PnL means a gate would remove profitable fills. Negative removed PnL is the interesting direction.

| Feature | Direction | Threshold | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| regime_reversal_pressure | ge | 0.5600 | 120 | $4,199.62 | $-22.34 | $22.34 | 25.00% | 48.33% |
| regime_sign_flip_rate | ge | 0.6000 | 67 | $2,692.34 | $92.32 | $-92.32 | 26.87% | 38.81% |
| regime_reversal_pressure | ge | 0.5000 | 152 | $5,334.05 | $99.41 | $-99.41 | 23.03% | 44.08% |
| regime_reversal_pressure | ge | 0.6200 | 67 | $2,242.97 | $142.43 | $-142.43 | 17.91% | 46.27% |
| risk_score | ge | 0.4737 | 348 | $13,437.36 | $169.51 | $-169.51 | 26.44% | 45.11% |
| side_edge_vs_fill | le | 0.0830 | 520 | $22,043.34 | $217.43 | $-217.43 | 21.15% | 35.38% |
| regime_reversal_pressure | ge | 0.4400 | 194 | $7,140.59 | $433.68 | $-433.68 | 21.13% | 41.75% |
| risk_score | ge | 0.3991 | 864 | $31,561.93 | $471.26 | $-471.26 | 23.84% | 41.90% |
| regime_path_efficiency | le | 0.0515 | 346 | $13,621.13 | $476.84 | $-476.84 | 21.97% | 39.60% |
| side_model_p | ge | 0.8976 | 359 | $17,580.09 | $511.07 | $-511.07 | 14.48% | 23.96% |
| confidence_score | le | 0.8430 | 344 | $13,855.97 | $581.91 | $-581.91 | 23.84% | 42.73% |
| regime_path_efficiency | le | 0.0731 | 517 | $20,608.41 | $585.47 | $-585.47 | 22.63% | 40.62% |
| prior_market_range_7d | ge | 0.6826 | 373 | $13,620.33 | $589.95 | $-589.95 | 18.77% | 33.51% |
| prior_market_range_1d | ge | 0.6891 | 372 | $14,165.95 | $600.13 | $-600.13 | 19.89% | 30.91% |
| market_yes_range_so_far | le | 0.3250 | 92 | $3,236.42 | $617.01 | $-617.01 | 29.35% | 42.39% |
| regime_sign_flip_rate | le | 0.2571 | 91 | $3,511.65 | $618.71 | $-618.71 | 12.09% | 40.66% |
| market_yes_range_so_far | ge | 0.5400 | 213 | $7,122.65 | $657.51 | $-657.51 | 17.37% | 37.09% |
| side_edge_vs_fill | le | 0.0940 | 691 | $29,651.80 | $668.52 | $-668.52 | 21.13% | 36.76% |
| prior_market_range_3d | ge | 0.6873 | 375 | $14,302.95 | $731.13 | $-731.13 | 17.33% | 30.67% |
| market_yes_range_so_far | ge | 0.4950 | 446 | $16,867.55 | $736.25 | $-736.25 | 19.73% | 38.79% |
| seconds_to_close | le | 40.5110 | 343 | $12,523.95 | $774.01 | $-774.01 | 20.99% | 36.15% |
| price | ge | 0.7964 | 354 | $17,946.82 | $774.64 | $-774.64 | 12.15% | 19.21% |
| side_model_p | ge | 0.8910 | 531 | $25,654.54 | $779.98 | $-779.98 | 15.44% | 25.99% |
| risk_score | ge | 0.3805 | 1036 | $38,220.91 | $791.94 | $-791.94 | 23.36% | 41.70% |
| risk_score | ge | 0.4444 | 519 | $19,197.16 | $801.08 | $-801.08 | 23.12% | 43.16% |
| regime_whipsaw_score | le | 0.2402 | 519 | $21,933.52 | $801.22 | $-801.22 | 21.97% | 40.27% |
| regime_path_efficiency | le | 0.0945 | 689 | $27,399.36 | $821.63 | $-821.63 | 21.77% | 39.19% |
| prior_market_range_1d | ge | 0.6848 | 563 | $19,004.49 | $832.27 | $-832.27 | 20.78% | 35.17% |
| side_edge_vs_fill | le | 0.0569 | 349 | $14,538.03 | $834.47 | $-834.47 | 16.05% | 28.94% |
| regime_whipsaw_score | le | 0.2244 | 345 | $14,202.20 | $882.82 | $-882.82 | 20.29% | 39.71% |

## Two-Feature Candidate Scan

| Candidate | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confirm_low_edge_reversal | 303 | $12,423.20 | $293.01 | $-293.01 | 20.46% | 36.30% |
| price_high_edge_low | 365 | $17,590.12 | $881.48 | $-881.48 | 12.60% | 22.19% |
| fav_high_price_chop | 311 | $16,381.08 | $1,036.59 | $-1,036.59 | 13.83% | 27.33% |
| price_high_reversal | 319 | $15,160.01 | $1,069.56 | $-1,069.56 | 12.85% | 24.76% |
| obs_mid_high_signflip | 951 | $38,610.47 | $2,320.46 | $-2,320.46 | 20.19% | 38.80% |
| high_reversal_low_eff | 924 | $37,189.16 | $3,363.71 | $-3,363.71 | 19.37% | 38.31% |
| high_signflip_low_eff | 1158 | $47,426.12 | $3,540.53 | $-3,540.53 | 20.29% | 38.77% |
