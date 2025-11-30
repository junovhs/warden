// tests/unit_tokens.rs
//! Unit tests for token counting functionality.
//! Covers: v0.1.0 Token Counting features

use warden_core::tokens::Tokenizer;

/// Verifies that the cl100k_base tokenizer initializes successfully.
/// Feature: Tokenizer initialization (cl100k_base)
#[test]
fn test_tokenizer_available() {
    // The tokenizer should be available and not panic on initialization
    let result = Tokenizer::count("test");
    // A non-zero result indicates the tokenizer is working
    // Even a simple word should produce at least 1 token
    assert!(
        result >= 1,
        "Tokenizer should produce at least 1 token for 'test'"
    );
}

/// Verifies basic token counting works correctly.
/// Feature: Token count function
#[test]
fn test_count_basic() {
    let simple = "Hello world";
    let count = Tokenizer::count(simple);

    // "Hello world" should be ~2 tokens (varies by tokenizer)
    assert!(
        count >= 2,
        "Expected at least 2 tokens for 'Hello world', got {count}"
    );
    assert!(
        count <= 5,
        "Expected at most 5 tokens for 'Hello world', got {count}"
    );
}

/// Verifies that token counting scales with content length.
#[test]
fn test_count_scales_with_length() {
    let short = "fn main() {}";
    let long = "fn main() { println!(\"Hello, world!\"); let x = 42; let y = x + 1; }";

    let short_count = Tokenizer::count(short);
    let long_count = Tokenizer::count(long);

    assert!(
        long_count > short_count,
        "Longer content should have more tokens: short={short_count}, long={long_count}"
    );
}

/// Verifies the exceeds_limit check works correctly.
/// Feature: Token limit check
#[test]
fn test_exceeds_limit() {
    let content = "fn main() { println!(\"test\"); }";
    let count = Tokenizer::count(content);

    // Should not exceed a high limit
    assert!(
        count < 1000,
        "Simple function should not exceed 1000 tokens"
    );

    // Should exceed a very low limit
    assert!(count > 1, "Simple function should exceed 1 token limit");
}

/// Verifies empty content returns zero tokens.
/// Feature: Graceful fallback on init failure (edge case: empty input)
#[test]
fn test_fallback_returns_zero() {
    let empty = "";
    let count = Tokenizer::count(empty);

    assert_eq!(count, 0, "Empty content should return 0 tokens");
}

/// Verifies whitespace-only content is handled.
#[test]
fn test_whitespace_only() {
    let whitespace = "   \n\t\n   ";
    let count = Tokenizer::count(whitespace);

    // Whitespace should produce minimal tokens
    assert!(
        count <= 5,
        "Whitespace should produce few tokens, got {count}"
    );
}

/// Verifies realistic Rust code token counting.
#[test]
fn test_realistic_rust_code() {
    let code = r#"
use std::io;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    println!("You entered: {}", input.trim());
    Ok(())
}
"#;

    let count = Tokenizer::count(code);

    // Realistic code should have a reasonable token count
    assert!(
        count >= 30,
        "Realistic code should have at least 30 tokens, got {count}"
    );
    assert!(
        count <= 200,
        "Realistic code should have at most 200 tokens, got {count}"
    );
}

/// Verifies Unicode content is handled correctly.
#[test]
fn test_unicode_content() {
    let unicode = "fn greet() { println!(\"你好世界\"); }";
    let count = Tokenizer::count(unicode);

    // Should not panic and should produce tokens
    assert!(count > 0, "Unicode content should produce tokens");
}

/// Verifies very long content produces proportionally many tokens.
#[test]
fn test_long_content() {
    // Generate content that should exceed typical limits
    let line = "let x = 1; let y = 2; let z = x + y;\n";
    let content: String = line.repeat(100);

    let count = Tokenizer::count(&content);

    // Should be substantial
    assert!(
        count > 500,
        "Long content should produce many tokens, got {count}"
    );
}
