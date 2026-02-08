# Redox + Iron Experimental Report (Exploratory Phase)

Date: 2026-02-08

Status: exploratory phase concluded; scale-up phase approved

## 1) Executive Summary

This report documents the full process from transpiler proof-of-concept to multi-seed model evaluation for the hypothesis:

> A natural-language, lexical superset of a programming language (Iron over Rust) can improve code-generation quality for compute/size constrained models.

Key outcomes:

- A deterministic Rust <-> Iron pipeline (Redox) was built and hardened for a focused Rust feature set.
- A full research loop was implemented: corpus validation -> dataset generation -> paired finetuning -> compile/test evaluation.
- Across early controlled runs, Iron showed strong functional performance and, after protocol corrections, improved over Rust on aggregate metrics in this pilot.
- A prompt-contract mismatch (Rust signatures in Iron eval prompts) was identified, fixed, and isolated via strict-attribution analysis.
- Post-fix, Iron regained high transform reliability and strong compile/test performance.

This does not yet establish broad generality, but it provides strong early directional evidence that the hypothesis is worth scaling.

## 2) Research Question and Hypothesis

### Research question

Can a natural-language-style coding representation improve generated-code quality for small finetuned models, despite increased token verbosity relative to condensed syntax?

### Hypothesis under test

Given LLM strengths in natural language, a lexical natural-language superset (Iron) of Rust may improve generated code correctness compared with direct Rust outputs, especially under small-model constraints.

## 3) System Built for the Study

### 3.1 Redox (transpiler/oxidizer)

Repository modules (core):

- `src/parser.rs`: Rust AST (`syn`) -> Iron emission
- `src/iron_tokenizer.rs`, `src/iron_parser.rs`, `src/iron_ast.rs`: Iron parse stack
- `src/oxidation.rs`: Iron AST -> Rust source
- `src/mappings.rs`, `src/keywords.rs`, `src/emitter.rs`: mapping/syntax support
- `src/lib.rs`: API surface (`transpile`, `oxidize`, validation)
- `src/main.rs`: CLI (`reduce`, `oxidize`, `validate`)

Design goals maintained:

- deterministic reduction behavior,
- stable emitted syntax,
- compile-correct oxidation for supported features,
- explicit, lexicalized constructs in Iron.

### 3.2 Validation and experimentation tooling

- `scripts/dataset_validator.py`
- `scripts/export_unsloth_dataset.py`
- `scripts/generate_foundation_v1.py`
- `scripts/generate_foundation_v2.py`
- `scripts/evaluate_predictions.py`
- `scripts/aggregate_eval_reports.py`

### 3.3 Corpus and roundtrip harnesses

- `tests/roundtrip_tests.rs` for supported-feature fidelity
- `tests/anyhow_roundtrip_tests.rs` for userland-pattern status tracking

`anyhow` status classes:

- `TranspileFailed`
- `OxidizeFailed`
- `RoundtripCompileFailed`
- `RoundtripCompiled`

## 4) Experimental Protocol

Canonical protocol and parity rules are now documented in `docs/EVAL_PROTOCOL.md`.

### 4.1 Model and training setup (pilot)

From `notebooks/Qwen3_(4B)_Instruct_Redox_AllInOne.ipynb`:

- Base model: `unsloth/Qwen3-4B-Instruct-2507`
- Arms: `rust`, `iron`
- Seeds: `3407`, `2108`
- Sequence length: `2048`
- Max new tokens: `320`
- Epochs: `1`
- 4-bit load enabled
- LoRA:
  - `r=32`, `lora_alpha=32`
  - target modules: `q_proj`, `k_proj`, `v_proj`, `o_proj`, `gate_proj`, `up_proj`, `down_proj`
- Training batch config:
  - `per_device_train_batch_size=2`
  - `gradient_accumulation_steps=4`
  - `learning_rate=2e-4`
- Generation config:
  - `do_sample=False`
  - `temperature=0.0`

### 4.2 Datasets

#### foundation_v1

- Manifest: `data/pilot/foundation_v1/manifest.v1_candidate.jsonl`
- Total tasks: `220`
- Splits: `train=160`, `val=30`, `test=30`
- Families: `12`
- Validator gates all pass in `data/pilot/foundation_v1/report.v1_candidate.json`

#### foundation_v2

- Manifest: `data/pilot/foundation_v2/manifest.v2_candidate.jsonl`
- Total tasks: `380`
- Splits: `train=320`, `val=30`, `test=30`
- Families: `15`
- Added task count: `160` (`data/pilot/foundation_v2/generation_summary.json`)
- New targeted training families:
  - `result_unwrap_or_train`
  - `vec_pop_train`
  - `option_unwrap_or_train`

