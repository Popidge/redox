#!/usr/bin/env python3
"""Evaluate Rust and Iron prediction JSONL files for compile@1 and test@1."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description="Evaluate Rust/Iron prediction files")
    parser.add_argument(
        "--rust", type=Path, required=True, help="Rust predictions JSONL"
    )
    parser.add_argument(
        "--iron", type=Path, required=True, help="Iron predictions JSONL"
    )
    parser.add_argument(
        "--redox-cmd",
        default="target/debug/redox",
        help="Command path used for `redox oxidize`",
    )
    parser.add_argument(
        "--out", type=Path, default=None, help="Optional output report JSON"
    )
    return parser.parse_args()


def read_jsonl(path: Path) -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def oxidize_iron(prediction: str, redox_cmd: str) -> tuple[bool, str, str]:
    with tempfile.TemporaryDirectory(prefix="redox_eval_iron_") as tmp:
        iron_path = Path(tmp) / "input.iron"
        iron_path.write_text(prediction, encoding="utf-8")
        proc = subprocess.run(
            [redox_cmd, "oxidize", str(iron_path)],
            text=True,
            capture_output=True,
            check=False,
        )
        if proc.returncode != 0:
            return False, "", compact(proc.stderr)
        return True, proc.stdout, ""


def compile_rust(source: str, crate_name: str) -> tuple[bool, str]:
    with tempfile.TemporaryDirectory(prefix="redox_eval_compile_") as tmp:
        src = Path(tmp) / "pred.rs"
        out = Path(tmp) / "pred.rlib"
        src.write_text(source, encoding="utf-8")
        proc = subprocess.run(
            [
                "rustc",
                "--crate-name",
                sanitize_crate_name(crate_name),
                "--crate-type",
                "lib",
                "--edition",
                "2024",
                "-A",
                "dead_code",
                "-o",
                str(out),
                str(src),
            ],
            text=True,
            capture_output=True,
            check=False,
        )
        return proc.returncode == 0, compact(proc.stderr)


def behavior_test(
    source: str, row: dict[str, Any], crate_name: str
) -> tuple[bool, str]:
    family = row.get("family", "")
    prompt = row.get("prompt", "")
    fn_name = extract_function_name(source)
    if fn_name is None:
        return False, "No function definition found"

    with tempfile.TemporaryDirectory(prefix="redox_eval_behavior_") as tmp:
        src = Path(tmp) / "behavior.rs"
        out = Path(tmp) / "behavior_bin"

        test_code = build_behavior_program(source, fn_name, family, prompt)
        if test_code is None:
            return False, f"Unsupported family for behavior checks: {family}"

        src.write_text(test_code, encoding="utf-8")
        proc = subprocess.run(
            [
                "rustc",
                "--crate-name",
                sanitize_crate_name(crate_name + "_behavior"),
                "--edition",
                "2024",
                "-A",
                "dead_code",
                "-o",
                str(out),
                str(src),
            ],
            text=True,
            capture_output=True,
            check=False,
        )
        if proc.returncode != 0:
            return False, compact(proc.stderr)

        run = subprocess.run([str(out)], text=True, capture_output=True, check=False)
        if run.returncode != 0:
            return False, compact(run.stderr or run.stdout)

    return True, ""


def extract_function_name(source: str) -> str | None:
    pub_matches = re.findall(
        r"(?m)^\s*pub\s+fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", source
    )
    if pub_matches:
        return pub_matches[0]

    any_matches = re.findall(r"(?m)^\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(", source)
    if any_matches:
        return any_matches[0]

    return None


def build_behavior_program(
    source: str, fn_name: str, family: str, prompt: str
) -> str | None:
    if family == "closure_shift_const":
        m = re.search(r"adds\s+(\d+)\s+to\s+input", prompt)
        if not m:
            return None
        k = int(m.group(1))
        return (
            f"{source}\n\n"
            "fn main() {\n"
            f"    let got = {fn_name}(7);\n"
            f"    assert_eq!(got, {7 + k});\n"
            "}\n"
        )

    if family == "result_unwrap_or_const":
        m = re.search(r"returns\s+(\d+)\.", prompt)
        if not m:
            return None
        fallback = int(m.group(1))
        return (
            f"{source}\n\n"
            "fn main() {\n"
            f"    let ok = {fn_name}(Ok(123));\n"
            "    assert_eq!(ok, 123);\n"
            f'    let err = {fn_name}(Err(String::from("x")));\n'
            f"    assert_eq!(err, {fallback});\n"
            "}\n"
        )

    if family == "vec_pop_basic":
        m = re.search(r"vec!\[(\d+),\s*(\d+),\s*(\d+)\]", prompt)
        if not m:
            return None
        expected = int(m.group(3))
        return (
            f"{source}\n\n"
            "fn main() {\n"
            f"    let got = {fn_name}();\n"
            f"    assert_eq!(got, Some({expected}));\n"
            "}\n"
        )

    return None


def evaluate_arm(
    rows: list[dict[str, Any]], arm: str, redox_cmd: str
) -> dict[str, Any]:
    results = []
    for row in rows:
        task_id = row.get("id", "unknown")
        pred = row.get("prediction", "")
        family = row.get("family", "")

        transform_ok = True
        rust_code = pred
        transform_error = ""

        if arm == "iron":
            transform_ok, rust_code, transform_error = oxidize_iron(pred, redox_cmd)

        compile_ok = False
        compile_error = ""
        if transform_ok:
            compile_ok, compile_error = compile_rust(rust_code, f"{arm}_{task_id}")

        test_ok = False
        test_error = ""
        if compile_ok:
            test_ok, test_error = behavior_test(rust_code, row, f"{arm}_{task_id}")

        results.append(
            {
                "id": task_id,
                "family": family,
                "arm": arm,
                "transform_ok": transform_ok,
                "compile_ok": compile_ok,
                "test_ok": test_ok,
                "transform_error": transform_error,
                "compile_error": compile_error,
                "test_error": test_error,
            }
        )

    return summarize_results(results, arm)


def classify_error(message: str, phase: str) -> str:
    text = message.lower()
    if not text:
        return "none"

    if phase == "transform":
        if "unexpectedtoken" in text or "parse error" in text:
            return "iron_parse_error"
        if "oxidation failed" in text:
            return "oxidation_error"
        return "transform_other"

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
    if "no function definition found" in text:
        return "missing_function"
    if "assertion failed" in text:
        return "behavior_assertion"
    return "other"


def summarize_results(results: list[dict[str, Any]], arm: str) -> dict[str, Any]:
    total = len(results)
    transform_pass = sum(1 for r in results if r["transform_ok"])
    compile_pass = sum(1 for r in results if r["compile_ok"])
    test_pass = sum(1 for r in results if r["test_ok"])

    per_family: dict[str, dict[str, int]] = defaultdict(
        lambda: {"total": 0, "compile": 0, "test": 0}
    )
    failure_phase_counts: Counter[str] = Counter()
    failure_taxonomy: Counter[str] = Counter()
    per_family_failure_phase_counts: dict[str, Counter[str]] = defaultdict(Counter)
    per_family_failure_taxonomy: dict[str, Counter[str]] = defaultdict(Counter)

    for r in results:
        fam = r["family"]
        per_family[fam]["total"] += 1
        if r["compile_ok"]:
            per_family[fam]["compile"] += 1
        if r["test_ok"]:
            per_family[fam]["test"] += 1

        if not r["transform_ok"]:
            phase = "transform"
            label = classify_error(r.get("transform_error", ""), "transform")
            failure_phase_counts["transform"] += 1
            failure_taxonomy[label] += 1
            per_family_failure_phase_counts[fam][phase] += 1
            per_family_failure_taxonomy[fam][label] += 1
        elif not r["compile_ok"]:
            phase = "compile"
            label = classify_error(r.get("compile_error", ""), "compile")
            failure_phase_counts["compile"] += 1
            failure_taxonomy[label] += 1
            per_family_failure_phase_counts[fam][phase] += 1
            per_family_failure_taxonomy[fam][label] += 1
        elif not r["test_ok"]:
            phase = "test"
            label = classify_error(r.get("test_error", ""), "test")
            failure_phase_counts["test"] += 1
            failure_taxonomy[label] += 1
            per_family_failure_phase_counts[fam][phase] += 1
            per_family_failure_taxonomy[fam][label] += 1

    return {
        "arm": arm,
        "total": total,
        "transform_pass": transform_pass,
        "compile_pass": compile_pass,
        "test_pass": test_pass,
        "compile_at_1": (compile_pass / total) if total else 0.0,
        "test_at_1": (test_pass / total) if total else 0.0,
        "per_family": per_family,
        "failure_phase_counts": dict(failure_phase_counts),
        "failure_taxonomy": dict(failure_taxonomy.most_common()),
        "per_family_failure_phase_counts": {
            family: dict(counts)
            for family, counts in sorted(per_family_failure_phase_counts.items())
        },
        "per_family_failure_taxonomy": {
            family: dict(counts.most_common())
            for family, counts in sorted(per_family_failure_taxonomy.items())
        },
        "rows": results,
    }


def sanitize_crate_name(value: str) -> str:
    name = re.sub(r"[^a-zA-Z0-9_]", "_", value)
    if not name:
        return "pred"
    if name[0].isdigit():
        return "pred_" + name
    return name


def compact(text: str) -> str:
    normalized = " ".join(text.strip().split())
    return normalized[:300] + ("..." if len(normalized) > 300 else "")


def print_summary(report: dict[str, Any]) -> None:
    for arm_key in ("rust", "iron"):
        data = report[arm_key]
        print(f"{arm_key.upper()}:")
        print(f"- total: {data['total']}")
        print(f"- transform pass: {data['transform_pass']}/{data['total']}")
        print(
            f"- compile@1: {data['compile_pass']}/{data['total']} = {data['compile_at_1']:.3f}"
        )
        print(
            f"- test@1: {data['test_pass']}/{data['total']} = {data['test_at_1']:.3f}"
        )
        print("- per_family:")
        for family, stats in sorted(data["per_family"].items()):
            c = stats["compile"]
            t = stats["test"]
            n = stats["total"]
            print(f"  - {family}: compile {c}/{n}, test {t}/{n}")
        phase_counts = data.get("failure_phase_counts", {})
        if phase_counts:
            transform_fail = phase_counts.get("transform", 0)
            compile_fail = phase_counts.get("compile", 0)
            test_fail = phase_counts.get("test", 0)
            print(
                "- failure phase counts: "
                f"transform={transform_fail}, compile={compile_fail}, test={test_fail}"
            )

        taxonomy = data.get("failure_taxonomy", {})
        if taxonomy:
            print("- top failure classes:")
            for label, count in list(taxonomy.items())[:5]:
                print(f"  - {label}: {count}")

        family_taxonomy = data.get("per_family_failure_taxonomy", {})
        if family_taxonomy:
            print("- top failure classes by family:")
            for family, counts in sorted(family_taxonomy.items()):
                top = list(counts.items())[:3]
                if not top:
                    continue
                rendered = ", ".join(f"{label}={count}" for label, count in top)
                print(f"  - {family}: {rendered}")
        print("")


def main() -> int:
    args = parse_args()
    rust_rows = read_jsonl(args.rust)
    iron_rows = read_jsonl(args.iron)

    rust_report = evaluate_arm(rust_rows, "rust", args.redox_cmd)
    iron_report = evaluate_arm(iron_rows, "iron", args.redox_cmd)

    report = {
        "inputs": {"rust": str(args.rust), "iron": str(args.iron)},
        "rust": rust_report,
        "iron": iron_report,
    }

    print_summary(report)

    if args.out is not None:
        args.out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
        print(f"Wrote report: {args.out}")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
