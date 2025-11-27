// src/apply/mod.rs
pub mod extractor;
pub mod manifest;
pub mod messages;
pub mod types;
pub mod validator;
pub mod writer;

use crate::clipboard;
use anyhow::{Context, Result};
use colored::Colorize;
use std::io::{self, Write};
use std::path::Path;
use types::{ApplyOutcome, ExtractedFiles, Manifest};

/// Runs the apply command logic.
///
/// # Errors
/// Returns error if clipboard access fails or extraction fails.
pub fn run_apply(dry_run: bool, force: bool) -> Result<ApplyOutcome> {
    let content = clipboard::read_clipboard().context("Failed to read clipboard")?;
    process_input(&content, dry_run, force, None)
}

pub fn print_result(outcome: &ApplyOutcome) {
    messages::print_outcome(outcome);
}

/// Processes input content directly.
///
/// # Errors
/// Returns error if extraction or writing fails.
pub fn process_input(
    content: &str,
    dry_run: bool,
    force: bool,
    root: Option<&Path>,
) -> Result<ApplyOutcome> {
    if content.trim().is_empty() {
        return Ok(ApplyOutcome::ParseError("Clipboard/Input is empty".to_string()));
    }

    // 1. Extract and display the plan (if any)
    if let Some(plan) = extractor::extract_plan(content) {
        println!("{}", "📋 PROPOSED PLAN:".cyan().bold());
        println!("{}", "─".repeat(50).dimmed());
        println!("{}", plan.trim());
        println!("{}", "─".repeat(50).dimmed());
        
        if !force && !dry_run {
            print!("Apply these changes? [y/N] ");
            io::stdout().flush()?;
            
            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            
            if !input.trim().eq_ignore_ascii_case("y") {
                return Ok(ApplyOutcome::ParseError("Operation cancelled by user.".to_string()));
            }
            println!();
        }
    }

    let validation = parse_and_validate(content);

    match validation {
        ApplyOutcome::Success { .. } => {
            if dry_run {
                return Ok(ApplyOutcome::Success {
                    written: vec!["(Dry Run) Files verified".to_string()],
                    backed_up: false,
                });
            }
            // Extract again (inefficient but safe separation of concerns)
            let extracted = extractor::extract_files(content)?;
            writer::write_files(&extracted, root)
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