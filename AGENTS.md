# AGENTS.md - Redox Agent Guide

## Project Overview

Redox is a Rust-to-Iron transpiler with reverse oxidation support.
Iron is a lexically expanded Rust-like language designed for reliable tokenization.
This repository is a Cargo project on Rust edition `2024`.

Primary goals for agents:
- preserve deterministic transpilation/oxidation behavior,
- maintain compile correctness,
- keep emitted syntax stable and readable,
- add or update tests when behavior changes.

## Repository Shape

- `src/lib.rs`: public API (`transpile`, `oxidize`, validation, `TranspileError`)
- `src/main.rs`: CLI commands (`reduce`, `validate`, `oxidize`)
- `src/parser.rs`: Rust AST (`syn`) -> Iron emitter flow
- `src/emitter.rs`: Iron output formatting utilities
- `src/iron_tokenizer.rs`: tokenization of Iron source
- `src/iron_parser.rs`: Iron token stream -> Iron AST
- `src/iron_ast.rs`: AST types for Iron language
- `src/oxidation.rs`: Iron AST -> Rust source generation
- `src/mappings.rs`: Rust/Iron mapping helpers
- `src/keywords.rs`: keyword handling and identifier sanitization
- `tests/roundtrip_tests.rs`: integration roundtrip coverage
- `tests/roundtrip/mod.rs`: roundtrip helper harness and compile checks

## Build, Lint, and Test Commands

Use these exact commands unless you have a clear reason to narrow scope.

```bash
# Build
cargo build
cargo build --release
cargo build --bin redox

# Format
cargo fmt
cargo fmt -- --check

# Lint
cargo clippy
cargo clippy --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings

# Full test suite
cargo test
cargo test --lib
cargo test --test roundtrip_tests
cargo test --doc
```

### Running a single test (important)

```bash
# Single unit test in library
cargo test --lib test_transpile_simple_function

# Single integration test by test binary + test name
cargo test --test roundtrip_tests test_vec_push

# Show captured output for one test
cargo test --test roundtrip_tests test_vec_push -- --nocapture

# List test names when unsure
cargo test --lib -- --list
cargo test --test roundtrip_tests -- --list
```

### Focused development loops

```bash
# Fast check without linking binaries
cargo check

# Exercise CLI manually
cargo run --bin redox -- --help
cargo run --bin redox -- reduce input.rs
cargo run --bin redox -- oxidize input.iron
```

## Code Style and Conventions

Follow existing code patterns first; this section describes observed project norms.

### Formatting

- Use `rustfmt` defaults (4-space indentation; trailing commas where rustfmt expects).
- Prefer <=100 columns when practical, even if rustfmt may wrap differently.
- Run `cargo fmt` after edits.
- Keep module declarations and top-level `use` blocks near file start.

### Imports

- Group imports in this order:
  1) `std` imports,
  2) external crates,
  3) `crate::` local modules.
- Prefer explicit imports over wildcard imports.
- Keep `use` lists minimal and remove unused imports.

### Naming

- Modules/files: `snake_case`.
- Functions/methods/variables: `snake_case`.
- Structs/enums/traits: `PascalCase`.
- Enum variants: `PascalCase`.
- Constants/statics: `SCREAMING_SNAKE_CASE`.
- Test names: descriptive `test_*` format.

### Types and signatures

- Keep explicit types on public function signatures.
- Use `&str` for borrowed input text and `String` for owned results.
- Return `Result<T, TranspileError>` in library fallible paths.
- Prefer concrete types unless generics materially improve API clarity.

### Error handling

- In library code, propagate with `?` and map errors with contextual `map_err`.
- Keep user-facing error messages specific and actionable.
- Avoid `unwrap`/`expect` in production code paths.
- In tests, `expect` is acceptable with meaningful failure messages.
- CLI wrappers may return `Result<(), Box<dyn std::error::Error>>` for ergonomics.

### Control flow and architecture hygiene

- Prefer `match` for exhaustive branching; use `if let` for single-pattern paths.
- Avoid deeply nested conditionals; extract helpers when branches grow.
- Keep parser/tokenizer/AST/emitter/oxidizer responsibilities separated.
- Do not introduce cross-layer shortcuts that bypass AST or mapping logic.

### Documentation

- Use `//!` for module docs where the module has non-trivial behavior.
- Use `///` docs for public APIs and key types.
- For public functions, document arguments and return/error behavior.
- Keep docs accurate to current behavior and syntax.

## Testing Expectations

- Unit tests live near code in `#[cfg(test)] mod tests`.
- Integration and fidelity tests live under `tests/`.
- For transpilation features, prefer roundtrip tests that also compile output.
- When fixing parser/emitter bugs, add a regression test that fails pre-fix.
- Keep assertions precise (check structure/tokens, not only `is_ok`).

## Agent Instruction Sources (Cursor/Copilot)

Checked in this repository:
- `.cursorrules`
- `.cursor/rules/`
- `.github/copilot-instructions.md`

Current status: none of these files are present.
If any are later added, treat them as additional mandatory guidance and update this file.

## Recommended Change Workflow for Agents

1. Read relevant modules fully before editing parser/tokenizer/AST interactions.
2. Make minimal, behavior-focused changes.
3. Run `cargo fmt`.
4. Run targeted tests first (single test), then broader suite.
5. Run `cargo clippy --all-targets --all-features`.
6. Summarize behavioral impact and test evidence in PR/commit notes.
