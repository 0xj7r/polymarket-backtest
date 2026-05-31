# BTC5m Regime-Aware Architecture: Design Spec

Date: 2026-05-29
Status: Proposed (design approved in shape; pending spec review)
Strategy: `bonereaper_v2` ("br2"), BTC 5-minute directional taker
Scope: regime detection + regime-conditional behavior to maintain positive P&L through regime change. The world-class favourite model is the explicit NEXT epoch (see end), out of scope here.

## 1. Objective

The engine is strongly profitable historically (23,705-market full history +$8,990, Sharpe ~3.4) but the recent ~30 days collapsed to roughly flat (+$45 last_30d, -$24 last_7d). The required outcome is not merely smaller drawdown: the engine must yield **clearly positive P&L in the current regime, and stay positive across future regime changes.**

## 2. Diagnosis (measured)

All numbers from the full-history funnel + fills (`/tmp/btc5m_tail08_markets.jsonl`, config `..._exact_profile_mem128_cf8`, identical to the in-flight `062901` run minus post-fill-path logging).

1. **It is not fewer markets or fewer setups.** Markets evaluated (`mkts_with_checks`) stays ~0.99 across all windows. What collapses is the emit/check rate.
2. **Participation collapse is a static vol floor meeting a lower-vol regime.** A static realized-vol floor (`min_realized_vol_180s_bps`, runtime ~1.25 bps) increasingly rejects candidates as the regime's volatility compresses. `low_vol_fail` share of checks rose first_30d -> last_7d: high_skew 48%->84%; late_favourite 32%->62%; late_confirm 35%->62%. Active rate fell 20.5%->10.9%->5.6%->4.0%.
3. **The signature of the regime is a sign-flip, invisible to per-fill features.** The lowest-vol tranche (mean fill realized vol 1.25-1.5 bps) made **+$5.63/mkt (+$2,105) in first_30d but -$11.57/mkt (-$1,365) in last_30d**, with identical fill-time features. Only the rolling background differs. This is why prior per-fill toxic-fill classifiers inverted out-of-sample (e.g. `toxic_midwide` test AUC 0.44).
4. **The recent regime is not uniformly unprofitable; its losses are concentrated.** last_30d decomposes to **+EV cells +$2,317 vs -EV cells -$2,272, net +$45**. The -EV book concentrates in the floor-band (1.25-1.5 bps) across all three momentum lanes:

   | Lane | 1.25-1.5 band (last_30d) |
   |---|---|
   | late_favourite | -$927 (-$20.59/mkt, 67% win) |
   | high_skew | -$341 (-$14.21/mkt, 62% win) |
   | late_confirm | -$303 (-$7.22/mkt, 69% win) |
   | subtotal | **-$1,571** |

   The 1.5-3.0 bands stayed +EV recently (late_confirm 1.5-2.0 +$620/81% win; late_favourite 2.0-3.0 +$333; high_skew 1.5-2.0 +$214). `convex_tail` is +EV in every bucket.
5. **Per-lane vol robustness.** Over full history `high_skew_load` and `convex_tail` are profitable in every vol bucket, but high_skew's *lowest* band sign-flipped recently like the others (its full-history +$8.32/mkt at 1.25-1.5 was a first_30d artifact). So the vol floor is not too high; the toxic band sits at the floor, and lowering it would feed toxicity.

**Implication.** Cutting the toxic floor-band in the detected regime takes last_30d from +$45 toward ~+$1,600 -- in a near-flat window, removing the concentrated loss *is* the positive P&L. This is achievable, but only if a **replay-safe** regime signal separates the toxic 1.25-1.5-recent cell from the profitable 1.25-1.5-first_30d cell prospectively (no hindsight).

**Honest ceiling.** Even perfect cell separation yields ~+$2.3k/30d, below the historical +$5.4k, because the high-vol setups (3.0+ bps) that drove the extra ~$3k genuinely stopped occurring. Realistic target: **solidly positive (order +$1.5-2.3k/30d), below the historical peak.**

## 3. Architecture: one signal, three depths

