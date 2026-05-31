# Strategy Narrative — End of Day 2026-05-30

Status: FINAL. All threads settled: the live-port equivalence test (the safety gate) is **PROVEN**, the ETH auto-tune is **settled** (thin, non-diversifying), and the vol-floor tighten is **REJECTED on the full history** (45-69% return cut for a Sharpe/drawdown gain - the trade the drawdown-tolerant objective declines). The 15m directional test is parked (data-staging bugs to fix before it is trustworthy). The live config is locked to the 062901 champion, unchanged.

## 0. The live port (the priority) - equivalence PROVEN

Carrying the BTC-5m base to live via **Option A** (path-dep the br2 crates into the existing `~/go/polymarket-agent`; live == backtest at the code level). Phases 1-3 are DONE:
- **Crate reuse compiles**: `pm-strategy`/`pm-types`/`pm-model`/`pm-risk` (zero nautilus deps) consumed as path deps - the live agent runs the EXACT `BonereaperV2::on_event` + the real frozen 062901 calibrator (`updates=903876, beta_enabled, isotonic_pts=187`, confirmed genuine).
- **Shadow adapter** feeds br2 from live Binance-spot + PM-book, logs decisions, places no orders.
- **EQUIVALENCE TEST PASSED**: 40 BTC-5m markets, 12,000/12,000 events identical = **100.0000% match, 341 decisions, 0 mismatches** across all four lanes. Negative control (breaking the gate) trips 255 mismatches, so the harness has teeth. Two adapter-only divergences were found and fixed (the external model gate conf≥0.68/risk≤0.72/edge≥0.00 applied after `on_event`; br2's position-awareness) - **br2 crates untouched**.
- The `is_buyer_maker` flow flag (dropped before) now flows from the Binance `m` field into br2's signed-flow features.

**Verdict: live-backtest equivalence is proven at the decision layer. Phase 4 (real taker execution + kill-switch) is CLEARED.** Remaining to live: Phase 4 execution wiring + a hard kill-switch, then Phase 5 paper/small-size. (One pre-existing snag: uncommitted WIP in `bonereaper_mm.rs` breaks `cargo test -p polymarket-exec`; the bins + lib build clean.)

## 1. Where the strategy sits

**The base BTC-5m br2 is genuinely strong and is the product.** Full history Feb12-May20 *including* the drawdown: **+$10,466 @ $2,800 (maxDD 15.4%, Sharpe ~3.3) / +$8,990 @ $1k** (authoritative run 98353). Edge concentrated in late_favourite (lane Sharpe 6.58) + high_skew (4.27); late_confirm volatile (2.42); convex_tail is a net cost (-1.42). It is highly selective (hit-rate 0.064, trades ~2,016 of 23,705 markets) - that selectivity IS the edge.

**The May drawdown is transient and self-healing, not a structural break.** It was a thin crossed-mid tail (~139 toxic loads) unmasked when participation collapses in a *calm/low-vol* regime (vol 2.06->1.72; the vol floor disengages, the residual that still fires crosses mid 54-56% vs 35-44% normal). No data problem. And br2 **self-recovered**: 8 straight green days May 19-26 (+$328), a -$200 stumble May 27 (the toxic tail can still flare), then +$100 May 28; net **+$354 over May 19-28 (Sharpe ~7.9)**, with *no* vol-regime change (vol flat ~8-12bps). Choppy at the front edge but transient on aggregate.

**Overall P&L through May 28 (the latest published data): ~+$10,727 @ $2,800** (the +$10,466 base run through May 20, plus +$261 net over the May 21-28 recovery days *including* the May 27 stumble). So the recent drawdown-and-recovery nets *slightly positive* and the dip cost essentially nothing on the full-history total. (May 29-30 not yet published by Telonex.)

**The maker direction is further debunked by the live data.** The pure-arb whale 0xb27b RESUMED at huge scale May 27-28 (133k/204k fills, $2.3M/$3.5M notional) but LOST heavily (-$2,542 / -$6,758/day; the -$6.8k matches the user's live UI -$5k day) - classic stranded-inventory adverse selection. Even the best whale bleeds ~$2.5-6.8k/day when active. And it moves *opposite* to br2 day-to-day, so its participation is not a usable signal.

**The hard discipline this session enforced: validate that an edge survives realistic execution before building it.** It killed two would-be wins before they cost a build (the perp gate on latency; the maker overlay on adverse selection).

## 2. The honest ledger - what we rigorously killed (do not revisit)

- **Week-level regime gate**: the toxic drawdown weeks and profitable trending weeks are ex-ante indistinguishable (same trailing range). Cutting lanes costs multiples more than it saves.
- **Fill-level flagging (spot+PM / Phase-1 signals)**: no separating power over the existing reversal score (AUC ~0.71); toxic loads interleaved with winners.
- **Perp-confirmation gate**: looked like THE win (OOS +$8k, fixes May) but is a **latency artifact** of the clean 1m-kline close - dies by ~30-60s of realistic lag; the fresh sub-minute tape carries no edge. Untradeable live.
- **Other signals**: the real two-sided (NO) book is a *mechanical mirror* of YES (confirmed byte-exact from S3 across 39.4M cells - synthesized NO was always correct, zero independent signal). Slow perp (funding/OI) latency-robust but not predictive. Cross-asset spot dies on latency.
- **Low-vol participation (taker)**: calm markets are genuinely efficient coinflips (book agrees with outcome only 45.6%); mean-reversion fade EV ~0 after spread, fair-value flat, sell-convexity has no inventory. No taker edge in calm.
- **Maker overlay (5m AND 15m)**: adverse selection eats it (-5.3c/mkt @ 5m, -5.42c @ 15m; the 10bps rebate closes ~3%). Calm is NOT lower-adverse-selection. The *entire* loss is the stranded residual; only zero-net-inventory matched fills are positive (+2c/+1.3c). The longer 15m window does NOT help - more time is cancelled by ~3x larger per-fill moves. Even the pure-arb whale (0xb27b) carries a naked stranded residual and has -$2k/-$5k days. So maker arb is build-to-validate on the unproven anti-stranding repair IP, not a safe income lever.
- **Pinned-break taker, 15m momentum/trend, pure momentum**: all dead (the markets are efficient coinflips at the bar level).

## 3. The architecture (the correct cross-market frame)

ONE engine (br2 code), **N per-asset auto-tuned configs**. BTC-5m = the fixed 062901 champion (untouched). Any net-new market (ETH/SOL/...) needs its OWN model + params, auto-tuned for that market's books - you cannot blind-copy the BTC-tuned gates (the failed ETH run did, over-participated at hit-rate 0.43 vs BTC's 0.064, and inverted to -$963). Markets meet only at the portfolio layer, with exposure calibrated for correlation (e.g. BTC-5m and BTC-15m are both BTC-directional - no double-stacking).

## 4. The live shots (being tested now)

- **ETH per-asset auto-tune (SETTLED): a thin positive edge, NOT a meaningful diversifier.** Sweeping ETH's own selectivity gates fixes the over-participation (broken blind-copy a0 = hit 0.391 / -$429; BTC-gates a1 = +$258 but maxDD 40.8%). The properly-tuned winner **a3_tight: +$306 @ $1k, hit-rate 0.026, daily-Sharpe 2.16, maxDD 33.2%, 779/21,747 markets** (gate_conf 0.74, lf_edge 0.12, lf_ask 0.78, vol_floor 2.0). So ETH HAS an edge once tuned, but ~29x smaller absolute than BTC ($306 vs $8,990 @ $1k) and far lower risk-adjusted (Sharpe 2.16 vs BTC 7.63 daily) with a WORSE drawdown (33.2% vs 19.1%). The correlation hope is dead: tuned BTC-ETH **daily r = 0.15 (CI -0.19..0.43, spans zero), weekly r = -0.23 (CI -0.81..0.74, meaningless)** - effectively zero, not the -0.69 hoped. Combined equal-$1k portfolio: +$9,296, daily-Sharpe 7.70 vs BTC-alone 7.63 (a +0.9% bump from the near-zero correlation), maxDD 20.3% vs 19.1%. **Read: ETH earns only a small future slot as a separate per-asset strategy; it is not a diversifier and does not move the needle.** (Validated at $1k; not re-run at $2,800 - scaling is ~linear and does not change the marginal verdict.)
- **15m directional (PARKED)**: the staged 15m run had data bugs (markets.jsonl outcome="Up" for all rows; slug=close_ts misparsed as open) that produced bogus results. Needs correct outcome/slug staging before it can be trusted. The mechanistic prior stands (a 15m favourite at a fixed seconds-to-close has more elapsed variance / less remaining time, so the reversal should be weaker), but this is unverified - do not carry it.
- **Vol-floor tighten (slice DONE, full-history in flight)**: raising the directional-lane realized-vol floor cuts the toxic calm-regime residual, and on the May drawdown slice it is decisive. Full 5-arm sweep (late_favourite + late_confirm floor, $1k):

  | floor (bps) | PnL$ | maxDD% | Sharpe | fills | worst-mkt$ |
  |---|---|---|---|---|---|
  | 1.25 (champ) | -103.09 | 15.03 | -0.86 | 285 | -49.83 |
  | 1.56 | -59.17 | 7.77 | -0.68 | 199 | -51.78 |
  | 1.88 | -50.76 | 6.80 | -0.80 | 150 | -52.43 |
  | **2.50** | **+8.12** | **2.26** | **+0.41** | 112 | **-11.09** |
  | 3.75 | +6.74 | 1.22 | +0.46 | 104 | -5.28 |

  A floor of **2.50 flips the drawdown slice from -$103 to +$8**, cuts maxDD 15%->2.3%, and the worst single-market loss -$49.83->-$11.09 - exactly the diagnosed mechanism (the toxic loads fire in low realized vol; floor them out). BUT the slice only shows the *bad* regime where the floor helps. The decisive test is the FULL-HISTORY net: does floor 2.50 give up the trending-regime profit that was earned at floor 1.25?

  **The full-history A/B is now DONE and the verdict is definitive (run `20260530T144155Z-fullhist-configB-18049`, all 23,705 eval markets). REJECT.** The volbase arm reproduced the authoritative baseline EXACTLY (volbase $1k = +$8,990.21 vs 98353's +$8,990 - clean sanity check, no contamination), so the comparison is trustworthy:

  | size | volbase 1.25 PnL | voltight 2.50 PnL | PnL delta | volbase maxDD | voltight maxDD | volbase Sharpe | voltight Sharpe |
  |---|---|---|---|---|---|---|---|
  | $1k | +$8,990 | +$2,813 | **-68.7%** | 23.7% | 10.2% | 3.20 | 3.85 |
  | $2,800 | +$10,466 | +$5,907 | **-44.6%** | 15.4% | 9.7% | 3.11 | 4.09 |

  Over the full history, tightening the vol floor to 2.50 nearly halves-to-thirds the ABSOLUTE return (-45% to -69%) while improving Sharpe (3.2->3.9) and halving maxDD. It is a pure return-for-drawdown / return-for-Sharpe trade: it cuts the profitable low-vol directional participation that compounds the trending profit. That is exactly the trade the drawdown-tolerant objective rejects (maximize persistent absolute return, accept the drawdown). So the floor does NOT improve both return and drawdown - it sacrifices a large chunk of return - and is rejected for the live config.

  One honest mechanism note from the slice's per-fill cross-mid rates: late_confirm IS toxic in calm (cm-rate 0.595), but late_favourite is closer to a coinflip (0.457, not the 0.54-0.56 the original diagnosis cited), and high_skew is healthy (0.349). So the floor does not surgically target only toxic loads; it removes near-coinflip-but-trending-profitable loads too.

## 5. What to carry to live (interim)

1. **The base BTC-5m br2 as-is** - it is a strong, self-healing persistent return (+$10.5k @ $2,800, Sharpe 3.3) and already clears the drawdown-tolerant bar. This is live-ready today.
2. **The vol-floor tighten: REJECTED (full-history confirmed)** - over all 23,705 markets it cuts absolute return 45-69% (volbase $1k +$8,990 -> voltight +$2,813) to improve Sharpe 3.2->3.9 and halve maxDD. A pure return-for-drawdown trade, which the drawdown-tolerant objective rejects. For the May tail, prefer convex_tail hedge sizing (protection without amputating profitable lanes). Sanity check passed (volbase reproduced 98353's +$8,990 exactly).
3. **ETH (a3_tight) as a small future per-asset strategy** - it has a real but thin tuned edge (+$306/$1k, Sharpe 2.16) and ~zero correlation, so it can be added later sized small, but it is NOT a diversifier and is NOT part of this port. Each future asset (ETH/SOL) needs its own bounded auto-tuning run, never a copy.
4. **NOT live**: the perp gate (untradeable, latency), the maker overlay (build-to-validate on unproven anti-stranding repair), 15m (unverified, data bugs), any blind cross-market copy.

**Bottom line:** the proven, shippable strategy is the strong self-healing BTC-5m base, and it is now **equivalence-proven for live** (Phase 4 execution wiring is the only thing between here and paper trading). Real upside beyond it requires per-asset / per-horizon *re-tuning* (ETH is thin, 15m unverified), not free replication, and every "clever" overlay died on execution realism. The one open question is whether the vol-floor tighten is a free drawdown-cut that keeps the trending profit - the full-history run decides that, and it is the only remaining gate on the final config.
