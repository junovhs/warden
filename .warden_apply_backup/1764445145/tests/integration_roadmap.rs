//! Integration tests for warden roadmap functionality.
//!
//! Tests verify:
//! - Roadmap parsing
//! - Command parsing
//! - Command execution

use warden_core::roadmap::{CommandBatch, Roadmap, TaskStatus};

// =============================================================================
// ROADMAP PARSING
// =============================================================================

#[test]
fn test_parse_simple_roadmap() {
    let content = r#"# My Project Roadmap

## v0.1.0

- [x] Initial setup
- [ ] Add feature A
- [ ] Add feature B

## v0.2.0

- [ ] Major refactor
"#;
    
    let roadmap = Roadmap::parse(content);
    
    assert_eq!(roadmap.title, "My Project Roadmap");
    assert!(!roadmap.sections.is_empty());
}

#[test]
fn test_parse_extracts_tasks() {
    let content = r#"# Roadmap

## v0.1.0

- [x] Done task
- [ ] Pending task
"#;
    
    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();
    
    assert_eq!(tasks.len(), 2);
    
    let done: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Complete).collect();
    let pending: Vec<_> = tasks.iter().filter(|t| t.status == TaskStatus::Pending).collect();
    
    assert_eq!(done.len(), 1);
    assert_eq!(pending.len(), 1);
}

#[test]
fn test_stats_are_correct() {
    let content = r#"# Roadmap

## v0.1.0

- [x] One
- [x] Two
- [ ] Three
"#;
    
    let roadmap = Roadmap::parse(content);
    let stats = roadmap.stats();
    
    assert_eq!(stats.total, 3);
    assert_eq!(stats.complete, 2);
    assert_eq!(stats.pending, 1);
}

// =============================================================================
// COMMAND PARSING
// =============================================================================

#[test]
fn test_parse_check_command() {
    let input = r#"
===ROADMAP===
CHECK my-task
===END===
"#;
    
    let batch = CommandBatch::parse(input);
    
    assert_eq!(batch.commands.len(), 1);
    assert!(batch.errors.is_empty());
}

#[test]
fn test_parse_multiple_commands() {
    let input = r#"
===ROADMAP===
CHECK task-one
UNCHECK task-two
ADD v0-1-0 "New task"
===END===
"#;
    
    let batch = CommandBatch::parse(input);
    
    assert_eq!(batch.commands.len(), 3);
}

#[test]
fn test_parse_add_with_after() {
    let input = r#"
===ROADMAP===
ADD v0-1-0 "New task" AFTER existing-task
===END===
"#;
    
    let batch = CommandBatch::parse(input);
    
    assert_eq!(batch.commands.len(), 1);
    assert!(batch.errors.is_empty());
}

#[test]
fn test_parse_ignores_comments() {
    let input = r#"
===ROADMAP===
# This is a comment
CHECK my-task
// Another comment
===END===
"#;
    
    let batch = CommandBatch::parse(input);
    
    assert_eq!(batch.commands.len(), 1);
}

#[test]
fn test_parse_extracts_from_larger_text() {
    let input = r#"
Here's some prose from Claude explaining the changes.

I've updated the feature as requested.

===ROADMAP===
CHECK my-feature
===END===

Let me know if you need anything else!
"#;
    
    let batch = CommandBatch::parse(input);
    
    assert_eq!(batch.commands.len(), 1);
}

#[test]
fn test_summary_format() {
    let input = r#"
===ROADMAP===
CHECK one
CHECK two
ADD v0-1-0 "new"
===END===
"#;
    
    let batch = CommandBatch::parse(input);
    let summary = batch.summary();
    
    assert!(summary.contains("CHECK"));
    assert!(summary.contains("ADD"));
}

// =============================================================================
// SLUGIFICATION
// =============================================================================

#[test]
fn test_slugify_basic() {
    use warden_core::roadmap::slugify;
    
    assert_eq!(slugify("Hello World"), "hello-world");
    assert_eq!(slugify("My Feature"), "my-feature");
}

#[test]
fn test_slugify_special_chars() {
    use warden_core::roadmap::slugify;
    
    assert_eq!(slugify("v0.1.0 — Feature"), "v0-1-0-feature");
    assert_eq!(slugify("**Bold Text**"), "bold-text");
}

#[test]
fn test_slugify_preserves_numbers() {
    use warden_core::roadmap::slugify;
    
    assert_eq!(slugify("v0.5.0"), "v0-5-0");
    assert_eq!(slugify("Feature 123"), "feature-123");
}

// =============================================================================
// TASK FINDING
// =============================================================================

#[test]
fn test_find_task_by_path() {
    let content = r#"# Roadmap

## v0.1.0

- [ ] My feature
"#;
    
    let roadmap = Roadmap::parse(content);
    let task = roadmap.find_task("v0-1-0/my-feature");
    
    assert!(task.is_some());
    assert_eq!(task.unwrap().text, "My feature");
}

#[test]
fn test_compact_state_format() {
    let content = r#"# Test Roadmap

## v0.1.0

- [x] Done
- [ ] Pending
"#;
    
    let roadmap = Roadmap::parse(content);
    let compact = roadmap.compact_state();
    
    assert!(compact.contains("Test Roadmap"));
    assert!(compact.contains("✓") || compact.contains("[1/2]"));
}