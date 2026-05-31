# BTC-5m Maker Rebate Overlay: Build Plan

Date: 2026-05-30. Status: design only, no code changed.

## Framing
The maker overlay is the REGIME-COMPLEMENTARY partner to br2's directional core: directional (br2) participates in vol/trend regimes; the maker overlay participates in the CALM/low-vol/pinned regime where directional participation collapses (the May drawdown is exactly this: calm regime -> vol floor disengages -> participation 12%->2% -> the thin residual that fires is toxic). Calm is the IDEAL maker regime (lowest adverse selection). So we do not force the toxic directional residual; we switch on maker quoting there. The regime signal is easy: the existing vol floor firing IS the calm-regime detector.

The overlay is NOT a new strategy. It runs as br2 Lane 0 (the existing "participation sleeve"), harvesting rebates + spread on the balanced inventory br2 accumulates/releases each bar. The reversal-risk score gates BOTH layers.

## What already exists (do not rebuild)
- Participation sleeve stub: `bonereaper_v2.rs` Lane 0 (~1571-1636) already emits BuyYes/BuyNo maker limits at `yes_ask-tick` and `(1-yes_bid)-tick`, respects `participation_max_pair_cost`, `participation_max_inventory_delta_shares`, `participation_repair_inventory_delta_shares`; anti-stranding repair logic present. OFF by default (`participation_clip_frac: 0.0`).
- Maker fill infra: `runner.rs` `check_resting_fills` (book-cross) + `check_trade_driven_resting_fills` (trade-tick), rebate already credited (`maker_rebate_bps`, ~1899-1901); `Fill` has `rebate_usdc`, `maker`; `RunnerConfig.max_inventory_imbalance_shares` cancels heavy-side orders.
- Real YES book loaded (`book_snapshot.rs` -> ReplayEvent.bids/asks). NO side discarded.

## What is missing -> the two infra gates
### Gate 1: real two-sided (NO) book
Today NO is synthesized `1 - yes_bid - tick` (`maker_participation_native_prices` ~1011), wrong when the real NO spread differs. ReplayEvent is size-capped at 128 bytes so NO depth goes in a side channel.
- `discovery.rs`: add `down_asset_id: Option<String>` to `MarketHandle`.
- `main.rs`: `canonical_up_asset_for_row` returns UP + DOWN asset_ids; second `load_book_snapshot_async` for DOWN; fall back to synthesized NO + warn if DOWN book missing (must not block YES run).
- new `crates/pm-types/src/paired_book.rs`: `PairedBookSnapshot { ts_ns, no_bids, no_asks }`; sorted Vec per market.
- `runner.rs`: add `paired_book: &[PairedBookSnapshot]` param + cursor; expose `current_no_book` per event.
- `lib.rs Ctx`: add `no_book: Option<&PairedBookSnapshot>` (default None -> all strategies compile unchanged).
- `bonereaper_v2.rs`: `maker_participation_native_prices` uses real `no_asks[0].price - tick` when present, else synthesized fallback.

### Gate 2: maker-fill + adverse-selection model
Current fills are book-cross with no queue position / no adverse selection -> overstates fill rate + understates the exact stranding cost that killed the live engine.
- `RestingOrder` += `queue_ahead_shares` (from resting depth at submit) and `reversal_risk_at_submit`.
- fill-probability gate: `fill_frac = s/(queue_ahead+order.shares)`; `RunnerConfig.maker_queue_fill_frac_threshold` (default 0.0 = current behaviour).
- adverse-selection haircut: `effective_rebate = maker_rebate_bps * (1 - reversal_risk_at_submit * adverse_selection_scale)`; `RunnerConfig.maker_adverse_selection_scale` (default 0.0, inert; stress at 0.5-1.0). A stress dial, not ground truth — calibrate against whale onchain fills.
- rebate accounting already exists; add `reversal_risk_at_submit: Option<f32>` to `Fill` for attribution.

