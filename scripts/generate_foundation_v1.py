#!/usr/bin/env python3
"""Generate the phase-1 foundation_v1 candidate task set."""

from __future__ import annotations

import json
import re
from pathlib import Path


def main() -> None:
    base = Path("data/pilot/foundation_v1")
    prompts = base / "prompts"
    rust = base / "rust"
    manifest = base / "manifest.v1_candidate.jsonl"

    prompts.mkdir(parents=True, exist_ok=True)
    rust.mkdir(parents=True, exist_ok=True)

    records: list[dict[str, object]] = []

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
        signature = rust_signature_for_task(rust_code, task_id)
        if signature is None:
            return prompt_text.strip()

        return (
            f"{prompt_text.strip()}\n\n"
            "Required interface contract (must match exactly):\n"
            f"`{signature}`\n"
            "Do not change the function name, parameters, or return type."
        )

    def add_task(
        task_id: str,
        split: str,
        family: str,
        prompt_text: str,
        rust_code: str,
    ) -> None:
        (prompts / f"{task_id}.md").write_text(
            prompt_with_contract(prompt_text, rust_code, task_id) + "\n",
            encoding="utf-8",
        )
        (rust / f"{task_id}.rs").write_text(rust_code.strip() + "\n", encoding="utf-8")
        records.append(
            {
                "id": task_id,
                "split": split,
                "family": family,
                "prompt_path": f"prompts/{task_id}.md",
                "rust_path": f"rust/{task_id}.rs",
                "deps": [],
                "unsafe": False,
            }
        )

    for i in range(1, 31):
        task = f"add_const_{i:03d}"
        add_task(
            task,
            "train",
            "arith_add_const",
            f"# Add Constant\n\nImplement a function that adds {i} to an input `i32`.",
            f"pub fn {task}(x: i32) -> i32 {{\n    x + {i}\n}}",
        )

    for i in range(1, 31):
        k = i + 1
        task = f"mul_const_{i:03d}"
        add_task(
            task,
            "train",
            "arith_mul_const",
            f"# Multiply Constant\n\nImplement a function that multiplies an input `i32` by {k}.",
            f"pub fn {task}(x: i32) -> i32 {{\n    x * {k}\n}}",
        )

    for i in range(1, 21):
        task = f"option_some_{i:03d}"
        v = 10 + i
        add_task(
            task,
            "train",
            "option_some_const",
            f"# Return Some Constant\n\nImplement a function that returns `Some({v})`.",
            f"pub fn {task}() -> Option<i32> {{\n    Some({v})\n}}",
        )

    for i in range(1, 21):
        task = f"option_map_add_{i:03d}"
        add_task(
            task,
            "train",
            "option_map_add_const",
            f"# Map Option Add\n\nImplement a function that adds {i} to an `Option<i32>` using `map`.",
            f"pub fn {task}(input: Option<i32>) -> Option<i32> {{\n    input.map(|n| n + {i})\n}}",
        )

    for i in range(1, 21):
        task = f"vec_push_{i:03d}"
        v = i * 3
        add_task(
            task,
            "train",
            "vec_push_const",
            f"# Push Value To Vec\n\nImplement a function that pushes {v} into a new vector and returns it.",
            f"pub fn {task}() -> Vec<i32> {{\n    let mut v = Vec::new();\n    v.push({v});\n    v\n}}",
        )

    for i in range(1, 21):
        task = f"vec_capacity_{i:03d}"
        cap = i + 5
        add_task(
            task,
            "train",
            "vec_with_capacity",
            f"# Vec With Capacity\n\nImplement a function that returns a vector with capacity {cap}.",
            f"pub fn {task}() -> Vec<i32> {{\n    Vec::with_capacity({cap})\n}}",
        )

    for i in range(1, 21):
        task = f"helper_inc_{i:03d}"
        add_task(
            task,
            "train",
            "helper_call_inc",
            f"# Helper Call\n\nImplement a helper function and call it to add {i}.",
            (
                f"fn inc_{i:03d}(x: i32) -> i32 {{\n    x + {i}\n}}\n\n"
                f"pub fn {task}(x: i32) -> i32 {{\n    inc_{i:03d}(x)\n}}"
            ),
        )

    for i in range(1, 21):
        task = f"result_ok_{i:03d}"
        v = 100 + i
        add_task(
            task,
            "val",
            "result_ok_const",
            f"# Return Ok Constant\n\nImplement a function that returns `Ok({v})` with unit error type.",
            f"pub fn {task}() -> Result<i32, ()> {{\n    Ok({v})\n}}",
        )

    for i in range(1, 11):
        task = f"alias_result_{i:03d}"
        v = 200 + i
        add_task(
            task,
            "val",
            "type_alias_result",
            f"# Result Type Alias\n\nDefine a generic result alias and return `Ok({v})`.",
            (
                f"type Alias{i:03d}<T> = Result<T, ()>;\n\n"
                f"pub fn {task}() -> Alias{i:03d}<i32> {{\n    Ok({v})\n}}"
            ),
        )

    for i in range(1, 11):
        task = f"closure_shift_{i:03d}"
        k = i + 2
        add_task(
            task,
            "test",
            "closure_shift_const",
            f"# Closure Shift\n\nImplement a function using a closure that adds {k} to input.",
            f"pub fn {task}(x: i32) -> i32 {{\n    let f = |n| n + {k};\n    f(x)\n}}",
        )

    for i in range(1, 11):
        task = f"result_unwrap_or_{i:03d}"
        add_task(
            task,
            "test",
            "result_unwrap_or_const",
            f"# Result Unwrap Or\n\nImplement a function that unwraps `Result<i32, String>` or returns {i}.",
            f"pub fn {task}(input: Result<i32, String>) -> i32 {{\n    input.unwrap_or({i})\n}}",
        )

    for i in range(1, 11):
        task = f"vec_pop_{i:03d}"
        a = i
        b = i + 1
        c = i + 2
        add_task(
            task,
            "test",
            "vec_pop_basic",
            f"# Pop From Vector\n\nImplement a function that pops from `vec![{a}, {b}, {c}]`.",
            f"pub fn {task}() -> Option<i32> {{\n    let mut v = vec![{a}, {b}, {c}];\n    v.pop()\n}}",
        )

    with manifest.open("w", encoding="utf-8") as f:
        for rec in records:
            f.write(json.dumps(rec, separators=(",", ":")) + "\n")

    print(f"Generated {len(records)} tasks at {manifest}")


if __name__ == "__main__":
    main()
