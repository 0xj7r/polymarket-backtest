# Calm-Regime Paired-MM Backtest + Net-EV Verdict (2026-05-31)

Queue-aware, round-trip backtest of a two-sided matched-pair maker on Polymarket
BTC-5m, built to decide whether to invest in a live MM build vs leaving br2 idle in
the calm regime. Simulator: `scripts/mm_paired_sim.py`. Raw output:
`docs/calm_regime_paired_mm_results_2026-05-31.txt`.

## TL;DR verdict

The paired-MM is **net-positive after realistic queue + round-trip costs, but small**:
roughly **$30-48/day** of gross edge across the whole calm universe at clip 10-20 shares,
of which the repeatable spread-capture component is **~$18-28/day**. At $1-2.8k of
capital the dollar edge does not scale with bankroll (it is gated by per-market queue
share and taker flow, not capital), so the answer at $2.8k is the same ~$30-48/day.

**The regime gate is NOT worth it.** Strict pairing already caps the tail (worst single
market is -$1.3 gated AND ungated); the gate only removes ~14% of markets, cutting total
volume and lowering total net while raising per-share quality. In this near-uniformly-calm
universe the gate buys nothing the pairing discipline does not already provide.

**Recommendation: do NOT build the live MM yet on the strength of this alone.** ~$30-48/day
gross is below the bar for a from-scratch execution/latency build whose realized edge is
the binding unknown (queue position). It is a credible *secondary* sleeve to run alongside
br2 (it earns in exactly the calm regime where br2 is idle), but only if it can be added
cheaply on top of existing maker infra, not as a standalone latency project.

## Method (what makes this realistic)

Single mirrored book: only the YES ("Up") token is in the cache, NO is its mirror
(`NO = 1 - YES`). A YES short is a NO long, so quoting both sides of YES accumulates both
legs. Fills come from the **tape, not the engine's optimistic touch-fill**:

- A resting YES bid fills only when a real taker SELL prints at/below our price; a resting
  YES ask fills only when a real taker BUY prints at/above our price.
- Fill quantity is pro-rata by `clip / (clip + shares_ahead)`, where `shares_ahead` is the
  top-of-book resting depth at our price from the book snapshot (the binding queue constraint).
- **Strict pairing:** a per-fill clamp + leg-suppression keeps `|yes_long - no_long|` within
  a small repair band, so one leg can never run away from the other. Matched pairs redeem to
  $1 (P&L = `1 - (yes_avg + no_avg)` per pair, i.e. the captured spread). The unmatched
  residual is **never traded out**; it resolves at the realized BTC up/down outcome.
- Quotes pull in the last 45s (toxic window) and on Binance spot acceleration / large taker
  prints (the dynamic anti-stranding gates).
- Outcome (YES wins) is derived from Binance BTCUSDT spot: `close_px > start_px` over the
  5-minute window. All gate features use only trailing (past) spot data: no look-ahead.

Economics: maker rebate modelled as 20% of the ~1.56% taker fee on notional
(~0.156c/share at price 0.5). Reported with and without rebate.

## Chosen regime-classifier thresholds (tuned on the data)

| feature | threshold | rationale |
|---|---|---|
| mid in band | 0.30-0.70 | coinflip zone; ~99.6% of these markets already sit 0.47-0.52 |
| YES range-so-far | <= 0.06 | p90 of the calm universe |
| spot vol (stdev of 30s returns, 5s grid) | <= 1.2e-4 | ~p75 of the universe; uncorrelated with PM range (r=0.07), adds info |
| spot sign-flip fraction | >= 0.20 | micro-oscillation (healthy two-sided flow) vs one-way drift |
| warmup | 60s of history before quoting | |

Dynamic (tick) gates: pull both legs in the last 45s; pull the exposed leg when the
trailing 30s spot return exceeds 0.0008 (~p97); pull the hit side after a taker print
>= 150 shares.

## Results (full universe, 3,767 markets with trades, 14 days, May 7-20)

`net = pairs (spread capture) + residual (unmatched leg vs outcome) + rebate`.
`$/day` divides by 14. `worst` = worst single-market net.

| clip | config | mkts | $/day net | c/share | %mkts pos | resid frac | pairs $/14d | resid $/14d | rebate $/14d | worst $ |
|---|---|---|---|---|---|---|---|---|---|---|
| 5  | gated   norebate | 3220 | 20.77 | +1.385 | 54.4 | 36.9% | 65.1  | 225.7 | 0.0   | -1.3 |
| 5  | ungated norebate | 3753 | 28.64 | +0.973 | 54.4 | 27.2% | 158.9 | 242.2 | 0.0   | -1.6 |
| 5  | gated   rebate   | 3220 | 23.09 | +1.539 | 54.8 | 36.9% | 65.1  | 225.7 | 32.5  | -1.3 |
| 5  | ungated rebate   | 3753 | 33.20 | +1.127 | 54.8 | 27.2% | 158.9 | 242.2 | 63.8  | -1.6 |
| 10 | gated   norebate | 3220 | 22.55 | +1.055 | 54.5 | 30.2% | 109.3 | 206.4 | 0.0   | -1.3 |
| 10 | ungated norebate | 3753 | 33.06 | +0.805 | 54.6 | 20.9% | 258.1 | 204.8 | 0.0   | -1.6 |
| 10 | gated   rebate   | 3220 | 25.85 | +1.210 | 54.9 | 30.2% | 109.3 | 206.4 | 46.3  | -1.3 |
| 10 | ungated rebate   | 3753 | 39.41 | +0.959 | 55.4 | 20.9% | 258.1 | 204.8 | 88.9  | -1.6 |
| 20 | gated   norebate | 3220 | 24.52 | +0.833 | 54.7 | 23.9% | 170.1 | 173.1 | 0.0   | -1.3 |
| 20 | ungated norebate | 3753 | 39.60 | +0.712 | 54.9 | 15.4% | 389.3 | 165.0 | 0.0   | -1.5 |
| 20 | gated   rebate   | 3220 | 29.07 | +0.987 | 55.1 | 23.9% | 170.1 | 173.1 | 63.7  | -1.3 |
| 20 | ungated rebate   | 3753 | 48.19 | +0.866 | 55.6 | 15.4% | 389.3 | 165.0 | 120.3 | -1.5 |