## Reversal gate on the maker quotes (the live-failure fix)
All in `bonereaper_v2.rs` Lane 0, mirroring the directional lane gating:
- hard suppress: `participation_max_reversal_score` (default 1.0 inert); score > thr -> pull BOTH legs this tick.
- continuous shrink: clip *= lerp(`participation_reversal_size_floor`, 1.0, 1-score).
- asymmetric leg suppression: when score high AND yes_bid rising fast, suppress only the leg that would add to the stranded side (Stoikov skew ported to quote suppression).
The reversal score (AUC 0.71, `binance_adverse_vol_30s`) detects the adverse move ~30s ahead -> with thr ~0.65 the about-to-be-picked-off leg is never placed -> no stranding.

## Repair state (capital-recycler logic)
`participation_in_repair` flag: set when `|yes_shares - no_shares| > participation_repair_inventory_delta_shares`; while in repair emit ONLY the underfilled leg and only if `pair_cost_if_filled <= participation_max_pair_cost`; clear at delta/2. Never-chase guard: skip the order if `yes_px >= event.yes_ask` (would cross -> taker). Combined inventory uses global `ctx.yes_shares` (directional + participation), so the overlay never piles onto a leg br2 already took.

## Build sequence (phased, each independently testable)
- Phase 1: preserve DOWN asset_id + audit NO-book S3 coverage (additive data only; no behaviour change). Bar: DOWN asset_id populated; coverage >=80%.
- Phase 2: load + merge NO book into replay (still unused since sleeve off -> baseline byte-identical). Diagnostic `--dump-no-book`: real_no_mid ~= 1-yes_mid but spreads differ.
- Phase 3: real NO prices in participation + activate sleeve + add reversal gate. Validate fills are maker w/ rebate, pair-cost gate holds, reversal gate cuts high-score fills.
- Phase 4: maker-fill model (queue position + adverse-selection haircut). Validate the haircut delta is negative + concentrated in high-reversal-score fills.
- Phase 5: full-history A/B at $1k AND $2,800. A=off; B=overlay conservative (clip 0.05, rebate 10bps, pair-cost 0.99, max-reversal 0.65, size-floor 0.3, adverse-scale 0.5); C=B with adverse-scale 1.0 (worst case). Bar: B adds persistent return over A on the Feb-May tape AND on an OOS holdout (recent weeks NOT in the reversal-score fit); robust even at rebate=0 (spread capture alone).

## Validation vs whale onchain fills (de-risk the fill model)
`scripts/validate_maker_fill_model.py`: load PM onchain fills (`load_pm_onchain_async` exists) for the same markets; per maker fill compute queue depth ahead + reversal score + post-fill YES-mid move (5/15/30s). Metrics: adverse-selection concentration (fraction of fills in top-20% reversal-risk), queue-fill fraction for fills vs cancels, avg post-fill adverse move. If onchain shows >40% adverse concentration, the backtest underestimates costs below adverse_selection_scale=1.0.

## Risks
1. DOWN-token S3 coverage may be patchy -> fall back to synthesized NO + caveat.
2. Pair-cost gate fooled by stale/illiquid NO ask -> require both legs `size > min_no_book_depth` (start 5).
3. Rebate programs are discretionary -> validate at rebate=0 and 10bps; must be positive at 0.
4. Reversal-gate look-ahead: coeffs fit on the same tape -> OOS holdout mandatory; set the gate threshold only after the OOS result.
5. Never-chase vs br2 takers: `submit_maker_order` (~1981) already converts a crossing limit to a taker; verify no double-exposure in tests.

## Files
Create: `crates/pm-types/src/paired_book.rs`, `scripts/validate_maker_fill_model.py`.
Modify: `discovery.rs` (down_asset_id), `main.rs` (dual load + CLI), `book_snapshot.rs` (load_no_book), `pm-types/lib.rs` (re-export), `pm-strategy/lib.rs` (Ctx.no_book, OrderRequest.reversal_score), `bonereaper_v2.rs` (real NO prices, reversal-gate fields, repair state, never-chase), `runner.rs` (RestingOrder fields, paired_book param, fill-prob gate, RunnerConfig dials, Fill attribution).
