// tests/unit_roadmap_cmd.rs
//! Tests for roadmap command parsing and execution.

use slopchop_core::roadmap::cmd_runner::apply_commands;
use slopchop_core::roadmap::types::{Command, CommandBatch, Roadmap};

#[test]
fn test_section_command() {
    let input = r#"SECTION "v0.11.0 — New Features""#;
    let batch = CommandBatch::parse(input);

    assert!(batch.errors.is_empty(), "Parse errors: {:?}", batch.errors);
    assert_eq!(batch.commands.len(), 1);

    match &batch.commands[0] {
        Command::AddSection { heading } => {
            assert_eq!(heading, "v0.11.0 — New Features");
        }
        other => panic!("Expected AddSection, got {other:?}"),
    }
}

#[test]
fn test_section_command_unquoted() {
    let input = "SECTION v0.12.0 — Performance";
    let batch = CommandBatch::parse(input);

    assert!(batch.errors.is_empty(), "Parse errors: {:?}", batch.errors);
    assert_eq!(batch.commands.len(), 1);

    match &batch.commands[0] {
        Command::AddSection { heading } => {
            assert_eq!(heading, "v0.12.0 — Performance");
        }
        other => panic!("Expected AddSection, got {other:?}"),
    }
}

#[test]
fn test_section_execution() {
    let roadmap_content = r"# Test Roadmap

## v0.1.0 — Existing

- [x] **Task one**
";
    let mut roadmap = Roadmap::parse(roadmap_content);
    let batch = CommandBatch::parse(r#"SECTION "v0.2.0 — New Section""#);

    let results = apply_commands(&mut roadmap, &batch);

    assert_eq!(results.len(), 1);
    assert!(roadmap.raw.contains("## v0.2.0 — New Section"));
}

#[test]
fn test_section_in_batch() {
    let input = r#"
===ROADMAP===
SECTION v0.99.0 — Future
ADD v0.99.0 "New feature"
===END===
"#;
    let batch = CommandBatch::parse(input);

    assert!(batch.errors.is_empty(), "Parse errors: {:?}", batch.errors);
    assert_eq!(batch.commands.len(), 2);
    assert!(matches!(&batch.commands[0], Command::AddSection { .. }));
    assert!(matches!(&batch.commands[1], Command::Add { .. }));
}