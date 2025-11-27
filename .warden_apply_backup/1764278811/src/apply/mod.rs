// src/apply/mod.rs
pub mod extractor;
pub mod git;
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod writer;

use crate::clipboard;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use types::{ApplyConfig, ApplyOutcome, ExtractedFiles, Manifest};

/// Runs the apply command logic.
pub fn run_apply(config: ApplyConfig) -> Result<ApplyOutcome> {
    let content = clipboard::read_clipboard().context("Failed to read clipboard")?;
    process_input(&content, config)
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

/// Processes input content directly.
pub fn process_input(content: &str, config: ApplyConfig) -> Result<ApplyOutcome> {
    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Clipboard/Input is empty".to_string()));
    }

    let plan_opt = extractor::extract_plan(content);

    if !handle_plan_interaction(plan_opt.as_deref(), &config)? {
        return Ok(ApplyOutcome::ParseError("Operation cancelled by user.".to_string()));
    }

    execute_apply(content, &config, plan_opt.as_deref())
}

fn handle_plan_interaction(plan: Option<&str>, config: &ApplyConfig) -> Result<bool> {
    let Some(p) = plan else {
        return Ok(true);
    };

    println!("{}", "📋 PROPOSED PLAN:".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!("{}", p.trim());
    println!("{}", "─".repeat(50).dimmed());

    if config.force || config.dry_run {
        return Ok(true);
    }

    print!("Apply these changes? [y/N] ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    Ok(input.trim().eq_ignore_ascii_case("y"))
}

fn execute_apply(content: &str, config: &ApplyConfig, plan: Option<&str>) -> Result<ApplyOutcome> {
    let validation = parse_and_validate(content);

    match validation {
        ApplyOutcome::Success { .. } => {
            if config.dry_run {
                return Ok(ApplyOutcome::Success {
                    written: vec!["(Dry Run) Files verified".to_string()],
                    backed_up: false,
                });
            }

            let extracted = extractor::extract_files(content)?;
            let outcome = writer::write_files(&extracted, config.root)?;

            if config.commit {
                if let ApplyOutcome::Success { ref written, .. } = outcome {
                    if let Err(e) = git::commit_changes(written, plan, config.root) {
                        eprintln!("{} Failed to commit: {e}", "⚠️".yellow());
                    }
                }
            }
            Ok(outcome)
        }
        _ => Ok(validation),
    }
}

fn parse_and_validate(content: &str) -> ApplyOutcome {
    let manifest = match parse_manifest_step(content) {
        Ok(m) => m,
        Err(e) => return ApplyOutcome::ParseError(e),
    };

    let extracted = match extract_files_step(content) {
        Ok(e) => e,
        Err(e) => return ApplyOutcome::ParseError(e),
    };

    validator::validate(&manifest, &extracted)
}

fn parse_manifest_step(content: &str) -> Result<Manifest, String> {
    match manifest::parse_manifest(content) {
        Ok(Some(m)) => Ok(m),
        Ok(None) => Ok(Vec::new()),
        Err(e) => Err(format!("Manifest Error: {e}")),
    }
}

fn extract_files_step(content: &str) -> Result<ExtractedFiles, String> {
    extractor::extract_files(content).map_err(|e| format!("Extraction Error: {e}"))
}