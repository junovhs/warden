// tests/integration_core.rs
//! Integration tests for the 3 Laws enforcement.
//! Covers: v0.2.0 The 3 Laws features

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use warden_core::analysis::RuleEngine;
use warden_core::config::{Config, RuleConfig};

fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

fn create_test_config(rules: RuleConfig) -> Config {
    Config {
        rules,
        ..Default::default()
    }
}

fn default_strict_rules() -> RuleConfig {
    RuleConfig {
        max_file_tokens: 2000,
        max_cyclomatic_complexity: 5,
        max_nesting_depth: 3,
        max_function_args: 5,
        max_function_words: 5,
        ignore_tokens_on: vec![],
        ignore_naming_on: vec![],
    }
}

// =============================================================================
// LAW OF ATOMICITY
// =============================================================================

/// Verifies clean files pass atomicity check.
/// Feature: File token counting
#[test]
fn test_atomicity_clean_file_passes() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("small.rs");

    // Create a small, clean file
    fs::write(
        &file_path,
        r#"
fn main() {
    println!("Hello");
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    assert_eq!(
        report.total_violations, 0,
        "Small file should have no violations"
    );
}

/// Verifies large files trigger atomicity violation.
/// Feature: Token limit violation
#[test]
fn test_atomicity_large_file_fails() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("large.rs");

    // Create a file that exceeds token limit
    let mut content = String::new();
    for i in 0..500 {
        content.push_str(&format!("fn function_{i}() {{ let x = {i}; }}\n"));
    }

    fs::write(&file_path, &content).expect("Failed to write file");

    let mut rules = default_strict_rules();
    rules.max_file_tokens = 100; // Very low limit to trigger violation

    let config = create_test_config(rules);
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    assert!(
        report.total_violations > 0,
        "Large file should trigger atomicity violation"
    );

    let file_report = &report.files[0];
    let has_atomicity = file_report
        .violations
        .iter()
        .any(|v| v.law == "LAW OF ATOMICITY");
    assert!(has_atomicity, "Should have LAW OF ATOMICITY violation");
}

// =============================================================================
// LAW OF COMPLEXITY - CYCLOMATIC
// =============================================================================

/// Verifies simple functions pass complexity check.
/// Feature: Rust complexity query (if/match/for/while/&&/||)
#[test]
fn test_complexity_simple_function_passes() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("simple.rs");

    fs::write(
        &file_path,
        r#"
fn add(a: i32, b: i32) -> i32 {
    a + b
}

fn greet(name: &str) {
    println!("Hello, {}", name);
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let complexity_violations = report.files[0]
        .violations
        .iter()
        .filter(|v| v.message.contains("Complexity"))
        .count();

    assert_eq!(
        complexity_violations, 0,
        "Simple functions should have no complexity violations"
    );
}

/// Verifies complex functions trigger complexity violation.
/// Feature: Complexity violation detection
#[test]
fn test_complexity_branchy_function_fails() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("complex.rs");

    // Create a function with high cyclomatic complexity
    fs::write(
        &file_path,
        r#"
fn complex_logic(a: i32, b: i32, c: i32) -> i32 {
    if a > 0 {
        if b > 0 {
            if c > 0 {
                return a + b + c;
            } else if c < -10 {
                return a + b;
            } else {
                return a;
            }
        } else if b < -10 {
            return b;
        } else {
            return 0;
        }
    } else if a < -10 {
        match c {
            0 => return 0,
            1 => return 1,
            2 => return 2,
            _ => return -1,
        }
    }
    -1
}
"#,
    )
    .expect("Failed to write file");

    let mut rules = default_strict_rules();
    rules.max_cyclomatic_complexity = 3; // Low limit to trigger

    let config = create_test_config(rules);
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let has_complexity_violation = report.files[0]
        .violations
        .iter()
        .any(|v| v.message.contains("Complexity") || v.law == "LAW OF COMPLEXITY");

    assert!(
        has_complexity_violation,
        "Complex function should trigger complexity violation"
    );
}

// =============================================================================
// LAW OF COMPLEXITY - NESTING DEPTH
// =============================================================================

/// Verifies shallow nesting passes.
/// Feature: Depth calculation (block/body traversal)
#[test]
fn test_nesting_shallow_passes() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("shallow.rs");

    fs::write(
        &file_path,
        r#"
fn shallow() {
    if true {
        println!("level 1");
    }
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let nesting_violations = report.files[0]
        .violations
        .iter()
        .filter(|v| v.message.contains("Nesting") || v.message.contains("depth"))
        .count();

    assert_eq!(nesting_violations, 0, "Shallow nesting should pass");
}

/// Verifies deep nesting triggers violation.
/// Feature: Deep nesting violation
#[test]
fn test_nesting_deep_fails() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("deep.rs");

    fs::write(
        &file_path,
        r#"
fn deeply_nested() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        println!("way too deep");
                    }
                }
            }
        }
    }
}
"#,
    )
    .expect("Failed to write file");

    let mut rules = default_strict_rules();
    rules.max_nesting_depth = 2; // Low limit to trigger

    let config = create_test_config(rules);
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let has_nesting_violation = report.files[0]
        .violations
        .iter()
        .any(|v| v.message.contains("Nesting") || v.message.contains("depth"));

    assert!(
        has_nesting_violation,
        "Deep nesting should trigger violation"
    );
}

