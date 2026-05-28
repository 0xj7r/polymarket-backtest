# Active BTC 5m Experiments

Last updated: 2026-05-28 19:10 UTC.

Scope for this lane is BTC 5m only. Multi-market BTC/ETH and 15m/1h expansion is
paused until the BTC 5m engine has a clean full-history profile.

## Current Verified Leader

Run: `20260528T145725Z-portfolio-grid-24140`

Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_vol1p25`

Configuration:

- Starting capital: `$1,000`
- Market family: BTC 5m only
- Markets: `23,705`
- Tails: disabled

Result:

- PnL: `+$8,823.43`
- Return: `+882.34%`
- Max drawdown: `31.14%`
- Fills: `6,010`
- Fill log loss: `0.5458`

Attribution:

- `br2_late_favourite_load`: `+$4,054.44`
- `br2_late_confirm`: `+$3,092.26`
- `br2_high_skew_load`: `+$1,676.73`

This is the benchmark to reproduce at `$5,000` starting capital before live
paper deployment. It does not include convex tail coverage.

## Active 5K Runs

All runs below use:

- Starting capital: `$5,000`
- Market family: BTC 5m only
- Date range: `2026-02-12` to `2026-05-27`
- Training window: first `4,500` markets
- Gross cap: `$1,250`
- Exposure fraction cap: `0.12`
- Clip fraction: `0.015`
- Kelly fraction: `0.5`
- Taker latency: `500ms`

### Scaled No-Tail Reproduction

Run: `20260528T185235Z-portfolio-grid-79610`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_exact_leader_scaled_notails`

Purpose: reproduce the old BTC5m no-tail leader at `$5,000` starting capital.

Latest checkpoint observed:

- Markets: `750`
- PnL: `-$256.10`
- Return: `-5.12%`
- Max drawdown: `11.35%`
- Fills: `356`
- Markets with orders: `179`

Early attribution:

- `br2_late_favourite_load`: `+$302.34`
- `br2_high_skew_load`: `+$149.78`
- `br2_late_confirm`: `-$708.22`

Interpretation: early loss is concentrated in `late_confirm`; favourite and
high-skew lanes are positive in the same checkpoint.

### Pre-Fix Tail Variant

Run: `20260528T185359Z-portfolio-grid-82390`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_exact_leader_scaled_tail`

Purpose: tail-enabled comparison launched before the directional-tail anchoring
fix.

Status: stopped after preserving artifacts to S3. This run is not promotable
because it predates the tail anchoring fix, but its partial checkpoint remains
useful for rough shape comparison.

Final preserved checkpoint before stop:

- Markets: `2,500`
- PnL: `+$1,968.64`
- Return: `+39.37%`
- Max drawdown: `27.33%`

Earlier attribution at `500` markets:

- `br2_late_favourite_load`: `+$475.32`
- `br2_high_skew_load`: `+$24.14`
- `br2_late_confirm`: `-$739.99`
- `br2_convex_tail`: `-$50.53`

Interpretation: early drag is still mostly `late_confirm`, not convex tails.

### Fixed Directional-Tail Variant

Run: `20260528T190005Z-portfolio-grid-94612`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_directional_tail_fix`

Purpose: same tail variant after commit `df968bcf`, where convex tails are
anchored to all directional exposure rather than only `late_favourite_load`.

Latest checkpoint observed:

- Markets: `3,000`
- PnL: `+$3,990.27`
- Return: `+79.81%`
- Max drawdown: `26.73%`
- Fills: `1,993`
- Markets with orders: `851`

Attribution:

- `br2_late_confirm`: `+$1,977.59`
- `br2_late_favourite_load`: `+$2,119.26`
- `br2_high_skew_load`: `+$489.97`
- `br2_convex_tail`: `-$596.56`

Comparable checkpoint versus no-tail:

- Through the first `2,500` markets, fixed-tail was `-$299.15` behind no-tail:
  `+$1,957.17` versus `+$2,256.31`.
- Directional lanes were roughly comparable; the gap was almost entirely
  convex-tail insurance spend (`br2_convex_tail = -$314.30`).
- Max drawdown was slightly lower with tails (`26.73%` versus `27.57%`).

Tail price buckets through `2,500` markets:

- `4-6c`: `13` fills, `2` wins, `+$151.99`
- `6-8c`: `25` fills, `2` wins, `+$53.98`
- `8-10c`: `124` fills, `5` wins, `-$520.26`

Interpretation: cheap tails should be judged as portfolio insurance, not as a
standalone alpha lane. The current evidence says `4-8c` tails fill and have
positive convex payoff in this slice, while `8-10c` dominates the bleed. A
future candidate should test `tail_max_ask = 0.08` or a much smaller 8-10c
size, but only after the active full-history runs complete.

### Late-Confirm Range-Gated Variant

Run: `20260528T191002Z-portfolio-grid-14707`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_tail_fix_lc_range50`

Purpose: test the post-hoc failure isolation from the 750-market no-tail
checkpoint using a clean forward replay.

Only intentional strategy change versus the fixed-tail run:

```text
--br2-late-confirm-max-observed-range 0.50
```

Hypothesis:

- `late_confirm` is profitable only before the market has already traversed a
  large observed YES range.
- Above `0.50` observed range, the lane is often chasing reversal-prone
  dislocations where the model reports high edge but realized payoff is poor.
- The gate should reduce reversion losses without touching profitable
  `late_favourite_load` or `high_skew_load` exposure.

Evidence from the 750-market no-tail checkpoint:

- Baseline checkpoint PnL: `-$256.10`
- Non-`late_confirm` PnL: `+$452.12`
- `late_confirm` PnL: `-$708.22`
- Post-hoc replay of existing fills with `late_confirm` range capped at `0.50`
  estimated total PnL around `+$1,034` on that same checkpoint.

This post-hoc result is a hypothesis generator only. Promotion requires a clean
full-history replay with the gate applied during order generation.

### No-Tail Late-Confirm Range-Gated Isolation

Run: `20260528T192139Z-portfolio-grid-37354`

Label: `clip_0p015_gross_1250_expfrac_0p12_lat500ms_cap5k_btc_5m_notail_lc_range50`

Purpose: isolate the `late_confirm_max_observed_range = 0.50` change without
convex tails. This should be compared directly against the scaled no-tail
reproduction run.

Only intentional strategy changes versus the scaled no-tail reproduction:

```text
--br2-late-confirm-max-observed-range 0.50
--br2-tail-clip-frac 0.0
--br2-tail-max-clips 0
```

Status at launch: active on instance `i-0924b2f000a266257`.

## Decision Rules

Promote a BTC5m candidate only if it beats the verified no-tail leader on a
clean full-history replay after matching market universe and training window.

For each candidate, compare:

- Full-history PnL and compounded return
- Max drawdown and worst local drawdown window
- Active market percentage
- Per-tag PnL for `late_confirm`, `late_favourite_load`, `high_skew_load`, and
  `br2_convex_tail`
- Fill log loss
- Tail spend, payoff, hit rate, and price buckets

Do not promote from a partial checkpoint alone.
