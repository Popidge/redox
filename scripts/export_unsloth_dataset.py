#!/usr/bin/env python3
"""Export manifest tasks into Qwen/Unsloth JSONL conversation datasets."""

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Export Rust/Iron conversation datasets"
    )
    parser.add_argument("manifest", type=Path, help="Path to candidate manifest JSONL")
    parser.add_argument(
        "--out-dir",
        type=Path,
        default=Path("data/pilot/foundation_v1/unsloth"),
        help="Output directory for JSONL files",
    )
    parser.add_argument(
        "--redox-cmd",
        default="target/debug/redox",
        help="Command used to run redox reduce",
    )
    parser.add_argument(
        "--fail-on-reduce-error",
        action="store_true",
        help="Abort if any task fails Rust->Iron reduction",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    manifest = args.manifest.resolve()
    if not manifest.exists():
        print(f"ERROR: manifest not found: {manifest}", file=sys.stderr)
        return 2

    redox_cmd = shlex.split(args.redox_cmd)
    if not redox_cmd:
        print("ERROR: --redox-cmd cannot be empty", file=sys.stderr)
        return 2

    records = load_manifest(manifest)
    base_dir = manifest.parent

    out_dir = args.out_dir.resolve()
    out_dir.mkdir(parents=True, exist_ok=True)

    rust_rows: dict[str, list[dict]] = defaultdict(list)
    iron_rows: dict[str, list[dict]] = defaultdict(list)
    reduce_failures: list[str] = []

    for record in records:
        task_id = record["id"]
        split = record["split"]
        family = record["family"]

        prompt_path = (base_dir / record["prompt_path"]).resolve()
        rust_path = (base_dir / record["rust_path"]).resolve()

        prompt_text = prompt_path.read_text(encoding="utf-8").strip()
        rust_code = rust_path.read_text(encoding="utf-8").strip()

        rust_rows[split].append(
            build_row(
                task_id=task_id,
                family=family,
                split=split,
                source="redox-foundation-v1-rust",
                user_prompt=build_user_prompt(
                    task_prompt=prompt_text,
                    language="Rust",
                    required_signature=extract_rust_signature(rust_code, task_id),
                ),
                assistant_code=rust_code,
                language="rust",
            )
        )

        iron_code = reduce_to_iron(redox_cmd, rust_code)
        if iron_code is None:
            msg = f"{task_id}: reduce failed"
            reduce_failures.append(msg)
            if args.fail_on_reduce_error:
                print(f"ERROR: {msg}", file=sys.stderr)
                return 1
            continue

        iron_rows[split].append(
            build_row(
                task_id=task_id,
                family=family,
                split=split,
                source="redox-foundation-v1-iron",
                user_prompt=build_user_prompt(
                    task_prompt=prompt_text,
                    language="Iron",
                    required_signature=extract_iron_signature(
                        iron_code.strip(), task_id
                    ),
                ),
                assistant_code=iron_code.strip(),
                language="iron",
            )
        )

    write_split_files(out_dir, "rust", rust_rows)
    write_split_files(out_dir, "iron", iron_rows)

    summary = {
        "manifest": str(manifest),
        "total_tasks": len(records),
        "rust_counts": {k: len(v) for k, v in rust_rows.items()},
        "iron_counts": {k: len(v) for k, v in iron_rows.items()},
        "reduce_failures": reduce_failures,
    }
    (out_dir / "export_summary.json").write_text(
        json.dumps(summary, indent=2) + "\n",
        encoding="utf-8",
    )

    print("Export complete")
    print(f"- Output dir: {out_dir}")
    print(f"- Rust rows: {sum(len(v) for v in rust_rows.values())}")
    print(f"- Iron rows: {sum(len(v) for v in iron_rows.values())}")
    if reduce_failures:
        print(f"- Reduce failures: {len(reduce_failures)}")
    else:
        print("- Reduce failures: 0")

    return 0


def load_manifest(path: Path) -> list[dict]:
    records: list[dict] = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if not line.strip():
            continue
        records.append(json.loads(line))
    return records


def build_user_prompt(
    task_prompt: str,
    language: str,
    required_signature: str | None,
) -> str:
    prompt_parts = [strip_existing_contract(task_prompt)]

    if required_signature:
        prompt_parts.append(
            "Required interface contract (must match exactly):\n"
            f"`{required_signature}`\n"
            "Do not change the function name, parameters, or return type. "
            "You may add helper functions only if needed."
        )

    prompt_parts.append(
        f"Write only valid {language} code for this task. Do not include explanations."
    )

    return "\n\n".join(prompt_parts)


def strip_existing_contract(task_prompt: str) -> str:
    lines = task_prompt.strip().splitlines()
    marker = "Required interface contract (must match exactly):"
    out: list[str] = []
    i = 0

    while i < len(lines):
        if lines[i].strip() != marker:
            out.append(lines[i])
            i += 1
            continue

        i += 1
        while i < len(lines) and lines[i].strip():
            i += 1
        while i < len(lines) and not lines[i].strip():
            i += 1

    return "\n".join(out).strip()


def extract_rust_signature(rust_code: str, task_id: str) -> str | None:
    named_pattern = re.compile(
        rf"(?m)^\s*(pub\s+fn\s+{re.escape(task_id)}\s*\([^\n]*\)\s*(?:->\s*[^\{{\n]+)?)\s*\{{"
    )
    m = named_pattern.search(rust_code)
    if m:
        return " ".join(m.group(1).split())

    fallback_pattern = re.compile(
        r"(?m)^\s*(pub\s+fn\s+[A-Za-z_][A-Za-z0-9_]*\s*\([^\n]*\)\s*(?:->\s*[^\{\n]+)?)\s*\{"
    )
    m = fallback_pattern.search(rust_code)
    if m:
        return " ".join(m.group(1).split())
    return None


def extract_iron_signature(iron_code: str, task_id: str) -> str | None:
    lines = [line.rstrip() for line in iron_code.splitlines()]

    def collect_signature(start_idx: int) -> str | None:
        sig_lines: list[str] = []
        for idx in range(start_idx, len(lines)):
            text = lines[idx].strip()
            if not text:
                continue
            if text == "begin":
                break
            sig_lines.append(text)
        if not sig_lines:
            return None
        return " | ".join(sig_lines)

    target_prefix = f"function {task_id}"
    for i, line in enumerate(lines):
        if line.strip() == target_prefix:
            return collect_signature(i)

    for i, line in enumerate(lines):
        if line.strip().startswith("function "):
            return collect_signature(i)

    return None


def build_row(
    task_id: str,
    family: str,
    split: str,
    source: str,
    user_prompt: str,
    assistant_code: str,
    language: str,
) -> dict:
    return {
        "id": task_id,
        "family": family,
        "split": split,
        "language": language,
        "source": source,
        "conversations": [
            {"role": "user", "content": user_prompt},
            {"role": "assistant", "content": assistant_code},
        ],
    }


def reduce_to_iron(redox_cmd: list[str], rust_source: str) -> str | None:
    with tempfile.TemporaryDirectory(prefix="redox_export_") as tmp:
        rust_path = Path(tmp) / "input.rs"
        rust_path.write_text(rust_source, encoding="utf-8")

        proc = subprocess.run(
            [*redox_cmd, "reduce", str(rust_path)],
            text=True,
            capture_output=True,
            check=False,
        )
        if proc.returncode != 0:
            return None
        return proc.stdout


def write_split_files(
    out_dir: Path, prefix: str, rows_by_split: dict[str, list[dict]]
) -> None:
    for split in ("train", "val", "test"):
        rows = rows_by_split.get(split, [])
        out_file = out_dir / f"{prefix}_{split}.jsonl"
        with out_file.open("w", encoding="utf-8") as f:
            for row in rows:
                f.write(json.dumps(row, separators=(",", ":")) + "\n")


if __name__ == "__main__":
    raise SystemExit(main())
