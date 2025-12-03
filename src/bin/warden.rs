// src/bin/warden.rs
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use warden_core::analysis::RuleEngine;
use warden_core::apply;
use warden_core::apply::types::ApplyContext;
use warden_core::config::Config;
use warden_core::discovery;
use warden_core::pack::{self, OutputFormat, PackOptions};
use warden_core::prompt::PromptGenerator;
use warden_core::reporting;
use warden_core::roadmap::cli::{handle_command, RoadmapCommand};
use warden_core::tui::state::App;
use warden_core::types::ScanReport;
use warden_core::wizard;

#[derive(Parser)]
#[command(name = "warden")]
#[command(version)]
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
    Apply,
    Clean {
        #[arg(long, short)]
        commit: bool,
    },
    Config,
    #[command(subcommand)]
    Roadmap(RoadmapCommand),
    Pack {
        #[arg(long, short)]
        stdout: bool,
        #[arg(long, short)]
        copy: bool,
        #[arg(long)]
        noprompt: bool,
        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,
        #[arg(long)]
        skeleton: bool,
        #[arg(long)]
        git_only: bool,
        #[arg(long)]
        no_git: bool,
        #[arg(long)]
        code_only: bool,
        #[arg(long, short)]
        verbose: bool,
        #[arg(value_name = "TARGET")]
        target: Option<PathBuf>,
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
        return wizard::run();
    }

    ensure_config_exists();
    dispatch_command(&cli)
}

fn dispatch_command(cli: &Cli) -> Result<()> {
    match &cli.command {
        Some(cmd) => dispatch_subcommand(cmd),
        None => dispatch_default(cli.ui),
    }
}

fn dispatch_subcommand(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Check => run_command("check"),
        Commands::Fix => run_command("fix"),
        Commands::Config => warden_core::tui::run_config(),
        Commands::Apply => handle_apply(),
        _ => dispatch_with_args(cmd),
    }
}

fn dispatch_with_args(cmd: &Commands) -> Result<()> {
    match cmd {
        Commands::Prompt { copy } => handle_prompt(*copy),
        Commands::Clean { commit } => warden_core::clean::run(*commit),
        Commands::Roadmap(sub) => handle_command(sub.clone()),
        Commands::Pack { .. } => handle_pack(cmd),
        _ => Ok(()),
    }
}

fn handle_pack(cmd: &Commands) -> Result<()> {
    let Commands::Pack {
        stdout,
        copy,
        noprompt,
        format,
        skeleton,
        git_only,
        no_git,
        code_only,
        verbose,
        target,
    } = cmd
    else {
        return Ok(());
    };

    pack::run(&PackOptions {
        stdout: *stdout,
        copy: *copy,
        prompt: !*noprompt,
        format: format.clone(),
        skeleton: *skeleton,
        git_only: *git_only,
        no_git: *no_git,
        code_only: *code_only,
        verbose: *verbose,
        target: target.clone(),
    })
}

fn dispatch_default(ui: bool) -> Result<()> {
    if ui {
        run_tui()
    } else {
        run_scan()
    }
}

fn handle_apply() -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let ctx = ApplyContext::new(&config);
    let outcome = apply::run_apply(&ctx)?;
    apply::print_result(&outcome);
    Ok(())
}

fn ensure_config_exists() {
    if Path::new("warden.toml").exists() {
        return;
    }
    let project = warden_core::project::ProjectType::detect();
    let content =
        warden_core::project::generate_toml(project, warden_core::project::Strictness::Standard);
    if fs::write("warden.toml", &content).is_ok() {
        eprintln!("{}", "📝 Created warden.toml".dimmed());
    }
}

fn handle_prompt(copy: bool) -> Result<()> {
    let mut config = Config::new();
    config.load_local_config();
    let gen = PromptGenerator::new(config.rules.clone());
    let prompt = gen.generate()?;
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

    let Some(commands) = config.commands.get(name) else {
        eprintln!(
            "{} No '{}' command configured in warden.toml",
            "error:".red(),
            name
        );
        process::exit(1);
    };

    println!("{} Running '{}' pipeline...", "🚀".green(), name);
    execute_command_list(commands)
}

fn execute_command_list(commands: &[String]) -> Result<()> {
    for cmd_str in commands {
        execute_single_command(cmd_str)?;
    }
    Ok(())
}

fn execute_single_command(cmd_str: &str) -> Result<()> {
    println!("   {} {}", "exec:".dimmed(), cmd_str.dimmed());
    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let (prog, args) = parts.split_first().unwrap_or((&"", &[]));

    match Command::new(prog).args(args).status() {
        Ok(s) if s.success() => Ok(()),
        Ok(s) => exit_with_failure(s.code().unwrap_or(1)),
        Err(e) => exit_with_exec_error(&e, prog),
    }
}

fn exit_with_failure(code: i32) -> Result<()> {
    eprintln!("{} Command failed with exit code {code}", "❌".red());
    process::exit(code);
}

fn exit_with_exec_error(e: &io::Error, prog: &str) -> Result<()> {
    if e.kind() == io::ErrorKind::NotFound {
        eprintln!("{} Command not found: {prog}", "error:".red());
        eprintln!("  Check that the program is installed and in PATH");
    } else {
        eprintln!("{} Failed to execute: {e}", "error:".red());
    }
    process::exit(1);
}

fn run_scan() -> Result<()> {
    let config = load_config();
    let files = discovery::discover(&config)?;
    let report = scan_files(&config, files);
    reporting::print_report(&report)?;
    if report.has_errors() {
        process::exit(1);
    }
    Ok(())
}

fn run_tui() -> Result<()> {
    let config = load_config();
    let files = discovery::discover(&config)?;
    let report = scan_files(&config, files);
    run_tui_with_report(report)
}

fn load_config() -> Config {
    let mut config = Config::new();
    config.load_local_config();
    config
}

fn scan_files(config: &Config, files: Vec<PathBuf>) -> ScanReport {
    RuleEngine::new(config.clone()).scan(files)
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
