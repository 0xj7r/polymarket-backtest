# Regime-Allocation Ensemble: Design + Light Assessment

Date: 2026-05-30. Status: design + offline assessment only. No engine code changed, no EC2 launched. Pure Rust/Python stack.

This document combines the existing strategy components into a single regime-allocation ensemble and gives a grounded, caveated estimate of its persistent risk-adjusted-return benefit over directional-alone. It builds directly on the authoritative findings: directional br2 is +$10,466 @ $2,800 over Feb-May20 (Sharpe ~3.3, maxDD 15.4%); the May7-20 residual is -$596; lane Sharpes are late_favourite 6.58, high_skew 4.27, late_confirm 2.42, convex_tail -1.42; the perp lead-lag signal lifts toxic-load separation to combined OOS AUC 0.744; the maker overlay is regime-complementary to the calm regime where directional disengages.

---

## 0. The core thesis (why an ensemble at all)

Every blunt defense tried against the May drawdown (week-level range gate, global config B convex, global lane amputation H0) costs MORE in the good regimes than it saves in the bad one, because the toxic loads are interleaved with winners that look identical at week granularity. The decisive reframe (user objective, 2026-05-30) is to stop flattening the drawdown and instead MAXIMIZE PERSISTENT RISK-ADJUSTED RETURN, drawdown-tolerant.

That reframe is what makes an ensemble the right move rather than a gate. The directional core does not need to be fixed; it needs to be (a) sharpened at the fill level where the toxic residual actually lives, and (b) PARTNERED with strategies that earn in exactly the conditions where the directional core sits idle. The May diagnosis is the proof: participation collapses 12% -> 2% in the calm regime, so for ~weeks the capital is doing nothing while a thin toxic residual bleeds. The ensemble's job is to put that idle capital to work in a regime-appropriate way, NOT to force the toxic directional residual.

---

## 1. Component strategies and the regime each owns

| Component | Owns regime | Mechanism | Status | Evidence |
|---|---|---|---|---|
| **A. Directional br2 core** | Vol / trend (favourites hold) | Late-favourite + high-skew + late-confirm taker loads | LIVE, validated | +$10,466 @ $2,800 Feb-May, Sharpe 3.3; lane Sharpes 6.58 / 4.27 / 2.42 |
| **B. Perp-confirmation gate** | Refines A's entries within vol/trend | Load late_favourite/late_confirm only when perp lead-lag CONFIRMS the move; skip/shrink when perp is flat-or-against | signal validated on May slice (n=112), full-history validation pending | spot+PM 0.593 -> +perp 0.716 -> combined 0.744 OOS AUC; CI on delta-AUC excludes zero |
| **C. Maker rebate overlay** | Calm / low-vol / pinned (favourites pin, directional disengages) | Two-sided maker quotes around br2's balanced inventory, harvest spread + rebate, lowest adverse selection here | design complete (build plan), not built | calm = lowest adverse selection; directional participation collapses 12%->2% here = the idle-capital gap |
| **D. Cross-market diversification** | Orthogonal, always-on across assets | Run A (+B, +C) on ETH-5m (built today), then SOL/XRP (one-line spot-symbol add) | BTC live; ETH plumbed (`discovery.rs` parses `eth-updown-`); SOL/XRP need spot symbol | each asset's per-market edge is the SAME mechanism (intra-bar mean-reversion of late favourites) but the realizations are largely uncorrelated bar-to-bar |

Key structural point: A and C are NOT competing strategies fighting over the same markets. They concentrate in DIFFERENT regimes. The convex_tail lane (Sharpe -1.42, a net cost) is explicitly NOT a standalone component; it stays as targeted insurance inside A, fired only on perp/flow-flagged-fragile loads (config-B mechanism), never as a primary allocation.

### The lane map inside component A
- late_favourite (Sharpe 6.58): the durable core. Never amputate. B gates its entries.
- high_skew (Sharpe 4.27): the most regime-robust lane; in the May local slice it was the entire positive P&L when the other lanes were cut. Keep ungated or lightly gated.
- late_confirm (Sharpe 2.42, volatile): the largest contributor to the May residual alongside late_favourite. B's primary target.
- convex_tail (Sharpe -1.42): insurance carry, targeted not blanket.

---

## 2. The allocation signal

