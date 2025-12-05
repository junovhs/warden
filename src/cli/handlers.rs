// src/cli/handlers.rs
//! Command handlers for the slopchop CLI.

use std::io::{self, Write};
use std::path::Path;
use std::process::{self, Command, Stdio};

use anyhow::Result;
use colored::Colorize;

use crate::apply::{self, types::ApplyContext};
use crate::config::Config;
use crate::context::{self, ContextOptions};
use crate::pack::{self, OutputFormat, PackOptions};
use crate::prompt::PromptGenerator;
use crate::trace::{self, TraceOptions};

/// Runs the check pipeline.
pub fn handle_check() {
    run_pipeline("check");
}

/// Runs the fix pipeline.
pub fn handle_fix() {
    run_pipeline("fix");
}

/// Displays the repository map.
///
/// # Errors
/// Returns error if trace fails.
pub fn handle_map(show_deps: bool) -> Result<()> {
    println!("{}", trace::map(show_deps)?);
    Ok(())
}

/// Traces dependencies from a file.
///
/// # Errors
/// Returns error if trace fails.
pub fn handle_trace(file: &Path, depth: usize, budget: usize) -> Result<()> {
    let opts = TraceOptions {
        anchor: file.to_path_buf(),
        depth,
        budget,
    };
    println!("{}", trace::run(&opts)?);
    Ok(())
}

/// Generates and displays context map.
///
/// # Errors
/// Returns error if context generation or clipboard fails.
pub fn handle_context(verbose: bool, copy: bool) -> Result<()> {
    let opts = ContextOptions { verbose };
    let output = context::run(&opts)?;

    if copy {
        crate::clipboard::copy_to_clipboard(&output)?;
        println!("{}", "? Context map copied to clipboard".green());
    } else {
        println!("{output}");
    }

    Ok(())
}

/// Generates and optionally copies the prompt.
///
/// # Errors
/// Returns error if prompt generation or clipboard fails.
pub fn handle_prompt(copy: bool) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let prompt = PromptGenerator::new(config.rules.clone()).generate()?;

    if copy {
        crate::clipboard::copy_to_clipboard(&prompt)?;
        println!("{}", "? Copied to clipboard".green());
    } else {
        println!("{prompt}");
    }
    Ok(())
}

/// Applies changes from clipboard.
///
/// # Errors
/// Returns error if apply fails.
pub fn handle_apply() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let outcome = apply::run_apply(&ApplyContext::new(&config))?;
    apply::print_result(&outcome);
    Ok(())
}

/// Pack command arguments.
#[allow(clippy::struct_excessive_bools)]
pub struct PackArgs {
    pub stdout: bool,
    pub copy: bool,
    pub noprompt: bool,
    pub format: OutputFormat,
    pub skeleton: bool,
    pub git_only: bool,
    pub no_git: bool,
    pub code_only: bool,
    pub verbose: bool,
    pub target: Option<std::path::PathBuf>,
    pub focus: Vec<std::path::PathBuf>,
    pub depth: usize,
}

/// Runs the pack command.
///
/// # Errors
/// Returns error if packing fails.
pub fn handle_pack(args: PackArgs) -> Result<()> {
    pack::run(&PackOptions {
        stdout: args.stdout,
        copy: args.copy,
        prompt: !args.noprompt,
        format: args.format,
        skeleton: args.skeleton,
        git_only: args.git_only,
        no_git: args.no_git,
        code_only: args.code_only,
        verbose: args.verbose,
        target: args.target,
        focus: args.focus,
        depth: args.depth,
    })
}

fn run_pipeline(name: &str) {
    let mut config = Config::new();
    config.load_local_config();

    let Some(commands) = config.commands.get(name) else {
        eprintln!("{} No '{}' command configured", "?".red(), name);
        process::exit(1);
    };

    println!("{} Running '{}' pipeline...", ">".cyan(), name);
    for cmd in commands {
        if !exec_cmd_filtered(cmd) {
            process::exit(1);
        }
    }
    
    println!("{}", "? All checks passed!".green().bold());
}

fn exec_cmd_filtered(cmd: &str) -> bool {
    print!("   {} {} ", ">".blue(), cmd.dimmed());
    let _ = io::stdout().flush();

    let parts: Vec<&str> = cmd.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        println!("{}", "?".green());
        return true;
    };

    match execute_command(prog, args) {
        Ok(output) => handle_command_output(cmd, &output),
        Err(e) => {
            println!("{}", "?".red());
            eprintln!("     {} {e}", "error:".red());
            false
        }
    }
}

fn execute_command(prog: &str, args: &[&str]) -> std::io::Result<std::process::Output> {
    Command::new(prog)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
}

fn handle_command_output(cmd: &str, output: &std::process::Output) -> bool {
    if output.status.success() {
        println!("{}", "?".green());
        return true;
    }

    println!("{}", "?".red());
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);

    print_failure_details(cmd, &stdout, &stderr);
    false
}

fn print_failure_details(cmd: &str, stdout: &str, stderr: &str) {
    if cmd.contains("cargo test") {
        print_test_failures(stdout, stderr);
    } else if cmd.contains("clippy") {
        print_clippy_errors(stderr);
    } else {
        print_generic_error(stdout, stderr);
    }
}

fn print_test_failures(stdout: &str, stderr: &str) {
    println!("\n{}", "��� Test Failures ���".red().bold());

    for line in stdout.lines().chain(stderr.lines()) {
        if line.contains("FAILED") {
            println!("  {} {}", "?".red(), line.trim());
        }
    }

    for line in stderr.lines() {
        if line.contains("panicked at") {
            println!("\n  {}", line.trim().yellow());
        }
    }
    println!();
}

fn print_clippy_errors(stderr: &str) {
    println!("\n{}", "��� Clippy Errors ���".red().bold());

    for line in stderr.lines() {
        if !process_clippy_line(line) {
            break;
        }
    }
    println!();
}

fn process_clippy_line(line: &str) -> bool {
    let trimmed = line.trim();
    if trimmed.starts_with("error[") || trimmed.starts_with("error:") {
        println!("  {}", trimmed.red());
    } else if trimmed.starts_with("-->") {
        println!("    {}", trimmed.dimmed());
    } else if trimmed.contains("could not compile") {
        return false;
    }
    true
}

fn print_generic_error(stdout: &str, stderr: &str) {
    println!("\n{}", "��� Output ���".red().bold());
    let combined = format!("{stdout}\n{stderr}");
    for line in combined.lines().take(20) {
        if !line.trim().is_empty() {
            println!("  {line}");
        }
    }
    println!();
}