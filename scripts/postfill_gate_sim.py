#!/usr/bin/env python3
"""Walk-forward simulation of post-fill reversal-risk gates."""

from __future__ import annotations

import argparse
import datetime as dt
import math
from collections import defaultdict
from pathlib import Path
from typing import Any

import numpy as np

import postfill_reversal_model as prm


LANES = [
    "all",
    "br2_high_skew_load",
    "br2_late_confirm",
    "br2_late_favourite_load",
]


def money(value: float) -> str:
    return f"${value:,.2f}"


def pct(value: float) -> str:
    return f"{value:.2%}"


def acc() -> dict[str, float]:
    return defaultdict(float)


def add_candidate(
    row: dict[str, float],
    test_pairs: list[tuple[dict[str, Any], float]],
    threshold: float,
) -> None:
    removed = [fill for fill, prob in test_pairs if prob >= threshold]
    kept = [fill for fill, prob in test_pairs if prob < threshold]
    removed_pnl = sum(float(fill["pnl"]) for fill in removed)
    kept_pnl = sum(float(fill["pnl"]) for fill in kept)
    base_pnl = removed_pnl + kept_pnl
    row["folds"] += 1
    row["base_pnl"] += base_pnl
    row["removed_pnl"] += removed_pnl
    row["kept_pnl"] += kept_pnl
    row["removed_fills"] += len(removed)
    row["tested_fills"] += len(test_pairs)
    row["removed_target"] += sum(1 for fill in removed if fill["target"])
    row["removed_crossed_mid"] += sum(1 for fill in removed if fill["post_crossed_mid"])
    row["removed_cost"] += sum(float(fill["notional"]) for fill in removed)


def train_predict(
    train: list[dict[str, Any]],
    test: list[dict[str, Any]],
    epochs: int,
    learning_rate: float,
    l2: float,
) -> tuple[np.ndarray, np.ndarray, np.ndarray, np.ndarray]:
    x_train, y_train, _ = prm.matrix(train)
    x_test, y_test, _ = prm.matrix(test)
    mean = x_train.mean(axis=0)
    std = x_train.std(axis=0)
    std[std < 1e-8] = 1.0
    weights = prm.train_weighted_logistic(
        (x_train - mean) / std,
        y_train,
        epochs,
        learning_rate,
        l2,
    )
    p_train = prm.predict(weights, (x_train - mean) / std)
    p_test = prm.predict(weights, (x_test - mean) / std)
    return y_train, p_train, y_test, p_test


def auc(y: np.ndarray, p: np.ndarray) -> float:
    return prm.auc(y, p)


def log_loss(y: np.ndarray, p: np.ndarray) -> float:
    return prm.log_loss(y, p)


