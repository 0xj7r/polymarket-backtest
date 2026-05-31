# BTC5m Late-Break Feature Contrast

Source: `/tmp/btc5m_postfill_watch_markets.jsonl`
Fills: `2579` late-confirm/favourite fills
Calendar: `2026-02-27T15:45:00+00:00` to `2026-05-20T17:15:00+00:00`
PnL: `$7,106.43`
Toxic fills: `565` (`21.91%`)

This diagnostic contrasts failed late breaks against profitable late breaks using fill-time features only. Post-fill labels are used only to define the offline target.

## By Lane

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| br2_late_confirm | 1187 | $2,379.56 | $76,509.40 | 73.46% | 25.44% | 44.82% |
| br2_late_favourite_load | 1392 | $4,726.87 | $68,650.78 | 80.60% | 18.89% | 36.35% |

## By Post-Fill Path

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| crossed_mid_after_fill | 1038 | $-20,443.00 | $60,835.58 | 45.76% | 54.24% | 100.00% |
| held_side | 1207 | $22,961.49 | $66,430.73 | 99.17% | 0.00% | 0.00% |
| moderate_adverse_no_cross | 334 | $4,587.94 | $17,893.87 | 96.41% | 0.60% | 0.00% |

## Feature Contrast: Toxic vs Profitable Non-Toxic Late Breaks

| Feature | Toxic Mean | Profitable Non-Toxic Mean | Difference | Std Diff | Toxic N | Profitable N |
|---|---:|---:|---:|---:|---:|---:|
| price | 0.7075 | 0.7396 | -0.0321 | -0.380 | 565 | 1994 |
| side_model_p | 0.8090 | 0.8382 | -0.0292 | -0.364 | 565 | 1994 |
| risk_score | 0.4207 | 0.4071 | 0.0136 | 0.171 | 565 | 1994 |
| regime_realized_vol_180s_bps | 1.8817 | 1.9902 | -0.1085 | -0.131 | 565 | 1994 |
| prior_market_range_1d | 0.7322 | 0.7244 | 0.0078 | 0.116 | 565 | 1994 |
| prior_market_range_7d | 0.7273 | 0.7206 | 0.0067 | 0.104 | 565 | 1994 |
| prior_market_range_3d | 0.7300 | 0.7232 | 0.0067 | 0.103 | 565 | 1994 |
| seconds_to_close | 65.5426 | 68.4897 | -2.9471 | -0.101 | 565 | 1994 |
| confidence_score | 0.8721 | 0.8780 | -0.0059 | -0.096 | 565 | 1994 |
| market_yes_range_so_far | 0.4484 | 0.4559 | -0.0075 | -0.096 | 565 | 1994 |
| regime_whipsaw_score | 0.2778 | 0.2845 | -0.0067 | -0.081 | 565 | 1994 |
| side_edge_vs_fill | 0.1015 | 0.0986 | 0.0029 | 0.061 | 565 | 1994 |
| regime_path_efficiency | 0.1386 | 0.1430 | -0.0044 | -0.042 | 565 | 1994 |
| regime_sign_flip_rate | 0.4226 | 0.4193 | 0.0033 | 0.036 | 565 | 1994 |
| regime_reversal_pressure | 0.3234 | 0.3204 | 0.0030 | 0.028 | 565 | 1994 |

## Quartiles: price

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| price:q1 | 644 | $4,726.02 | $39,432.72 | 68.17% | 30.43% | 58.39% |
| price:q2 | 587 | $-664.04 | $25,127.84 | 74.79% | 24.53% | 42.42% |
| price:q3 | 702 | $2,631.46 | $38,229.39 | 81.77% | 17.81% | 36.32% |
| price:q4 | 646 | $412.98 | $42,370.23 | 83.90% | 15.48% | 24.46% |

## Quartiles: side_model_p

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| side_model_p:q1 | 644 | $2,771.74 | $41,568.49 | 68.01% | 30.75% | 56.83% |
| side_model_p:q2 | 645 | $930.81 | $29,618.20 | 75.19% | 23.72% | 42.64% |
| side_model_p:q3 | 644 | $2,217.76 | $33,268.81 | 81.52% | 18.01% | 36.80% |
| side_model_p:q4 | 646 | $1,186.12 | $40,704.69 | 84.52% | 15.17% | 24.77% |

