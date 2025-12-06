// src/roadmap_v2/mod.rs
pub mod cli;
pub mod generator;
pub mod parser;
pub mod store;
pub mod types;

use std::path::Path;
use anyhow::{Context, Result};

// Added Task to exports
pub use types::{RoadmapCommand, TaskStatus, TaskStore, Task};
pub use cli::{handle_command, RoadmapV2Command};
pub use parser::parse_commands;

/// Handles raw string input from the clipboard or stdin, parsing it and applying commands to the roadmap.
///
/// # Errors
/// Returns error if the store cannot be loaded, parsing fails, or saving fails.
pub fn handle_input(path: &Path, content: &str) -> Result<Vec<String>> {
    let mut store = TaskStore::load(Some(path))?;
    let commands = parser::parse_commands(content).map_err(|e| anyhow::anyhow!("{e}"))?;
    
    if commands.is_empty() {
        return Ok(vec![]);
    }

    let mut results = Vec::new();
    let mut success_count = 0;

    for cmd in commands {
        match store.apply(cmd.clone()) {
            Ok(()) => {
                success_count += 1;
                results.push(format!("Applied: {cmd:?}"));
            }
            Err(e) => {
                results.push(format!("Failed: {cmd:?} - {e}"));
            }
        }
    }

    if success_count > 0 {
        store.save(Some(path)).context("Failed to save roadmap")?;
    }

    Ok(results)
}