### 4.3 Evaluation setup

Evaluation scripts:

- per-seed: `scripts/evaluate_predictions.py`
- multi-seed aggregate: `scripts/aggregate_eval_reports.py`

Held-out test set (30 tasks):

- `closure_shift_const` (10)
- `result_unwrap_or_const` (10)
- `vec_pop_basic` (10)

Metrics:

- `transform_pass` / transform rate (Iron includes oxidation stage)
- `compile@1`
- `test@1`
- per-family compile/test rates
- failure phase and failure taxonomy

## 5) Engineering and Fidelity Milestones

### 5.1 Core transpiler milestone

The project progressed from initial parser/oxidizer bring-up to a stable core with support for:

- associated functions and method calls,
- constructor patterns (`Some`, `Ok`, etc.),
- macros (including bracket awareness),
- closures (including use in method arguments),
- generic bounds and common control flow.

Historical milestone docs:

- `plans/notebook.md`
- `FIDELITY_REPORT.md`

### 5.2 Current test status (local verification)

`cargo test` run on 2026-02-08:

- unit tests: `13 passed`
- anyhow corpus tests: `3 passed`
- integration roundtrip tests: `13 passed`
- doc tests: `2 passed`

## 6) Results Across Experiment Versions

### 6.1 Aggregate and key-run metrics

| Run | Rust compile@1 | Rust test@1 | Rust transform | Iron compile@1 | Iron test@1 | Iron transform | Delta compile (Iron-Rust) | Delta test |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| v1 aggregate (2 seeds) | 0.450 | 0.067 | 1.000 | 0.333 | 0.333 | 0.667 | -0.117 | +0.267 |
| v2 seed2108 | 0.400 | 0.067 | 1.000 | 0.600 | 0.600 | 0.933 | +0.200 | +0.533 |
| v2.5 aggregate (2 seeds) | 0.717 | 0.717 | 1.000 | 0.667 | 0.667 | 0.733 | -0.050 | -0.050 |
| v2.5 after parser fix (2 seeds) | 0.717 | 0.717 | 1.000 | 1.000 | 1.000 | 1.000 | +0.283 | +0.283 |
| v2.6 aggregate (2 seeds) | 0.917 | 0.917 | 1.000 | 0.967 | 0.967 | 0.983 | +0.050 | +0.050 |
| v2.6 strict attribution (old Rust + new Iron) | 0.717 | 0.717 | 1.000 | 0.967 | 0.967 | 0.983 | +0.250 | +0.250 |

Sources:

- `eval/v1/report_aggregate_2seeds.json`
- `eval/v2/report_seed2108.json`
- `eval/v2_5/report_aggregate_2seeds.json`
- `eval/v2_5/report_aggregate_2seeds_after_parser_fix.json`
- `eval/v2_6/report_aggregate_2seeds.json`
- `eval/v2_6/report_aggregate_2seeds_iron_prompt_fix_only.json`

### 6.2 Interpretation by phase

#### v1 baseline

- Rust had higher compile reliability than Iron, but much lower behavioral pass rate.
- Iron transform reliability was a major bottleneck (`0.667`).

#### v2 targeted data expansion

- On seed 2108, Iron improved materially (compile/test to `0.600`, transform to `0.933`).
- Improvement aligned with targeted weak-family augmentation.

#### v2.5 regression window

- Rust and Iron both improved over earlier baselines, but Iron transform remained depressed (`0.733`).
- Failure taxonomy heavily concentrated in Iron transform/oxidation classes.

#### v2.6 prompt-contract corrected

- Iron reached `0.967` compile/test and `0.983` transform in 2-seed aggregate.
- Iron exceeded Rust by `+0.050` on compile and test in as-run aggregate.
- Strict attribution (holding Rust predictions fixed to v2.5) showed a larger Iron gain: `+0.250` compile/test delta.

## 7) Prompt-Contract Incident and Isolation Analysis

### 7.1 Root cause

Iron evaluation prompts were carrying Rust function signatures (`pub fn ...`) rather than Iron signatures.

This was corrected in dataset export logic:

- `scripts/export_unsloth_dataset.py`
  - `build_user_prompt(...)` now strips and replaces any existing contract block.
  - `strip_existing_contract(...)` added.

Result:

- Iron prompt contracts in `data/pilot/foundation_v2/unsloth/iron_test.jsonl` now use Iron-form signatures (`function ... | takes ... | returns ...`).

### 7.2 Strict-attribution evidence

To isolate prompt-contract effects, evaluation used old Rust predictions with newly generated Iron predictions:

- `eval/v2_6/report_aggregate_2seeds_iron_prompt_fix_only.json`

