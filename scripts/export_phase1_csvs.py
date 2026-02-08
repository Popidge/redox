#!/usr/bin/env python3
"""Export figure/table-ready CSVs from phase-1 experiment reports."""

from __future__ import annotations

import argparse
import csv
import json
import sys
from collections import Counter, defaultdict
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class RunSpec:
    key: str
    label: str
    kind: str
    report_path: Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export phase-1 metrics and error breakdown CSVs"
    )
    parser.add_argument(
        "--repo-root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="Repository root (default: inferred from script location)",
    )
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("eval/tables/phase1"),
        help="Output directory for generated CSV files",
    )
    return parser.parse_args()


def r6(value: float) -> float:
    return round(float(value), 6)


def build_run_specs(repo_root: Path) -> list[RunSpec]:
    return [
        RunSpec(
            key="v1_agg",
            label="v1 aggregate (2 seeds)",
            kind="aggregate",
            report_path=repo_root / "eval/v1/report_aggregate_2seeds.json",
        ),
        RunSpec(
            key="v2_seed2108",
            label="v2 seed2108",
            kind="seed",
            report_path=repo_root / "eval/v2/report_seed2108.json",
        ),
        RunSpec(
            key="v2_5_agg",
            label="v2.5 aggregate (2 seeds)",
            kind="aggregate",
            report_path=repo_root / "eval/v2_5/report_aggregate_2seeds.json",
        ),
        RunSpec(
            key="v2_5_after_parser_fix_agg",
            label="v2.5 after parser fix (2 seeds)",
            kind="aggregate",
            report_path=repo_root
            / "eval/v2_5/report_aggregate_2seeds_after_parser_fix.json",
        ),
        RunSpec(
            key="v2_6_agg",
            label="v2.6 aggregate (2 seeds)",
            kind="aggregate",
            report_path=repo_root / "eval/v2_6/report_aggregate_2seeds.json",
        ),
        RunSpec(
            key="v2_6_iron_prompt_fix_only_agg",
            label="v2.6 strict attribution (old Rust + new Iron)",
            kind="aggregate",
            report_path=repo_root
            / "eval/v2_6/report_aggregate_2seeds_iron_prompt_fix_only.json",
        ),
    ]


def load_json(path: Path) -> dict[str, Any]:
    return json.loads(path.read_text(encoding="utf-8"))


