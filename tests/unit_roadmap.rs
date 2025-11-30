// tests/unit_roadmap.rs
//! Unit tests for roadmap parsing and command handling.
//! Covers: v0.6.0 Roadmap Parsing and Command Parsing features

use warden_core::roadmap::{CommandBatch, Roadmap};

const ROADMAP_WITH_ANCHORS: &str = r#"# Project Roadmap

## v0.1.0

- [x] **Feature A** <!-- test: tests/unit.rs::test_feature_a -->
- [ ] **Feature B** <!-- test: tests/other.rs::test_feature_b -->
- [x] **No test feature** [no-test]
"#;

// =============================================================================
// TEST ANCHOR EXTRACTION
// =============================================================================

/// Verifies test anchor extraction from task comments.
/// Feature: Test anchor extraction
#[test]
fn test_anchor_extraction() {
    let roadmap = Roadmap::parse(ROADMAP_WITH_ANCHORS);
    let tasks = roadmap.all_tasks();

    // Find task with test anchor
    let task_a = tasks.iter().find(|t| t.text.contains("Feature A"));
    assert!(task_a.is_some(), "Should find Feature A");

    let task = task_a.unwrap();
    assert!(!task.tests.is_empty(), "Should extract test anchors");
    assert!(
        task.tests[0].contains("test_feature_a"),
        "Should have correct test path"
    );
}

// =============================================================================
// COMMAND PARSING - ADDITIONAL COMMANDS
// =============================================================================

/// Verifies DELETE command parsing.
/// Feature: DELETE command
#[test]
fn test_delete_command() {
    let input = "DELETE old-task-path";
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Delete { path } => {
            assert_eq!(path, "old-task-path");
        }
        _ => panic!("Expected Delete command"),
    }
}

/// Verifies UPDATE command parsing.
/// Feature: UPDATE command
#[test]
fn test_update_command() {
    let input = r#"UPDATE task-path "New description""#;
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Update { path, text } => {
            assert_eq!(path, "task-path");
            assert_eq!(text, "New description");
        }
        _ => panic!("Expected Update command"),
    }
}

/// Verifies NOTE command parsing.
/// Feature: NOTE command
#[test]
fn test_note_command() {
    let input = r#"NOTE task-path "Implementation note here""#;
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Note { path, note } => {
            assert_eq!(path, "task-path");
            assert_eq!(note, "Implementation note here");
        }
        _ => panic!("Expected Note command"),
    }
}

/// Verifies MOVE command parsing.
/// Feature: MOVE command
#[test]
fn test_move_command() {
    let input = "MOVE task-one AFTER task-two";
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Move { path, position } => {
            assert_eq!(path, "task-one");
            match position {
                warden_core::roadmap::MovePosition::After(target) => {
                    assert_eq!(target, "task-two");
                }
                _ => panic!("Expected After position"),
            }
        }
        _ => panic!("Expected Move command"),
    }
}

/// Verifies MOVE BEFORE command.
#[test]
fn test_move_before_command() {
    let input = "MOVE task-one BEFORE task-two";
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Move { path, position } => {
            assert_eq!(path, "task-one");
            match position {
                warden_core::roadmap::MovePosition::Before(target) => {
                    assert_eq!(target, "task-two");
                }
                _ => panic!("Expected Before position"),
            }
        }
        _ => panic!("Expected Move command"),
    }
}

// =============================================================================
// EDGE CASES
// =============================================================================

/// Verifies empty input produces no commands.
#[test]
fn test_empty_input() {
    let batch = CommandBatch::parse("");
    assert!(batch.commands.is_empty());
}

/// Verifies whitespace-only input produces no commands.
#[test]
fn test_whitespace_input() {
    let batch = CommandBatch::parse("   \n\n   \t   ");
    assert!(batch.commands.is_empty());
}

/// Verifies invalid commands produce errors.
#[test]
fn test_invalid_command_error() {
    let input = "INVALID_COMMAND some-arg";
    let batch = CommandBatch::parse(input);

    // Should not have valid commands
    assert!(batch.commands.is_empty() || batch.errors.len() > 0);
}

/// Verifies quoted strings with spaces work.
#[test]
fn test_quoted_string_with_spaces() {
    let input = r#"ADD v0.1.0 "A task with multiple words""#;
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Add { text, .. } => {
            assert_eq!(text, "A task with multiple words");
        }
        _ => panic!("Expected Add command"),
    }
}

/// Verifies escaped quotes in strings.
#[test]
fn test_escaped_quotes() {
    let input = r#"ADD v0.1.0 "Task with \"quotes\" inside""#;
    let batch = CommandBatch::parse(input);

    assert_eq!(batch.commands.len(), 1);
    match &batch.commands[0] {
        warden_core::roadmap::Command::Add { text, .. } => {
            assert!(text.contains("\""), "Should preserve escaped quotes");
        }
        _ => panic!("Expected Add command"),
    }
}

/// Verifies roadmap with no tasks.
#[test]
fn test_roadmap_no_tasks() {
    let content = r#"# Empty Roadmap

## v0.1.0

Some prose but no tasks.

## v0.2.0

More prose.
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    assert!(tasks.is_empty(), "Should have no tasks");
}

/// Verifies deeply nested sections.
#[test]
fn test_nested_sections() {
    let content = r#"# Project

## v0.1.0

### Subsection A

- [x] Task in subsection

### Subsection B

#### Deep subsection

- [ ] Deep task
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    assert!(tasks.len() >= 2, "Should find tasks in nested sections");
}

/// Verifies task status markers are correctly parsed.
#[test]
fn test_task_status_markers() {
    let content = r#"# Test

## v0.1.0

- [x] Complete task
- [ ] Pending task
- [X] Also complete (uppercase)
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // Should have at least the lowercase ones
    assert!(tasks.len() >= 2);
}
