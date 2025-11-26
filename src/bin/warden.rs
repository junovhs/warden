// src/bin/warden.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::fs;
use std::io;
use std::process::{self, Command};

use warden_core::apply;
use warden_core::clipboard;
use warden_core::config::{Config, GitMode};
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;
use warden_core::prompt::PromptGenerator;
use warden_core::reporting;
use warden_core::rules::RuleEngine;
use warden_core::tui::state::App;
use warden_core::types::ScanReport;

#[derive(Subcommand)]
enum Commands {
    Prompt {
        #[arg(long, short)]
        copy: bool,
        #[arg(long, short)]
        short: bool,
    },
    Run {
        name: String,
    },
    Apply {
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Parser)]
#[command(name = "warden")]
#[command(about = "Structural linter for Code With Intent")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[arg(long, short)]
    verbose: bool,
    #[arg(long)]
    git_only: bool,
    #[arg(long)]
    no_git: bool,
    #[arg(long)]
    code_only: bool,
    #[arg(long)]
    init: bool,
    #[arg(long)]
    ui: bool,

    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(index = 1)]
    legacy_command: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.init {
        return init_config();
    }

    let config = load_config(&cli)?;

    if let Some(cmd) = &cli.command {
        return exec_subcommand(cmd, &config);
    }

    if let Some(cmd_name) = &cli.legacy_command {
        run_alias(&config, cmd_name);
    }

    run_scan(&config, cli.ui)
}

fn init_config() -> Result<()> {
    if std::path::Path::new("warden.toml").exists() {
        println!("{}", "⚠️ warden.toml already exists.".yellow());
    } else {
        // Default template now suggests auto-detection
        let default_toml = r#"# warden.toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10
max_nesting_depth = 4
max_function_args = 5
max_function_words = 3
ignore_naming_on = ["tests", "spec"]

# Commands are auto-detected if not specified here.
# To override:
# [commands]
# check = "npx.cmd @biomejs/biome check src/"
"#;
        fs::write("warden.toml", default_toml)?;
        println!("{}", "✅ Created warden.toml".green());
    }
    Ok(())
}

fn load_config(cli: &Cli) -> Result<Config> {
    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;
    config.git_mode = if cli.git_only {
        GitMode::Yes
    } else if cli.no_git {
        GitMode::No
    } else {
        GitMode::Auto
    };
    config.load_local_config();
    config.validate()?;
    Ok(config)
}

fn exec_subcommand(cmd: &Commands, config: &Config) -> Result<()> {
    match cmd {
        Commands::Prompt { copy, short } => show_prompt(config, *copy, *short),
        Commands::Run { name } => {
            run_alias(config, name);
            Ok(())
        }
        Commands::Apply { dry_run } => {
            run_apply(*dry_run);
            Ok(())
        }
    }
}

fn show_prompt(config: &Config, copy: bool, short: bool) -> Result<()> {
    let generator = PromptGenerator::new(config.rules.clone());
    let output = if short {
        generator.generate_reminder()?
    } else {
        generator.wrap_header()?
    };

    if copy {
        clipboard::copy_to_clipboard(&output)?;
        println!("{}", "✅ Copied to clipboard".green());
    } else {
        println!("{output}");
    }
    Ok(())
}

fn run_alias(config: &Config, name: &str) {
    if let Some(cmd_str) = config.commands.get(name) {
        println!("🚀 Running '{}': {}", name.cyan(), cmd_str.yellow());
        execute_command_string(cmd_str);
    } else {
        println!(
            "⚠️ Unknown command: '{}'. Auto-detection failed or not configured.",
            name.yellow()
        );
        process::exit(1);
    }
}

fn execute_command_string(cmd_str: &str) {
    let mut parts = cmd_str.split_whitespace();
    if let Some(prog) = parts.next() {
        // Verbose error handling for command execution
        let result = Command::new(prog).args(parts).status();

        match result {
            Ok(status) => {
                if !status.success() {
                    println!(
                        "{}",
                        format!(
                            "❌ Command failed with exit code {}",
                            status.code().unwrap_or(1)
                        )
                        .red()
                    );
                    process::exit(status.code().unwrap_or(1));
                }
            }
            Err(e) => {
                println!("{}", format!("❌ Failed to execute '{prog}': {e}").red());
                if cfg!(windows) {
                    println!(
                        "{}",
                        "💡 Tip: On Windows, npm commands need '.cmd' (e.g., 'npx.cmd').".yellow()
                    );
                }
                process::exit(1);
            }
        }
    }
}

fn run_apply(dry_run: bool) {
    if dry_run {
        println!("{}", "🔍 Dry run mode - no files will be written".yellow());
    }

    match apply::run_apply(dry_run) {
        Ok(outcome) => {
            apply::print_result(&outcome);
            match outcome {
                apply::types::ApplyOutcome::Success { .. } => {}
                _ => {
                    process::exit(1);
                }
            }
        }
        Err(e) => {
            println!("{}", format!("❌ Error: {e}").red());
            process::exit(1);
        }
    }
}

fn run_scan(config: &Config, use_ui: bool) -> Result<()> {
    let files = discover_files(config)?;
    if files.is_empty() {
        println!("No files to scan.");
        return Ok(());
    }

    let report = RuleEngine::new(config.clone()).scan(files);

    if use_ui {
        run_tui_app(report)
    } else {
        reporting::print_report(&report)?;
        if report.total_violations > 0 {
            process::exit(1);
        }
        Ok(())
    }
}

fn discover_files(config: &Config) -> Result<Vec<std::path::PathBuf>> {
    let raw = FileEnumerator::new(config.clone()).enumerate()?;
    let heuristic = HeuristicFilter::new().filter(raw);
    let filtered = FileFilter::new(config)?.filter(heuristic);
    Ok(filtered)
}

fn run_tui_app(report: ScanReport) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = ratatui::Terminal::new(backend)?;

    let mut app = App::new(report);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }
    Ok(())
}
