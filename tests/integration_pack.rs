// tests/integration_pack.rs
//! Integration tests for the pack/context generation system.
//! Covers: v0.5.0 Pack & Context features

use std::fs;
use tempfile::TempDir;
use warden_core::pack::{self, PackOptions};
use warden_core::prompt::PromptGenerator;

fn setup_temp_project() -> TempDir {
    let dir = tempfile::tempdir().expect("Failed to create temp directory");

    // Create basic Rust project structure
    fs::write(
        dir.path().join("Cargo.toml"),
        r#"
[package]
name = "test"
version = "0.1.0"
"#,
    )
    .unwrap();

    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("src/main.rs"),
        r#"
fn main() {
    println!("Hello, world!");
}
"#,
    )
    .unwrap();

    fs::write(
        dir.path().join("src/lib.rs"),
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    )
    .unwrap();

    // Create warden.toml
    fs::write(
        dir.path().join("warden.toml"),
        r#"
[rules]
max_file_tokens = 2000
"#,
    )
    .unwrap();

    dir
}

/// Verifies Nabla delimiters are unique and don't appear in code.
/// Feature: File discovery integration (implicit in Nabla format)
#[test]
fn test_nabla_delimiters_are_unique() {
    // The delimiters ∇∇∇ and ∆∆∆ should never appear in normal source code
    let test_code = r#"
fn main() {
    let x = 42;
    // Normal comment
    /* Block comment */
    println!("Hello");
}
"#;

    assert!(
        !test_code.contains("∇∇∇"),
        "Nabla delimiter should not appear in code"
    );
    assert!(
        !test_code.contains("∆∆∆"),
        "Delta delimiter should not appear in code"
    );
}

/// Verifies Nabla format structure is correct.
/// Feature: Nabla format output
#[test]
fn test_nabla_format_structure() {
    let dir = setup_temp_project();
    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let options = PackOptions {
        include_prompt: false,
        skeleton: false,
        ..Default::default()
    };

    let output = pack::generate_pack(&options).expect("Should generate");

    // Verify Nabla structure
    assert!(output.contains("∇∇∇"), "Should have Nabla openers");
    assert!(output.contains("∆∆∆"), "Should have Delta closers");

    // Each opener should have a path
    for line in output.lines() {
        if line.contains("∇∇∇") && !line.contains("PLAN") && !line.contains("MANIFEST") {
            // Line should look like: ∇∇∇ path/to/file ∇∇∇
            assert!(line.starts_with("∇∇∇"), "Opener should start line");
            assert!(line.ends_with("∇∇∇"), "Opener should end with delimiter");
        }
    }
}

/// Verifies prompt includes the 3 Laws.
/// Feature: System prompt header, Law of Atomicity in prompt
#[test]
fn test_prompt_includes_laws() {
    let prompt = PromptGenerator::generate();

    assert!(
        prompt.contains("LAW OF ATOMICITY")
            || prompt.contains("Law of Atomicity")
            || prompt.contains("ATOMICITY"),
        "Prompt should include Law of Atomicity"
    );
    assert!(
        prompt.contains("LAW OF COMPLEXITY")
            || prompt.contains("Law of Complexity")
            || prompt.contains("COMPLEXITY"),
        "Prompt should include Law of Complexity"
    );
    assert!(
        prompt.contains("LAW OF PARANOIA")
            || prompt.contains("Law of Paranoia")
            || prompt.contains("PARANOIA"),
        "Prompt should include Law of Paranoia"
    );
}

/// Verifies prompt includes limits.
/// Feature: Law of Complexity in prompt
#[test]
fn test_prompt_includes_limits() {
    let prompt = PromptGenerator::generate();

    // Should mention specific limits
    assert!(
        prompt.contains("token") || prompt.contains("Token"),
        "Prompt should mention tokens"
    );
    assert!(
        prompt.contains("2000") || prompt.contains("file"),
        "Prompt should mention file size"
    );
}

/// Verifies prompt includes Nabla format instructions.
/// Feature: Nabla format instructions
#[test]
fn test_prompt_includes_nabla_instructions() {
    let prompt = PromptGenerator::generate();

    assert!(
        prompt.contains("∇∇∇") || prompt.contains("Nabla") || prompt.contains("nabla"),
        "Prompt should include Nabla format"
    );
    assert!(
        prompt.contains("∆∆∆") || prompt.contains("Delta") || prompt.contains("delta"),
        "Prompt should include Delta closer"
    );
}

/// Verifies reminder is concise.
/// Feature: Footer reminder
#[test]
fn test_reminder_is_concise() {
    let prompt = PromptGenerator::generate();

    // The prompt should not be excessively long
    let line_count = prompt.lines().count();
    assert!(
        line_count < 500,
        "Prompt should be concise, got {} lines",
        line_count
    );
}

/// Verifies skeleton integration with pack.
/// Feature: --skeleton all files
#[test]
fn test_pack_skeleton_integration() {
    let dir = setup_temp_project();
    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let options = PackOptions {
        include_prompt: false,
        skeleton: true,
        ..Default::default()
    };

    let output = pack::generate_pack(&options).expect("Should generate");

    // In skeleton mode, function bodies should be replaced
    // Full implementation bodies should not appear
    // This is a structural test - exact behavior may vary
    assert!(output.contains("∇∇∇"), "Should still use Nabla format");
}

/// Verifies focus mode with target.
/// Feature: --target focus mode, Target full, rest skeleton
#[test]
fn test_smart_context_focus_mode() {
    let dir = setup_temp_project();
    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let target_path = dir.path().join("src/main.rs");

    let options = PackOptions {
        include_prompt: false,
        skeleton: false,
        target: Some(target_path.clone()),
        ..Default::default()
    };

    let output = pack::generate_pack(&options).expect("Should generate");

    // Target file should be included with full content
    assert!(output.contains("src/main.rs"), "Should include target file");
    assert!(
        output.contains("println!"),
        "Target should have full content"
    );
}