def brier(y: np.ndarray, p: np.ndarray) -> float:
    return prm.brier(y, p)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("markets_jsonl")
    parser.add_argument("--strategy", default="bonereaper_v2")
    parser.add_argument(
        "--target",
        choices=[
            "crossed_mid_after_fill",
            "adverse_soft_finish",
            "toxic_crossed_mid",
            "toxic_reversal_path",
        ],
        default="toxic_reversal_path",
    )
    parser.add_argument("--min-train-fills", type=int, default=600)
    parser.add_argument("--test-fills", type=int, default=300)
    parser.add_argument("--step-fills", type=int, default=300)
    parser.add_argument("--epochs", type=int, default=1200)
    parser.add_argument("--learning-rate", type=float, default=0.035)
    parser.add_argument("--l2", type=float, default=0.02)
    parser.add_argument("--source-label")
    parser.add_argument("--out-md", required=True)
    args = parser.parse_args()

    fills = prm.load_fills(args.markets_jsonl, args.strategy, args.target)
    fills.sort(key=lambda fill: (int(fill["ts"]), str(fill["tag"])))
    if len(fills) < args.min_train_fills + args.test_fills:
        raise RuntimeError(
            f"not enough fills for walk-forward: fills={len(fills)} "
            f"required={args.min_train_fills + args.test_fills}"
        )

    candidates: dict[str, dict[str, float]] = defaultdict(acc)
    quality = acc()
    quantiles = [0.70, 0.80, 0.90, 0.95]
    fold_count = 0
    fold_lines: list[str] = []

    start = args.min_train_fills
    while start + args.test_fills <= len(fills):
        train = fills[:start]
        test = fills[start : start + args.test_fills]
        y_train, p_train, y_test, p_test = train_predict(
            train,
            test,
            args.epochs,
            args.learning_rate,
            args.l2,
        )
        fold_count += 1
        quality["folds"] += 1
        quality["test_fills"] += len(test)
        quality["test_pnl"] += sum(float(fill["pnl"]) for fill in test)
        quality["test_targets"] += float(y_test.sum())
        quality["test_logloss_sum"] += log_loss(y_test, p_test) * len(test)
        quality["test_brier_sum"] += brier(y_test, p_test) * len(test)
        fold_auc = auc(y_test, p_test)
        if not math.isnan(fold_auc):
            quality["auc_sum"] += fold_auc
            quality["auc_folds"] += 1

        test_start = dt.datetime.fromtimestamp(int(test[0]["ts"]), tz=dt.timezone.utc)
        test_end = dt.datetime.fromtimestamp(int(test[-1]["ts"]), tz=dt.timezone.utc)
        fold_lines.append(
            f"| {fold_count} | {len(train)} | {len(test)} | "
            f"{test_start.date()} | {test_end.date()} | "
            f"{money(sum(float(fill['pnl']) for fill in test))} | "
            f"{float(y_test.mean()):.2%} | {log_loss(y_test, p_test):.4f} | "
            f"{fold_auc:.4f} |"
        )

        for quantile in quantiles:
            global_threshold = float(np.quantile(p_train, quantile))
            add_candidate(
                candidates[f"all_lanes:q{quantile:.2f}:global_threshold"],
                list(zip(test, p_test)),
                global_threshold,
            )
            for lane in LANES[1:]:
                train_lane_probs = np.asarray(
                    [prob for fill, prob in zip(train, p_train) if fill["tag"] == lane],
                    dtype=np.float64,
                )
                test_pairs = [(fill, prob) for fill, prob in zip(test, p_test) if fill["tag"] == lane]
                if len(train_lane_probs) < 25 or not test_pairs:
                    continue
                threshold = float(np.quantile(train_lane_probs, quantile))
                add_candidate(candidates[f"{lane}:q{quantile:.2f}:lane_threshold"], test_pairs, threshold)
        start += args.step_fills

    lines = [
        "# BTC5m Post-Fill Gate Simulation",
        "",
        f"Source: `{args.source_label or args.markets_jsonl}`",
        f"Target: `{args.target}`",
        f"Fills: `{len(fills)}`",
        f"Min train fills: `{args.min_train_fills}`",
        f"Test fills per fold: `{args.test_fills}`",
        f"Step fills: `{args.step_fills}`",
        "",
        "This is an offline diagnostic. It does not prove live performance, but it is stricter than a single split because thresholds are fit only on earlier fills and applied to later fills.",
        "",
        "## Fold Quality",
        "",
        "| Folds | Test Fills | Test PnL | Target Rate | Log Loss | Brier | Mean Fold AUC |",
        "|---:|---:|---:|---:|---:|---:|---:|",
        f"| {int(quality['folds'])} | {int(quality['test_fills'])} | {money(quality['test_pnl'])} | "
        f"{pct(quality['test_targets'] / quality['test_fills']) if quality['test_fills'] else '0.00%'} | "
        f"{quality['test_logloss_sum'] / quality['test_fills'] if quality['test_fills'] else 0.0:.4f} | "
        f"{quality['test_brier_sum'] / quality['test_fills'] if quality['test_fills'] else 0.0:.4f} | "
        f"{quality['auc_sum'] / quality['auc_folds'] if quality['auc_folds'] else 0.0:.4f} |",
        "",
        "## Candidate Gate Outcomes",
        "",
        "Improvement assumes full removal of high-risk fills. `Half-Throttle Improvement` assumes high-risk fill size is cut by 50%, so PnL contribution is also halved.",
        "",
        "| Candidate | Folds | Tested Fills | Removed Fills | Removed Cost | Removed PnL | Kept PnL | Full-Removal Improvement | Half-Throttle Improvement | Removed Target Rate | Removed Cross-Mid Rate |",
        "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|",
    ]
    ranked = sorted(
        candidates.items(),
        key=lambda item: -(-item[1]["removed_pnl"]),
    )
    for name, row in ranked:
        removed_fills = int(row["removed_fills"])
        if removed_fills == 0:
            continue
        full_improvement = -row["removed_pnl"]
        half_improvement = -row["removed_pnl"] * 0.5
        lines.append(
            f"| {name} | {int(row['folds'])} | {int(row['tested_fills'])} | {removed_fills} | "
            f"{money(row['removed_cost'])} | {money(row['removed_pnl'])} | {money(row['kept_pnl'])} | "
            f"{money(full_improvement)} | {money(half_improvement)} | "
            f"{pct(row['removed_target'] / removed_fills)} | "
            f"{pct(row['removed_crossed_mid'] / removed_fills)} |"
        )

    lines.extend(
        [
            "",
            "## Folds",
            "",
            "| Fold | Train Fills | Test Fills | Test Start | Test End | Test PnL | Target Rate | Log Loss | AUC |",
            "|---:|---:|---:|---|---|---:|---:|---:|---:|",
            *fold_lines,
            "",
        ]
    )

    Path(args.out_md).parent.mkdir(parents=True, exist_ok=True)
    Path(args.out_md).write_text("\n".join(lines))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