def load_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def write_csv(path: Path, fieldnames: list[str], rows: list[dict[str, Any]]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(rows)


def summarize_run(spec: RunSpec, data: dict[str, Any]) -> dict[str, Any]:
    if spec.kind == "aggregate":
        rust = data["summary"]["rust"]
        iron = data["summary"]["iron"]

        row = {
            "run_key": spec.key,
            "run_label": spec.label,
            "report_path": spec.report_path.as_posix(),
            "num_reports": int(data.get("num_reports", 1)),
            "rust_compile_at_1_mean": r6(rust["compile_at_1"]["mean"]),
            "rust_compile_at_1_std": r6(rust["compile_at_1"].get("std", 0.0)),
            "rust_test_at_1_mean": r6(rust["test_at_1"]["mean"]),
            "rust_test_at_1_std": r6(rust["test_at_1"].get("std", 0.0)),
            "rust_transform_rate_mean": r6(rust["transform_rate"]["mean"]),
            "rust_transform_rate_std": r6(rust["transform_rate"].get("std", 0.0)),
            "iron_compile_at_1_mean": r6(iron["compile_at_1"]["mean"]),
            "iron_compile_at_1_std": r6(iron["compile_at_1"].get("std", 0.0)),
            "iron_test_at_1_mean": r6(iron["test_at_1"]["mean"]),
            "iron_test_at_1_std": r6(iron["test_at_1"].get("std", 0.0)),
            "iron_transform_rate_mean": r6(iron["transform_rate"]["mean"]),
            "iron_transform_rate_std": r6(iron["transform_rate"].get("std", 0.0)),
            "delta_iron_minus_rust_compile": r6(
                data["delta_iron_minus_rust"]["compile_at_1_mean"]
            ),
            "delta_iron_minus_rust_test": r6(
                data["delta_iron_minus_rust"]["test_at_1_mean"]
            ),
        }
        return row

    rust = data["rust"]
    iron = data["iron"]
    rust_transform = (
        rust["transform_pass"] / rust["total"] if rust.get("total", 0) else 0.0
    )
    iron_transform = (
        iron["transform_pass"] / iron["total"] if iron.get("total", 0) else 0.0
    )

    return {
        "run_key": spec.key,
        "run_label": spec.label,
        "report_path": spec.report_path.as_posix(),
        "num_reports": 1,
        "rust_compile_at_1_mean": r6(rust["compile_at_1"]),
        "rust_compile_at_1_std": 0.0,
        "rust_test_at_1_mean": r6(rust["test_at_1"]),
        "rust_test_at_1_std": 0.0,
        "rust_transform_rate_mean": r6(rust_transform),
        "rust_transform_rate_std": 0.0,
        "iron_compile_at_1_mean": r6(iron["compile_at_1"]),
        "iron_compile_at_1_std": 0.0,
        "iron_test_at_1_mean": r6(iron["test_at_1"]),
        "iron_test_at_1_std": 0.0,
        "iron_transform_rate_mean": r6(iron_transform),
        "iron_transform_rate_std": 0.0,
        "delta_iron_minus_rust_compile": r6(
            iron["compile_at_1"] - rust["compile_at_1"]
        ),
        "delta_iron_minus_rust_test": r6(iron["test_at_1"] - rust["test_at_1"]),
    }


def collect_per_family_rows(
    spec: RunSpec, data: dict[str, Any]
) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []

    if spec.kind == "aggregate":
        for arm in ("rust", "iron"):
            families = data["per_family"].get(arm, {})
            for family, metrics in sorted(families.items()):
                rows.append(
                    {
                        "run_key": spec.key,
                        "run_label": spec.label,
                        "report_path": spec.report_path.as_posix(),
                        "arm": arm,
                        "family": family,
                        "compile_mean": r6(metrics["compile"]["mean"]),
                        "compile_std": r6(metrics["compile"].get("std", 0.0)),
                        "test_mean": r6(metrics["test"]["mean"]),
                        "test_std": r6(metrics["test"].get("std", 0.0)),
                    }
                )
        return rows

    for arm in ("rust", "iron"):
        families = data[arm].get("per_family", {})
        for family, stats in sorted(families.items()):
            n = stats.get("total", 0)
            compile_mean = (stats.get("compile", 0) / n) if n else 0.0
            test_mean = (stats.get("test", 0) / n) if n else 0.0
            rows.append(
                {
                    "run_key": spec.key,
                    "run_label": spec.label,
                    "report_path": spec.report_path.as_posix(),
                    "arm": arm,
                    "family": family,
                    "compile_mean": r6(compile_mean),
                    "compile_std": 0.0,
                    "test_mean": r6(test_mean),
                    "test_std": 0.0,
                }
            )

    return rows


def collect_taxonomy_rows(spec: RunSpec, data: dict[str, Any]) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []

    if spec.kind == "aggregate":
        taxonomy = data.get("error_taxonomy", {})
        for arm in ("rust", "iron"):
            for failure_class, count in sorted(
                taxonomy.get(arm, {}).items(), key=lambda item: (-item[1], item[0])
            ):
                rows.append(
                    {
                        "run_key": spec.key,
                        "run_label": spec.label,
                        "report_path": spec.report_path.as_posix(),
                        "arm": arm,
                        "failure_class": failure_class,
                        "count": int(count),
                    }
                )
        return rows

    for arm in ("rust", "iron"):
        taxonomy = data[arm].get("failure_taxonomy", {})
        for failure_class, count in sorted(
            taxonomy.items(), key=lambda item: (-item[1], item[0])
        ):
            rows.append(
                {
                    "run_key": spec.key,
                    "run_label": spec.label,
                    "report_path": spec.report_path.as_posix(),
                    "arm": arm,
                    "failure_class": failure_class,
                    "count": int(count),
                }
            )

    return rows


def collect_v26_seed_metrics(repo_root: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for seed in (3407, 2108):
        report_path = repo_root / f"eval/v2_6/report_seed{seed}.json"
        if not report_path.exists():
            continue
        data = load_json(report_path)

        for arm in ("rust", "iron"):
            arm_data = data[arm]
            total = int(arm_data["total"])
            transform_pass = int(arm_data["transform_pass"])
            rows.append(
                {
                    "seed": seed,
                    "arm": arm,
                    "report_path": report_path.as_posix(),
                    "total": total,
                    "transform_pass": transform_pass,
                    "transform_rate": r6(transform_pass / total if total else 0.0),
                    "compile_pass": int(arm_data["compile_pass"]),
                    "compile_at_1": r6(arm_data["compile_at_1"]),
                    "test_pass": int(arm_data["test_pass"]),
                    "test_at_1": r6(arm_data["test_at_1"]),
                }
            )

    return rows


def collect_v26_failure_rows(repo_root: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []

    for seed in (3407, 2108):
        report_path = repo_root / f"eval/v2_6/report_seed{seed}.json"
        if not report_path.exists():
            continue
        data = load_json(report_path)

        for arm in ("rust", "iron"):
            for row in data[arm].get("rows", []):
                transform_ok = bool(row.get("transform_ok", False))
                compile_ok = bool(row.get("compile_ok", False))
                test_ok = bool(row.get("test_ok", False))
                if transform_ok and compile_ok and test_ok:
                    continue

                if not transform_ok:
                    failure_phase = "transform"
                elif not compile_ok:
                    failure_phase = "compile"
                else:
                    failure_phase = "test"

                rows.append(
                    {
                        "seed": seed,
                        "arm": arm,
                        "report_path": report_path.as_posix(),
                        "id": row.get("id", ""),
                        "family": row.get("family", ""),
                        "failure_phase": failure_phase,
                        "transform_ok": transform_ok,
                        "compile_ok": compile_ok,
                        "test_ok": test_ok,
                        "transform_error": row.get("transform_error", ""),
                        "compile_error": row.get("compile_error", ""),
                        "test_error": row.get("test_error", ""),
                    }
                )

    return rows


def collect_dataset_rows(
    repo_root: Path,
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    overview_rows: list[dict[str, Any]] = []
    family_rows: list[dict[str, Any]] = []

    manifests = [
        (
            "foundation_v1",
            repo_root / "data/pilot/foundation_v1/manifest.v1_candidate.jsonl",
            repo_root / "data/pilot/foundation_v1/report.v1_candidate.json",
            repo_root / "data/pilot/foundation_v1/unsloth/export_summary.json",
            None,
        ),
        (
            "foundation_v2",
            repo_root / "data/pilot/foundation_v2/manifest.v2_candidate.jsonl",
            None,
            repo_root / "data/pilot/foundation_v2/unsloth/export_summary.json",
            repo_root / "data/pilot/foundation_v2/generation_summary.json",
        ),
    ]

    for (
        dataset_version,
        manifest_path,
        report_path,
        export_summary_path,
        gen_summary_path,
    ) in manifests:
        if not manifest_path.exists():
            continue

        rows = load_jsonl(manifest_path)
        split_counter: Counter[str] = Counter(row.get("split", "") for row in rows)
        families = sorted({row.get("family", "") for row in rows if row.get("family")})

        family_split_counts: defaultdict[str, Counter[str]] = defaultdict(Counter)
        for row in rows:
            family = row.get("family", "")
            split = row.get("split", "")
            family_split_counts[family][split] += 1

        gate_all_pass = ""
        if report_path is not None and report_path.exists():
            report_data = load_json(report_path)
            gate_all_pass = bool(report_data.get("gates", {}).get("all_pass", False))

        added_tasks = ""
        families_added = ""
        if gen_summary_path is not None and gen_summary_path.exists():
            summary = load_json(gen_summary_path)
            added_tasks = summary.get("added_tasks", "")
            families_added = ";".join(summary.get("families_added", []))

        export_rust_train = ""
        export_rust_val = ""
        export_rust_test = ""
        export_iron_train = ""
        export_iron_val = ""
        export_iron_test = ""
        if export_summary_path.exists():
            export_summary = load_json(export_summary_path)
            export_rust_train = export_summary.get("rust_counts", {}).get("train", "")
            export_rust_val = export_summary.get("rust_counts", {}).get("val", "")
            export_rust_test = export_summary.get("rust_counts", {}).get("test", "")
            export_iron_train = export_summary.get("iron_counts", {}).get("train", "")
            export_iron_val = export_summary.get("iron_counts", {}).get("val", "")
            export_iron_test = export_summary.get("iron_counts", {}).get("test", "")

        overview_rows.append(
            {
                "dataset_version": dataset_version,
                "manifest_path": manifest_path.as_posix(),
                "total_tasks": len(rows),
                "family_count": len(families),
                "train_count": split_counter.get("train", 0),
                "val_count": split_counter.get("val", 0),
                "test_count": split_counter.get("test", 0),
                "gate_all_pass": gate_all_pass,
                "added_tasks": added_tasks,
                "families_added": families_added,
                "export_rust_train": export_rust_train,
                "export_rust_val": export_rust_val,
                "export_rust_test": export_rust_test,
                "export_iron_train": export_iron_train,
                "export_iron_val": export_iron_val,
                "export_iron_test": export_iron_test,
            }
        )

        for family in sorted(family_split_counts):
            counts = family_split_counts[family]
            family_rows.append(
                {
                    "dataset_version": dataset_version,
                    "family": family,
                    "total_count": counts.get("train", 0)
                    + counts.get("val", 0)
                    + counts.get("test", 0),
                    "train_count": counts.get("train", 0),
                    "val_count": counts.get("val", 0),
                    "test_count": counts.get("test", 0),
                }
            )

    return overview_rows, family_rows


def main() -> int:
    args = parse_args()
    repo_root = args.repo_root.resolve()
    out_dir = (repo_root / args.out_dir).resolve()

    specs = build_run_specs(repo_root)
    available_specs: list[RunSpec] = []
    for spec in specs:
        if spec.report_path.exists():
            available_specs.append(spec)
        else:
            print(f"warning: missing report {spec.report_path}", file=sys.stderr)

    if not available_specs:
        raise FileNotFoundError("No configured reports were found")

    aggregate_rows: list[dict[str, Any]] = []
    per_family_rows: list[dict[str, Any]] = []
    taxonomy_rows: list[dict[str, Any]] = []

    for spec in available_specs:
        data = load_json(spec.report_path)
        aggregate_rows.append(summarize_run(spec, data))
        per_family_rows.extend(collect_per_family_rows(spec, data))
        taxonomy_rows.extend(collect_taxonomy_rows(spec, data))

    v26_seed_rows = collect_v26_seed_metrics(repo_root)
    v26_failure_rows = collect_v26_failure_rows(repo_root)
    dataset_overview_rows, dataset_family_rows = collect_dataset_rows(repo_root)

    write_csv(
        out_dir / "aggregate_metrics_by_run.csv",
        [
            "run_key",
            "run_label",
            "report_path",
            "num_reports",
            "rust_compile_at_1_mean",
            "rust_compile_at_1_std",
            "rust_test_at_1_mean",
            "rust_test_at_1_std",
            "rust_transform_rate_mean",
            "rust_transform_rate_std",
            "iron_compile_at_1_mean",
            "iron_compile_at_1_std",
            "iron_test_at_1_mean",
            "iron_test_at_1_std",
            "iron_transform_rate_mean",
            "iron_transform_rate_std",
            "delta_iron_minus_rust_compile",
            "delta_iron_minus_rust_test",
        ],
        aggregate_rows,
    )

    write_csv(
        out_dir / "per_family_metrics_by_run.csv",
        [
            "run_key",
            "run_label",
            "report_path",
            "arm",
            "family",
            "compile_mean",
            "compile_std",
            "test_mean",
            "test_std",
        ],
        per_family_rows,
    )

    write_csv(
        out_dir / "failure_taxonomy_by_run.csv",
        [
            "run_key",
            "run_label",
            "report_path",
            "arm",
            "failure_class",
            "count",
        ],
        taxonomy_rows,
    )

    write_csv(
        out_dir / "v2_6_seed_metrics.csv",
        [
            "seed",
            "arm",
            "report_path",
            "total",
            "transform_pass",
            "transform_rate",
            "compile_pass",
            "compile_at_1",
            "test_pass",
            "test_at_1",
        ],
        v26_seed_rows,
    )

    write_csv(
        out_dir / "v2_6_failure_rows.csv",
        [
            "seed",
            "arm",
            "report_path",
            "id",
            "family",
            "failure_phase",
            "transform_ok",
            "compile_ok",
            "test_ok",
            "transform_error",
            "compile_error",
            "test_error",
        ],
        v26_failure_rows,
    )

    write_csv(
        out_dir / "dataset_overview.csv",
        [
            "dataset_version",
            "manifest_path",
            "total_tasks",
            "family_count",
            "train_count",
            "val_count",
            "test_count",
            "gate_all_pass",
            "added_tasks",
            "families_added",
            "export_rust_train",
            "export_rust_val",
            "export_rust_test",
            "export_iron_train",
            "export_iron_val",
            "export_iron_test",
        ],
        dataset_overview_rows,
    )

    write_csv(
        out_dir / "dataset_family_counts.csv",
        [
            "dataset_version",
            "family",
            "total_count",
            "train_count",
            "val_count",
            "test_count",
        ],
        dataset_family_rows,
    )

    print(f"Wrote CSV files to {out_dir}")
    print("- aggregate_metrics_by_run.csv")
    print("- per_family_metrics_by_run.csv")
    print("- failure_taxonomy_by_run.csv")
    print("- v2_6_seed_metrics.csv")
    print("- v2_6_failure_rows.csv")
    print("- dataset_overview.csv")
    print("- dataset_family_counts.csv")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
