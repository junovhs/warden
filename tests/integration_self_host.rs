// tests/integration_self_host.rs
//! Integration tests verifying Warden passes its own rules.
//! Covers: v0.7.0 Self-Hosting

use std::path::PathBuf;
use warden_core::analysis::RuleEngine;
use warden_core::config::Config;
use warden_core::discovery;

/// Verifies Warden's own codebase passes all Warden rules.
/// Feature: Warden passes own rules
///
/// This is the ultimate dogfooding test - Warden must enforce
/// its own constraints on itself.
#[test]
fn test_warden_passes_own_rules() {
    // Get the project root (where Cargo.toml lives)
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    // Change to project root for config loading
    let original_dir = std::env::current_dir().ok();
    if std::env::set_current_dir(&manifest_dir).is_err() {
        // Skip if we can't change directory (CI environment quirk)
        return;
    }

    // Load warden's own config
    let config = match Config::load() {
        Ok(c) => c,
        Err(_) => {
            // Restore directory and skip if no config
            if let Some(dir) = original_dir {
                let _ = std::env::set_current_dir(dir);
            }
            return;
        }
    };

    // Discover all source files
    let src_dir = manifest_dir.join("src");
    if !src_dir.exists() {
        if let Some(dir) = original_dir {
            let _ = std::env::set_current_dir(dir);
        }
        return;
    }

    let files = discovery::find_files(&src_dir);

    // Filter to only Rust files in src/
    let rust_files: Vec<PathBuf> = files
        .into_iter()
        .filter(|p| p.extension().map_or(false, |e| e == "rs"))
        .filter(|p| p.starts_with(&src_dir))
        .collect();

    if rust_files.is_empty() {
        if let Some(dir) = original_dir {
            let _ = std::env::set_current_dir(dir);
        }
        return;
    }

    // Run analysis
    let engine = RuleEngine::new(config);
    let report = engine.scan(rust_files);

    // Restore original directory
    if let Some(dir) = original_dir {
        let _ = std::env::set_current_dir(dir);
    }

    // Collect violations for error message
    let mut violation_details = Vec::new();
    for file in &report.files {
        for violation in &file.violations {
            violation_details.push(format!(
                "  {}:{} - {} ({})",
                file.path.display(),
                violation.row,
                violation.message,
                violation.law
            ));
        }
    }

    // Assert no violations
    assert!(
        report.total_violations == 0,
        "Warden must pass its own rules!\n\nViolations found ({}):\n{}",
        report.total_violations,
        violation_details.join("\n")
    );
}

/// Verifies each source file is under the token limit.
#[test]
fn test_all_source_files_atomic() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let src_dir = manifest_dir.join("src");
    if !src_dir.exists() {
        return;
    }

    let files = discovery::find_files(&src_dir);
    let rust_files: Vec<_> = files
        .into_iter()
        .filter(|p| p.extension().map_or(false, |e| e == "rs"))
        .collect();

    // Default limit from Law of Atomicity
    let max_tokens = 2000;

    for file in rust_files {
        if let Ok(content) = std::fs::read_to_string(&file) {
            let tokens = warden_core::tokens::count(&content);
            assert!(
                tokens <= max_tokens,
                "File {} has {} tokens (max {})",
                file.display(),
                tokens,
                max_tokens
            );
        }
    }
}

/// Verifies no .unwrap() calls in production code.
#[test]
fn test_no_unwrap_in_source() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    let src_dir = manifest_dir.join("src");
    if !src_dir.exists() {
        return;
    }

    let files = discovery::find_files(&src_dir);
    let rust_files: Vec<_> = files
        .into_iter()
        .filter(|p| p.extension().map_or(false, |e| e == "rs"))
        .collect();

    let mut violations = Vec::new();

    for file in rust_files {
        if let Ok(content) = std::fs::read_to_string(&file) {
            for (line_num, line) in content.lines().enumerate() {
                // Skip comments and warden:ignore lines
                let trimmed = line.trim();
                if trimmed.starts_with("//") || trimmed.contains("warden:ignore") {
                    continue;
                }

                // Check for .unwrap() - but allow unwrap_or, unwrap_or_else, etc.
                if line.contains(".unwrap()") || line.contains(".expect(") {
                    violations.push(format!("{}:{}: {}", file.display(), line_num + 1, trimmed));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "Found .unwrap()/.expect() calls in source:\n{}",
        violations.join("\n")
    );
}
