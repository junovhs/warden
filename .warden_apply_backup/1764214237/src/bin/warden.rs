// src/bin/warden.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::io;
use std::path::Path;
use std::process::{self, Command};

use warden_core::apply;
use warden_core::config::{Config, GitMode};
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;
use warden_core::project;
use warden_core::prompt::PromptGenerator;
use warden_core::reporting;
use warden_core::rules::RuleEngine;
use warden_core::tui::state::App;
use warden_core::types::ScanReport;

#[derive(Parser)]
#[command(name = "warden")]
#[command(about = "Code quality guardian", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    #[arg(long)]
    ui: bool,
    #[arg(long)]
    init: bool,
}

#[derive(Subcommand)]
enum Commands {
    Prompt {
        #[arg(long, short)]
        copy: bool,
    },
    Check,
    Fix,
    Apply {
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e}", "error:".red().bold());
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    if cli.init {
        return init_config();
    }

    ensure_config_exists();

    match cli.command {
        Some(Commands::Prompt { copy }) => handle_prompt(copy),
        Some(Commands::Check) => run_command("check"),
        Some(Commands::Fix) => run_command("fix"),
        Some(Commands::Apply { dry_run }) => apply::run(dry_run),
        None if cli.ui => run_tui(),
        None => run_scan(),
    }
}

fn ensure_config_exists() {
    if Path::new("warden.toml").exists() {
        return;
    }
    let content = project::generate_toml();
    if fs::write("warden.toml", &content).is_ok() {
        eprintln!("{}", "📝 Created warden.toml".dimmed());
    }
}

fn init_config() -> Result<()> {
    let content = project::generate_toml();
    fs::write("warden.toml", &content)?;
    println!("{}", "✓ Created warden.toml".green());
    Ok(())
}

fn handle_prompt(copy: bool) -> Result<()> {
    let prompt = PromptGenerator::generate();
    if copy {
        warden_core::clipboard::copy_to_clipboard(&prompt)?;
        println!("{}", "✓ Copied to clipboard".green());
    } else {
        println!("{prompt}");
    }
    Ok(())
}

fn run_command(name: &str) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();

    let Some(cmd_str) = config.commands.get(name) else {
        eprintln!(
            "{} No '{}' command configured in warden.toml",
            "error:".red(),
            name
        );
        process::exit(1);
    };

    println!("{} Running '{}': {}", "🚀".green(), name, cmd_str.dimmed());

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let (prog, args) = parts.split_first().unwrap_or((&"", &[]));

    let status = Command::new(prog).args(args).status();

    match status {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => {
            let code = s.code().unwrap_or(1);
            eprintln!("{} Command failed with exit code {code}", "❌".red());
            process::exit(code);
        }
        Err(e) => {
            handle_exec_error(&e, prog);
            process::exit(1);
        }
    }
}

fn handle_exec_error(e: &std::io::Error, prog: &str) {
    if e.kind() == io::ErrorKind::NotFound {
        eprintln!("{} Command not found: {prog}", "error:".red());
        eprintln!("  Check that the program is installed and in PATH");
    } else {
        eprintln!("{} Failed to execute: {e}", "error:".red());
    }
}

fn run_scan() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();

    let files = FileEnumerator::new(config.clone()).enumerate()?;
    let files = FileFilter::new(&config).filter(files);
    let files = HeuristicFilter::new(&config).filter(files);

    let engine = RuleEngine::new(config.rules.clone());
    let report = engine.analyze(&files)?;

    reporting::print_report(&report);

    if report.has_errors() {
        process::exit(1);
    }
    Ok(())
}

fn run_tui() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();

    let files = FileEnumerator::new(config.clone()).enumerate()?;
    let files = FileFilter::new(&config).filter(files);
    let files = HeuristicFilter::new(&config).filter(files);

    let engine = RuleEngine::new(config.rules.clone());
    let report = engine.analyze(&files)?;

    run_tui_with_report(report)
}

fn run_tui_with_report(report: ScanReport) -> Result<()> {
    use crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    };
    use ratatui::backend::CrosstermBackend;
    use ratatui::Terminal;

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(report);
    let res = app.run(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    res
}
