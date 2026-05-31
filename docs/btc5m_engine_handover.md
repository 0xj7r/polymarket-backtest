# BTC5m Engine Handover

## Current Context and Objective
- Repository: `polymarket-backtest`
- Goal: continue BTC 5m directional/backtest engine work on BTC5m only, with strong post-fill diagnostics, and derive walk-forward, replay-safe late-break controls to improve last-window behavior.
- Status of current cycle: ongoing; prior long-history pass and newer mid-run diagnostics were generated, but final full-window run for the newest launch is not yet complete.

## Active Run Being Monitored
- Run ID: `20260529T062901Z-portfolio-grid-5265`
- Label: `clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8`
- Source commit used by run: `40ac170edc295cf05b9311af27823ca03e423ad0`
- Region for EC2/SSM: `us-east-1` (important; default profile is eu-west-2)
- S3 results prefix:
  - `s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/`
- Instance: `i-069194bfa6e6aa332` (running during last check)
- Monitoring session: `pm_postfill_watch`

## Confirmed Data Snapshot (latest checked)
- Downloaded markets file currently contains `10500` markets.
- Artifact file: `/tmp/btc5m_postfill_markets_062901.jsonl`
- Equivalent report metrics at 10500 markets:
  - calendar: `2026-02-27T15:40:00Z` to `2026-04-05T02:35:00Z`
  - total PnL: `+$6,309.67` (from `start 1,000` baseline)
  - active late fills and lane behavior indicate strong continuation in many windows but persistent late-crossed-mid losses.

## Diagnostics Already Generated (new at 10500)
All files written in `docs/` with source `/tmp/btc5m_postfill_markets_062901.jsonl`:
- `docs/btc5m_postfill_checkpoint_10500_regime_evolution.md`
- `docs/btc5m_postfill_checkpoint_10500_reversal_tail.md`
- `docs/btc5m_postfill_checkpoint_10500_toxic_reversal_path_model.md`
- `docs/btc5m_postfill_checkpoint_10500_crossed_mid_after_fill_model.md`
- `docs/btc5m_postfill_checkpoint_10500_gate_sim.md`
- `docs/btc5m_postfill_checkpoint_10500_late_break_gate_search.md`
- `docs/btc5m_postfill_checkpoint_10500_late_break_feature_contrast.md`
- `docs/btc5m_postfill_checkpoint_10500_late_break_gate_portfolio_sim.md`

Command used:
- `python3 scripts/run_postfill_diagnostics.py /tmp/btc5m_postfill_markets_062901.jsonl --strategy bonereaper_v2 --recent-days 30 --last-markets 10500 --test-days 30 --out-prefix docs/btc5m_postfill_checkpoint_10500 --min-fills 200 --gate-min-train-fills 600 --gate-test-fills 200 --gate-step-fills 200 --gate-epochs 800`

## Key Quantitative Findings (10500-checkpoint)

### 1) Cross-mid path remains dominant failure mode
From `reversal_tail` and model diagnostics:
- Crossed-mid path is the damaging subset.
- `crossed_mid_after_fill` summary: `714` fills, `-$10,543.62` PnL, `100%` cross-mid by definition.
- `held_side` summary: `895` fills, `$13,521.65` PnL, `0%` cross-mid.
- Late favourite confirm lanes remain profitable if held, but drawdowns are concentrated in late-cross events.

### 2) Live-safe feature/label distinction
- `regime` tags like final-range buckets are diagnostic unless path can be generated pre-close.
- Cross-mid labels are post-fill path labels; useful for offline model quality and candidates, not direct live gates.

### 3) Lane-level behavior from diagnostic contrast
- late favourite and late confirm still dominate edge and also majority of late-cross damage.
- In final range decomposition, late favourite and late confirm are profitable in narrow markets but negative in wide ranges (`mid`/`wide` bands), especially after final-range resolved bucketing.

### 4) Candidate search outcome quality
- Most direct thresholds degrade overall full-PnL when used as hard gates.
- Promising walk-forward candidates are small, throttled, and lane-specific.
- In `late_break_gate_search`, top stable candidate in tested folds:
  - `side_model_p:q4&regime_reversal_pressure:q2`
    - active folds: 3/3, removed toxic fold-level PnL: `-$372.18`, removed cross-mid `35.42%`.
