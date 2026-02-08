# Evaluation Protocol

This document defines the canonical evaluation protocol for Rust-vs-Iron model comparisons.

## Goals

- Preserve fair, symmetric comparison conditions between Rust and Iron arms.
- Make run-to-run differences attributable to known factors.
- Ensure outputs are reproducible and auditable.

## Protocol versioning

- Every evaluation run must declare a protocol version (for example `v2_6`).
- Any change to prompt contract wording, generation settings, or scoring rules must increment protocol version.

## Arm parity rules

1. **Task parity**: both arms use the same task ids and split definitions.
2. **Seed parity**: both arms run the same seed set.
3. **Sampling parity**: generation config must be identical across arms unless explicitly documented.
4. **No asymmetric hinting**: do not add one-off instruction tweaks to only one arm to rescue failures.

## Interface contract rules

1. Prompts must include an explicit required interface contract.
2. Contract language must match the arm language:
   - Rust arm uses Rust signatures (for example `pub fn ...`).
   - Iron arm uses Iron signatures (for example `function ... | takes ... | returns ...`).
3. Contract block must be generated deterministically from canonical solutions.

## Generation rules

- Log model base, adapter reference, seed, and generation settings.
- Prefer deterministic decoding for eval (`do_sample=false`, fixed token limits).
- Save raw per-task prediction JSONL files under `eval/<version>/`.

## Scoring rules

Evaluation uses:

1. `transform_ok` (Iron only path includes oxidation)
2. `compile_ok`
3. `test_ok`

Reported metrics:

- `transform_pass/total`
- `compile@1`
- `test@1`
- per-family compile/test rates
- failure taxonomy and failure phase counts

## Required artifacts per run

For each protocol version, commit:

- predictions: `eval/<version>/predictions_{arm}_seed{seed}.jsonl`
- per-seed reports: `eval/<version>/report_seed{seed}.json`
- aggregate report: `eval/<version>/report_aggregate_*.json`
- run manifest: `experiments/<run_tag>.json`

## Change attribution guidance

When investigating regressions or improvements:

1. Keep one axis changed at a time (parser, prompts, dataset, or model/runtime).
2. Use strict attribution reruns when possible (for example old Rust predictions + new Iron predictions).
3. Document conclusions and caveats in run notes.
