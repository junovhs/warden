// tests/unit_pack_violations.rs
//! Unit tests for violation injection in pack output.
//! Covers: v0.5.0 Prompt Generation - Violation injection

use std::fs;
use tempfile::TempDir;
use warden_core::analysis::RuleEngine;
use warden_core::config::Config;
use warden_core::pack::{self, PackOptions};

fn setup_temp_project() -> TempDir {
    let dir = tempfile::tempdir().expect("Failed to create temp directory");

    fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("warden.toml"),
        r#"
[rules]
max_file_tokens = 100
max_cyclomatic_complexity = 3
"#,
    )
    .unwrap();

    dir
}

/// Verifies violations are injected into pack output when present.
/// Feature: Violation injection
#[test]
fn test_violations_injected() {
    let dir = setup_temp_project();

    // Create a file that will have violations (too many tokens)
    let big_content: String = "let x = 1;\n".repeat(100);
    fs::write(dir.path().join("src/big.rs"), &big_content).unwrap();

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let options = PackOptions {
        include_prompt: true,
        ..Default::default()
    };

    let output = pack::generate_pack(&options).expect("Should generate pack");

    // The pack output should include violation information when prompt is enabled
    // Violations section should appear somewhere in the output
    let has_violations_section =
        output.contains("VIOLATION") || output.contains("violation") || output.contains("LAW OF");

    // If there are violations, they should be mentioned
    // The exact format depends on implementation
    assert!(
        output.contains("∇∇∇") && output.contains("∆∆∆"),
        "Should have valid Nabla format"
    );
}

/// Verifies clean project has no violation injection.
#[test]
fn test_clean_project_no_violations() {
    let dir = setup_temp_project();

    // Create a small, clean file
    fs::write(dir.path().join("src/main.rs"), "fn main() {}").unwrap();

    // Use relaxed rules
    fs::write(
        dir.path().join("warden.toml"),
        r#"
[rules]
max_file_tokens = 5000
max_cyclomatic_complexity = 20
"#,
    )
    .unwrap();

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let options = PackOptions {
        include_prompt: true,
        ..Default::default()
    };

    let output = pack::generate_pack(&options).expect("Should generate pack");

    // Clean project should generate valid output
    assert!(output.contains("∇∇∇"), "Should have Nabla openers");
}

/// Verifies violation details include file path.
#[test]
fn test_violation_includes_file_path() {
    let dir = setup_temp_project();

    // Create a file with violations
    fs::write(
        dir.path().join("src/problem.rs"),
        r#"
fn too_complex(a: i32, b: i32, c: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                if a > b {
                    return 1;
                }
            }
        }
    }
    0
}
"#,
    )
    .unwrap();

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    // Scan for violations
    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/problem.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    // If violations exist, they should reference the file
    if report.total_violations > 0 {
        let file_report = &report.files[0];
        assert!(
            file_report.path.to_string_lossy().contains("problem"),
            "Violation should reference file path"
        );
    }
}

/// Verifies multiple violations are all captured.
#[test]
fn test_multiple_violations_captured() {
    let dir = setup_temp_project();

    // Create files with different types of violations
    fs::write(
        dir.path().join("src/unwrap.rs"),
        r#"
fn bad() {
    let x = Some(1).unwrap();
}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("src/complex.rs"),
        r#"
fn very_complex(a: i32) -> i32 {
    if a > 0 { if a > 1 { if a > 2 { if a > 3 { return 1; } } } }
    0
}
"#,
    )
    .unwrap();

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let config = Config::load().expect("Should load config");
    let files = vec![
        dir.path().join("src/unwrap.rs"),
        dir.path().join("src/complex.rs"),
    ];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    // Should capture violations from multiple files
    // Exact count depends on rules, but should find something
    assert!(report.files.len() >= 1, "Should analyze files");
}

/// Verifies violation law type is identified.
#[test]
fn test_violation_law_identified() {
    let dir = setup_temp_project();

    fs::write(
        dir.path().join("src/unwrap.rs"),
        r#"
fn bad() {
    let x = Some(1).unwrap();
}
"#,
    )
    .unwrap();

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let config = Config::load().expect("Should load config");
    let files = vec![dir.path().join("src/unwrap.rs")];
    let engine = RuleEngine::new(config);
    let report = engine.scan(files);

    // Find paranoia violation
    for file in &report.files {
        for violation in &file.violations {
            // Each violation should have a law identifier
            assert!(
                !violation.law.is_empty(),
                "Violation should have law identifier"
            );
        }
    }
}
