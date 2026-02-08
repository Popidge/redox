#!/usr/bin/env python3
"""Validate phase-1 Rust/Iron finetune datasets against Redox quality gates."""

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
import tempfile
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any


ALLOWED_SPLITS = {"train", "val", "test"}


@dataclass
class TaskRecord:
    task_id: str
    split: str
    family: str
    prompt_path: Path
    rust_path: Path
    tests_path: Path | None
    deps: list[str]
    unsafe: bool


@dataclass
class TaskResult:
    task_id: str
    split: str
    family: str
    errors: list[str] = field(default_factory=list)
    warnings: list[str] = field(default_factory=list)
    used_verbatim: bool = False
    reduce_deterministic: bool = False
    reduce_ok: bool = False
    oxidize_ok: bool = False
    roundtrip_compile_ok: bool = False


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Validate dataset manifest against Redox experiment gates",
    )
    parser.add_argument(
        "manifest",
        type=Path,
        help="Path to JSONL manifest",
    )
    parser.add_argument(
        "--redox-cmd",
        default="target/debug/redox",
        help="Redox command prefix (default: target/debug/redox)",
    )
    parser.add_argument(
        "--purity-threshold",
        type=float,
        default=0.98,
        help="Minimum non-verbatim ratio required for phase-1 gates",
    )
    parser.add_argument(
        "--allow-deps",
        action="store_true",
        help="Allow non-empty deps metadata for phase-1",
    )
    parser.add_argument(
        "--allow-unsafe",
        action="store_true",
        help="Allow unsafe=true metadata for phase-1",
    )
    parser.add_argument(
        "--report-json",
        type=Path,
        default=None,
        help="Optional path to write machine-readable report JSON",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifest_path = args.manifest.resolve()

    if not manifest_path.exists():
        print(f"ERROR: manifest not found: {manifest_path}", file=sys.stderr)
        return 2

    redox_prefix = shlex.split(args.redox_cmd)
    if not redox_prefix:
        print("ERROR: --redox-cmd cannot be empty", file=sys.stderr)
        return 2

    if redox_prefix[0].endswith("redox") and not Path(redox_prefix[0]).exists():
        print(
            "ERROR: redox binary not found. Run `cargo build --bin redox` or pass --redox-cmd.",
            file=sys.stderr,
        )
        return 2

    records, schema_errors = load_manifest(manifest_path)
    if schema_errors:
        print("Manifest schema errors:")
        for err in schema_errors:
            print(f"- {err}")
        return 1

    split_errors = validate_split_hygiene(records)

    task_results = [validate_task(record, redox_prefix, args) for record in records]

    gate_results = compute_gate_results(
        records, task_results, split_errors, args.purity_threshold
    )
    print_report(
        records, task_results, split_errors, gate_results, args.purity_threshold
    )

    if args.report_json is not None:
        write_report_json(
            args.report_json, records, task_results, split_errors, gate_results
        )

    if gate_results["all_pass"]:
        return 0
    return 1


def load_manifest(manifest_path: Path) -> tuple[list[TaskRecord], list[str]]:
    base_dir = manifest_path.parent
    records: list[TaskRecord] = []
    errors: list[str] = []
    seen_ids: set[str] = set()

    for idx, line in enumerate(
        manifest_path.read_text(encoding="utf-8").splitlines(), start=1
    ):
        stripped = line.strip()
        if not stripped:
            continue
        try:
            raw = json.loads(stripped)
        except json.JSONDecodeError as exc:
            errors.append(f"line {idx}: invalid JSON ({exc})")
            continue

        if not isinstance(raw, dict):
            errors.append(f"line {idx}: each JSONL record must be an object")
            continue

        line_errors, record = parse_record(raw, base_dir, idx)
        errors.extend(line_errors)
        if record is None:
            continue

        if record.task_id in seen_ids:
            errors.append(f"line {idx}: duplicate id '{record.task_id}'")
            continue

        seen_ids.add(record.task_id)
        records.append(record)

    if not records:
        errors.append("manifest contains no task records")

    return records, errors


def parse_record(
    raw: dict[str, Any], base_dir: Path, line_no: int
) -> tuple[list[str], TaskRecord | None]:
    errors: list[str] = []

    task_id = raw.get("id")
    split = raw.get("split")
    family = raw.get("family")
    prompt_path_raw = raw.get("prompt_path")
    rust_path_raw = raw.get("rust_path")
    tests_path_raw = raw.get("tests_path")
    deps = raw.get("deps", [])
    unsafe = raw.get("unsafe", False)

    if not isinstance(task_id, str) or not task_id:
        errors.append(f"line {line_no}: 'id' must be a non-empty string")

    if not isinstance(split, str) or split not in ALLOWED_SPLITS:
        errors.append(
            f"line {line_no}: 'split' must be one of {sorted(ALLOWED_SPLITS)}"
        )

    if not isinstance(family, str) or not family:
        errors.append(f"line {line_no}: 'family' must be a non-empty string")

    if not isinstance(prompt_path_raw, str) or not prompt_path_raw:
        errors.append(f"line {line_no}: 'prompt_path' must be a non-empty string")

    if not isinstance(rust_path_raw, str) or not rust_path_raw:
        errors.append(f"line {line_no}: 'rust_path' must be a non-empty string")

    if tests_path_raw is not None and not isinstance(tests_path_raw, str):
        errors.append(f"line {line_no}: 'tests_path' must be a string when present")

    if not isinstance(deps, list) or not all(isinstance(dep, str) for dep in deps):
        errors.append(f"line {line_no}: 'deps' must be a string list")

    if not isinstance(unsafe, bool):
        errors.append(f"line {line_no}: 'unsafe' must be boolean")

    if errors:
        return errors, None

    assert isinstance(task_id, str)
    assert isinstance(split, str)
    assert isinstance(family, str)
    assert isinstance(prompt_path_raw, str)
    assert isinstance(rust_path_raw, str)
    assert isinstance(deps, list)
    assert isinstance(unsafe, bool)
    assert tests_path_raw is None or isinstance(tests_path_raw, str)

    prompt_path = resolve_path(base_dir, prompt_path_raw)
    rust_path = resolve_path(base_dir, rust_path_raw)
    tests_path = resolve_path(base_dir, tests_path_raw) if tests_path_raw else None

    for label, path in (
        ("prompt_path", prompt_path),
        ("rust_path", rust_path),
        ("tests_path", tests_path),
    ):
        if path is not None and not path.exists():
            errors.append(f"line {line_no}: {label} does not exist: {path}")

    if errors:
        return errors, None

    return (
        errors,
        TaskRecord(
            task_id=task_id,
            split=split,
            family=family,
            prompt_path=prompt_path,
            rust_path=rust_path,
            tests_path=tests_path,
            deps=deps,
            unsafe=unsafe,
        ),
    )


def resolve_path(base_dir: Path, value: str) -> Path:
    p = Path(value)
    if p.is_absolute():
        return p
    return (base_dir / p).resolve()


def validate_split_hygiene(records: list[TaskRecord]) -> list[str]:
    family_splits: dict[str, set[str]] = {}
    for record in records:
        family_splits.setdefault(record.family, set()).add(record.split)

    errors: list[str] = []
    for family, splits in sorted(family_splits.items()):
        if len(splits) > 1:
            joined = ", ".join(sorted(splits))
            errors.append(f"family '{family}' appears in multiple splits: {joined}")

    return errors


def validate_task(
    record: TaskRecord, redox_prefix: list[str], args: argparse.Namespace
) -> TaskResult:
    result = TaskResult(
        task_id=record.task_id, split=record.split, family=record.family
    )

    if record.deps and not args.allow_deps:
        result.errors.append(
            "deps is non-empty (phase-1 requires minimal dependencies)"
        )

    if record.unsafe and not args.allow_unsafe:
        result.errors.append("unsafe=true is disallowed for phase-1")

    source = record.rust_path.read_text(encoding="utf-8")
    reduce_1 = run_reduce(redox_prefix, source, record.rust_path)
    reduce_2 = run_reduce(redox_prefix, source, record.rust_path)

    if reduce_1.returncode != 0:
        result.errors.append(f"reduce failed: {single_line(reduce_1.stderr)}")
        return result

    if reduce_2.returncode != 0:
        result.errors.append(f"second reduce failed: {single_line(reduce_2.stderr)}")
        return result

    result.reduce_ok = True
    iron_1 = reduce_1.stdout
    iron_2 = reduce_2.stdout

    if iron_1 == iron_2:
        result.reduce_deterministic = True
    else:
        result.errors.append("reduce output is non-deterministic")

    result.used_verbatim = 'verbatim item "' in iron_1

    oxidize_proc = run_oxidize(redox_prefix, iron_1)
    if oxidize_proc.returncode != 0:
        result.errors.append(f"oxidize failed: {single_line(oxidize_proc.stderr)}")
        return result

    result.oxidize_ok = True
    roundtrip_rust = oxidize_proc.stdout

    compile_proc = compile_rust_source(roundtrip_rust, record.task_id)
    if compile_proc.returncode != 0:
        result.errors.append(
            f"roundtrip compile failed: {single_line(compile_proc.stderr)}"
        )
        return result

    result.roundtrip_compile_ok = True
    return result


def run_reduce(
    redox_prefix: list[str], source: str, source_path: Path
) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory(prefix="redox_reduce_") as tmp:
        in_path = Path(tmp) / source_path.name
        in_path.write_text(source, encoding="utf-8")
        cmd = [*redox_prefix, "reduce", str(in_path)]
        return subprocess.run(cmd, text=True, capture_output=True, check=False)


def run_oxidize(redox_prefix: list[str], iron: str) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory(prefix="redox_oxidize_") as tmp:
        in_path = Path(tmp) / "input.iron"
        in_path.write_text(iron, encoding="utf-8")
        cmd = [*redox_prefix, "oxidize", str(in_path)]
        return subprocess.run(cmd, text=True, capture_output=True, check=False)


def compile_rust_source(
    rust_source: str, task_id: str
) -> subprocess.CompletedProcess[str]:
    with tempfile.TemporaryDirectory(prefix="redox_compile_") as tmp:
        source_path = Path(tmp) / "roundtrip.rs"
        source_path.write_text(rust_source, encoding="utf-8")

        crate_name = sanitize_crate_name(task_id)
        output_path = Path(tmp) / "out.rlib"

        cmd = [
            "rustc",
            "--crate-name",
            crate_name,
            "--crate-type",
            "lib",
            "--edition",
            "2024",
            "-A",
            "dead_code",
            "-o",
            str(output_path),
            str(source_path),
        ]
        return subprocess.run(cmd, text=True, capture_output=True, check=False)


def sanitize_crate_name(task_id: str) -> str:
    name = re.sub(r"[^a-zA-Z0-9_]", "_", task_id)
    if not name:
        return "task"
    if name[0].isdigit():
        return f"task_{name}"
    return name


def compute_gate_results(
    records: list[TaskRecord],
    results: list[TaskResult],
    split_errors: list[str],
    purity_threshold: float,
) -> dict[str, Any]:
    total = len(results)
    stable_count = sum(1 for r in results if r.reduce_ok and r.oxidize_ok)
    deterministic_count = sum(1 for r in results if r.reduce_deterministic)
    compile_count = sum(1 for r in results if r.roundtrip_compile_ok)
    non_verbatim_count = sum(1 for r in results if not r.used_verbatim)
    purity = non_verbatim_count / total if total else 0.0

    per_task_errors = sum(1 for r in results if r.errors)

    return {
        "gate_pipeline_stability": stable_count == total,
        "gate_determinism": deterministic_count == total,
        "gate_data_quality": compile_count == total,
        "gate_iron_purity": purity >= purity_threshold,
        "gate_split_hygiene": len(split_errors) == 0,
        "all_pass": (
            stable_count == total
            and deterministic_count == total
            and compile_count == total
            and purity >= purity_threshold
            and len(split_errors) == 0
        ),
        "stats": {
            "total_tasks": total,
            "tasks_with_errors": per_task_errors,
            "stable_count": stable_count,
            "deterministic_count": deterministic_count,
            "compile_count": compile_count,
            "non_verbatim_count": non_verbatim_count,
            "purity_ratio": purity,
            "families": len({r.family for r in records}),
        },
    }


def print_report(
    records: list[TaskRecord],
    results: list[TaskResult],
    split_errors: list[str],
    gate_results: dict[str, Any],
    purity_threshold: float,
) -> None:
    stats = gate_results["stats"]

    print("Dataset Validation Report")
    print("=========================")
    print(f"Tasks: {stats['total_tasks']}")
    print(f"Families: {stats['families']}")
    print(f"Tasks with errors: {stats['tasks_with_errors']}")
    print("")

    print("Gate Results")
    print("------------")
    print(gate_line("Pipeline stability", gate_results["gate_pipeline_stability"]))
    print(gate_line("Determinism", gate_results["gate_determinism"]))
    print(gate_line("Data quality", gate_results["gate_data_quality"]))
    purity_msg = (
        f"{stats['purity_ratio']:.3f} >= {purity_threshold:.3f}"
        if gate_results["gate_iron_purity"]
        else f"{stats['purity_ratio']:.3f} < {purity_threshold:.3f}"
    )
    print(gate_line("Iron purity", gate_results["gate_iron_purity"], purity_msg))
    print(gate_line("Split hygiene", gate_results["gate_split_hygiene"]))
    print("")

    if split_errors:
        print("Split leakage issues:")
        for err in split_errors:
            print(f"- {err}")
        print("")

    failing = [r for r in results if r.errors]
    if failing:
        print("Per-task failures:")
        for result in failing:
            print(f"- {result.task_id} ({result.split}/{result.family})")
            for err in result.errors:
                print(f"    * {err}")
    else:
        print("All tasks passed per-task checks.")

    print("")
    if gate_results["all_pass"]:
        print("Overall: PASS")
    else:
        print("Overall: FAIL")


def gate_line(name: str, ok: bool, details: str | None = None) -> str:
    marker = "PASS" if ok else "FAIL"
    if details:
        return f"- {name}: {marker} ({details})"
    return f"- {name}: {marker}"


def write_report_json(
    output_path: Path,
    records: list[TaskRecord],
    results: list[TaskResult],
    split_errors: list[str],
    gate_results: dict[str, Any],
) -> None:
    payload = {
        "gates": {k: v for k, v in gate_results.items() if k != "stats"},
        "stats": gate_results["stats"],
        "split_errors": split_errors,
        "tasks": [
            {
                "id": result.task_id,
                "split": result.split,
                "family": result.family,
                "used_verbatim": result.used_verbatim,
                "reduce_ok": result.reduce_ok,
                "reduce_deterministic": result.reduce_deterministic,
                "oxidize_ok": result.oxidize_ok,
                "roundtrip_compile_ok": result.roundtrip_compile_ok,
                "errors": result.errors,
                "warnings": result.warnings,
            }
            for result in results
        ],
        "manifest_task_count": len(records),
    }
    output_path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def single_line(text: str) -> str:
    normalized = " ".join(text.strip().split())
    if len(normalized) > 220:
        return normalized[:217] + "..."
    return normalized


if __name__ == "__main__":
    raise SystemExit(main())
