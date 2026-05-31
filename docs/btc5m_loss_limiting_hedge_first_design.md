# BTC5m Loss-Limiting: Hedge-First Restructure

Date: 2026-05-29
Status: Proposed (supersedes `docs/btc5m_regime_aware_architecture_design.md`, whose regime-detection thesis the data invalidated)
Strategy: `bonereaper_v2` ("br2"), BTC 5-minute directional taker

## TL;DR

`br2` is strongly profitable historically (+$8,990 / Sharpe 3.4 on 12,500 markets) but the recent ~30 days went flat/negative. After three independent investigations, the cause and the fix are clear, and the fix is STRUCTURAL, not predictive: emulate the real profitable BTC-5m operator's hedge-first, arb-anchored book. The directional bet becomes a small overlay on a pre-funded sub-$1 hedge, so a crossed-mid reversal costs the overlay, not the whole position.

## What the data established (the investigation journey)

Three approaches were tested and falsified before the answer:

1. **Regime detection — DEAD.** The profitable middle window and the toxic recent window are statistically identical on every market-structure feature (prior_range_7d 0.810 vs 0.813; frac_decisive 0.542 vs 0.541) and per-fill feature (separation <0.1). No regime signal separates them.
2. **Act-at-the-cross (stop/hedge) — DEAD.** crossed_mid_after_fill fills recover 46.6% of the time; a blind exit/hedge stops out the recoverers and nets ~zero to negative (-$3.8k to +$2.3k). The earlier "cap doubles PnL" numbers were an oracle artifact: you cannot stop a binary at -cap mid-market.
3. **Reversal prediction (incl. spot-buffer) — DEAD.** No fill-time feature separates crossed-mid recoverers from reversers. Logged features (conf/risk) are faint (best combo helps ~11%); the spot distance-to-strike buffer is noise (AUC ~0.51, even slightly inverted: deeper-ITM crossed-mid favourites recover LESS). The reversal is essentially exogenous at decision time.

**Mechanism of the recent flat P&L:** the loss is the persistent crossed_mid_after_fill tail (favourite loaded, price reverses through mid, ~$200-250/market), unmasked when the static realized-vol floor (`min_realized_vol_180s_bps`) met a low short-horizon-vol regime and cut participation ~4x (active rate 20.5% -> 4.0%), so the clean held-side wins that used to mask the tail dried up. In the recent 30d, ~5 reversal markets = 89% of the low-vol loss; 69% of markets still won.

## The breakthrough: real-operator loss-limiting recipe

Source: `~/go/polymarket-research/data/whales/`. Two BTC-5m whales, 8-day window 2026-05-09..05-16 (the same recent regime).

- **Whale A `0xb27b...` (BTC-5m only, the disciplined target):** +$28,022, median +$36.8/market, 58% markets profitable, **worst single market only 6.6% of net P&L**, max day-drawdown -$3,984 recovered next day. Structural edge: median combined pair price **0.971** (buys YES+NO for ~0.97, redeems at 1.00 = ~2.9c locked arb), 64% of markets sub-$1.00 pair, **99.5% two-sided / 93% balanced**, **93% of notional early+mid / 6% late / 1.6% post**, clip p50 $3.33 / p99 $46 / max $191.
- **Whale B `0xeebd...` (BTC+ETH, aggressive — what br2 resembles):** +$10,409 on 2.5x the capital, **worst single market 33.5% of net P&L**, max clip $5,004, only 88% two-sided, 35% of notional post-bar. Fat-tailed, naked-ish directional.

**The loss-limiter is not a stop-loss; it is a structural sub-$1 hedge that makes the directional bet a small, pre-funded overlay.**

### Recipe (portable br2 parameters)

| # | Lever | Whale A number | br2 change |
|---|---|---|---|
| a | Two-sided + combined price | 99.5% both legs; median combined 0.971 | Never naked. Require a both-legs base, blended YES+NO cost < $0.98. If pair unavailable sub-1.00, skip the market. |
| b | Balance | minority leg median 29% of notional (>=20%) | Minority leg floor ~20% of book notional; both legs established before any tilt. |
| c | Phase timing | 93% notional early+mid, 6% late, 1.6% post | Build the hedged base in the first 240s. Reserve <=6% per-market notional for the late directional add. No last-15s / post-bar loading. |
| d | Clip discipline | p50 $3.33, p99 $46, max $191 | Clip cap ~$50 (hard max ~$190); slice into many small fills; ban lumpy clips. |
| e | Late tilt on top of base | late ~57/43 lean, rides hedged book | Late directional is a thin overlay (<=6%, mild lean) on a profitable hedged base; the base carries the P&L. |
| f | Convex hedge ratio late | convex_hedge_count_ratio_late 1.32 | Keep hedging into the tilt (~1.3x hedge:directional fill rate) rather than abandoning the hedge late. |

### Top 3 structural changes (ranked)

1. **Require a sub-1.00 hedged base before any directional exposure.** Converts the crossed-mid reversal from a full-notional loss into a loss on the <=6% overlay. Single biggest lever; the edge lives in the base (arb), not the tilt.
2. **Cap late/post directional notional to ~6% of per-market book; forbid post-bar chasing.** The reversal hurts because size lands late at adverse prices; move it forward and shrink it.
3. **Hard clip cap ~$50 (max ~$190) + 20% minority-leg floor.** Mechanically bounds per-market loss to ~one bar's pair-cost; eliminates single-market blowups.

## Implications for the prior plan

- **#1 (convex_tail) was the right instinct but too small.** The real change is the hedge-first base; convex_tail is the seed of it.
- **#2 (order-flow reversal prediction) is largely moot.** The whale does not predict reversals; he structures around them. Deprioritize unless the structural change underperforms.
- The regime-aware spec is superseded; regime detection is not the lever.

## Implementation plan

0. **Engine capability gate (do first).** Confirm the backtest engine: (a) credits $1 per winning share at resolution so a sub-$1 pair is mechanically +EV (redeem/merge accounting); (b) lets a strategy buy BOTH legs early and hold them; (c) the seam to add an early hedged-base lane + a late-notional cap. Identify whether new plumbing is needed or convex_tail / existing lanes can be extended.
1. **Hedged-base lane:** in the first ~240s, buy YES+NO toward a blended combined cost < ~0.98, minority leg >= 20%. Config-gated, off by default.
2. **Late-overlay cap:** cap late+post directional notional to ~6% of per-market book; forbid post-bar adds; clip cap ~$50.
3. **Validate on EC2** (PM order-book replay data lives in S3, not local) over full history + last_30d + last_7d, at BOTH $1,000 and ~$2,800 starting cash (the live account; note sizing is sublinear due to thin books).

### Promotion bar

- Recent slice (last_30d / last_7d) clearly positive.
- Per-market loss tail bounded (worst market a small % of net, approaching Whale A's 6.6% rather than br2's blowups).
- Full-history PnL up or comparable with materially lower drawdown.
- No new loss category in the post_fill_path distribution.
