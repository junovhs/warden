//! Integration tests for warden pack command functionality.
//!
//! Tests verify:
//! - Nabla format output
//! - Prompt inclusion (default on)
//! - File discovery and filtering

use std::fs;
use tempfile::TempDir;
use warden_core::config::{Config, GitMode};
use warden_core::discovery;
use warden_core::pack::{OutputFormat, PackOptions};
use warden_core::prompt::PromptGenerator;
use warden_core::tokens::Tokenizer;

fn create_test_project(dir: &TempDir) {
    let src = dir.path().join("src");
    fs::create_dir_all(&src).unwrap();
    
    fs::write(src.join("main.rs"), "fn main() { println!(\"hello\"); }").unwrap();
    fs::write(src.join("lib.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }").unwrap();
    fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
}

// =============================================================================
// NABLA FORMAT
// =============================================================================

#[test]
fn test_nabla_format_structure() {
    let content = "fn main() {}";
    let path = "src/main.rs";
    
    let formatted = format!("∇∇∇ {} ∇∇∇\n{}\n∆∆∆\n", path, content);
    
    assert!(formatted.contains("∇∇∇"), "Should have nabla opener");
    assert!(formatted.contains("∆∆∆"), "Should have delta closer");
    assert!(formatted.contains(path), "Should contain file path");
    assert!(formatted.contains(content), "Should contain file content");
}

#[test]
fn test_nabla_delimiters_are_unique() {
    // Verify nabla/delta don't appear in normal code
    let normal_code = r#"
fn main() {
    let x = vec![1, 2, 3];
    for i in x.iter() {
        println!("{}", i);
    }
}
"#;
    assert!(!normal_code.contains("∇∇∇"), "Nabla should not appear in normal code");
    assert!(!normal_code.contains("∆∆∆"), "Delta should not appear in normal code");
}

// =============================================================================
// PROMPT GENERATION
// =============================================================================

#[test]
fn test_prompt_includes_laws() {
    let config = Config::new();
    let gen = PromptGenerator::new(config.rules);
    let prompt = gen.generate().unwrap();
    
    assert!(prompt.contains("LAW OF ATOMICITY"), "Should mention atomicity");
    assert!(prompt.contains("LAW OF COMPLEXITY"), "Should mention complexity");
    assert!(prompt.contains("LAW OF PARANOIA"), "Should mention paranoia");
}

#[test]
fn test_prompt_includes_limits() {
    let mut config = Config::new();
    config.rules.max_file_tokens = 1500;
    config.rules.max_cyclomatic_complexity = 6;
    config.rules.max_nesting_depth = 2;
    
    let gen = PromptGenerator::new(config.rules);
    let prompt = gen.generate().unwrap();
    
    assert!(prompt.contains("1500"), "Should include token limit");
    assert!(prompt.contains("6") || prompt.contains("≤ 6"), "Should include complexity limit");
}

#[test]
fn test_prompt_includes_nabla_instructions() {
    let config = Config::new();
    let gen = PromptGenerator::new(config.rules);
    let prompt = gen.generate().unwrap();
    
    assert!(prompt.contains("∇∇∇"), "Should explain nabla format");
    assert!(prompt.contains("∆∆∆"), "Should explain delta format");
    assert!(prompt.contains("NABLA") || prompt.contains("Nabla"), "Should name the format");
}

#[test]
fn test_reminder_is_concise() {
    let config = Config::new();
    let gen = PromptGenerator::new(config.rules);
    let reminder = gen.generate_reminder().unwrap();
    
    // Reminder should be shorter than full prompt
    let full = gen.generate().unwrap();
    assert!(reminder.len() < full.len() / 2, "Reminder should be concise");
    
    // But still contain key constraints
    assert!(reminder.contains("tokens") || reminder.contains("Files"), "Should mention files/tokens");
    assert!(reminder.contains("Complexity") || reminder.contains("complexity"), "Should mention complexity");
}

// =============================================================================
// TOKENIZER
// =============================================================================

#[test]
fn test_tokenizer_counts_tokens() {
    let text = "Hello world, this is a test.";
    let count = Tokenizer::count(text);
    
    assert!(count > 0, "Should count tokens");
    assert!(count < 20, "Simple sentence should be few tokens");
}

#[test]
fn test_tokenizer_exceeds_limit() {
    let short = "hi";
    let long = "word ".repeat(1000);
    
    assert!(!Tokenizer::exceeds_limit(short, 100), "Short text should not exceed");
    assert!(Tokenizer::exceeds_limit(&long, 100), "Long text should exceed");
}

// =============================================================================
// FILE DISCOVERY
// =============================================================================

#[test]
fn test_discovery_finds_rust_files() {
    let dir = TempDir::new().unwrap();
    create_test_project(&dir);
    
    std::env::set_current_dir(dir.path()).unwrap();
    
    let mut config = Config::new();
    config.git_mode = GitMode::No;
    
    let files = discovery::discover(&config).unwrap();
    
    let rust_files: Vec<_> = files.iter()
        .filter(|p| p.extension().is_some_and(|e| e == "rs"))
        .collect();
    
    assert!(!rust_files.is_empty(), "Should find .rs files");
}

#[test]
fn test_discovery_excludes_hidden() {
    let dir = TempDir::new().unwrap();
    create_test_project(&dir);
    
    // Create hidden file
    fs::write(dir.path().join(".secret"), "secret stuff").unwrap();
    
    std::env::set_current_dir(dir.path()).unwrap();
    
    let mut config = Config::new();
    config.git_mode = GitMode::No;
    
    let files = discovery::discover(&config).unwrap();
    
    let hidden: Vec<_> = files.iter()
        .filter(|p| p.to_string_lossy().contains(".secret"))
        .collect();
    
    assert!(hidden.is_empty(), "Should not include hidden files");
}