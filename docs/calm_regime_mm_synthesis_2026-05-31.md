# Calm-Regime Strategy Synthesis (2026-05-31) — the MM pivot

Context: br2 (directional taker) is OOS-negative in the current calm regime (trades nothing in calm). The whales profit by MARKET-MAKING. Four-prong analysis (web + unlawful reverse-engineer + split/merge + our-data quantification) → verdict.

## VERDICT: calm-regime edge = passive two-sided spread-capture MM. REAL but MODEST + QUEUE-DOMINATED.

### The hard numbers (from OUR book data, 14 calm days, 3,987 markets, 330k simulated fills; script scripts/mm_calm_opportunity.py)
- Spread is **1c ~91% of the time** (>=2c only ~9%) -> maker raw edge = **0.5c half-spread**, not 1c.
- Adverse selection is SMALL in calm: **-0.072c/share @ +5s** (median 0); fills ~even long/short (self-neutralizing). The -5.3c "adverse selection" prior is only the **worst-1% toxic tail**, concentrated in the last ~30s.
- **Net +0.536c/filled share, 86% of fills profitable.**
- BINDING CONSTRAINT = **QUEUE POSITION**: ~200 shares rest ahead of you, addressable taker flow ~1,555 sh/market; a clip captures pro-rata clip/(clip+200). Estimated (266 calm mkts/day, two-sided, NO rebate): clip 5sh ~$54/day, 10sh ~$106/day, 20sh ~$203/day (halve if real fills worse). Maker rebate (if present) ~additive on top.
- Pull quotes the **last ~30s** (toxic window; same crossed-mid toxicity that hurt br2).

### unlawful_shear recipe (from 298k real fills) — confirms the design
Two-sided near mid (~1.35c each side), tiny ~$3 clips, ~274 fills/market, **strict pairing: 95% matched Up+Down pairs -> redeem to $1 -> adverse selection structurally capped at the ~5% residual** (this is why its making works where our 1-sided overlay bled). Pulls quotes last 60s. Captures ~2.7c/matched-pair + rebate. Profit = thin margin x volume x structural neutrality, NOT direction. (Whale net-PnL magnitude is uncertain/contradictory across sources: direct redeem-minus-buys = +$28k/week, Polyanna +$595k/90d, one clean-room pass implausibly said -$3M; rely on OUR data EV above, not whale-PnL estimates.)

### Dead ends (confirmed, do not pursue)
- SPLIT/MERGE arb: MIRAGE on a mirrored book (crossed book 0.026% of time, vanishes <1s, sub-$1). Not a standalone edge. (It's just an inventory-unwind path.)
- Liquidity-reward FARMING: dead (quadratic in distance from mid; ~10% APY bonus at best).
- Latency-taker arb vs Binance: dead for small players (no cancel window; sub-100ms bots take >70%).

## THE STRATEGY DESIGN (paired-MM, stranding-proof)
Direction-NEUTRAL matched-pair accumulator. Edge = spread + rebate. The residual (unmatched leg) is the only risk -> hard-cap ~5%, re-pair via the opposite quote (never trade out at a loss in a whipsaw), let small residual resolve. Pull last ~30-60s.

REGIME CLASSIFIER (user's, validated-in-principle, tune thresholds on data): low spot vol + narrow range-so-far + healthy sign-flip (micro-oscillation, NOT macro-whipsaw, the range+vol gates exclude the dangerous kind) + mid in ~0.30-0.70 band -> market is MM-eligible.
DYNAMIC GATES (tick-by-tick, the actual anti-stranding): **Binance spot LEADING signal** (we see spot before the PM book reprices -> pull/widen/skew the exposed side as spot moves; our unique edge + the #1 stranding fix); + last-window pull; + large-taker toxicity pull. Reuses br2's vol/whipsaw/reversal/momentum + the Binance feed.

KEY REALIZATION: P&L is dominated by **queue position / fill priority** (200 sh ahead of you), so the build is as much an EXECUTION/latency problem (fast quote/cancel, repost discipline, front-of-queue) as a signal problem. Signals pick the safe markets; execution determines whether you actually get filled.

## DATA caveat for backtesting
Single mirrored book = complete on STATE (25-level depth). BUT snapshots(~1s)+trades only, NO order-level/queue events -> cannot model queue position precisely (the binding constraint). The engine's maker fill model has NO queue -> OVERSTATES fills. So the MM backtest MUST use a conservative queue/pro-rata fill model (fill only when a taker print crosses our price, pro-rata by clip/(clip+ahead), drop fills that go adverse fast). The tape analysis (scripts/mm_calm_opportunity.py) is the realistic basis, not the engine's optimistic fill.

## NEXT (the cook): build + backtest the paired-MM
Use engine's existing delta_neutral_mm/paired_mm + docs/btc5m_maker_overlay_build_plan.md. Add: the regime classifier as quote-gate, the Binance-lead + late-pull dynamic gates, the conservative queue-aware fill model. Measure net EV (with + without rebate) conditional on the regime, at $1-2.8k, clip 5/10/20sh. Validate: does the regime gate reduce the toxic tail + lift net EV vs ungated? Is it net-positive after realistic queue + a round-trip (both legs)? Then OOS-split it. Decision gate: is the modest (~$50-200/day pre-rebate) edge worth the execution build, vs just running br2 (idle in calm) + waiting for trends?
