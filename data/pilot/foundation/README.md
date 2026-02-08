# Foundation Candidate Set (v0)

This directory contains a starter candidate pool for phase-1 small-model finetuning.

## Contents

- `manifest.v0_candidate.jsonl`: 18 task records
- `prompts/`: prompt text for each task
- `rust/`: canonical Rust references
- `report.v0_candidate.json`: latest validator report

## Coverage

The set focuses on dependency-free, safe Rust foundations:

- arithmetic and helper function composition
- vector basics (`new`, `with_capacity`, `push`, `pop`)
- option/result basics (`Some`, `map`, `Ok`, `unwrap_or`)
- closures
- type alias usage

## Validate

```bash
python3 scripts/dataset_validator.py data/pilot/foundation/manifest.v0_candidate.jsonl \
  --report-json data/pilot/foundation/report.v0_candidate.json
```

Current status: all phase-1 entry gates pass on this set.
