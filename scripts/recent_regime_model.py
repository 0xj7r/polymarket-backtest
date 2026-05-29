#!/usr/bin/env python3
"""Train an offline recent-regime logistic gate from walk-forward fills."""

import argparse
import datetime as dt
import json
import math
import os
import subprocess
import sys
from pathlib import Path
from typing import Any, Iterable, Iterator, TextIO

import numpy as np


AWS_PROFILE: str | None = None

NUMERIC_FEATURES = [
    "price",
    "side_model_p",
    "side_edge_vs_fill",
    "confidence_score",
    "risk_score",
    "market_yes_range_so_far",
    "seconds_to_close",
    "regime_whipsaw_score",
    "regime_path_efficiency",
    "regime_reversal_pressure",
    "regime_sign_flip_rate",
    "regime_realized_vol_180s_bps",
    "volatility_range",
]

TAG_FEATURES = [
    "br2_high_skew_load",
    "br2_late_confirm",
    "br2_late_favourite_load",
    "br2_convex_tail",
]


def open_input(path: str) -> tuple[TextIO, subprocess.Popen[str] | None]:
    if path == "-":
        return sys.stdin, None
    if path.startswith("s3://"):
        env = os.environ.copy()
        if AWS_PROFILE:
            env["AWS_PROFILE"] = AWS_PROFILE
        proc = subprocess.Popen(
            ["aws", "s3", "cp", path, "-"],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            env=env,
        )
        assert proc.stdout is not None
        return proc.stdout, proc
    return Path(path).open(), None


def iter_rows(f: Iterable[str]) -> Iterator[dict[str, Any]]:
    for line in f:
        line = line.strip()
        if line:
            yield json.loads(line)


def close_ts(row: dict[str, Any]) -> int:
    value = row.get("close_ts")
    if value is not None:
        return int(value)
    return int(str(row.get("slug") or "").rsplit("-", 1)[1]) + 300


def strategy_result(row: dict[str, Any], strategy: str) -> dict[str, Any]:
    return ((row.get("per_strategy") or {}).get(strategy)) or {}


def fill_won(row: dict[str, Any], side: str) -> bool | None:
    outcome = str(row.get("outcome_label") or "").lower()
    if side == "BuyYes":
        return outcome in ("yes", "up")
    if side == "BuyNo":
        return outcome in ("no", "down")
    return None


def fill_pnl(row: dict[str, Any], fill: dict[str, Any]) -> tuple[float, int]:
    won = fill_won(row, str(fill.get("side") or ""))
    shares = float(fill.get("shares") or 0.0)
    notional = float(fill.get("notional") or 0.0)
    rebate = float(fill.get("rebate_usdc") or 0.0)
    return (shares if won else 0.0) - notional + rebate, 1 if won else 0


def load_fills(path: str, strategy: str) -> list[dict[str, Any]]:
    f, proc = open_input(path)
    fills: list[dict[str, Any]] = []
    try:
        for row in iter_rows(f):
            strat = strategy_result(row, strategy)
            ts = close_ts(row)
            for fill in strat.get("fills_detail") or []:
                pnl, won = fill_pnl(row, fill)
                item = {
                    "ts": ts,
                    "date": dt.datetime.fromtimestamp(ts, tz=dt.timezone.utc).date(),
                    "tag": str(fill.get("tag") or "unknown"),
                    "side": str(fill.get("side") or "unknown"),
                    "pnl": pnl,
                    "won": won,
                    "notional": float(fill.get("notional") or 0.0),
                    "price": float(fill.get("price") or 0.0),
                    "volatility_range": float(row.get("volatility_range") or 0.0),
                }
                for key in NUMERIC_FEATURES:
                    if key in item:
                        continue
                    value = fill.get(key)
                    item[key] = float(value) if value is not None else 0.0
                fills.append(item)
    finally:
        if proc is not None:
            assert proc.stderr is not None
            stderr = proc.stderr.read()
            rc = proc.wait()
            if rc != 0:
                raise RuntimeError(stderr.strip())
        elif f is not sys.stdin:
            f.close()
    return fills