Observed shifts vs v2.5 baseline aggregate:

- Rust unchanged: compile/test `0.717`
- Iron compile: `0.667 -> 0.967` (`+0.300`)
- Iron test: `0.667 -> 0.967` (`+0.300`)
- Iron transform: `0.733 -> 0.983` (`+0.250`)

Failure taxonomy shift:

- Iron v2.5 baseline: `{'iron_parse_or_oxidize': 16, 'other': 4}`
- Iron v2.6 strict attribution: `{'iron_parse_or_oxidize': 1, 'name_resolution': 1}`

This strongly supports prompt-contract mismatch as a major contributor to the observed Iron regression.

## 8) Remaining Failure Analysis (v2.6)

Residual Iron failures are narrow and concentrated:

- Both seeds fail only `closure_shift_008` in `closure_shift_const`.
- Seed 3407: transform failure from malformed `macro closure` hybrid syntax.
- Seed 2108: compile failure from unresolved macro (`add_10!`) where closure behavior was expected.

Rust on the same task id produced normal closure patterns and passed.

Interpretation:

- Remaining risk is primarily lexical confusion between macro and closure forms in one family/task pattern, not broad systemic failure.

## 9) Threats to Validity and Limitations

### 9.1 Scope limitations

- Small held-out evaluation set (30 tasks, 3 families).
- Single base model family (Qwen3 4B instruct with LoRA setup).
- Current feature set does not yet represent full Rust complexity.

### 9.2 Attribution complexity

- Parser/oxidizer improvements and model/data changes can interact.
- Strict-attribution reruns improve confidence but do not eliminate all confounds.

### 9.3 Protocol sensitivity

- Prompt contract formatting materially affects outcomes.
- Asymmetric prompt tweaks can bias conclusions if not controlled.

Protocol controls now formalized in `docs/EVAL_PROTOCOL.md`.

## 10) Conclusions

Within this exploratory setup:

- The Redox + Iron approach is technically viable and reproducible for a meaningful Rust subset.
- Early experimental signals are positive for the hypothesis.
- After correcting protocol drift, Iron shows strong compile/test performance and competitive or superior aggregate performance in this pilot.

Most importantly, the project now has an operational, auditable loop capable of testing this hypothesis rigorously at larger scale.

## 11) Recommended Scale-Up Plan (Phase 2)

1. Expand Iron/Redox feature coverage while preserving deterministic round-trip semantics.
2. Increase real-world corpus testing under `tests/corpus/` and track status classes over time.
3. Generate larger, more diverse paired datasets with locked protocol versions.
4. Run multi-seed comparisons on larger models (for example 8B class).
5. Continue strict-attribution runs for any protocol or parser change.

Roadmap reference: `docs/ROADMAP.md`

## 12) Reproducibility Appendix

### 12.1 Key artifacts

- Protocol: `docs/EVAL_PROTOCOL.md`
- Roadmap: `docs/ROADMAP.md`
- Run manifest: `experiments/v2_6_iron_eval_prompt_fix.json`
- Evaluation reports:
  - `eval/v1/report_aggregate_2seeds.json`
  - `eval/v2/report_seed2108.json`
  - `eval/v2_5/report_aggregate_2seeds.json`
  - `eval/v2_5/report_aggregate_2seeds_after_parser_fix.json`
  - `eval/v2_6/report_aggregate_2seeds.json`
  - `eval/v2_6/report_aggregate_2seeds_iron_prompt_fix_only.json`

### 12.2 Common commands

Build and test:

```bash
cargo build --bin redox
cargo test
```

Export paired datasets:

```bash
python3 scripts/export_unsloth_dataset.py data/pilot/foundation_v2/manifest.v2_candidate.jsonl \
  --out-dir data/pilot/foundation_v2/unsloth
```

Evaluate predictions per seed:

```bash
python3 scripts/evaluate_predictions.py \
  --rust eval/v2_6/predictions_rust_seed3407.jsonl \
  --iron eval/v2_6/predictions_iron_seed3407.jsonl \
  --redox-cmd target/debug/redox \
  --out eval/v2_6/report_seed3407.json
```

Aggregate multi-seed reports:

```bash
python3 scripts/aggregate_eval_reports.py \
  eval/v2_6/report_seed3407.json \
  eval/v2_6/report_seed2108.json \
  --out eval/v2_6/report_aggregate_2seeds.json
```

### 12.3 Notes for white paper drafting

- Present this phase as exploratory evidence, not final proof.
- Emphasize protocol controls and strict-attribution methodology.
- Separate claims about representation effects from claims about parser maturity.
- Include the prompt-contract incident as a concrete example of how subtle protocol mismatch can mask true model behavior.
