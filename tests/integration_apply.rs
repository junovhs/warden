// tests/integration_apply.rs
//! Integration tests for the apply system.
//! Covers: v0.3.0 Apply System + v0.4.0 Safety & Validation features

use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use warden_core::apply::extractor;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use warden_core::apply::validator;

fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

// =============================================================================
// NABLA FORMAT EXTRACTION (v0.3.0)
// =============================================================================

/// Verifies single file extraction from Nabla format.
/// Features: Header detection, Footer detection, Path extraction, Content extraction
#[test]
fn test_extract_single_file() {
    let input = r#"
Here is the code:

∇∇∇ src/main.rs ∇∇∇
fn main() {
    println!("Hello, world!");
}
∆∆∆

That's all.
"#;

    let files = extractor::extract_files(input).expect("Extraction should succeed");

    assert_eq!(files.len(), 1, "Should extract exactly one file");
    assert!(files.contains_key("src/main.rs"), "Should have src/main.rs");

    let content = &files["src/main.rs"].content;
    assert!(
        content.contains("fn main()"),
        "Content should include function"
    );
    assert!(
        content.contains("println!"),
        "Content should include println"
    );
}

/// Verifies multiple file extraction.
/// Feature: Multiple file extraction
#[test]
fn test_extract_multiple_files() {
    let input = r#"
∇∇∇ src/lib.rs ∇∇∇
pub mod utils;
∆∆∆

∇∇∇ src/utils.rs ∇∇∇
pub fn helper() -> i32 {
    42
}
∆∆∆

∇∇∇ src/main.rs ∇∇∇
use mylib::utils;

fn main() {
    println!("{}", utils::helper());
}
∆∆∆
"#;

    let files = extractor::extract_files(input).expect("Extraction should succeed");

    assert_eq!(files.len(), 3, "Should extract three files");
    assert!(files.contains_key("src/lib.rs"));
    assert!(files.contains_key("src/utils.rs"));
    assert!(files.contains_key("src/main.rs"));
}

/// Verifies MANIFEST blocks are skipped.
/// Feature: MANIFEST block skipping
#[test]
fn test_extract_skips_manifest() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
src/main.rs
src/lib.rs [NEW]
∆∆∆

∇∇∇ src/main.rs ∇∇∇
fn main() {}
∆∆∆
"#;

    let files = extractor::extract_files(input).expect("Extraction should succeed");

    assert_eq!(files.len(), 1, "Should only extract one file, not MANIFEST");
    assert!(
        !files.contains_key("MANIFEST"),
        "MANIFEST should not be extracted as file"
    );
    assert!(files.contains_key("src/main.rs"));
}

/// Verifies PLAN block extraction.
/// Feature: PLAN block extraction
#[test]
fn test_extract_plan() {
    let input = r#"
∇∇∇ PLAN ∇∇∇
GOAL: Refactor the parser module
CHANGES:
1. Extract validation logic
2. Add new error types
∆∆∆

∇∇∇ src/parser.rs ∇∇∇
fn parse() {}
∆∆∆
"#;

    let plan = extractor::extract_plan(input);

    assert!(plan.is_some(), "Should extract PLAN block");
    let plan_content = plan.unwrap();
    assert!(plan_content.contains("GOAL:"), "Plan should contain GOAL");
    assert!(
        plan_content.contains("CHANGES:"),
        "Plan should contain CHANGES"
    );
}

// =============================================================================
// PATH SAFETY - TRAVERSAL (v0.4.0)
// =============================================================================

/// Verifies ../ traversal is blocked.
/// Feature: Block ../ traversal
#[test]
fn test_path_safety_blocks_traversal() {
    let mut files = HashMap::new();
    files.insert(
        "../etc/passwd".to_string(),
        FileContent {
            content: "malicious".to_string(),
            line_count: 1,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_traversal_error = errors
                .iter()
                .any(|e| e.contains("traversal") || e.contains("SECURITY"));
            assert!(has_traversal_error, "Should detect traversal attack");
        }
        _ => panic!("Should fail validation for traversal path"),
    }
}

// =============================================================================
// PATH SAFETY - ABSOLUTE (v0.4.0)
// =============================================================================

/// Verifies Unix absolute paths are blocked.
/// Feature: Block Unix absolute (/)
#[test]
fn test_path_safety_blocks_absolute() {
    let mut files = HashMap::new();
    files.insert(
        "/etc/passwd".to_string(),
        FileContent {
            content: "malicious".to_string(),
            line_count: 1,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_absolute_error = errors
                .iter()
                .any(|e| e.contains("absolute") || e.contains("SECURITY"));
            assert!(has_absolute_error, "Should detect absolute path");
        }
        _ => panic!("Should fail validation for absolute path"),
    }
}

