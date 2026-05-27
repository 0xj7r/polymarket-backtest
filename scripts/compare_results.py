#!/usr/bin/env python3
"""
Simple result comparison helper for Polymarket backtest summaries + manifests.

Usage:
  python scripts/compare_results.py results/run1/summary.json results/run2/summary.json ...
"""

import json
import sys
from pathlib import Path
from typing import Any, Dict, List


def load_summary(path: Path) -> Dict[str, Any]:
    with open(path) as f:
        return json.load(f)


def get_bonereaper(summary: Dict) -> Dict:
    per_strat = summary.get("per_strategy", {})
    return per_strat.get("bonereaper_v2", {}) or per_strat.get("bonereaper_lite", {})


def fmt_num(value: Any, decimals: int = 2) -> str:
    if value is None:
        return "N/A"
    try:
        return f"{float(value):.{decimals}f}"
    except (TypeError, ValueError):
        return "N/A"


def main(paths: List[str]):
    rows = []
    for p in paths:
        path = Path(p)
        try:
            s = load_summary(path)
            br = get_bonereaper(s)
            manifest_path = path.parent / "run_manifest.json"
            manifest = {}
            if manifest_path.exists():
                with open(manifest_path) as mf:
                    manifest = json.load(mf)

            rows.append(
                {
                    "path": str(path),
                    "markets": s.get("markets_succeeded", 0),
                    "compounded_return_pct": br.get("compounded_return_pct"),
                    "max_drawdown_pct": br.get("path_max_drawdown_pct"),
                    "sharpe": br.get("sharpe_ratio"),
                    "profile": manifest.get("profile_path"),
                    "git_sha": (manifest.get("git_sha") or "")[:8],
                }
            )
        except Exception as e:
            print(f"Error loading {path}: {e}", file=sys.stderr)

    # Sort by compounded return desc
    rows.sort(key=lambda r: (r["compounded_return_pct"] or -999), reverse=True)

    print(
        f"{'Path':<45} {'Markets':>8} {'Return%':>10} {'MaxDD%':>9} {'Sharpe':>8} {'Profile':<30} {'SHA'}"
    )
    print("-" * 130)
    for r in rows:
        print(
            f"{r['path']:<45} {r['markets']:>8} "
            f"{fmt_num(r['compounded_return_pct']):>10} "
            f"{fmt_num(r['max_drawdown_pct']):>9} "
            f"{fmt_num(r['sharpe']):>8} "
            f"{str(r['profile'] or ''):<30} {r['git_sha']}"
        )


if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python scripts/compare_results.py <summary1.json> [summary2.json ...]")
        sys.exit(1)
    main(sys.argv[1:])
