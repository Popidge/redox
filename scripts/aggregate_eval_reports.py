#!/usr/bin/env python3
"""Aggregate multiple eval report JSON files across seeds."""

from __future__ import annotations

import argparse
import json
import re
import statistics
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Aggregate Redox eval reports")
    parser.add_argument(
        "reports",
        nargs="+",
        type=Path,
        help="Paths to report_seedXXXX.json files",
    )
    parser.add_argument(
        "--out", type=Path, default=None, help="Optional output JSON path"
    )
    return parser.parse_args()


def mean_std(values: list[float]) -> dict[str, float]:
    if not values:
        return {"mean": 0.0, "std": 0.0}
    if len(values) == 1:
        return {"mean": values[0], "std": 0.0}
    return {"mean": statistics.mean(values), "std": statistics.pstdev(values)}


def classify_error(message: str, phase: str) -> str:
    text = message.lower()
    if not text:
        return "none"
    if phase == "transform":
        return "iron_parse_or_oxidize"

    if "expected value, found crate" in text:
        return "name_resolution"
    if "cannot find" in text:
        return "name_resolution"
    if "this function takes" in text and "argument" in text:
        return "wrong_signature_or_call"
    if "mismatched types" in text:
        return "type_mismatch"
    if "unexpectedtoken" in text or "parse error" in text:
        return "parse_error"
    if "expected `i32`, found closure" in text:
        return "closure_return_shape"
    return "other"


def aggregate(reports: list[dict[str, Any]]) -> dict[str, Any]:
    arms = ("rust", "iron")
    arm_metrics: dict[str, dict[str, list[float]]] = {
        arm: {"compile_at_1": [], "test_at_1": [], "transform_rate": []} for arm in arms
    }

    per_family_rates: dict[str, dict[str, dict[str, list[float]]]] = {
        arm: defaultdict(lambda: {"compile": [], "test": []}) for arm in arms
    }

    error_taxonomy: dict[str, Counter[str]] = {arm: Counter() for arm in arms}

    for report in reports:
        for arm in arms:
            data = report[arm]
            total = data["total"]
            arm_metrics[arm]["compile_at_1"].append(float(data["compile_at_1"]))
            arm_metrics[arm]["test_at_1"].append(float(data["test_at_1"]))
            arm_metrics[arm]["transform_rate"].append(
                data["transform_pass"] / total if total else 0.0
            )

            for family, stats in data["per_family"].items():
                n = stats["total"]
                per_family_rates[arm][family]["compile"].append(
                    stats["compile"] / n if n else 0.0
                )
                per_family_rates[arm][family]["test"].append(
                    stats["test"] / n if n else 0.0
                )

            for row in data.get("rows", []):
                if not row.get("transform_ok", True):
                    error_taxonomy[arm][
                        classify_error(row.get("transform_error", ""), "transform")
                    ] += 1
                elif not row.get("compile_ok", False):
                    error_taxonomy[arm][
                        classify_error(row.get("compile_error", ""), "compile")
                    ] += 1
                elif not row.get("test_ok", False):
                    error_taxonomy[arm][
                        classify_error(row.get("test_error", ""), "test")
                    ] += 1

    out: dict[str, Any] = {
        "num_reports": len(reports),
        "summary": {},
        "per_family": {},
        "error_taxonomy": {},
    }

    for arm in arms:
        out["summary"][arm] = {
            "compile_at_1": mean_std(arm_metrics[arm]["compile_at_1"]),
            "test_at_1": mean_std(arm_metrics[arm]["test_at_1"]),
            "transform_rate": mean_std(arm_metrics[arm]["transform_rate"]),
        }

        out["per_family"][arm] = {}
        for family, metrics in sorted(per_family_rates[arm].items()):
            out["per_family"][arm][family] = {
                "compile": mean_std(metrics["compile"]),
                "test": mean_std(metrics["test"]),
            }

        out["error_taxonomy"][arm] = dict(error_taxonomy[arm].most_common())

    # Derived deltas for quick interpretation
    out["delta_iron_minus_rust"] = {
        "compile_at_1_mean": out["summary"]["iron"]["compile_at_1"]["mean"]
        - out["summary"]["rust"]["compile_at_1"]["mean"],
        "test_at_1_mean": out["summary"]["iron"]["test_at_1"]["mean"]
        - out["summary"]["rust"]["test_at_1"]["mean"],
    }

    return out


def print_human(agg: dict[str, Any]) -> None:
    print(f"Reports aggregated: {agg['num_reports']}")
    for arm in ("rust", "iron"):
        s = agg["summary"][arm]
        print(f"\n{arm.upper()}")
        print(
            f"- compile@1 mean±std: {s['compile_at_1']['mean']:.3f} ± {s['compile_at_1']['std']:.3f}"
        )
        print(
            f"- test@1 mean±std: {s['test_at_1']['mean']:.3f} ± {s['test_at_1']['std']:.3f}"
        )
        print(
            f"- transform mean±std: {s['transform_rate']['mean']:.3f} ± {s['transform_rate']['std']:.3f}"
        )
        print("- top failure classes:")
        for k, v in list(agg["error_taxonomy"][arm].items())[:5]:
            print(f"  - {k}: {v}")

    d = agg["delta_iron_minus_rust"]
    print("\nIRON - RUST")
    print(f"- compile@1 mean delta: {d['compile_at_1_mean']:+.3f}")
    print(f"- test@1 mean delta: {d['test_at_1_mean']:+.3f}")


def main() -> int:
    args = parse_args()
    reports = [json.loads(p.read_text(encoding="utf-8")) for p in args.reports]
    agg = aggregate(reports)
    print_human(agg)

    if args.out is not None:
        args.out.write_text(json.dumps(agg, indent=2) + "\n", encoding="utf-8")
        print(f"\nWrote aggregate report: {args.out}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