## Quartiles: risk_score

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| risk_score:q1 | 644 | $4,052.60 | $39,924.36 | 82.14% | 17.55% | 34.78% |
| risk_score:q2 | 645 | $2,484.32 | $35,960.31 | 79.07% | 20.31% | 39.84% |
| risk_score:q3 | 644 | $373.80 | $32,665.91 | 75.16% | 24.07% | 40.84% |
| risk_score:q4 | 646 | $195.71 | $36,609.60 | 72.91% | 25.70% | 45.51% |

## Quartiles: regime_realized_vol_180s_bps

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| regime_realized_vol_180s_bps:q1 | 644 | $1,933.02 | $43,068.26 | 76.09% | 22.98% | 43.32% |
| regime_realized_vol_180s_bps:q2 | 645 | $563.60 | $38,074.02 | 74.11% | 24.81% | 44.03% |
| regime_realized_vol_180s_bps:q3 | 644 | $1,131.02 | $34,082.08 | 75.93% | 23.14% | 40.99% |
| regime_realized_vol_180s_bps:q4 | 646 | $3,478.79 | $29,935.83 | 83.13% | 16.72% | 32.66% |

## Quartiles: prior_market_range_1d

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| prior_market_range_1d:q1 | 644 | $2,781.25 | $29,114.07 | 78.88% | 20.34% | 39.91% |
| prior_market_range_1d:q2 | 643 | $1,364.47 | $25,536.21 | 77.92% | 21.46% | 40.90% |
| prior_market_range_1d:q3 | 646 | $793.53 | $33,451.52 | 78.48% | 20.28% | 35.60% |
| prior_market_range_1d:q4 | 646 | $2,167.18 | $57,058.38 | 73.99% | 25.54% | 44.58% |

## Quartiles: prior_market_range_7d

| Bucket | Fills | PnL | Cost | Win Rate | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| prior_market_range_7d:q1 | 644 | $2,893.50 | $33,007.78 | 78.42% | 20.96% | 38.20% |
| prior_market_range_7d:q2 | 645 | $1,043.11 | $22,091.61 | 77.05% | 22.33% | 43.26% |
| prior_market_range_7d:q3 | 644 | $574.03 | $32,778.53 | 78.42% | 20.03% | 36.65% |
| prior_market_range_7d:q4 | 646 | $2,595.79 | $57,282.27 | 75.39% | 24.30% | 42.88% |

## Single-Feature Removal Scan

Positive removed PnL means a gate would remove profitable fills. Negative removed PnL is the interesting direction.

