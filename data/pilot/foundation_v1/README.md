# Foundation v1 Candidate Set

This is the first scaled candidate pool targeting ~200 stable, dependency-free tasks.

## Snapshot

- Total tasks: 220
- Split mix:
  - train: 160
  - val: 30
  - test: 30
- Families: 12
- deps: all empty
- unsafe: all false

## Files

- `manifest.v1_candidate.jsonl`
- `prompts/`
- `rust/`
- `report.v1_candidate.json` (latest validator output)

## Validation

```bash
python3 scripts/dataset_validator.py data/pilot/foundation_v1/manifest.v1_candidate.jsonl \
  --report-json data/pilot/foundation_v1/report.v1_candidate.json
```

Current status: all phase-1 entry gates pass.

## Regenerate

```bash
python3 scripts/generate_foundation_v1.py
```