### Candidates considered
1. **Realized vol level** (spot 180s bps, or the existing `min_realized_vol_180s_bps` floor state). Leakage-proof, already computed, already drives participation. US-session vol ~2x overnight; May vol 2.06 -> 1.72 is what disengaged the floor.
2. **Participation / gate-engagement rate** (fraction of markets where br2 actually fires). This is a DERIVED function of (1) plus the model gates; it IS the observable that collapsed 12% -> 2%.
3. **Perp-confirmation state** (per-load, not per-regime): does the perp lead-lag confirm this specific favourite.
4. **Whipsaw / vol regime snapshot** (`regime_whipsaw_score`, `regime_sign_flip_rate`, `regime_path_efficiency`): these FAILED as a week-level lane on/off gate (trending and drawdown weeks were statistically identical on them).

### The choice: the allocation is largely SELF-SELECTING, with vol-level as the only explicit dial.

The honest finding from all prior work is that an EXPLICIT regime switch (4) does not exist at the granularity that would help: the regimes are not separable ex-ante at week/market level. But the ensemble does not need an explicit switch, because the components self-select by construction:

- **Directional A self-disengages in calm.** This is not a bug to fix; it is the mechanism. The vol floor firing in low-vol IS the calm detector, and it already cuts A's participation. No new signal needed: when vol is low, A naturally goes quiet.
- **Maker C self-engages in calm.** Calm/pinned is precisely C's prime regime (lowest adverse selection). The SAME vol-floor-firing event that silences A is the trigger to switch C on. One signal, two opposite jobs.
- **Perp gate B is a per-load refinement, not a regime allocator.** It operates inside A's active periods, deciding which individual favourites to trust. It does not move capital between components; it sharpens A's expectancy.
- **Cross-market D is always-on by asset** and needs no regime signal at all; its benefit is correlation, not timing.

So the "ensemble" is mostly automatic: A and C are coupled to the single, leakage-proof, already-computed signal `realized_vol_180s` (equivalently, the vol-floor state). The only explicit dial is the vol-floor threshold that hands off between A and C, and that threshold ALREADY EXISTS in the engine (`min_realized_vol_180s_bps`). This is the crucial advantage over the failed week-level range gate: we are not trying to predict which trending-looking week is secretly toxic; we are responding to an observable, contemporaneous vol state and assigning the regime-appropriate strategy to it.

**Verdict on the signal: vol-level (the existing floor) as the single explicit hand-off dial between A and C; B and D require no allocation signal.** The ensemble is ~80% self-selecting and ~20% an explicit vol-floor hand-off.

Why NOT participation rate (2) as the driver: it is downstream of vol plus the model gates, so using it as the allocator is circular and laggier than vol itself. Why NOT the whipsaw snapshot (4): it demonstrably cannot separate the regimes ex-ante (the regime_conditional_gate negative result). Vol level is upstream, contemporaneous, and leakage-proof.

---

## 3. Assessment of expected benefit from available data

All figures @ $2,800 bankroll, Feb27-May20 window, baseline directional = +$10,466, Sharpe ~3.3, maxDD 15.4%. The May7-20 residual is -$596 (against +$11,313 entering May).

### 3a. Perp-confirmation gate (B) on the toxic residual
The May residual is a THIN tail: ~139 fills crossing mid at 54-56% (vs 35-44% healthy). The perp gate separates toxic from clean directional loads at combined OOS AUC 0.744. A confirmation gate that loads only when perp confirms would, at that AUC, avoid roughly the top half of the toxic mass while keeping most clean loads.

Grounded estimate: of the -$596 May residual, a 0.744-AUC gate plausibly recovers $250-$450 (avoids most of the strongly-flagged toxic loads, keeps the borderline ones). Crucially, applied at the FILL level (not week level), the collateral damage to the +$7.2k of healthy directional-lane profit is small, unlike config B applied globally, which gave up ~$3k. The honest caveat: the perp signal is validated on only n=112 directional loads / 57 toxic (participation had collapsed), so the lower CI on the lift is just +0.01; the full-history validation (thousands of loads) is what converts this from "plausible" to "bankable." If the lift holds at scale, B is a modest but persistent expectancy improver: think +2-4% on annualized directional return via cleaner entries, plus a meaningful maxDD reduction concentrated in calm-regime weeks.