def matrix(fills: list[dict[str, Any]]) -> tuple[np.ndarray, np.ndarray, list[str]]:
    names = list(NUMERIC_FEATURES)
    names.extend(["buy_yes"])
    names.extend([f"tag:{tag}" for tag in TAG_FEATURES])
    names.extend(
        [
            "whipsaw_x_reversal",
            "whipsaw_x_low_efficiency",
            "range_x_reversal",
            "price_x_model_p",
            "edge_x_confidence",
        ]
    )
    rows = []
    y = []
    for fill in fills:
        row = [float(fill.get(key) or 0.0) for key in NUMERIC_FEATURES]
        row.append(1.0 if fill.get("side") == "BuyYes" else 0.0)
        row.extend(1.0 if fill.get("tag") == tag else 0.0 for tag in TAG_FEATURES)
        whipsaw = float(fill.get("regime_whipsaw_score") or 0.0)
        reversal = float(fill.get("regime_reversal_pressure") or 0.0)
        path_eff = float(fill.get("regime_path_efficiency") or 0.0)
        observed_range = float(fill.get("market_yes_range_so_far") or 0.0)
        price = float(fill.get("price") or 0.0)
        model_p = float(fill.get("side_model_p") or 0.5)
        edge = float(fill.get("side_edge_vs_fill") or 0.0)
        conf = float(fill.get("confidence_score") or 0.0)
        row.extend(
            [
                whipsaw * reversal,
                whipsaw * (1.0 - path_eff),
                observed_range * reversal,
                price * model_p,
                edge * conf,
            ]
        )
        rows.append(row)
        y.append(float(fill["won"]))
    return np.asarray(rows, dtype=np.float64), np.asarray(y, dtype=np.float64), names


def sigmoid(x: np.ndarray) -> np.ndarray:
    return 1.0 / (1.0 + np.exp(-np.clip(x, -35.0, 35.0)))


def train_logistic(
    x_train: np.ndarray,
    y_train: np.ndarray,
    epochs: int,
    learning_rate: float,
    l2: float,
) -> np.ndarray:
    weights = np.zeros(x_train.shape[1] + 1, dtype=np.float64)
    x_aug = np.c_[np.ones(len(x_train)), x_train]
    for _ in range(epochs):
        p = sigmoid(x_aug @ weights)
        grad = (x_aug.T @ (p - y_train)) / len(y_train)
        grad[1:] += l2 * weights[1:]
        weights -= learning_rate * grad
    return weights


def predict(weights: np.ndarray, x: np.ndarray) -> np.ndarray:
    return sigmoid(np.c_[np.ones(len(x)), x] @ weights)


def log_loss(y: np.ndarray, p: np.ndarray) -> float:
    p = np.clip(p, 1e-5, 1.0 - 1e-5)
    return float(-(y * np.log(p) + (1.0 - y) * np.log(1.0 - p)).mean())


def brier(y: np.ndarray, p: np.ndarray) -> float:
    return float(((p - y) ** 2).mean())


def summarize_gate(name: str, fills: list[dict[str, Any]], probs: np.ndarray, threshold: float) -> dict[str, Any]:
    selected = [fill for fill, p in zip(fills, probs) if p - float(fill.get("price") or 0.0) >= threshold]
    pnl = sum(float(fill["pnl"]) for fill in selected)
    wins = sum(int(fill["won"]) for fill in selected)
    cost = sum(float(fill["notional"]) for fill in selected)
    return {
        "name": name,
        "fills": len(selected),
        "pnl": pnl,
        "wins": wins,
        "win_rate": wins / len(selected) if selected else 0.0,
        "cost": cost,
    }


