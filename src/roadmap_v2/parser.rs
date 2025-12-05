// src/roadmap_v2/parser.rs
use crate::error::SlopChopError;
use super::types::{RoadmapCommand, Task, TaskUpdate};

const BLOCK_START: &str = "===ROADMAP===";

/// Parse all roadmap command blocks from input text.
///
/// # Errors
/// Returns error if a command block is malformed or has missing required fields.
pub fn parse_commands(input: &str) -> Result<Vec<RoadmapCommand>, SlopChopError> {
    let blocks = extract_blocks(input);
    let mut commands = Vec::new();

    for block in blocks {
        let cmd = parse_single_block(&block)?;
        commands.push(cmd);
    }

    Ok(commands)
}

fn extract_blocks(input: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current = String::new();

    for line in input.lines() {
        let trimmed = line.trim();
        if trimmed == BLOCK_START {
            if in_block {
                blocks.push(current.clone());
                current.clear();
            }
            in_block = !in_block;
            continue;
        }
        if in_block {
            current.push_str(line);
            current.push('\n');
        }
    }

    blocks
}

fn parse_single_block(block: &str) -> Result<RoadmapCommand, SlopChopError> {
    let lines: Vec<&str> = block.lines().collect();
    let first_line = lines.first().copied().unwrap_or("").trim();

    match first_line.to_uppercase().as_str() {
        "CHECK" => parse_check(&lines[1..]),
        "UNCHECK" => parse_uncheck(&lines[1..]),
        "ADD" => parse_add(&lines[1..]),
        "UPDATE" => parse_update(&lines[1..]),
        "DELETE" => parse_delete(&lines[1..]),
        other => Err(SlopChopError::Other(format!(
            "Unknown roadmap command: {other}"
        ))),
    }
}

fn parse_check(lines: &[&str]) -> Result<RoadmapCommand, SlopChopError> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Check { id })
}

fn parse_uncheck(lines: &[&str]) -> Result<RoadmapCommand, SlopChopError> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Uncheck { id })
}

fn parse_delete(lines: &[&str]) -> Result<RoadmapCommand, SlopChopError> {
    let id = require_field(lines, "id")?;
    Ok(RoadmapCommand::Delete { id })
}

fn parse_add(lines: &[&str]) -> Result<RoadmapCommand, SlopChopError> {
    let id = require_field(lines, "id")?;
    let task_text = require_field(lines, "text")?;
    let section = require_field(lines, "section")?;
    let group = optional_field(lines, "group");
    let test_anchor = optional_field(lines, "test");

    let task = Task {
        id,
        text: task_text,
        status: super::types::TaskStatus::Pending,
        section,
        group,
        test: test_anchor,
        order: 0,
    };

    Ok(RoadmapCommand::Add(task))
}

fn parse_update(lines: &[&str]) -> Result<RoadmapCommand, SlopChopError> {
    let id = require_field(lines, "id")?;
    let fields = TaskUpdate {
        text: optional_field(lines, "text"),
        test: optional_field(lines, "test"),
        section: optional_field(lines, "section"),
        group: optional_field(lines, "group"),
    };

    Ok(RoadmapCommand::Update { id, fields })
}

fn require_field(lines: &[&str], key: &str) -> Result<String, SlopChopError> {
    optional_field(lines, key).ok_or_else(|| {
        SlopChopError::Other(format!("Missing required field: {key}"))
    })
}

fn optional_field(lines: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key} = ");
    for line in lines {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix(&prefix) {
            return Some(value.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_check() {
        let input = "===ROADMAP===\nCHECK\nid = my-task\n===ROADMAP===";
        let cmds = parse_commands(input).unwrap_or_default();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(&cmds[0], RoadmapCommand::Check { id } if id == "my-task"));
    }

    #[test]
    fn test_parse_add() {
        let input = "===ROADMAP===\nADD\nid = new-feature\ntext = Support Go\nsection = v0.8.0\ngroup = Lang\ntest = tests/unit.rs::test_go\n===ROADMAP===";
        let cmds = parse_commands(input).unwrap_or_default();
        assert_eq!(cmds.len(), 1);
        assert!(matches!(&cmds[0], RoadmapCommand::Add(t) if t.id == "new-feature"));
    }
}