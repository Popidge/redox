# Experiments: Structure and Migration Plan

This directory is the control plane for future Redox experiments.

## Why this exists

The exploratory phase produced strong signals and many useful artifacts. Going forward, we want cleaner attribution, easier reruns, and a repository layout that scales.

## Target repository structure

Top-level layout to converge toward:

- `src/`, `tests/`: Rust transpiler/oxidizer code and fidelity tests.
- `scripts/`: dataset, validation, prediction evaluation, and aggregation tooling.
- `data/`: versioned task manifests, prompts, canonical Rust solutions, and exported dataset files.
- `eval/`: per-run predictions and reports, grouped by protocol version.
- `docs/`: roadmap, protocol definitions, milestone notes, and design decisions.
- `experiments/`: run manifests and reproducibility metadata.
- `notebooks/` (planned): move notebook files from repo root into organized subfolders.

Local-only (gitignored):

- `training/`: checkpoints, adapters, and optimizer states.
- `artifacts/` (planned): temporary local outputs and scratch exports.

## Experiment manifest format

Each run should have one manifest file:

- Path: `experiments/<run_tag>.json`
- Minimum fields:
  - `run_tag`
  - `date_utc`
  - `model_base`
  - `arms` (for example `rust`, `iron`)
  - `seeds`
  - `dataset_version`
  - `dataset_paths`
  - `prompt_contract_version`
  - `prediction_paths`
  - `report_paths`
  - `aggregate_report_path`
  - `notes`

Optional but recommended:

- `adapter_locations` (local path, Drive path, or remote reference)
- `runtime` (Colab, local GPU, container)
- `script_versions` (tool/script commit hash snapshot)

## Migration plan (incremental)

### Step 1: Freeze exploratory baseline (now)

- Keep current datasets/evals checked in.
- Exclude heavy training artifacts from git.
- Add roadmap and experiment conventions.

### Step 2: Introduce run manifests

- For each new run, add `experiments/<run_tag>.json`.
- Ensure every report in `eval/<version>/` is referenced from a manifest.

### Step 3: Organize notebooks

- Create `notebooks/` and move root notebooks there.
- Keep one canonical notebook per workflow (training+generation, eval-only).
- Add a short `notebooks/README.md` with execution order.

### Step 4: Standardize protocol docs

- Add `docs/EVAL_PROTOCOL.md` with locked rules for:
  - prompt contract wording,
  - seed policy,
  - per-arm parity constraints,
  - report metric definitions.

### Step 5: Add reproducibility helpers

- Add a simple task runner (`Makefile` or script) for common flows:
  - dataset export,
  - single-seed eval,
  - multi-seed aggregation,
  - summary diff between eval versions.

## Operational conventions

- Version eval results under `eval/vX_Y/`.
- Keep parser changes and eval protocol changes separate when feasible.
- Prefer explicit filenames that encode seed and variant (`seed3407`, `iron_prompt_fix_only`).
- Always include at least one aggregate report per run variant.

## Immediate next actions

1. Done: added manifest for `v2_6` (`experiments/v2_6_iron_eval_prompt_fix.json`).
2. Done: added canonical protocol doc (`docs/EVAL_PROTOCOL.md`).
3. Done: moved root notebooks into `notebooks/`.
4. Next: add a lightweight task runner for common export/eval/report flows.