### 3b. Maker overlay (C) in the calm regime
The calm regime is where A earns ~nothing (participation 2%) and bleeds the -$596 residual. If C instead participates there:
- It does NOT need to be a big number to dominate the calm-regime contribution, because A's calm-regime contribution is negative. Swapping a -$596 toxic residual for ANY positive rebate carry is a swing of $596 + whatever C earns.
- Rebate-only floor: at 10bps rebate on two-sided quotes over the ~weeks of calm in May, on a $2,800 book with conservative clips (0.05 frac), C's gross is small in absolute dollars but POSITIVE and direction-immune (the whale's matched-arb book is positive in every vol quartile incl the whippiest). Spread capture on top adds to it.
- The honest framing: C's value in this window is primarily (i) turning the -$596 residual off (by not forcing the toxic directional load) and (ii) adding a small positive carry, so the calm-regime swing is on the order of +$600 to +$1,200 over the May slice. The bigger value of C is PERSISTENCE: it earns in calm regimes that recur, so its annualized contribution compounds across every future calm stretch, not just May.
- Hard caveat: C is design-only. It requires the two-sided NO book + a realistic maker-fill/adverse-selection model (the build plan's Gate 1 + Gate 2), and must be validated at rebate=0 (spread capture alone must be positive) before any rebate is assumed. The whale data says the calm regime is also the MOST contested for pure arb, but as a REBATE overlay on our own directional inventory we are not racing incumbents for pure-arb flow, which softens that concern.

### 3c. Cross-market diversification (D) on portfolio Sharpe
This is the largest and most robust lever for RISK-ADJUSTED return specifically. The directional mechanism (intra-bar mean-reversion of late favourites) is the same on ETH/SOL/XRP, so each asset carries a similar positive per-market edge, but the bar-to-bar realizations are largely uncorrelated (different order flow, different pin levels, idiosyncratic 5m moves).

Portfolio-Sharpe math (standard, caveated): combining N strategies of equal Sharpe S and average pairwise correlation rho gives portfolio Sharpe S * sqrt(N / (1 + (N-1)*rho)). With the BTC core at S ~3.3 and adding ETH (and later SOL/XRP) at comparable per-asset Sharpe:
- 2 assets (BTC+ETH), rho ~0.3: portfolio Sharpe ~3.3 * sqrt(2/1.3) ~= 4.1 (+24%).
- 3 assets, rho ~0.3: ~3.3 * sqrt(3/1.6) ~= 4.5 (+36%).
- 4 assets, rho ~0.3: ~3.3 * sqrt(4/1.9) ~= 4.8 (+45%).

These are the headline persistent-return numbers and they are the cleanest part of the case, BUT they assume (i) each new asset's edge replicates BTC's (must be measured, not assumed: ETH first), (ii) the cross-asset correlation is genuinely ~0.3 not ~0.7 (crypto 5m moves can co-move in stress; measure realized correlation on overlapping markets), and (iii) thin-book slippage does not eat the edge on smaller-cap assets (SOL/XRP 5m books are thinner; the engine models depth so re-run, do not extrapolate). The diversification benefit is to SHARPE, not necessarily to total dollars per unit capital: splitting $2,800 four ways means each book is smaller, which actually HELPS the thin-book slippage problem (smaller clips).

### 3d. Combined, grounded estimate
Stacking the three honestly:
- Persistent return: B adds a few percent via cleaner entries and turns the recurring calm-regime drag positive via C; the absolute-dollar uplift over a May-like window is on the order of +$850 to +$1,650 (residual recovery + maker carry), and it RECURS every calm stretch.
- Risk-adjusted return: the headline is the diversification Sharpe lift, plausibly from ~3.3 to ~4.0-4.8 at 2-4 assets, PLUS a maxDD reduction from B's fill-level gating concentrated in the exact weeks that hurt.
- Most-likely persistent outcome: **portfolio Sharpe in the 4.0-4.5 range (vs 3.3 directional-alone) with the calm-regime drag removed**, dominated by diversification (D), de-risked at the fill level by B, and with C converting idle calm-regime capital from a small negative to a small positive.

Confidence: the diversification math is the most reliable; B is plausible-pending-scale-validation; C is the least proven (design-only, needs the maker-fill model). All three point the same direction, which is the reassuring part: the ensemble does not rely on any single uncertain component.

---

## 4. Capital allocation + risk at $2,800

### Allocation (the components are regime-coupled, so the splits are soft caps, not fixed buckets)
- **Directional A (incl. perp gate B and convex insurance): up to ~70% of bankroll exposure cap in vol/trend regimes.** In calm regimes A self-throttles to near-zero via the vol floor; its allocation is regime-state-dependent by construction.
- **Maker C: engaged in the calm regime A vacates; conservative cap ~15-20% of bankroll in two-sided notional, with a vol-scaled naked-inventory cap driven toward zero as whipsaw rises** (harder than the whale, since we lack the big arb book to swamp residual losses). C and A are largely time-disjoint, so they rarely compete for the same capital.
- **Cross-market D: the bankroll is split PER ASSET first, then A/B/C run within each asset's slice.** At 2 assets, ~$1,400 each; at 4 assets, ~$700 each. Smaller per-asset books reduce thin-book slippage. Total exposure across all assets capped so aggregate gross does not exceed the single-asset cap that the engine already enforces (`max_per_market_exposure_frac` 0.12, `clip_fraction_of_equity` 0.015 scale off equity).

### Avoiding double-counting exposure (the critical risk)
The genuine hazard is A and C both going long YES on the same market (directional load + maker quote that fills on the same side), doubling the directional bet under one regime. Mitigations, in order:
1. **Regime disjointness does most of the work:** A is near-silent exactly when C is active (low vol), so same-market overlap is rare by construction.
2. **Shared inventory accounting:** C's combined inventory uses the GLOBAL `ctx.yes_shares` (directional + participation), per the build plan, so C's repair/quote logic already nets against whatever A holds and will not pile onto a leg A already took. This is the single most important guard and it is already in the design.
3. **Shared reversal/perp gate:** the SAME reversal-risk + perp signal that gates A's loads also pulls C's quotes (one detector, two jobs). When the regime is fragile, BOTH layers shrink together, so they cannot both lever into the same toxic move.
4. **Per-market gross cap is enforced on the SUM** of directional + maker notional, not per-component, so the engine cannot exceed the per-market exposure cap by splitting across components.
5. **Cross-market caps are independent per asset** but share the equity base, so a drawdown on one asset shrinks every other asset's clip (equity-scaled sizing): automatic portfolio de-risking.

### Risk posture
Drawdown-tolerant per the user objective: the ensemble does not chase a flat curve. The maxDD improvement comes for free from (a) B trimming the toxic tail at the fill level and (b) D's diversification lowering portfolio variance, NOT from degrading the directional base. The convex_tail stays as cheap targeted insurance only.

---

## 5. Build dependencies (what must exist first)

The full ensemble backtest is finalized only once the component results land. Dependency order:

1. **Directional A core: DONE** (live, validated +$10,466 @ $2,800).
2. **Perp-confirmation gate B: validate the 0.744 lift on FULL-history directional loads** (thousands, not the 112 May-slice loads) and estimate net return of a perp-confirmation gate. This is the cheapest next step (free Binance perp data, plugs into the existing reversal model) and gates whether B is bankable. BLOCKS the B-enhanced ensemble number.
3. **Maker overlay C: build per the build plan** (Phase 1-5: DOWN/NO book, maker-fill + adverse-selection model, reversal-gated quotes, full-history A/B at $1k and $2,800, must be positive at rebate=0). BLOCKS the C contribution; this is the largest build.
4. **Cross-market D: measure ETH-5m first** (engine already parses `eth-updown-`); run br2-A on ETH, measure per-asset Sharpe AND realized BTC-ETH correlation. Then add SOL/XRP via a one-line spot-symbol addition in `discovery.rs:infer_spot_symbol_from_slug`. BLOCKS the diversification Sharpe number (which is currently a model estimate assuming edge replication + rho ~0.3).
5. **Ensemble integration + joint backtest:** once B, C, D each clear their own bar, run the combined portfolio with the shared inventory accounting and shared gate, at $2,800, with an OOS holdout for persistence. The per-lane size multipliers and the vol-floor hand-off dial are the only new wiring; both already exist in the engine.

Components B, C, D are independent and can be validated in parallel (different data, different code paths), so the gating constraint is C (the maker-fill model build), not a serial chain.
</content>
</invoke>