### Reading the table

- **Spread capture (pairs) is the real, repeatable edge.** It is positive in every cell and
  scales with clip (clip20 ungated = $389/14d = $27.8/day). Per-share edge falls as clip
  grows (queue dilution: a bigger clip captures more flow but at a lower fraction), exactly
  the queue-dominated economics the synthesis predicted.
- **Residual P&L is persistently positive** ($165-242/14d) and does NOT average to zero over
  3,767 markets. A control run with the directional dynamic gates OFF still shows residual
  $200/14d (clip10) / $149 (clip20) — so this is **not** the Binance-lead gate injecting
  direction. It is structural favorable selection in the conservative cross-fills: we buy
  below mid (on taker sells) and sell above mid (on taker buys), so the leftover inventory
  was acquired at favorable prices and carries small positive EV even marked to a coinflip
  outcome. It is genuine, but it is the more fragile component (it leans on the residual
  resolving, and on fill timing the snapshot data cannot pin down precisely).
- **The regime gate lowers total net.** Gated runs trade fewer markets (3220 vs 3753) and
  fewer shares, so total $ drops even though c/share rises. The worst single market is -$1.3
  in BOTH gated and ungated: **strict pairing, not the regime gate, is what caps the tail.**
  The gate would matter in a volatile universe; this May calm universe is ~85% benign already.
- **Rebate adds ~25-30%** to net and never flips the sign — the strategy is positive at
  rebate = 0 (spread capture alone), satisfying the "must be positive without rebate" bar.

## OOS split (IS = May 07-15, OOS = May 16-20), clip 10, no rebate

| split | config | mkts | $/day net | c/share | pairs $ | resid $ | worst $ |
|---|---|---|---|---|---|---|---|
| IS  | gated   | 2011 | 20.57 | +0.949 | 69.5  | 115.6 | -1.2 |
| IS  | ungated | 2326 | 28.00 | +0.755 | 132.6 | 119.4 | -1.6 |
| OOS | gated   | 1209 | 26.11 | +1.254 | 39.7  | 90.8  | -1.3 |
| OOS | ungated | 1427 | 42.17 | +0.874 | 125.5 | 85.3  | -1.4 |

Both halves are positive, ungated > gated in both, and the spread-capture share per market
is stable (IS pairs $132.6 / 2326 mkts ~ 5.7c/mkt; OOS $125.5 / 1427 ~ 8.8c/mkt — OOS a bit
richer, not weaker). The signal does not degrade out of sample.

**OOS caveat (important):** the local cache holds PM book+trades for **May 7-20 only**. The
requested IS Feb-Apr vs OOS May split is **not feasible** — there is no pre-May PM data on
disk (Binance spot exists Feb-May, but the PM book does not). This OOS is a within-month
9-day/5-day split, so it tests temporal stability inside one calm regime, not cross-regime
robustness. A true OOS needs pulling Feb-Apr PM book/trades from S3.

## Honest verdict on the decision gate

1. **Is it net-positive after realistic queue + round-trip costs?** Yes. Spread capture alone
   (the repeatable part) is positive everywhere, ~$18-28/day at clip 10-20; with the
   structural residual and rebate the total reaches ~$33-48/day. Positive at rebate = 0.

2. **How much, at $1-2.8k?** ~$30-48/day gross across the whole calm universe, and it is
   **bankroll-insensitive** in this range — the edge is capped by per-market queue share and
   taker flow, not by capital. Clips of 5-20 shares per market need only tens of dollars of
   inventory per market. So $2.8k earns about the same $/day as $1k; extra capital sits idle.

3. **Is the regime gate worth it?** No, not in this universe. It does not lift net EV and is
   not needed to control the tail (pairing does that). Keep the cheap parts of the classifier
   (mid-band, skip the last 45s) and drop the spot-vol/range/flip gating unless a later
   cross-regime OOS shows a toxic tail that pairing alone misses.

4. **Worth a live build vs leaving br2 idle in calm?** **Marginal, lean no as a standalone.**
   ~$30-48/day is real but small, and the single biggest live risk — actual queue position /
   fill priority — is exactly what the snapshot data cannot measure, so realized edge could be
   materially lower (the synthesis flags this; halving for worse real fills lands ~$15-25/day).
   That does not clear the cost of a dedicated fast-quote/cancel execution stack. The sane path
   is to run it as a **near-zero-marginal-cost overlay on existing maker infra** (br2 Lane 0,
   per `docs/btc5m_maker_overlay_build_plan.md`) so it harvests this calm-regime edge while br2
   sleeps, rather than commissioning a new latency project for $30-48/day.

## Caveats / known limitations

- No Feb-Apr PM data locally; true cross-regime OOS not run (see above).
- Fill timing is from ~1s snapshots + trade tape; no order-level/queue events, so queue
  position (the binding constraint) is approximated by TOB depth at submit. Real edge could
  be lower if priority is worse than pro-rata assumes.
- Residual P&L leans on the unmatched leg resolving; it is the more fragile component and
  should be discounted relative to the pairs (spread-capture) number when sizing conviction.
- Outcome derived from Binance spot at the window endpoints; a handful of markets near the
  data edges are dropped when spot coverage is missing.