- In `late_break_gate_portfolio_sim` (candidate `side_model_p:q1&side_edge_vs_fill:q2`, half throttle):
  - base `+$6,309.67`, adjusted `+$6,319.16` (+`$9.49`) with DD 23.70% -> 18.23% in this window.
- Reversal classifiers are only moderate:
  - `crossed_mid_after_fill` test AUC ~0.6368, log loss ~0.6799 (better than random but not strong enough to hard-enforce)
  - coefficients suggest prior-range, sign-flip/reversal interactions, and `side_edge_vs_fill` are useful drivers.

### 5) What was worse earlier and what changed
- We have older full-history candidate context from previous runs (no-tails, older data/flags) with very strong aggregate PnL, but not the active 062901 run state and not replay-equivalent to current live-tail+regime settings.
- The current active run is still a strict, in-flight slice at 10,500/23,705 markets; avoid treating it as final yet.

## Current Working Docs to Read First
1. `docs/active_btc5m_experiments.md`
2. `docs/btc5m_late_regime_action_report.md`
3. `docs/btc5m_postfill_checkpoint_10500_regime_evolution.md`
4. `docs/btc5m_postfill_checkpoint_10500_late_break_feature_contrast.md`
5. `docs/btc5m_postfill_checkpoint_10500_gate_sim.md`
6. `docs/btc5m_postfill_checkpoint_10500_late_break_gate_portfolio_sim.md`
7. `docs/btc5m_postfill_checkpoint_10500_reversal_tail.md`

## Immediate Continuation Plan for New Agent
1. Confirm remote run completion status:
   - `AWS_PROFILE=visumlabs AWS_REGION=us-east-1 aws s3 ls s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl`
   - if still growing, compare expected final marker 23,705 (OOS market count).
2. If incomplete, keep watching existing `pm_postfill_watch` session and continue diagnostics at each checkpoint.
3. When final `23,705` is available, rerun full-window diagnostics (same command as above with `--last-markets 23705`, optionally keep all `--..._100` settings).
4. Before implementing any strategy code changes, validate candidate impact under walk-forward half-throttle first, then only harden as a narrow, lane-specific override.

Suggested first experiments after full-run completion:
- Compare these candidate sets as offline what-ifs:
  - `side_model_p:q4&regime_reversal_pressure:q2` with half-size scaling
  - `side_edge_vs_fill:q2&risk_score:q4`
  - `risk_score:q4&prior_market_range_7d:q1`
- If candidate still weak, add/validate regime feature for post-fill cross-mid proxy with only replay-safe, short-horizon features (eg `prior market range`, `sign_flip`, `path_efficiency`, `risk`, `side_edge_vs_fill`, `seconds_to_close`).

## Commands for a Reproducible 10500+ Continuation
- Download latest checkpoint:
  - `AWS_PROFILE=visumlabs aws s3 cp s3://pm-research-backtest-prod/results/20260529T062901Z-portfolio-grid-5265/clip_0p015_gross_250_expfrac_0p12_lat500ms_cap1k_btc_5m_tail08_lc_range50_exact_profile_postpath_mem128_cf8/markets.jsonl /tmp/btc5m_postfill_markets_062901.jsonl`
- Validate artifact line count:
  - `wc -l /tmp/btc5m_postfill_markets_062901.jsonl`
- Full report once complete:
  - `python3 scripts/run_postfill_diagnostics.py /tmp/btc5m_postfill_markets_062901.jsonl --strategy bonereaper_v2 --recent-days 30 --last-markets 23705 --test-days 30 --out-prefix docs/btc5m_postfill_checkpoint_full --min-fills 200 --gate-min-train-fills 600 --gate-test-fills 200 --gate-step-fills 200 --gate-epochs 800`

## Notes and Constraints to Carry Forward
- Do not mix post-hoc final-range gates into live logic without a replay-safe approximation.
- Keep all candidate gates replay-safe (fill-time features only) when used in production/risk path.
- 5m BTC full-history coverage remains the priority; skip ETH/multi-timeframe changes unless explicitly requested.