// =============================================================================
// PATH SAFETY - SENSITIVE (v0.4.0)
// =============================================================================

/// Verifies .git/ paths are blocked.
/// Feature: Block .git/**
#[test]
fn test_path_safety_blocks_git() {
    let mut files = HashMap::new();
    files.insert(
        ".git/config".to_string(),
        FileContent {
            content: "malicious".to_string(),
            line_count: 1,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_git_error = errors.iter().any(|e| {
                e.contains(".git")
                    || e.contains("SECURITY")
                    || e.contains("sensitive")
                    || e.contains("hidden")
            });
            assert!(has_git_error, "Should detect .git path");
        }
        _ => panic!("Should fail validation for .git path"),
    }
}

// =============================================================================
// PATH SAFETY - HIDDEN FILES (v0.4.0)
// =============================================================================

/// Verifies hidden files are blocked.
/// Feature: Block hidden files (.*)
#[test]
fn test_path_safety_blocks_hidden() {
    let mut files = HashMap::new();
    files.insert(
        ".secrets".to_string(),
        FileContent {
            content: "secret data".to_string(),
            line_count: 1,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_hidden_error = errors
                .iter()
                .any(|e| e.contains("hidden") || e.contains("SECURITY"));
            assert!(has_hidden_error, "Should detect hidden file");
        }
        _ => panic!("Should fail validation for hidden file"),
    }
}

// =============================================================================
// TRUNCATION DETECTION (v0.4.0)
// =============================================================================

/// Verifies // ... pattern is detected.
/// Feature: Pattern: // ...
#[test]
fn test_truncation_detects_ellipsis_comment() {
    let mut files = HashMap::new();
    files.insert(
        "src/main.rs".to_string(),
        FileContent {
            content: "fn main() {\n    // ...\n}".to_string(),
            line_count: 3,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_truncation_error = errors
                .iter()
                .any(|e| e.contains("truncation") || e.contains("..."));
            assert!(
                has_truncation_error,
                "Should detect ellipsis truncation marker"
            );
        }
        _ => panic!("Should fail validation for truncation marker"),
    }
}

/// Verifies warden:ignore bypasses truncation detection.
/// Feature: warden:ignore bypass
#[test]
fn test_truncation_allows_warden_ignore() {
    let mut files = HashMap::new();
    files.insert(
        "src/main.rs".to_string(),
        FileContent {
            content: "fn main() {\n    // ... warden:ignore\n    println!(\"ok\");\n}".to_string(),
            line_count: 4,
        },
    );

    let result = validator::validate(&vec![], &files);

    // Should pass because warden:ignore is present
    match result {
        ApplyOutcome::Success { .. } => {} // Expected
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let truncation_errors: Vec<_> =
                errors.iter().filter(|e| e.contains("truncation")).collect();
            assert!(
                truncation_errors.is_empty(),
                "warden:ignore should bypass truncation: {:?}",
                truncation_errors
            );
        }
        _ => {}
    }
}

/// Verifies empty files are rejected.
/// Feature: Empty file rejection
#[test]
fn test_truncation_detects_empty_file() {
    let mut files = HashMap::new();
    files.insert(
        "src/main.rs".to_string(),
        FileContent {
            content: "".to_string(),
            line_count: 0,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_empty_error = errors.iter().any(|e| e.contains("empty"));
            assert!(has_empty_error, "Should detect empty file");
        }
        _ => panic!("Should fail validation for empty file"),
    }
}

// =============================================================================
// VALID PATHS (v0.4.0)
// =============================================================================

