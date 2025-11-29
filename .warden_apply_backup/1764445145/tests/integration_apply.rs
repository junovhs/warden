//! Integration tests for warden apply command functionality.
//!
//! Tests verify:
//! - Nabla format parsing
//! - Path safety validation
//! - Truncation detection
//! - File writing

use std::collections::HashMap;
use warden_core::apply::extractor;
use warden_core::apply::types::{ApplyOutcome, ExtractedFiles, FileContent, Manifest};
use warden_core::apply::validator;

fn make_file(content: &str) -> FileContent {
    FileContent {
        content: content.to_string(),
        line_count: content.lines().count(),
    }
}

// =============================================================================
// NABLA EXTRACTION
// =============================================================================

#[test]
fn test_extract_single_file() {
    let input = r#"
∇∇∇ src/main.rs ∇∇∇
fn main() {
    println!("hello");
}