| Feature | Direction | Threshold | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| side_edge_vs_fill | le | 0.0825 | 779 | $51,500.11 | $-996.84 | $996.84 | 23.88% | 38.90% |
| risk_score | ge | 0.4742 | 518 | $29,779.89 | $-227.28 | $227.28 | 27.03% | 45.37% |
| side_edge_vs_fill | le | 0.0939 | 1034 | $66,060.34 | $-158.13 | $158.13 | 23.21% | 39.65% |
| regime_reversal_pressure | ge | 0.6200 | 95 | $5,025.99 | $114.16 | $-114.16 | 21.05% | 44.21% |
| price | ge | 0.7969 | 525 | $35,139.14 | $184.75 | $-184.75 | 15.05% | 22.48% |
| side_edge_vs_fill | le | 0.0568 | 523 | $33,699.72 | $210.32 | $-210.32 | 19.31% | 33.46% |
| regime_sign_flip_rate | ge | 0.6000 | 96 | $5,653.97 | $230.06 | $-230.06 | 25.00% | 41.67% |
| regime_reversal_pressure | ge | 0.5600 | 171 | $8,918.68 | $461.53 | $-461.53 | 24.56% | 45.03% |
| risk_score | ge | 0.4015 | 1290 | $69,275.51 | $569.51 | $-569.51 | 24.88% | 43.18% |
| risk_score | ge | 0.4243 | 1032 | $55,450.82 | $576.23 | $-576.23 | 24.42% | 43.02% |
| regime_realized_vol_180s_bps | le | 1.3706 | 517 | $33,708.75 | $694.38 | $-694.38 | 23.79% | 44.10% |
| regime_path_efficiency | le | 0.0750 | 772 | $43,952.54 | $706.89 | $-706.89 | 23.45% | 41.84% |
| regime_path_efficiency | le | 0.0518 | 516 | $30,000.86 | $795.83 | $-795.83 | 23.06% | 41.47% |
| regime_reversal_pressure | ge | 0.4800 | 237 | $12,996.85 | $808.44 | $-808.44 | 22.36% | 41.35% |
| side_edge_vs_fill | le | 0.0995 | 1292 | $81,510.27 | $910.52 | $-910.52 | 21.83% | 38.08% |
| risk_score | ge | 0.4467 | 774 | $43,316.52 | $947.21 | $-947.21 | 24.68% | 44.32% |
| regime_sign_flip_rate | le | 0.2571 | 146 | $8,107.73 | $957.74 | $-957.74 | 15.75% | 39.04% |
| regime_whipsaw_score | le | 0.2474 | 1034 | $63,291.94 | $989.29 | $-989.29 | 24.08% | 42.94% |
| regime_whipsaw_score | le | 0.2343 | 776 | $47,787.33 | $1,008.33 | $-1,008.33 | 23.97% | 43.17% |
| side_model_p | ge | 0.8904 | 793 | $48,602.56 | $1,039.09 | $-1,039.09 | 16.14% | 26.61% |
| market_yes_range_so_far | ge | 0.5500 | 272 | $10,377.44 | $1,091.60 | $-1,091.60 | 19.85% | 36.76% |
| regime_reversal_pressure | ge | 0.4200 | 317 | $17,245.04 | $1,104.20 | $-1,104.20 | 23.34% | 41.96% |
| side_model_p | ge | 0.8970 | 537 | $34,332.38 | $1,153.06 | $-1,153.06 | 15.08% | 24.39% |
| confidence_score | le | 0.8446 | 516 | $29,518.66 | $1,226.23 | $-1,226.23 | 24.61% | 44.38% |
| prior_market_range_3d | ge | 0.8114 | 458 | $40,191.60 | $1,231.62 | $-1,231.62 | 25.33% | 43.67% |
| price | ge | 0.7802 | 802 | $52,284.43 | $1,283.99 | $-1,283.99 | 14.96% | 25.44% |
| regime_sign_flip_rate | ge | 0.5429 | 308 | $16,622.72 | $1,311.50 | $-1,311.50 | 21.75% | 40.91% |
| market_yes_range_so_far | le | 0.3200 | 112 | $6,106.18 | $1,379.52 | $-1,379.52 | 25.89% | 39.29% |
| regime_whipsaw_score | le | 0.2194 | 515 | $31,416.30 | $1,386.82 | $-1,386.82 | 21.75% | 41.75% |
| regime_reversal_pressure | le | 0.2200 | 394 | $23,634.89 | $1,402.78 | $-1,402.78 | 19.80% | 42.39% |

## Two-Feature Candidate Scan

| Candidate | Removed Fills | Removed Cost | Removed PnL | Full-Removal Improvement | Toxic Rate | Cross-Mid Rate |
|---|---:|---:|---:|---:|---:|---:|
| confirm_low_edge_reversal | 441 | $27,661.13 | $-55.29 | $55.29 | 22.90% | 39.00% |
| price_high_edge_low | 559 | $36,394.59 | $637.31 | $-637.31 | 14.67% | 23.97% |
| price_high_reversal | 471 | $30,016.80 | $665.11 | $-665.11 | 14.86% | 25.69% |
| fav_high_price_chop | 473 | $30,019.48 | $1,713.21 | $-1,713.21 | 15.43% | 28.33% |
| obs_mid_high_signflip | 1428 | $79,311.43 | $3,784.13 | $-3,784.13 | 21.29% | 39.85% |
| high_signflip_low_eff | 1704 | $94,841.14 | $3,829.51 | $-3,829.51 | 22.07% | 40.90% |
| high_reversal_low_eff | 1322 | $72,010.97 | $4,230.47 | $-4,230.47 | 21.03% | 39.49% |
