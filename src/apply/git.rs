// src/apply/git.rs
use anyhow::{anyhow, Result};
use colored::Colorize;
use std::path::Path;
use std::process::Command;

/// Stages and commits the applied files.
///
/// # Errors
/// Returns error if git commands fail.
pub fn commit_changes(files: &[String], plan: Option<&str>, root: Option<&Path>) -> Result<()> {
    if files.is_empty() {
        return Ok(());
    }

    // 1. Git Add
    run_git(&["add"], files, root)?;

    // 2. Construct Commit Message
    let message = construct_message(files, plan);

    // 3. Git Commit
    run_git(&["commit", "-m", &message], &[], root)?;

    println!(
        "{} {}",
        "Git Commit:".green(),
        message.lines().next().unwrap_or("")
    );
    Ok(())
}

fn run_git(args: &[&str], files: &[String], root: Option<&Path>) -> Result<()> {
    let mut cmd = Command::new("git");

    if let Some(r) = root {
        cmd.current_dir(r);
    }

    cmd.args(args);
    cmd.args(files);

    let output = cmd.output()?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Git error: {err}"));
    }
    Ok(())
}

fn construct_message(files: &[String], plan: Option<&str>) -> String {
    if let Some(p) = plan {
        return p.trim().to_string();
    }

    // Fallback message
    let file_list = files.join(", ");
    if files.len() == 1 {
        format!("warden: update {file_list}")
    } else {
        format!("warden: update {} files ({file_list})", files.len())
    }
}
