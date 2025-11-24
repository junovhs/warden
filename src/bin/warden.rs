use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::fs;
use std::process;

use warden_core::config::{Config, GitMode};
use warden_core::detection::Detector;
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;
use warden_core::rules::RuleEngine;

const DEFAULT_TOML: &str = r#"# warden.toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 10
max_nesting_depth = 4
max_function_args = 5
max_function_words = 3
ignore_naming_on = ["tests", "spec"]
"#;

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
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if cli.init {
        return handle_init();
    }

    let config = initialize_config(&cli)?;
    let target_files = run_scan(&config)?;

    if target_files.is_empty() {
        println!("No files to scan.");
        return Ok(());
    }

    println!(
        "👮 Warden scanning {} files (AST + Token Analysis)...",
        target_files.len()
    );

    let engine = RuleEngine::new(config);
    let mut total_failures = 0;

    for path in target_files {
        if let Ok(passed) = engine.check_file(&path) {
            if !passed {
                total_failures += 1;
            }
        }
    }

    if total_failures > 0 {
        println!(
            "{}",
            format!("❌ Warden found {total_failures} violations.")
                .red()
                .bold()
        );
        process::exit(1);
    } else {
        println!(
            "{}",
            "✅ All Clear. Code structure is clean.".green().bold()
        );
        process::exit(0);
    }
}

fn handle_init() -> Result<()> {
    if std::path::Path::new("warden.toml").exists() {
        println!("{}", "⚠️ warden.toml already exists.".yellow());
    } else {
        fs::write("warden.toml", DEFAULT_TOML)?;
        println!("{}", "✅ Created warden.toml".green());
    }
    Ok(())
}

fn initialize_config(cli: &Cli) -> Result<Config> {
    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;

    if cli.git_only {
        config.git_mode = GitMode::Yes;
    } else if cli.no_git {
        config.git_mode = GitMode::No;
    }

    config.load_local_config();
    config.validate()?;
    Ok(config)
}

fn run_scan(config: &Config) -> Result<Vec<std::path::PathBuf>> {
    let enumerator = FileEnumerator::new(config.clone());
    let raw_files = enumerator.enumerate()?;

    // Optional: Detect ecosystem (informational)
    let detector = Detector::new();
    if let Ok(systems) = detector.detect_build_systems(&raw_files) {
        if !systems.is_empty() && config.verbose {
            let sys_list: Vec<String> = systems.iter().map(ToString::to_string).collect();
            println!("🔎 Detected Ecosystem: [{}]", sys_list.join(", ").cyan());
        }
    }

    let heuristic_filter = HeuristicFilter::new();
    let heuristics_files = heuristic_filter.filter(raw_files);

    let filter = FileFilter::new(config.clone())?;
    Ok(filter.filter(heuristics_files))
}
