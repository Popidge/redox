# Dataset Validator

`scripts/dataset_validator.py` validates pilot Rust/Iron training manifests against the phase-1 gates from `plans/small_model_finetune_experiment.md`.

## Usage

```bash
# Build redox binary once
cargo build --bin redox

# Validate a manifest
python3 scripts/dataset_validator.py data/pilot/examples/manifest.sample.jsonl

# Write machine-readable report
python3 scripts/dataset_validator.py data/pilot/examples/manifest.sample.jsonl \
  --report-json data/pilot/examples/report.json
```

## Manifest schema (JSONL)

Each line is one task object.

Required fields:

- `id` (string): unique task id
- `split` (string): `train`, `val`, or `test`
- `family` (string): semantic family label used for split leakage checks
- `prompt_path` (string): path to task prompt text
- `rust_path` (string): path to canonical Rust solution

Optional fields:

- `tests_path` (string): path to task tests
- `deps` (string array, default `[]`): dependency list metadata
- `unsafe` (boolean, default `false`): whether task uses `unsafe`

## Gates checked

1. Pipeline stability (`reduce` + `oxidize` succeed for every task)
2. Determinism (two `reduce` runs are byte-identical)
3. Data quality (Iron -> Rust output compiles with `rustc`)
4. Iron purity (non-verbatim ratio >= threshold, default `0.98`)
5. Split hygiene (`family` appears in exactly one split)

Use `--allow-deps` or `--allow-unsafe` to relax phase-1 constraints when needed.

## Foundation starter set

Validate the checked-in starter candidate pool:

```bash
python3 scripts/dataset_validator.py data/pilot/foundation/manifest.v0_candidate.jsonl \
  --report-json data/pilot/foundation/report.v0_candidate.json
```

Generate and validate the scaled v1 pool (~200 tasks):

```bash
python3 scripts/generate_foundation_v1.py
python3 scripts/dataset_validator.py data/pilot/foundation_v1/manifest.v1_candidate.jsonl \
  --report-json data/pilot/foundation_v1/report.v1_candidate.json
```

Generate and validate v2 pool (v1 carryover + targeted weak-family data):

```bash
python3 scripts/generate_foundation_v2.py
python3 scripts/dataset_validator.py data/pilot/foundation_v2/manifest.v2_candidate.jsonl \
  --report-json data/pilot/foundation_v2/report.v2_candidate.json
```

Export Unsloth-ready Rust and Iron JSONL files:

```bash
python3 scripts/export_unsloth_dataset.py data/pilot/foundation_v1/manifest.v1_candidate.jsonl \
  --out-dir data/pilot/foundation_v1/unsloth

python3 scripts/export_unsloth_dataset.py data/pilot/foundation_v2/manifest.v2_candidate.jsonl \
  --out-dir data/pilot/foundation_v2/unsloth
```

Evaluate Rust and Iron prediction files:

```bash
python3 scripts/evaluate_predictions.py \
  --rust eval/predictions_rust_seed3407.jsonl \
  --iron eval/predictions_iron_seed3407.jsonl \
  --out eval/report_seed3407.json
```

Aggregate multiple seed reports:

```bash
python3 scripts/aggregate_eval_reports.py \
  eval/report_seed3407.json \
  eval/report_seed2108.json \
  --out eval/report_aggregate_2seeds.json
```