The same replay-safe regime signal is applied at three depths -- belief, size, structure -- so the layers can never disagree about what regime it is. Levers, by decision:

- **B-lite (belief): IN, ship first.** Regime as pooled interaction features in the existing meta-calibrator.
- **A (size): IN, ship second, primary positive-P&L lever.** Band-targeted, two-sided regime-conditional sizing on the momentum lanes.
- **B-heavy (per-regime-band segmented calibrators): DROPPED.** The recent profit-driving vol bands hold only 84 fills (2.0-3.0) and 21 (3.0+); per-band-per-lane cells fall to 2-13. Segmenting reproduces the documented OOS inversion.
- **C (config / floor switch): DEFERRED to Phase 3, narrowed** to a single one-directional move (raise the fragile-lane floor in the toxic-hard band; never lower, never disable a lane).

### 3.1 Shared regime estimator (load-bearing)

New struct/function in `crates/pm-strategy/src/regime.rs`, beside `WhipsawRiskSnapshot` (the shared lower module consumed by both the strategy and model crates; avoids a circular dependency). Computed at fill time from prior-closed-market and fill-time inputs only (zero look-ahead).

Inputs (all already present, all verified replay-safe):
- `v_fill = whipsaw.realized_vol_180s_bps` -- the instantaneous reading the static floor tests (`regime.rs`, `WhipsawRiskSnapshot::from_history`).
- `v_bg_3d = ctx.prior_market_range_3d`, `v_bg_1d = ctx.prior_market_range_1d` -- slow rolling cross-market vol proxy from the deque of prior CLOSED markets (`bonereaper_v2.rs`, populated by `walkforward.rs prior_market_range_mean`, window 288 = one UTC day of 5m markets). This is the axis the sign-flip lives on.
- `flip = whipsaw.sign_flip_rate`, `eff = whipsaw.path_efficiency` -- fill-time churn signals.

Outputs:
```
continuous toxicity in [0,1]:
  low_vol_pressure = clamp((VOL_REF - v_fill) / VOL_REF, 0, 1)            // distance below healthy floor
  chop             = (1 - eff) * flip                                      // unproductive churn
  bg_divergence    = clamp((v_bg_3d - v_bg_1d) / v_bg_3d, 0, 1)            // background vol decaying = the collapse
  toxicity = clamp(0.55*low_vol_pressure + 0.30*chop + 0.15*bg_divergence, 0, 1)

discrete band (for C only):
  Healthy    : v_fill >= VOL_REF and toxicity < 0.4
  LowVolSoft : VOL_REF*0.6 <= v_fill < VOL_REF                            // the 1.25-1.5 bps tranche
  LowVolHard : v_fill < VOL_REF*0.6

confidence = min(prior_closed_count / 288, 1.0)                          // lerps every effect toward neutral during warmup
```

`VOL_REF` anchors at the healthy floor (~1.5 bps, the upper edge of the flip tranche), a **frozen full-history constant, never fit per-regime** (recent data is too thin). On degenerate inputs (empty warmup deque, `WhipsawRiskSnapshot::default()` zeros) `toxicity = 0`, so cold start is identically the baseline.

Why it separates the sign-flip: in first_30d the 1.25-1.5 tranche sat in a hot, non-collapsing background (`bg_divergence ~ 0`, moderate toxicity, keep size). In last_30d the genuine high-vol setups stopped, so `v_bg_1d` fell faster than `v_bg_3d`, pushing `bg_divergence` up while `v_fill` sat in the floor band (high toxicity, throttle). The estimator discriminates on the rolling background -- precisely the axis the per-fill model is blind to.

### 3.2 Layer B-lite -- regime as pooled interaction feature (belief)

Inject `toxicity`, `bg_divergence`, and the interaction `toxicity * side_edge_vs_fill` into `MetaFeatures`. The single pooled stack (84-feature logit + 10 depth-2 trees) then represents "in a toxic/contracting regime, discount edge" learned across full history where both tranches are populated -- it cannot starve because training stays pooled. A depth-2 tree split on `toxicity` above a split on vol literally carves first_30d from last_30d.

