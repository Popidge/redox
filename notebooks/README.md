# Notebooks

This directory contains notebook workflows used for model training, prediction generation, and evaluation orchestration.

## Current notebooks

- `Qwen3_(4B)_Instruct.ipynb`: baseline model notebook.
- `Qwen3_(4B)_Instruct_Redox.ipynb`: split-arm Redox training notebook.
- `Qwen3_(4B)_Instruct_Redox_AllInOne.ipynb`: combined train/generate/eval orchestration.

## Recommended usage

1. Prefer `Qwen3_(4B)_Instruct_Redox_AllInOne.ipynb` for reproducible end-to-end runs.
2. Record the resulting paths in an experiment manifest under `experiments/`.
3. Keep eval protocol parity with `docs/EVAL_PROTOCOL.md`.

## Hygiene

- Keep notebooks in this directory (not repo root).
- Do not commit local checkpoint paths from transient runtimes.
- Save generated predictions/reports under versioned `eval/<version>/` folders.
