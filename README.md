# Redox

Redox is a Rust <-> Iron transpiler for testing a research hypothesis:

> A natural-language, lexical superset of an existing programming language may improve AI-generated code quality in smaller, compute-constrained models.

Iron is a natural-language-style superset of Rust written in alphanumeric English-like syntax. Redox provides deterministic conversion between supported Rust and Iron forms, plus evaluation tooling for Rust-vs-Iron model comparisons.

## Why this exists

Most code-generation work optimizes for token compactness. This project explores the opposite direction: whether a verbose, explicit, language-like syntax can better match LLM strengths and improve downstream functional correctness.

## Current status

- Exploratory phase complete with positive early signals
- Reproducible dataset + eval pipeline in-repo
- Protocol, roadmap, and run manifests documented
- Next phase: expand Rust feature coverage, scale datasets, and run larger-model experiments

See:

- `docs/EXPERIMENTAL_REPORT_PHASE1.md`
- `docs/EVAL_PROTOCOL.md`
- `docs/ROADMAP.md`

## Repository overview

- `src/`: Redox core (parser, tokenizer, AST, oxidation, mappings, CLI)
- `tests/`: roundtrip and corpus validation tests
- `scripts/`: dataset generation/export and evaluation tooling
- `data/`: pilot datasets and manifests
- `eval/`: prediction outputs and evaluation reports
- `experiments/`: run manifests and reproducibility metadata
- `docs/`: protocol, roadmap, and experimental reporting
- `notebooks/`: notebook workflows for training/generation/eval orchestration

## Quick start

### Build and test

```bash
cargo build --bin redox
cargo test
```

### CLI usage

```bash
# Rust -> Iron
target/debug/redox reduce input.rs

# Iron -> Rust
target/debug/redox oxidize input.iron

# Validate Iron source
target/debug/redox validate input.iron
```

### Evaluation tooling

```bash
python3 scripts/evaluate_predictions.py --help
python3 scripts/aggregate_eval_reports.py --help
```

## Research artifacts (TBC links)

- arXiv submission (TBC): <TBC>
- Transparency post on AI-assisted research workflow (TBC): <TBC>

## License

TBC