Insertion (verified): populate the features in `MetaFeatures::from_raw_with_market_context` (`crates/pm-model/src/lib.rs`, the call site already receives `market_context` carrying the prior-range values); extend the `META_FEATURES` vector. No signature change to `predict_side_win_probability` or `fit_batch` (they consume `MetaFeatures` opaquely; the tree ensemble picks up new columns for free). Training path unchanged: pooled fit in `walkforward.rs`; `MetaTrainingSample.market_idx` lets the trainer attach per-market regime at sample-build time. `ModelState.meta_calibrator` is left whole -- no map-by-band, no snapshot-format change, zero blast radius.

### 3.3 Layer A -- band-targeted, two-sided regime-conditional sizing (size; primary lever)

A continuous multiplier driven by the shared `toxicity`, applied **only to the momentum lanes**, band-targeted so it concentrates where the loss concentrates:

```
regime_size_mult(lane, toxicity, vol_band):
  defensive (toxic floor-band): 1 - LAMBDA_DOWN * toxicity   floored at SIZE_FLOOR (~0.25-0.35, never 0)
  offensive (+EV mid bands):    1 + LAMBDA_UP   * (1-toxicity) capped at SIZE_CAP (~1.5), bounded by thin samples
```

- `late_favourite` -- defensive throttle concentrated on the 1.25-1.5 band (its -$927 cell is the single biggest toxic cell); modest offensive size-up on 2.0-3.0.
- `late_confirm` -- defensive throttle on 1.25-1.5 and the small toxic 2.0-3.0 cell; offensive size-up on the +EV 1.5-2.0 band.
- `high_skew_load` -- defensive throttle on 1.25-1.5 only (it sign-flipped there); keep 1.5-3.0.
- `convex_tail` -- **pinned to 1.0** (regime-robust, +EV in every bucket; touching it is pure downside).
- The robust portions of high_skew are otherwise left near 1.0; any regression to robust-lane PnL fails the promotion bar.

Insertion (verified): add `regime_size_mult` as one more factor in each fragile lane's existing taper product (`bonereaper_v2.rs`, late_favourite chain `clip * levels * price_taper * edge_taper * range_size_taper * fragile_taper * risk_size_mult`; late_confirm clip expression). New config fields (`recent_regime_size_lambda_down/up`, `recent_regime_size_floor/cap`, per-lane enables) near the existing `recent_regime_gate_*` block, all defaulting to the no-op (floor=cap=1.0 / disabled), mirroring `recent_regime_gate_enabled = false`.

Reuse the dormant `recent_regime_gate` plumbing (config block, snapshot delivery path, per-lane wiring sites) but feed it the new `toxicity` estimator and a throttle map, not the old logistic. The 24 baked gate coefficients are retired as suspect (fit in the lineage of the inverted classifier). Boolean `>= 0.08` gate becomes a continuous taper map.

Double-counting control: B-lite owns the smooth expectancy correction (discount edge); A owns the bounded, band-keyed variance/drawdown haircut and the offensive size-up. A's `LAMBDA_DOWN/UP` are tuned against the **B-lite-adjusted** baseline, not the raw baseline, so the two never double-correct.

### 3.4 Layer C -- regime floor switch (structure). DEFERRED, narrowed.

Only if Phases 1+2 leave a measurable structural participation loss that sizing and belief cannot reach. The defensible residual is a single one-directional move: in the `LowVolHard` band, *raise* the fragile-lane floor to starve the toxic sub-tranche at source. Never lower it, never disable a lane, never loosen in healthy regimes. Insertion: the floor comparisons are single-expression gates in `bonereaper_v2.rs` reading `self.cfg.{lane}_min_realized_vol_180s_bps`; a per-regime `effective_min_vol(cfg, lane, band)` swaps the read. Note: in-code floor defaults are `0.0` (the 1.25 bps is a runtime/sweep value), so C reads a regime-selected profile, not an assumed baseline. Broad static-param re-tuning is otherwise deprioritized: the engine works historically; the problem is regime-specific.

## 4. Phasing and validation

