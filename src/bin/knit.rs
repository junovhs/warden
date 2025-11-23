use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::fmt::Write;
use std::fs;

use warden_core::config::{Config, GitMode};
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;
use warden_core::tokens::Tokenizer;

#[derive(Parser)]
#[command(name = "knit")]
#[command(about = "Stitches atomic files into a single context file.")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    #[arg(long, short)]
    stdout: bool,
    #[arg(long, short)]
    verbose: bool,
    #[arg(long)]
    git_only: bool,
    #[arg(long)]
    no_git: bool,
    #[arg(long)]
    code_only: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let mut config = Config::new();
    config.verbose = cli.verbose;
    config.code_only = cli.code_only;

    if cli.git_only {
        config.git_mode = GitMode::Yes;
    } else if cli.no_git {
        config.git_mode = GitMode::No;
    }

    config.load_ignore_file();
    config.validate()?;

    if !cli.stdout {
        println!("🧶 Knitting repository...");
    }

    let enumerator = FileEnumerator::new(config.clone());
    let raw_files = enumerator.enumerate()?;

    let heuristic_filter = HeuristicFilter::new();
    let heuristics_files = heuristic_filter.filter(raw_files);

    let filter = FileFilter::new(config.clone())?;
    let target_files = filter.filter(heuristics_files);

    if cli.verbose {
        eprintln!("📦 Packing {} files...", target_files.len());
    }

    let mut full_context = String::with_capacity(100_000);

    for path in &target_files {
        let path_str = path.to_string_lossy();

        writeln!(
            full_context,
            "================================================================================"
        )
        .unwrap();

        writeln!(full_context, "FILE: {path_str}").unwrap();

        writeln!(
            full_context,
            "================================================================================"
        )
        .unwrap();

        match fs::read_to_string(path) {
            Ok(content) => {
                full_context.push_str(&content);
            }
            Err(e) => {
                writeln!(full_context, "<ERROR READING FILE: {e}>").unwrap();
            }
        }
        full_context.push_str("\n\n");
    }

    // Count tokens
    let token_count = Tokenizer::count(&full_context);

    if cli.stdout {
        print!("{full_context}");
        eprintln!(
            "\n📊 Context Size: {} tokens",
            token_count.to_string().yellow().bold()
        );
    } else {
        fs::write("context.txt", &full_context)?;
        println!("✅ Generated 'context.txt'");
        println!(
            "📊 Context Size: {} tokens",
            token_count.to_string().yellow().bold()
        );
    }

    Ok(())
}
