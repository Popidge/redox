//! Round-trip testing infrastructure
//!
//! Tests that Rust code can be transpiled to Iron and back to Rust
//! with semantic equivalence.

use std::fs;
use std::process::Command;

/// Test that a Rust file round-trips correctly through Iron
#[allow(dead_code)]
pub fn test_roundtrip(source_path: &str) -> Result<(), String> {
    let source = fs::read_to_string(source_path)
        .map_err(|e| format!("Failed to read {}: {}", source_path, e))?;

    // Step 1: Reduce to Iron
    let iron = redox::transpile(&source).map_err(|e| format!("Reduction failed: {}", e))?;

    // Step 2: Oxidize back to Rust
    let roundtrip = redox::oxidize(&iron).map_err(|e| format!("Oxidation failed: {}", e))?;

    // Step 3: Verify both compile
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let original_path = temp_dir.path().join("original.rs");
    let roundtrip_path = temp_dir.path().join("roundtrip.rs");

    fs::write(&original_path, &source).map_err(|e| format!("Failed to write original: {}", e))?;
    fs::write(&roundtrip_path, &roundtrip)
        .map_err(|e| format!("Failed to write roundtrip: {}", e))?;

    // Try to compile original
    let original_out = temp_dir.path().join("original.rlib");
    let original_compile = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "--edition",
            "2024",
            "-o",
            original_out.to_str().unwrap(),
            "-A",
            "dead_code", // Allow dead code warnings
            original_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run rustc on original: {}", e))?;

    if !original_compile.status.success() {
        return Err(format!(
            "Original code doesn't compile: {}",
            String::from_utf8_lossy(&original_compile.stderr)
        ));
    }

    // Try to compile roundtrip
    let roundtrip_out = temp_dir.path().join("roundtrip.rlib");
    let roundtrip_compile = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "--edition",
            "2024",
            "-o",
            roundtrip_out.to_str().unwrap(),
            "-A",
            "dead_code", // Allow dead code warnings
            roundtrip_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run rustc on roundtrip: {}", e))?;

    if !roundtrip_compile.status.success() {
        return Err(format!(
            "Round-trip code doesn't compile: {}\n\nOriginal:\n{}\n\nRound-trip:\n{}",
            String::from_utf8_lossy(&roundtrip_compile.stderr),
            source,
            roundtrip
        ));
    }

    // Step 4: Check semantic equivalence (simplified: compare text for now)
    // In a more sophisticated version, we'd compare ASTs
    if source.trim() != roundtrip.trim() {
        // Not identical, but let's see if it's semantically equivalent
        // For now, we allow minor whitespace differences
        let source_normalized: String = source.split_whitespace().collect();
        let roundtrip_normalized: String = roundtrip.split_whitespace().collect();

        if source_normalized != roundtrip_normalized {
            return Err(format!(
                "Round-trip code differs from original:\n\nOriginal:\n{}\n\nRound-trip:\n{}\n\nIron:\n{}",
                source, roundtrip, iron
            ));
        }
    }

    Ok(())
}

/// Test a specific function round-trips correctly
#[allow(dead_code)]
pub fn test_function_roundtrip(rust_code: &str) -> Result<(), String> {
    test_roundtrip_content(rust_code)
}

/// Test round-trip on code content directly
pub fn test_roundtrip_content(source: &str) -> Result<(), String> {
    // Step 1: Reduce to Iron
    let iron = redox::transpile(source).map_err(|e| format!("Reduction failed: {}", e))?;

    // Step 2: Oxidize back to Rust
    let roundtrip = redox::oxidize(&iron).map_err(|e| format!("Oxidation failed: {}", e))?;

    // Step 3: Verify compilation
    let temp_dir = tempfile::tempdir().map_err(|e| format!("Failed to create temp dir: {}", e))?;

    let roundtrip_path = temp_dir.path().join("roundtrip.rs");

    fs::write(&roundtrip_path, &roundtrip)
        .map_err(|e| format!("Failed to write roundtrip: {}", e))?;

    let roundtrip_out = temp_dir.path().join("roundtrip.rlib");
    let roundtrip_compile = Command::new("rustc")
        .args([
            "--crate-type",
            "lib",
            "--edition",
            "2024",
            "-o",
            roundtrip_out.to_str().unwrap(),
            "-A",
            "dead_code", // Allow dead code warnings
            roundtrip_path.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run rustc: {}", e))?;

    if !roundtrip_compile.status.success() {
        return Err(format!(
            "Round-trip code doesn't compile: {}\n\nRound-trip:\n{}",
            String::from_utf8_lossy(&roundtrip_compile.stderr),
            roundtrip
        ));
    }

    Ok(())
}