Shared harness: the existing walk-forward fold machinery (`walkforward.rs`), trained on strictly-prior folds (zero leakage for B-lite). Define folds whose **test windows are exactly last_30d and last_7d** (the negative slice), trained only on prior data, plus a **stratified report on the 1.25-1.5 bps tranche** (first_30d vs last_30d pnl/mkt) so the sign-flip is visible directly. Report per arm and per lane: PnL/mkt, Sharpe, max drawdown, fill count, `low_vol_fail` / `active_rate` participation stats, and the +EV/-EV cell decomposition.

- **Phase 0 -- shared estimator + sign-flip unit test.** Land `RegimeEstimate` in `regime.rs`, no behavior change. GATE: `toxicity` scores the first_30d 1.25-1.5 tranche low and the last_30d tranche high. If it does not separate, stop -- nothing downstream works.
- **Phase 1 -- B-lite (belief).** Add the three pooled features. Bar: OOS log-loss on the recent fold improves AND the 1.25-1.5 tranche loss shrinks toward zero, with no degradation to high_skew/convex_tail on any slice and full-history PnL within ~2% of +$8,990.
- **Phase 2 -- A (size), tuned on the B-lite-adjusted baseline.** Add band-targeted two-sided `regime_size_mult` to the momentum lanes. Bar: **last_30d and last_7d turn clearly positive** (target order +$1.5-2.3k/30d), no regression to +EV cells or robust lanes, healthy low-vol fills keep `regime_size_mult ~ 1` (offensive/defensive only fire under high toxicity), and negative-slice fill count does not drop more than ~15% (proves throttle not gate).
- **Phase 3 -- C (structure), gated on residual.** Only if a structural participation loss remains. One-directional floor-raise on fragile lanes; must not suppress the rare genuine high-vol (3.0+) setup.

Decide by the ablation, not the prior. Ship the smallest combination that clears the positive-P&L bar.

## 5. YAGNI and risks

Deliberately NOT doing: (1) segmenting the meta-calibrator by regime band (starves); (2) lowering the vol floor anywhere (feeds the toxic tranche); (3) disabling any lane in any regime; (4) throttling convex_tail or the robust high_skew bands; (5) reusing the 24 baked gate coefficients; (6) the `MarketResult` schema change for `mean_fill_realized_vol_180s_bps` unless the range proxy proves too noisy; (7) per-fill toxic-fill classifiers (invert because per-fill features are identical across the sign-flip); (8) broad static-param re-tuning.

Top risks and mitigations:
1. **B-lite re-inverts OOS.** Pooled, regularized interaction (3 of ~87 features, existing L2/weight-clip), frozen `VOL_REF`. The Phase-1 ablation is itself the falsification test.
2. **B-lite and A double-count, over-suppressing a still-+EV lane.** Sequencing: A tuned against the B-lite-adjusted baseline; `SIZE_FLOOR >= 0.25` so A never zeros a lane.
3. **A throttles healthy low-vol fills that happen to sit in the floor band.** `toxicity` includes `bg_divergence`, not just `low_vol_pressure`, so a low-vol fill in a still-healthy background scores low toxicity and keeps full size -- the exact axis separating first_30d (+$5.63) from last_30d (-$11.57). The Phase-0 sign-flip test guards it.
4. **Offensive size-up overfits thin +EV cells (16-32 mkts).** `SIZE_CAP` modest (~1.5), offensive gated on low toxicity, validated as a separable arm; drop if it does not clear the bar.

## 6. Next epoch (out of scope here): world-class favourite model

Once the regime architecture holds positive P&L through regime change, the next epoch is a **world-class favourite model**: predict whether a forming favourite will hold vs reverse, to capture P&L even in mid-range and strongly-reversing markets (not always possible, and that is acceptable). This is precisely the `crossed_mid_after_fill` problem -- the dominant loss mode (late_favourite/late_confirm crossing back through mid after fill). It is trained on full history (not per-regime-band, so it does not starve) and plugs into the same `meta_calibrator` seam as B-lite, consuming the same replay-safe feature plumbing. The regime architecture is designed not to preclude it: the favourite model becomes a stronger belief layer that the regime estimator continues to modulate.