// =============================================================================
// LAW OF COMPLEXITY - ARITY
// =============================================================================

/// Verifies functions with few arguments pass.
/// Feature: Parameter counting
#[test]
fn test_arity_few_args_passes() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("few_args.rs");

    fs::write(
        &file_path,
        r#"
fn two_args(a: i32, b: i32) -> i32 {
    a + b
}

fn three_args(x: f64, y: f64, z: f64) -> f64 {
    x + y + z
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let arity_violations = report.files[0]
        .violations
        .iter()
        .filter(|v| v.message.contains("Arity") || v.message.contains("argument"))
        .count();

    assert_eq!(arity_violations, 0, "Few arguments should pass");
}

/// Verifies functions with many arguments trigger violation.
/// Feature: High arity violation
#[test]
fn test_arity_many_args_fails() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("many_args.rs");

    fs::write(
        &file_path,
        r#"
fn too_many_args(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32, g: i32, h: i32) -> i32 {
    a + b + c + d + e + f + g + h
}
"#,
    )
    .expect("Failed to write file");

    let mut rules = default_strict_rules();
    rules.max_function_args = 5;

    let config = create_test_config(rules);
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let has_arity_violation = report.files[0]
        .violations
        .iter()
        .any(|v| v.message.contains("Arity") || v.message.contains("argument"));

    assert!(
        has_arity_violation,
        "Many arguments should trigger arity violation"
    );
}

// =============================================================================
// LAW OF PARANOIA
// =============================================================================

/// Verifies .unwrap() calls trigger violation.
/// Feature: Banned call query (.unwrap/.expect)
#[test]
fn test_paranoia_unwrap_fails() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("unwrap.rs");

    fs::write(
        &file_path,
        r#"
fn bad_code() {
    let result: Result<i32, &str> = Ok(42);
    let value = result.unwrap();
    println!("{}", value);
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let has_paranoia_violation = report.files[0]
        .violations
        .iter()
        .any(|v| v.message.contains("unwrap") || v.law == "LAW OF PARANOIA");

    assert!(
        has_paranoia_violation,
        ".unwrap() should trigger LAW OF PARANOIA violation"
    );
}

/// Verifies .expect() calls trigger violation.
/// Feature: .expect() detection
#[test]
fn test_paranoia_expect_fails() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("expect.rs");

    fs::write(
        &file_path,
        r#"
fn bad_code() {
    let result: Result<i32, &str> = Ok(42);
    let value = result.expect("This should not panic");
    println!("{}", value);
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let has_paranoia_violation = report.files[0]
        .violations
        .iter()
        .any(|v| v.message.contains("expect") || v.law == "LAW OF PARANOIA");

    assert!(
        has_paranoia_violation,
        ".expect() should trigger LAW OF PARANOIA violation"
    );
}

/// Verifies safe alternatives are allowed.
/// Feature: Safe alternatives allowed (.unwrap_or)
#[test]
fn test_paranoia_no_unwrap_passes() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("safe.rs");

    fs::write(
        &file_path,
        r#"
fn safe_code() -> i32 {
    let result: Result<i32, &str> = Ok(42);
    let value = result.unwrap_or(0);
    let opt: Option<i32> = Some(1);
    let v2 = opt.unwrap_or_default();
    value + v2
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    let paranoia_violations = report.files[0]
        .violations
        .iter()
        .filter(|v| v.law == "LAW OF PARANOIA")
        .count();

    assert_eq!(
        paranoia_violations, 0,
        "Safe alternatives should not trigger violations"
    );
}

// =============================================================================
// FILE IGNORES
// =============================================================================

/// Verifies warden:ignore comment skips file (C-style).
/// Feature: warden:ignore (C-style //)
#[test]
fn test_warden_ignore_skips_file() {
    let dir = setup_temp_dir();
    let file_path = dir.path().join("ignored.rs");

    fs::write(
        &file_path,
        r#"
// warden:ignore
fn has_unwrap() {
    let x = Some(1).unwrap();
}
"#,
    )
    .expect("Failed to write file");

    let config = create_test_config(default_strict_rules());
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![file_path]);

    // File should be completely skipped
    assert_eq!(
        report.files.len(),
        0,
        "Ignored file should not appear in report"
    );
}
