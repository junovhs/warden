//! Integration tests for core Warden scanning functionality.
//!
//! Tests verify that the 3 Laws are properly enforced:
//! - Law of Atomicity (file token limits)
//! - Law of Complexity (cyclomatic complexity, nesting, arity)
//! - Law of Paranoia (banned calls like .unwrap())

use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use warden_core::analysis::RuleEngine;
use warden_core::config::{Config, RuleConfig};

fn setup_temp_file(dir: &TempDir, name: &str, content: &str) -> PathBuf {
    let path = dir.path().join(name);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).ok();
    }
    fs::write(&path, content).expect("Failed to write temp file");
    path
}

fn make_config() -> Config {
    let mut config = Config::new();
    config.rules = RuleConfig {
        max_file_tokens: 100,
        max_cyclomatic_complexity: 5,
        max_nesting_depth: 2,
        max_function_args: 3,
        max_function_words: 5,
        ignore_naming_on: vec![],
        ignore_tokens_on: vec![],
    };
    config
}

// =============================================================================
// LAW OF ATOMICITY - Token limits
// =============================================================================

#[test]
fn test_atomicity_clean_file_passes() {
    let dir = TempDir::new().unwrap();
    let path = setup_temp_file(&dir, "small.rs", "fn main() {}");
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    assert!(!report.has_errors(), "Small file should pass");
}

#[test]
fn test_atomicity_large_file_fails() {
    let dir = TempDir::new().unwrap();
    // Create content that exceeds 100 tokens
    let big_content = "fn main() { ".to_string() + &"let x = 1; ".repeat(50) + "}";
    let path = setup_temp_file(&dir, "big.rs", &big_content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    assert!(report.has_errors(), "Large file should fail atomicity");
    let violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.law == "LAW OF ATOMICITY")
        .collect();
    assert!(!violations.is_empty(), "Should have atomicity violation");
}

// =============================================================================
// LAW OF COMPLEXITY - Cyclomatic complexity
// =============================================================================

#[test]
fn test_complexity_simple_function_passes() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn simple() -> i32 {
    42
}
"#;
    let path = setup_temp_file(&dir, "simple.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let complexity_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.message.contains("Complexity"))
        .collect();
    assert!(complexity_violations.is_empty(), "Simple function should pass");
}

#[test]
fn test_complexity_branchy_function_fails() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn branchy(x: i32) -> i32 {
    if x > 0 {
        if x > 10 {
            1
        } else if x > 5 {
            2
        } else {
            3
        }
    } else if x < -10 {
        4
    } else if x < -5 {
        5
    } else {
        6
    }
}
"#;
    let path = setup_temp_file(&dir, "branchy.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let complexity_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.message.contains("Complexity"))
        .collect();
    assert!(!complexity_violations.is_empty(), "Branchy function should fail");
}

// =============================================================================
// LAW OF COMPLEXITY - Nesting depth
// =============================================================================

#[test]
fn test_nesting_shallow_passes() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn shallow(x: i32) {
    if x > 0 {
        println!("positive");
    }
}
"#;
    let path = setup_temp_file(&dir, "shallow.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let depth_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.message.contains("Nesting"))
        .collect();
    assert!(depth_violations.is_empty(), "Shallow nesting should pass");
}

#[test]
fn test_nesting_deep_fails() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn deep(x: i32) {
    if x > 0 {
        if x > 5 {
            if x > 10 {
                if x > 15 {
                    println!("very big");
                }
            }
        }
    }
}
"#;
    let path = setup_temp_file(&dir, "deep.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let depth_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.message.contains("Nesting") || v.message.contains("Depth"))
        .collect();
    assert!(!depth_violations.is_empty(), "Deep nesting should fail");
}

// =============================================================================
// LAW OF COMPLEXITY - Function arity
// =============================================================================

#[test]
fn test_arity_few_args_passes() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn few_args(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    let path = setup_temp_file(&dir, "few_args.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let arity_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.message.contains("Arity") || v.message.contains("arguments"))
        .collect();
    assert!(arity_violations.is_empty(), "Few args should pass");
}

#[test]
fn test_arity_many_args_fails() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn many_args(a: i32, b: i32, c: i32, d: i32, e: i32, f: i32) -> i32 {
    a + b + c + d + e + f
}
"#;
    let path = setup_temp_file(&dir, "many_args.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let arity_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.message.contains("Arity") || v.message.contains("arguments"))
        .collect();
    assert!(!arity_violations.is_empty(), "Many args should fail");
}

// =============================================================================
// LAW OF PARANOIA - Banned calls
// =============================================================================

#[test]
fn test_paranoia_no_unwrap_passes() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn safe(opt: Option<i32>) -> i32 {
    opt.unwrap_or(0)
}
"#;
    let path = setup_temp_file(&dir, "safe.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let paranoia_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.law == "LAW OF PARANOIA")
        .collect();
    assert!(paranoia_violations.is_empty(), "unwrap_or should pass");
}

#[test]
fn test_paranoia_unwrap_fails() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn unsafe_code(opt: Option<i32>) -> i32 {
    opt.unwrap()
}
"#;
    let path = setup_temp_file(&dir, "unsafe.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let paranoia_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.law == "LAW OF PARANOIA")
        .collect();
    assert!(!paranoia_violations.is_empty(), "unwrap() should fail");
}

#[test]
fn test_paranoia_expect_fails() {
    let dir = TempDir::new().unwrap();
    let content = r#"
fn also_unsafe(opt: Option<i32>) -> i32 {
    opt.expect("oops")
}
"#;
    let path = setup_temp_file(&dir, "also_unsafe.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    let paranoia_violations: Vec<_> = report.files[0].violations.iter()
        .filter(|v| v.law == "LAW OF PARANOIA")
        .collect();
    assert!(!paranoia_violations.is_empty(), "expect() should fail");
}

// =============================================================================
// WARDEN:IGNORE - Skip specific files
// =============================================================================

#[test]
fn test_warden_ignore_skips_file() {
    let dir = TempDir::new().unwrap();
    let content = r#"
// warden:ignore
fn ignored() {
    let x = Some(1).unwrap();
}
"#;
    let path = setup_temp_file(&dir, "ignored.rs", content);
    
    let config = make_config();
    let engine = RuleEngine::new(config);
    let report = engine.scan(vec![path]);
    
    assert!(report.files.is_empty(), "Ignored file should be skipped entirely");
}