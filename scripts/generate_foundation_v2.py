#!/usr/bin/env python3

from __future__ import annotations

import json
import re
import shutil
from pathlib import Path


def load_jsonl(path: Path) -> list[dict]:
    rows = []
    for line in path.read_text(encoding="utf-8").splitlines():
        if line.strip():
            rows.append(json.loads(line))
    return rows


def main() -> None:
    root = Path("data/pilot")
    v1 = root / "foundation_v1"
    v2 = root / "foundation_v2"

    if not v1.exists():
        raise FileNotFoundError(f"Missing source dataset: {v1}")

    if v2.exists():
        shutil.rmtree(v2)

    (v2 / "prompts").mkdir(parents=True, exist_ok=True)
    (v2 / "rust").mkdir(parents=True, exist_ok=True)

    for src in (v1 / "prompts").glob("*.md"):
        shutil.copy2(src, v2 / "prompts" / src.name)
    for src in (v1 / "rust").glob("*.rs"):
        shutil.copy2(src, v2 / "rust" / src.name)

    rows = load_jsonl(v1 / "manifest.v1_candidate.jsonl")
    ids = {r["id"] for r in rows}

    def rust_signature_for_task(rust_code: str, task_id: str) -> str | None:
        pattern = re.compile(
            rf"(?m)^\s*(pub\s+fn\s+{re.escape(task_id)}\s*\([^\n]*\)\s*(?:->\s*[^\{{\n]+)?)\s*\{{"
        )
        m = pattern.search(rust_code)
        if m:
            return " ".join(m.group(1).split())

        fallback = re.compile(
            r"(?m)^\s*(pub\s+fn\s+[A-Za-z_][A-Za-z0-9_]*\s*\([^\n]*\)\s*(?:->\s*[^\{\n]+)?)\s*\{"
        )
        m = fallback.search(rust_code)
        if m:
            return " ".join(m.group(1).split())
        return None

    def prompt_with_contract(prompt_text: str, rust_code: str, task_id: str) -> str:
        marker = "Required interface contract (must match exactly):"
        cleaned = prompt_text.strip()
        if marker in cleaned:
            return cleaned

        signature = rust_signature_for_task(rust_code, task_id)
        if signature is None:
            return cleaned

        return (
            f"{cleaned}\n\n"
            "Required interface contract (must match exactly):\n"
            f"`{signature}`\n"
            "Do not change the function name, parameters, or return type."
        )

    for row in rows:
        task_id = row["id"]
        prompt_path = v2 / row["prompt_path"]
        rust_path = v2 / row["rust_path"]
        prompt_text = prompt_path.read_text(encoding="utf-8")
        rust_code = rust_path.read_text(encoding="utf-8")
        prompt_path.write_text(
            prompt_with_contract(prompt_text, rust_code, task_id) + "\n",
            encoding="utf-8",
        )

    def add_task(task_id: str, family: str, prompt: str, rust: str) -> None:
        if task_id in ids:
            raise ValueError(f"Duplicate id: {task_id}")
        ids.add(task_id)
        (v2 / "prompts" / f"{task_id}.md").write_text(
            prompt_with_contract(prompt, rust, task_id) + "\n",
            encoding="utf-8",
        )
        (v2 / "rust" / f"{task_id}.rs").write_text(
            rust.strip() + "\n", encoding="utf-8"
        )
        rows.append(
            {
                "id": task_id,
                "split": "train",
                "family": family,
                "prompt_path": f"prompts/{task_id}.md",
                "rust_path": f"rust/{task_id}.rs",
                "deps": [],
                "unsafe": False,
            }
        )

    for i in range(1, 61):
        task_id = f"result_unwrap_or_train_{i:03d}"
        fallback = i
        add_task(
            task_id,
            "result_unwrap_or_train",
            (
                "# Result Unwrap Or (Train)\n\n"
                f"Implement a function that takes `Result<i32, String>` and returns `input.unwrap_or({fallback})`."
            ),
            (
                f"pub fn {task_id}(input: Result<i32, String>) -> i32 {{\n"
                f"    input.unwrap_or({fallback})\n"
                "}"
            ),
        )

    for i in range(1, 61):
        task_id = f"vec_pop_train_{i:03d}"
        a, b, c = i, i + 1, i + 2
        add_task(
            task_id,
            "vec_pop_train",
            (
                "# Vec Pop (Train)\n\n"
                f"Implement a function that pops from `vec![{a}, {b}, {c}]` and returns `Option<i32>`."
            ),
            (
                f"pub fn {task_id}() -> Option<i32> {{\n"
                f"    let mut v = vec![{a}, {b}, {c}];\n"
                "    v.pop()\n"
                "}"
            ),
        )

    for i in range(1, 41):
        task_id = f"option_unwrap_or_train_{i:03d}"
        fallback = i + 2
        add_task(
            task_id,
            "option_unwrap_or_train",
            (
                "# Option Unwrap Or (Train)\n\n"
                f"Implement a function that takes `Option<i32>` and returns `input.unwrap_or({fallback})`."
            ),
            (
                f"pub fn {task_id}(input: Option<i32>) -> i32 {{\n"
                f"    input.unwrap_or({fallback})\n"
                "}"
            ),
        )

    manifest_path = v2 / "manifest.v2_candidate.jsonl"
    with manifest_path.open("w", encoding="utf-8") as f:
        for row in rows:
            f.write(json.dumps(row, separators=(",", ":")) + "\n")

    summary = {
        "base_source": str(v1),
        "total_tasks": len(rows),
        "added_tasks": 160,
        "families_added": [
            "result_unwrap_or_train",
            "vec_pop_train",
            "option_unwrap_or_train",
        ],
    }
    (v2 / "generation_summary.json").write_text(
        json.dumps(summary, indent=2) + "\n", encoding="utf-8"
    )

    print(f"Generated {len(rows)} tasks at {manifest_path}")


if __name__ == "__main__":
    main()
