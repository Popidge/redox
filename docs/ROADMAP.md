# Redox Roadmap

This roadmap captures the next phase after the successful exploratory validation of the Iron hypothesis.

## North Star

Build a robust, reversible Rust <-> Iron pipeline and evaluate whether natural-language-first code representations improve small and medium LLM code quality under controlled conditions.

## Phase 1 - Harden Core Transpiler

Status: in progress

Goals:
- Expand Iron coverage for common Rust features while preserving deterministic, lossless round-trip behavior for supported syntax.
- Keep Rust emission stable, readable, and compile-correct.
- Add regression tests for every parser/oxidation bug discovered in eval runs.

Priority feature areas:
- Pattern matching and richer `match` forms.
- Additional closure and function-expression variants.
- More method and associated function forms on standard library types.
- Broader generic type and trait-bound coverage.

Exit criteria:
- Targeted roundtrip and corpus tests pass consistently.
- Newly added syntax has deterministic `reduce` output and successful `oxidize` compile checks.

## Phase 2 - Validate on Real-World Code

Goals:
- Move beyond synthetic micro-tasks with curated real-world snippets.
- Track fidelity quality at both parse and behavioral levels.

Workstreams:
- Expand `tests/corpus/` with additional standard library and crate-inspired fixtures.
- Define a stable corpus gate for inclusion (determinism + compile + readability checks).
- Report failure taxonomy by syntax family to guide parser priorities.

Exit criteria:
- Real-world corpus coverage improves without regressions in existing feature families.

## Phase 3 - Scale Dataset Generation

Goals:
- Generate larger, more diverse Rust/Iron paired datasets while preserving split hygiene and contract parity.
- Keep dataset exports reproducible and versioned.

Workstreams:
- Add new task families for higher feature diversity.
- Maintain per-version manifests and generated reports.
- Preserve strict train/val/test split separation by family.

Exit criteria:
- New dataset version published with validator reports and clear changelog vs prior version.

## Phase 4 - Scale Model Experiments

Goals:
- Re-run protocol on larger models (for example 8B class) with fixed prompt contract rules.
- Measure whether Iron advantage persists as model capacity increases.

Workstreams:
- Keep experiment configs explicit (seeds, model, dataset, prompt contract variant).
- Store comparable per-seed reports and aggregate summaries.
- Record adapter locations and run metadata in experiment manifests.

Exit criteria:
- Multi-seed comparisons available for Rust vs Iron across at least one larger model tier.

## Research Guardrails

- Do not tune prompts asymmetrically across arms to chase outcomes.
- Keep interface contract format language-consistent per arm.
- Treat eval protocol as versioned; document any change before running.
- Separate parser fixes from model/prompt changes where possible to improve attribution.

## Milestone Deliverables

For each milestone:
- Short milestone note under `docs/` with objective, protocol version, and key results.
- Linked evaluation artifacts under `eval/<version>/`.
- Reproducibility manifest under `experiments/`.
