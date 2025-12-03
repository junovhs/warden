// src/clean.rs
use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;
use std::path::Path;
use std::process::Command;

const CONTEXT_FILE: &str = "context.txt";
const GITIGNORE_FILE: &str = ".gitignore";

/// Runs the clean command: removes context.txt and ensures gitignore.
///
/// # Errors
/// Returns error if file operations or git commands fail.
pub fn run(commit: bool) -> Result<()> {
    let mut actions = Vec::new();

    if ensure_gitignore()? {
        actions.push("Added context.txt to .gitignore");
    }

    if remove_context_file()? {
        actions.push("Removed context.txt");
    }

    if actions.is_empty() {
        println!("{}", "✓ Already clean".green());
        return Ok(());
    }

    for action in &actions {
        println!("{} {action}", "✓".green());
    }

    if commit && is_git_repo() {
        commit_changes(&actions)?;
    }

    Ok(())
}

fn ensure_gitignore() -> Result<bool> {
    let path = Path::new(GITIGNORE_FILE);

    let content = if path.exists() {
        fs::read_to_string(path).context("Failed to read .gitignore")?
    } else {
        String::new()
    };

    if content.lines().any(|line| line.trim() == CONTEXT_FILE) {
        return Ok(false);
    }

    let new_content = if content.is_empty() || content.ends_with('\n') {
        format!("{content}{CONTEXT_FILE}\n")
    } else {
        format!("{content}\n{CONTEXT_FILE}\n")
    };

    fs::write(path, new_content).context("Failed to write .gitignore")?;
    Ok(true)
}

fn remove_context_file() -> Result<bool> {
    let path = Path::new(CONTEXT_FILE);

    if !path.exists() {
        return Ok(false);
    }

    fs::remove_file(path).context("Failed to remove context.txt")?;
    Ok(true)
}

fn is_git_repo() -> bool {
    Path::new(".git").is_dir()
}

fn commit_changes(actions: &[&str]) -> Result<()> {
    let message = format!("chore: {}", actions.join(", ").to_lowercase());

    Command::new("git")
        .args(["add", GITIGNORE_FILE])
        .output()
        .context("Failed to stage .gitignore")?;

    let output = Command::new("git")
        .args(["commit", "-m", &message])
        .output()
        .context("Failed to commit")?;

    if output.status.success() {
        println!("{} Committed: {}", "✓".green(), message.dimmed());
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("nothing to commit") {
            println!("{}", "✓ Nothing to commit".dimmed());
        } else {
            println!("{} Git commit failed: {}", "⚠".yellow(), stderr.trim());
        }
    }

    Ok(())
}
