# Redox + Iron Early-Stage Report

Date: 2026-02-08

## Project summary

Redox is a deterministic Rust <-> Iron transpiler.

- **Iron**: a lexical, natural-language-style superset of Rust syntax.
- **Goal**: test whether this representation improves small-model code generation quality.
- **Hypothesis**: for 4B-20B class models, Iron may reduce brittle syntax failure and improve semantic correctness on practical coding tasks.

This phase focused on building the full experimental loop:

1. transpiler robustness and corpus validation,
2. quality-gated dataset generation,
3. paired Rust/Iron finetunes,
4. compile+behavior evaluation.

## Infrastructure completed

- Userland validation harness for extracted anyhow-style patterns.
- Dataset validator enforcing: stability, determinism, compile quality, purity, split hygiene.
- Unsloth export pipeline for paired Rust/Iron JSONL data.
- Local evaluator for `compile@1` and `test@1` with per-family breakdown.

## Dataset snapshots used

- **foundation_v1**: 220 tasks (phase-1 gates passing).
- **foundation_v2**: 380 tasks = v1 carryover + targeted weak-family training expansion.

Targeted v2 additions (train):

- `result_unwrap_or*`
- `vec_pop*`
- `option_unwrap_or*`

## Early finetune/eval results

Evaluation set: 30 held-out tasks across:

- `closure_shift_const`
- `result_unwrap_or_const`
- `vec_pop_basic`

### Baseline (v1) - two seeds aggregate

From `eval/v1/report_aggregate_2seeds.json`:

- **Rust**
  - `compile@1`: `0.450 ± 0.017`
  - `test@1`: `0.067 ± 0.000`
- **Iron**
  - `compile@1`: `0.333 ± 0.000`
  - `test@1`: `0.333 ± 0.000`
  - transform pass rate: `0.667 ± 0.033`

Interpretation: early Iron already showed stronger behavioral correctness on this eval set, but lower compile reliability due to generation/oxidation failures.

### After targeted data expansion (v2, seed 2108)

From `eval/v2/report_seed2108.json`:

- **Rust**
  - `compile@1`: `0.400`
  - `test@1`: `0.067`
- **Iron**
  - `compile@1`: `0.600`
  - `test@1`: `0.600`
  - transform pass rate: `0.933`

Delta vs v1 (same seed 2108):

- Rust: compile `-0.033`, test `+0.000`
- Iron: compile `+0.267`, test `+0.267`, transform `+0.300`

## Why this is notable

This is a strong early signal for Iron in this controlled setting:

- Iron not only improved compile reliability after targeted data,
- it produced substantially more behaviorally correct code on held-out tasks.

The improvement tracks directly with targeted weak-family training data, suggesting the loop is responsive and directionally valid.

## Current caveats

- Small eval slice (30 tasks, 3 families) and single v2 seed so far.
- Some failures are still representational/oxidation edge cases, not purely model competence.
- We still need broader family coverage before drawing general conclusions.

## Immediate next steps

1. Add prompt-contract tightening (explicit required function signature in prompt).
2. Add Iron generation guardrails/retry for oxidize failures.
3. Expand evaluation families and rerun multi-seed comparison.
4. Continue targeted data expansion where failure clusters remain.

## Bottom line (early stage)

The first full-loop experiment is promising: in this pilot regime, Iron shows a meaningful path to higher functional correctness for a 4B instruct finetune when paired with targeted training data.