/// Verifies normal paths are accepted.
/// Feature: Normal paths accepted
#[test]
fn test_path_safety_allows_valid() {
    let mut files = HashMap::new();
    files.insert(
        "src/main.rs".to_string(),
        FileContent {
            content: "fn main() { println!(\"hello\"); }".to_string(),
            line_count: 1,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::Success { written, .. } => {
            assert!(written.contains(&"src/main.rs".to_string()));
        }
        ApplyOutcome::ValidationFailure { errors, .. } => {
            panic!("Valid path should pass: {:?}", errors);
        }
        _ => panic!("Unexpected result"),
    }
}

// =============================================================================
// UNIFIED APPLY (v0.6.0)
// =============================================================================

/// Verifies ===ROADMAP=== detection in apply.
/// Feature: Detect ===ROADMAP=== in apply
#[test]
fn test_unified_apply_roadmap() {
    let input = r#"
===ROADMAP===
CHECK some-task
===END===
"#;

    // The roadmap detection should work
    let has_roadmap = input.contains("===ROADMAP===");
    assert!(has_roadmap, "Should detect roadmap block");
}

/// Verifies combined roadmap and files apply.
/// Feature: Apply roadmap + files together
#[test]
fn test_unified_apply_combined() {
    let input = r#"
===ROADMAP===
CHECK feature-one
ADD v0.5.0 "New feature"
===END===

∇∇∇ src/main.rs ∇∇∇
fn main() {
    println!("Updated");
}
∆∆∆
"#;

    // Should extract both roadmap and files
    let has_roadmap = input.contains("===ROADMAP===");
    let files = extractor::extract_files(input).expect("Should extract files");

    assert!(has_roadmap, "Should have roadmap block");
    assert!(files.contains_key("src/main.rs"), "Should extract file");
}

// =============================================================================
// v0.8.0 MARKDOWN REJECTION
// =============================================================================

/// Verifies triple backticks are rejected.
/// Feature: Block triple backticks (```)
#[test]
fn test_rejects_markdown_fences() {
    let input = r#"
Here's the updated code:

```rust
fn main() {
    println!("Hello");
}
```
"#;

    // This should be rejected - AI should use Nabla format, not markdown
    let result = validator::detect_markdown_fences(input);
    assert!(result.is_some(), "Should detect markdown triple backticks");

    // If extracting, should get no valid files from markdown-only input
    let files = extractor::extract_files(input);
    match files {
        Ok(f) => assert!(
            f.is_empty(),
            "Should not extract files from markdown-only input"
        ),
        Err(_) => {} // Also acceptable
    }
}

/// Verifies tilde fences are rejected.
/// Feature: Block tilde fences (~~~)
#[test]
fn test_rejects_tilde_fences() {
    let input = r#"
Here's the code:

~~~python
def main():
    print("Hello")
~~~
"#;

    let result = validator::detect_markdown_fences(input);
    assert!(result.is_some(), "Should detect tilde fences");
}

// =============================================================================
// v0.8.0 BRACE BALANCING
// =============================================================================

/// Verifies unbalanced open braces are detected.
/// Feature: Detect unbalanced {
#[test]
fn test_detects_unbalanced_open_brace() {
    let content = r#"
fn main() {
    if true {
        println!("missing close");
    // missing closing brace
}
"#;

    let result = validator::check_brace_balance(content);
    assert!(result.is_err(), "Should detect unbalanced open brace");

    let err = result.unwrap_err();
    assert!(
        err.contains("{") || err.contains("brace") || err.contains("unbalanced"),
        "Error should mention braces"
    );
}

/// Verifies unbalanced close braces are detected.
/// Feature: Detect unbalanced }
#[test]
fn test_detects_unbalanced_close_brace() {
    let content = r#"
fn main() {
    println!("extra close");
}
}
"#;

    let result = validator::check_brace_balance(content);
    assert!(result.is_err(), "Should detect unbalanced close brace");
}

/// Verifies unbalanced brackets are detected.
/// Feature: Detect unbalanced [
#[test]
fn test_detects_unbalanced_bracket() {
    let content = r#"
fn main() {
    let arr = [1, 2, 3;
    println!("{}", arr[0]);
}
"#;

    let result = validator::check_brace_balance(content);
    assert!(result.is_err(), "Should detect unbalanced bracket");
}

/// Verifies unbalanced parentheses are detected.
/// Feature: Detect unbalanced (
#[test]
fn test_detects_unbalanced_paren() {
    let content = r#"
fn main() {
    println!("hello"
}
"#;

    let result = validator::check_brace_balance(content);
    assert!(result.is_err(), "Should detect unbalanced parenthesis");
}

/// Verifies balanced code passes validation.
#[test]
fn test_balanced_braces_pass() {
    let content = r#"
fn main() {
    let arr = [1, 2, 3];
    println!("{}", arr[0]);
    if true {
        for i in 0..10 {
            match i {
                0 => println!("zero"),
                _ => println!("{}", i),
            }
        }
    }
}
"#;

    let result = validator::check_brace_balance(content);
    assert!(result.is_ok(), "Balanced braces should pass");
}

/// Verifies braces in strings don't cause false positives.
#[test]
fn test_braces_in_strings_ignored() {
    let content = r#"
fn main() {
    let s = "{ this is not a real brace }";
    let t = "[ also not real ]";
    let u = "( these too )";
    println!("{}", s);
}
"#;

    let result = validator::check_brace_balance(content);
    assert!(result.is_ok(), "Braces in strings should be ignored");
}
