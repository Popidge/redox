//! Roundtrip status checks for extracted anyhow-like patterns.
//!
//! These tests are a corpus validation harness, not a "must pass everything" gate.
//! They give us a stable baseline while we expand language coverage.

use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RoundtripStatus {
    TranspileFailed,
    OxidizeFailed,
    RoundtripCompileFailed,
    RoundtripCompiled,
}

fn anyhow_cases() -> Vec<(&'static str, RoundtripStatus)> {
    vec![
        ("box_dyn_error.rs", RoundtripStatus::RoundtripCompiled),
        ("context_method.rs", RoundtripStatus::RoundtripCompileFailed),
        ("downcast.rs", RoundtripStatus::RoundtripCompileFailed),
        (
            "from_conversion.rs",
            RoundtripStatus::RoundtripCompileFailed,
        ),
        (
            "impl_error_trait.rs",
            RoundtripStatus::RoundtripCompileFailed,
        ),
        ("impl_iterator.rs", RoundtripStatus::RoundtripCompiled),
        ("impl_trait.rs", RoundtripStatus::RoundtripCompileFailed),
        ("lazy_context.rs", RoundtripStatus::RoundtripCompileFailed),
        ("mod.rs", RoundtripStatus::RoundtripCompileFailed),
        ("result_alias.rs", RoundtripStatus::RoundtripCompileFailed),
        (
            "source_chaining.rs",
            RoundtripStatus::RoundtripCompileFailed,
        ),
    ]
}

fn compile_rust_snippet(rust_source: &str, crate_name: &str) -> Result<(), String> {
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let source_path = temp_dir.path().join("roundtrip.rs");
    let output_path = temp_dir.path().join("roundtrip.rlib");

    std::fs::write(&source_path, rust_source)
        .map_err(|e| format!("Failed to write roundtrip source: {}", e))?;

    let compile = Command::new("rustc")
        .args([
            "--crate-name",
            crate_name,
            "--crate-type",
            "lib",
            "--edition",
            "2024",
            "-A",
            "dead_code",
            "-o",
        ])
        .arg(&output_path)
        .arg(&source_path)
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    if compile.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&compile.stderr).to_string())
    }
}

fn status_for_case(file_name: &str) -> RoundtripStatus {
    let path = format!(
        "{}/tests/corpus/anyhow/{}",
        env!("CARGO_MANIFEST_DIR"),
        file_name
    );

    let source =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("Failed to read {}: {}", path, e));

    let iron = match redox::transpile(&source) {
        Ok(iron) => iron,
        Err(_) => return RoundtripStatus::TranspileFailed,
    };

    let rust = match redox::oxidize(&iron) {
        Ok(rust) => rust,
        Err(_) => return RoundtripStatus::OxidizeFailed,
    };

    let crate_name = format!("anyhow_case_{}", file_name.replace(".rs", ""));
    match compile_rust_snippet(&rust, &crate_name) {
        Ok(()) => RoundtripStatus::RoundtripCompiled,
        Err(_) => RoundtripStatus::RoundtripCompileFailed,
    }
}

fn compile_file(path: &str, crate_name: &str) -> bool {
    let temp_dir = match tempfile::tempdir() {
        Ok(dir) => dir,
        Err(_) => return false,
    };
    let output_path = temp_dir.path().join("out.rlib");

    let compile = match Command::new("rustc")
        .args([
            "--crate-name",
            crate_name,
            "--crate-type",
            "lib",
            "--edition",
            "2024",
            "-A",
            "dead_code",
            "-o",
        ])
        .arg(&output_path)
        .arg(path)
        .output()
    {
        Ok(output) => output,
        Err(_) => return false,
    };

    compile.status.success()
}

#[test]
fn test_anyhow_corpus_status_baseline() {
    let mut mismatches = Vec::new();

    for (file_name, expected) in anyhow_cases() {
        let actual = status_for_case(file_name);
        if actual != expected {
            mismatches.push(format!(
                "{}: expected {:?}, got {:?}",
                file_name, expected, actual
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "Anyhow corpus status changed:\n{}\n\nIf this is intentional progress, update test expectations.",
        mismatches.join("\n")
    );
}

#[test]
fn test_anyhow_corpus_has_roundtrip_success() {
    let successes = anyhow_cases()
        .into_iter()
        .filter(|(file_name, _)| status_for_case(file_name) == RoundtripStatus::RoundtripCompiled)
        .count();

    assert!(
        successes >= 1,
        "Expected at least one anyhow corpus case to roundtrip-compile"
    );
}

#[test]
fn test_anyhow_compile_parity_with_original() {
    let mut mismatches = Vec::new();

    for (file_name, _) in anyhow_cases() {
        let source_path = format!(
            "{}/tests/corpus/anyhow/{}",
            env!("CARGO_MANIFEST_DIR"),
            file_name
        );

        let crate_suffix = file_name.replace(".rs", "").replace('-', "_");

        let original_ok = compile_file(&source_path, &format!("orig_{}", crate_suffix));
        let roundtrip_ok = status_for_case(file_name) == RoundtripStatus::RoundtripCompiled;

        if original_ok != roundtrip_ok {
            mismatches.push(format!(
                "{}: original compile={} roundtrip compile={}",
                file_name, original_ok, roundtrip_ok
            ));
        }
    }

    assert!(
        mismatches.is_empty(),
        "Anyhow compile parity drifted:\n{}",
        mismatches.join("\n")
    );
}
