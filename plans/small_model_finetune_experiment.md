# Redox/Iron Small-Model Finetune Experiment Spec (v1)

## 1) Purpose

Test the hypothesis:

> For small coding models (8B-20B), training and prompting with Iron can improve practical code generation outcomes over Rust on dependency-light tasks.

This experiment is intentionally low-cost and early-stage. It optimizes for decisive signal, not maximal benchmark coverage.

## 2) Current Readiness Snapshot

Current repo status (already achieved):

- Core roundtrip suite is stable (`tests/roundtrip_tests.rs` passing).
- Anyhow corpus harness exists (`tests/anyhow_roundtrip_tests.rs`).
- Anyhow pipeline has no transpile/oxidize crashes; failures are compile-level fidelity issues.
- Compile parity test against originals exists for anyhow corpus.

Interpretation: the system is ready for a constrained pilot dataset where we avoid unsupported edge features.

## 3) Scope (Phase 1)

In-scope task profile:

- Common algorithms and patterns (search/sort/DP/graph basics/string/iterators/error handling).
- Minimal dependencies (prefer stdlib-only; optional tiny utility crates if needed).
- No `unsafe`, no macros-heavy APIs, no proc-macros.
- Avoid advanced language edge features in phase 1 (GATs, deep trait-object plumbing, complex macro metaprogramming).

Out of scope:

- Demonstrating superiority on full ecosystem Rust.
- Large-model training claims.
- Production reliability guarantees.

## 4) Entry Gates (Go/No-Go Before Training)

All gates must pass on the selected pilot corpus.

1. **Pipeline stability**: 0 transpile/oxidize crashes.
2. **Determinism**: same input -> byte-identical transpiler output.
3. **Data quality**: each training pair compiles after Iron->Rust conversion.
4. **Iron purity**: >=98% of phase-1 pairs are native Iron (not verbatim fallback items).
5. **Split hygiene**: no leakage across train/val/test by problem family.

If gate 4 fails, shrink the task domain rather than expanding transpiler scope immediately.

## 5) Dataset Plan

Target size (phase 1):

- 300-700 tasks total (start at ~400).
- Suggested split: 70% train / 15% val / 15% test.

Per-task artifacts:

- `prompt.md` (problem statement)
- `reference.rs` (canonical Rust solution)
- `reference.iron` (transpiled Iron form)
- `tests.rs` (unit tests)
- metadata (`difficulty`, `family`, `deps`, `unsafe=false`, `verbatim_used=false`)

Filtering rules:

- Drop items with transpiler fallback/verbatim in phase 1.
- Drop items that fail deterministic roundtrip checks.
- Keep task families balanced to avoid skew (e.g. too many array problems).

## 6) Training Arms

Use identical base model and nearly identical compute budgets.

- **Arm A (Rust model)**: finetune on prompt -> Rust solution.
- **Arm B (Iron model)**: finetune on prompt -> Iron solution.

Notes:

- Keep token budget per arm approximately equal.
- Use same optimization recipe (LoRA/QLoRA config, epochs, lr schedule, max length).
- Run at least 2 random seeds if budget allows.

## 7) Evaluation Protocol

Held-out test set only. Same prompts for both arms.

For each generated sample:

1. If Rust arm: compile/test directly.
2. If Iron arm: oxidize to Rust, then compile/test.

Primary metrics:

- `compile@1`
- `test@1`

Secondary metrics:

- median attempts-to-pass (if using pass@k generation)
- error distribution (syntax, type, borrow, logic)
- average generated tokens and latency

Report both absolute and delta metrics (`Iron - Rust`).

## 8) Decision Thresholds (Phase 1)

Define success for continuing investment:

- Iron `test@1` >= Rust `test@1` + 8 absolute points, **or**
- Iron `test@1` >= Rust `test@1` + 5 points plus materially lower syntax/type failure rate.

Define neutral/stop outcomes:

- Delta in `test@1` within +/-3 points -> inconclusive; do one focused iteration only.
- Iron worse by >3 points -> pause and reassess language design/data quality assumptions.

## 9) Minimal Timeline

Week 1:

- Finalize phase-1 task list and filters.
- Generate and validate Rust/Iron pairs.
- Freeze dataset v0.

Week 2:

- Train Rust and Iron arms.
- Run held-out evaluation and error analysis.
- Produce recommendation memo.

## 10) Risks and Mitigations

1. **Risk: false gains from easier tokenization only on trivial tasks**
   - Mitigation: stratify test set by difficulty and family.
2. **Risk: transpiler artifacts contaminate conclusions**
   - Mitigation: strict purity filter and compile parity checks.
3. **Risk: unfair token-budget mismatch**
   - Mitigation: enforce near-equal token/compute budgets.
4. **Risk: overfitting to templates**
   - Mitigation: dedupe by semantic family and hold out variants.

## 11) Deliverables

- `dataset_v0_manifest.jsonl` with quality flags
- Trained adapters/checkpoints for Rust and Iron arms
- Evaluation report with metric table + failure taxonomy
- Clear go/no-go recommendation for phase 2

## 12) Immediate Next Actions

1. Implement a dataset validator script that enforces entry gates.
2. Tag each candidate task with `family` and `verbatim_used`.
3. Build initial 400-task candidate pool and run filter pass.
4. Freeze the first training-ready split and begin pilot finetunes.
