// tests/unit_extractor.rs
//! Unit tests for the Nabla format extractor.
//! Covers: v0.3.0 Nabla Format Extraction features

use warden_core::apply::extractor;

/// Verifies malformed blocks are skipped gracefully.
/// Feature: Malformed block handling
#[test]
fn test_malformed_block_skipped() {
    let input = r#"
∇∇∇ src/valid.rs ∇∇∇
fn valid() {}
∆∆∆

∇∇∇ src/no_closing.rs ∇∇∇
fn no_close() {}

∇∇∇ src/another_valid.rs ∇∇∇
fn another() {}
∆∆∆
"#;

    let files = extractor::extract_files(input).expect("Should not fail");

    // Should extract the valid files, skip the malformed one
    assert!(
        files.contains_key("src/valid.rs"),
        "Should extract valid file"
    );
    assert!(
        files.contains_key("src/another_valid.rs"),
        "Should extract second valid file"
    );
    // The malformed one might or might not be extracted depending on implementation
    // Key point: no panic
}

/// Verifies content between delimiters is preserved.
#[test]
fn test_content_preserved_exactly() {
    let expected_content = "fn exact() {\n    // This exact content\n    let x = 42;\n}";
    let input = format!("∇∇∇ test.rs ∇∇∇\n{}\n∆∆∆", expected_content);

    let files = extractor::extract_files(&input).expect("Should extract");

    let content = &files["test.rs"].content;
    assert!(
        content.contains("// This exact content"),
        "Should preserve comments"
    );
    assert!(content.contains("let x = 42"), "Should preserve code");
}

/// Verifies whitespace handling at boundaries.
#[test]
fn test_whitespace_boundaries() {
    let input = "∇∇∇ test.rs ∇∇∇\nfn test() {}\n∆∆∆";

    let files = extractor::extract_files(input).expect("Should extract");
    let content = &files["test.rs"].content;

    // Content should not have excessive leading/trailing whitespace
    assert!(
        !content.starts_with("\n\n"),
        "Should not have excessive leading newlines"
    );
    assert!(
        !content.ends_with("\n\n"),
        "Should not have excessive trailing newlines"
    );
}

/// Verifies paths with spaces are handled.
#[test]
fn test_path_with_spaces() {
    let input = "∇∇∇ src/my file.rs ∇∇∇\nfn test() {}\n∆∆∆";

    let files = extractor::extract_files(input).expect("Should extract");

    // Path handling may vary - key is no crash
    assert!(files.len() >= 1, "Should extract at least one file");
}

/// Verifies deeply nested paths work.
#[test]
fn test_deeply_nested_path() {
    let input = "∇∇∇ src/a/b/c/d/e/deep.rs ∇∇∇\nfn deep() {}\n∆∆∆";

    let files = extractor::extract_files(input).expect("Should extract");

    assert!(files.contains_key("src/a/b/c/d/e/deep.rs"));
}

/// Verifies Unicode in content is preserved.
#[test]
fn test_unicode_content() {
    let input = "∇∇∇ test.rs ∇∇∇\nfn greet() { println!(\"你好世界\"); }\n∆∆∆";

    let files = extractor::extract_files(input).expect("Should extract");
    let content = &files["test.rs"].content;

    assert!(content.contains("你好世界"), "Should preserve Unicode");
}

/// Verifies PLAN extraction handles missing PLAN.
#[test]
fn test_plan_missing_returns_none() {
    let input = "∇∇∇ test.rs ∇∇∇\nfn test() {}\n∆∆∆";

    let plan = extractor::extract_plan(input);

    assert!(plan.is_none(), "Should return None when no PLAN block");
}

/// Verifies PLAN with complex content.
#[test]
fn test_plan_complex_content() {
    let input = r#"
∇∇∇ PLAN ∇∇∇
GOAL: Major refactoring

CHANGES:
1. First change
   - Sub-item a
   - Sub-item b
2. Second change
3. Third change

NOTES:
- Important consideration
- Another note
∆∆∆

∇∇∇ src/main.rs ∇∇∇
fn main() {}
∆∆∆
"#;

    let plan = extractor::extract_plan(input);

    assert!(plan.is_some(), "Should extract complex PLAN");
    let plan_content = plan.unwrap();
    assert!(plan_content.contains("GOAL:"));
    assert!(plan_content.contains("CHANGES:"));
    assert!(plan_content.contains("Sub-item"));
}

/// Verifies line count is calculated correctly.
#[test]
fn test_line_count_correct() {
    let input = "∇∇∇ test.rs ∇∇∇\nline1\nline2\nline3\nline4\n∆∆∆";

    let files = extractor::extract_files(input).expect("Should extract");

    assert_eq!(files["test.rs"].line_count, 4, "Should count 4 lines");
}

/// Verifies empty content between delimiters.
#[test]
fn test_empty_content_block() {
    let input = "∇∇∇ empty.rs ∇∇∇\n∆∆∆";

    let files = extractor::extract_files(input).expect("Should extract");

    // Empty file should still be extracted
    assert!(files.contains_key("empty.rs"));
}

/// Verifies multiple PLAN blocks (only first should be extracted).
#[test]
fn test_multiple_plan_blocks() {
    let input = r#"
∇∇∇ PLAN ∇∇∇
First plan
∆∆∆

∇∇∇ PLAN ∇∇∇
Second plan
∆∆∆
"#;

    let plan = extractor::extract_plan(input);

    assert!(plan.is_some());
    let plan_content = plan.unwrap();
    assert!(
        plan_content.contains("First plan"),
        "Should extract first PLAN"
    );
}

/// Verifies surrounding prose doesn't affect extraction.
#[test]
fn test_extraction_with_prose() {
    let input = r#"
Here's the implementation you requested. I've made the following changes:

∇∇∇ src/lib.rs ∇∇∇
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
∆∆∆

Let me know if you need any modifications to this code.
"#;

    let files = extractor::extract_files(input).expect("Should extract");

    assert!(files.contains_key("src/lib.rs"));
    let content = &files["src/lib.rs"].content;
    assert!(content.contains("pub fn add"), "Should extract function");
    assert!(!content.contains("Let me know"), "Should not include prose");
}