def fmt(value: float, decimals: int = 4) -> str:
    return f"{value:.{decimals}f}"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("markets_jsonl")
    parser.add_argument("--strategy", default="bonereaper_v2")
    parser.add_argument("--aws-profile")
    parser.add_argument("--test-days", type=int, default=30)
    parser.add_argument("--epochs", type=int, default=2500)
    parser.add_argument("--learning-rate", type=float, default=0.04)
    parser.add_argument("--l2", type=float, default=0.01)
    parser.add_argument("--edge-threshold", type=float, default=0.05)
    parser.add_argument("--out", help="write markdown report to this path")
    args = parser.parse_args()

    global AWS_PROFILE
    AWS_PROFILE = args.aws_profile
    fills = load_fills(args.markets_jsonl, args.strategy)
    if len(fills) < 100:
        raise RuntimeError(f"not enough fills: {len(fills)}")

    max_ts = max(fill["ts"] for fill in fills)
    test_start = max_ts - args.test_days * 86400 + 300
    train = [fill for fill in fills if fill["ts"] < test_start]
    test = [fill for fill in fills if fill["ts"] >= test_start]
    if len(train) < 100 or len(test) < 20:
        raise RuntimeError(f"not enough train/test fills: train={len(train)} test={len(test)}")

    x_train, y_train, names = matrix(train)
    x_test, y_test, _ = matrix(test)
    mean = x_train.mean(axis=0)
    std = x_train.std(axis=0)
    std[std < 1e-8] = 1.0
    x_train_z = (x_train - mean) / std
    x_test_z = (x_test - mean) / std

    weights = train_logistic(x_train_z, y_train, args.epochs, args.learning_rate, args.l2)
    p_train = predict(weights, x_train_z)
    p_test = predict(weights, x_test_z)
    existing_test = np.asarray([float(fill.get("side_model_p") or 0.5) for fill in test], dtype=np.float64)
    price_test = np.asarray([float(fill.get("price") or 0.0) for fill in test], dtype=np.float64)

    all_test_pnl = sum(float(fill["pnl"]) for fill in test)
    model_gate = summarize_gate("regime_logistic_edge", test, p_test, args.edge_threshold)
    existing_gate = summarize_gate("existing_model_edge", test, existing_test, args.edge_threshold)

    coef = sorted(zip(names, weights[1:] / std), key=lambda kv: abs(kv[1]), reverse=True)[:20]
    tag_rows = []
    for tag in sorted({fill["tag"] for fill in test}):
        idx = [i for i, fill in enumerate(test) if fill["tag"] == tag]
        if not idx:
            continue
        yt = y_test[idx]
        pt = p_test[idx]
        pe = existing_test[idx]
        pnl = sum(float(test[i]["pnl"]) for i in idx)
        tag_rows.append((tag, len(idx), pnl, log_loss(yt, pt), log_loss(yt, pe), brier(yt, pt), brier(yt, pe)))

    lines = []
    lines.append("# Recent Regime Logistic Gate Report")
    lines.append("")
    lines.append(f"Source: `{args.markets_jsonl}`")
    lines.append(f"Train fills: `{len(train)}` before `{dt.datetime.fromtimestamp(test_start, tz=dt.timezone.utc).isoformat()}`")
    lines.append(f"Test fills: `{len(test)}` in final `{args.test_days}` days")
    lines.append("")
    lines.append("## Probability Quality")
    lines.append("")
    lines.append("| Split | Model | Log Loss | Brier |")
    lines.append("|---|---|---:|---:|")
    lines.append(f"| train | regime logistic | {fmt(log_loss(y_train, p_train))} | {fmt(brier(y_train, p_train))} |")
    lines.append(f"| test | regime logistic | {fmt(log_loss(y_test, p_test))} | {fmt(brier(y_test, p_test))} |")
    lines.append(f"| test | existing side_model_p | {fmt(log_loss(y_test, existing_test))} | {fmt(brier(y_test, existing_test))} |")
    lines.append("")
    lines.append("## PnL Gate On Final Window")
    lines.append("")
    lines.append(f"All final-window fills: `{len(test)}`, PnL `${all_test_pnl:.2f}`.")
    lines.append("")
    lines.append("| Gate | Fills | PnL | Cost | Wins | Win Rate |")
    lines.append("|---|---:|---:|---:|---:|---:|")
    for row in (existing_gate, model_gate):
        lines.append(
            f"| {row['name']} | {row['fills']} | ${row['pnl']:.2f} | ${row['cost']:.2f} | "
            f"{row['wins']} | {row['win_rate'] * 100.0:.2f}% |"
        )
    lines.append("")
    lines.append("## Test Metrics By Fill Tag")
    lines.append("")
    lines.append("| Tag | Fills | PnL | Logistic LL | Existing LL | Logistic Brier | Existing Brier |")
    lines.append("|---|---:|---:|---:|---:|---:|---:|")
    for tag, count, pnl, ll_m, ll_e, br_m, br_e in tag_rows:
        lines.append(f"| {tag} | {count} | ${pnl:.2f} | {ll_m:.4f} | {ll_e:.4f} | {br_m:.4f} | {br_e:.4f} |")
    lines.append("")
    lines.append("## Largest Coefficients")
    lines.append("")
    lines.append("| Feature | Coefficient |")
    lines.append("|---|---:|")
    for name, value in coef:
        lines.append(f"| {name} | {value:.4f} |")
    lines.append("")
    output = "\n".join(lines)
    if args.out:
        Path(args.out).write_text(output + "\n")
    else:
        print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
