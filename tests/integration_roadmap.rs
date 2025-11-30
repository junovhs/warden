// tests/integration_roadmap.rs
//! Integration tests for the roadmap system.
//! Covers: v0.6.0 Roadmap System features

use warden_core::roadmap::{slugify, CommandBatch, Roadmap};

const SAMPLE_ROADMAP: &str = r#"# Test Project Roadmap

## v0.1.0 — Foundation

### Core Features
- [x] **Feature one** <!-- test: tests/unit.rs::test_one -->
- [ ] **Feature two** <!-- test: tests/unit.rs::test_two -->
- [x] **Feature three**

### Secondary
- [ ] Pending task
- [x] Done task

## v0.2.0 — Advanced

- [ ] Future feature
"#;

// =============================================================================
// ROADMAP PARSING
// =============================================================================

/// Verifies title extraction from # Title.
/// Feature: Title extraction (# Title)
#[test]
fn test_parse_simple_roadmap() {
    let roadmap = Roadmap::parse(SAMPLE_ROADMAP);

    assert_eq!(
        roadmap.title, "Test Project Roadmap",
        "Should extract title"
    );
    assert!(!roadmap.sections.is_empty(), "Should have sections");
}

/// Verifies task checkbox detection and status extraction.
/// Features: Task checkbox detection, Task status: pending, Task status: complete
#[test]
fn test_parse_extracts_tasks() {
    let roadmap = Roadmap::parse(SAMPLE_ROADMAP);
    let tasks = roadmap.all_tasks();

    assert!(tasks.len() >= 5, "Should extract multiple tasks");

    // Check for complete tasks
    let complete_count = tasks
        .iter()
        .filter(|t| t.status == warden_core::roadmap::TaskStatus::Complete)
        .count();
    assert!(complete_count >= 2, "Should have complete tasks");

    // Check for pending tasks
    let pending_count = tasks
        .iter()
        .filter(|t| t.status == warden_core::roadmap::TaskStatus::Pending)
        .count();
    assert!(pending_count >= 2, "Should have pending tasks");
}

/// Verifies stats calculation.
/// Feature: Stats calculation
#[test]
fn test_stats_are_correct() {
    let roadmap = Roadmap::parse(SAMPLE_ROADMAP);
    let stats = roadmap.stats();

    assert!(stats.total >= 5, "Should count total tasks");
    assert!(stats.complete >= 2, "Should count complete tasks");
    assert!(stats.pending >= 2, "Should count pending tasks");
    assert_eq!(
        stats.total,
        stats.complete + stats.pending,
        "Total should equal complete + pending"
    );
}

/// Verifies task path generation for lookup.
/// Feature: Task path generation
#[test]
fn test_find_task_by_path() {
    let roadmap = Roadmap::parse(SAMPLE_ROADMAP);
    let tasks = roadmap.all_tasks();

    // Each task should have a path
    for task in &tasks {
        assert!(
            !task.path.is_empty(),
            "Task should have path: {}",
            task.text
        );
    }

    // Should be able to find a task by its path
    if let Some(first_task) = tasks.first() {
        let found = roadmap.find_task(&first_task.path);
        assert!(found.is_some(), "Should find task by path");
    }
}

/// Verifies compact state display format.
/// Feature: Compact state display
#[test]
fn test_compact_state_format() {
    let roadmap = Roadmap::parse(SAMPLE_ROADMAP);
    let compact = roadmap.compact_state();

    assert!(
        compact.contains("Test Project Roadmap"),
        "Should include title"
    );
    assert!(
        compact.contains("✓") || compact.contains("○"),
        "Should have status markers"
    );
}

// =============================================================================
// SLUGIFICATION
// =============================================================================

/// Verifies lowercase conversion in slugify.
/// Feature: Lowercase conversion
#[test]
fn test_slugify_basic() {
    assert_eq!(slugify("Hello World"), "hello-world");
    assert_eq!(slugify("UPPERCASE"), "uppercase");
    assert_eq!(slugify("MixedCase"), "mixedcase");
}

/// Verifies special characters are converted to dashes.
/// Feature: Special char to dash
#[test]
fn test_slugify_special_chars() {
    assert_eq!(slugify("hello_world"), "hello-world");
    assert_eq!(slugify("hello.world"), "hello-world");
    assert_eq!(slugify("hello/world"), "hello-world");
    assert_eq!(slugify("hello@world#test"), "hello-world-test");
    assert_eq!(slugify("  spaces  "), "spaces");
}

/// Verifies numbers are preserved in slugify.
/// Feature: Number preservation
#[test]
fn test_slugify_preserves_numbers() {
    assert_eq!(slugify("v0.1.0"), "v0-1-0");
    assert_eq!(slugify("feature123"), "feature123");
    assert_eq!(slugify("2024 update"), "2024-update");
}

// =============================================================================
// COMMAND PARSING
// =============================================================================

/// Verifies ===ROADMAP=== block detection.
/// Feature: ===ROADMAP=== block detection
#[test]
fn test_parse_extracts_from_larger_text() {
    let input = r#"
Here is some prose before the commands.

===ROADMAP===
CHECK feature-one
ADD v0.1.0 "New feature"
===END===

And some text after.
"#;

    let batch = CommandBatch::parse(input);

    assert!(
        !batch.commands.is_empty(),
        "Should extract commands from block"
    );
    assert!(batch.commands.len() >= 2, "Should find both commands");
}

/// Verifies CHECK command parsing.
/// Feature: CHECK command
#[test]
fn test_parse_check_command() {
    let input = "CHECK some-task-path";
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1, "Should parse one command");
    // Verify it's a Check command
    match &batch.commands[0] {
        warden_core::roadmap::Command::Check { path } => {
            assert_eq!(path, "some-task-path");
        }
        _ => panic!("Expected Check command"),
    }
}

/// Verifies multiple command parsing.
/// Features: UNCHECK command, ADD command
#[test]
fn test_parse_multiple_commands() {
    let input = r#"
CHECK task-one
UNCHECK task-two
ADD v0.1.0 "New task"
"#;

    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 3, "Should parse all three commands");
}

/// Verifies ADD with AFTER positioning.
/// Feature: ADD with AFTER
#[test]
fn test_parse_add_with_after() {
    let input = r#"ADD v0.1.0 "New feature" AFTER existing-task"#;

    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Add {
            parent,
            text,
            after,
        } => {
            assert_eq!(parent, "v0.1.0");
            assert_eq!(text, "New feature");
            assert!(after.is_some(), "Should have AFTER position");
            assert_eq!(after.as_ref().unwrap(), "existing-task");
        }
        _ => panic!("Expected Add command"),
    }
}

/// Verifies comment lines are skipped.
/// Feature: Comment skipping
#[test]
fn test_parse_ignores_comments() {
    let input = r#"
# This is a comment
CHECK task-one
// Another comment
CHECK task-two
"#;

    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 2, "Should skip comments");
}

/// Verifies summary generation.
/// Feature: Summary generation
#[test]
fn test_summary_format() {
    let input = r#"
CHECK task-one
CHECK task-two
ADD v0.1.0 "New task"
"#;

    let batch = CommandBatch::parse(input);
    let summary = batch.summary();

    assert!(summary.contains("CHECK"), "Summary should mention CHECK");
    assert!(summary.contains("ADD"), "Summary should mention ADD");
}
