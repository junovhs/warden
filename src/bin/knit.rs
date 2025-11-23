use anyhow::Result;
use clap::Parser;
use std::fs::{self, File};
use std::io::{self, BufWriter, Write};

use warden_core::config::{Config, GitMode};
use warden_core::enumerate::FileEnumerator;
use warden_core::filter::FileFilter;
use warden_core::heuristics::HeuristicFilter;

#[derive(Parser)]
#[command(name = "knit")]
#[command(about = "Stitches atomic files into a single context file.")]
#[allow(clippy::struct_excessive_bools)]
struct Cli {
    /// Output to stdout instead of context.txt
    #[arg(long, short)]
    stdout: bool,

    /// Verbose logging
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

    // NEW: Load the .wardenignore file to remove noise
    config.load_ignore_file();
    config.validate()?;

    if !cli.stdout {
        println!("🧶 Knitting repository...");
    }

    // 1. Discovery
    let enumerator = FileEnumerator::new(config.clone());
    let raw_files = enumerator.enumerate()?;

    // 2. Heuristics
    let heuristic_filter = HeuristicFilter::new();
    let heuristics_files = heuristic_filter.filter(raw_files);

    // 3. Filtering
    let filter = FileFilter::new(config.clone())?;
    let target_files = filter.filter(heuristics_files);

    if cli.verbose {
        eprintln!("📦 Packing {} files...", target_files.len());
    }

    // 4. Output Setup
    // We use Box<dyn Write> to switch between File and Stdout transparently
    let writer: Box<dyn Write> = if cli.stdout {
        Box::new(io::stdout())
    } else {
        Box::new(File::create("context.txt")?)
    };

    let mut buffer = BufWriter::new(writer);

    for path in target_files {
        let path_str = path.to_string_lossy();

        writeln!(
            buffer,
            "================================================================================"
        )?;
        writeln!(buffer, "FILE: {path_str}")?;
        writeln!(
            buffer,
            "================================================================================"
        )?;

        match fs::read_to_string(&path) {
            Ok(content) => {
                writeln!(buffer, "{content}")?;
            }
            Err(e) => {
                writeln!(buffer, "<ERROR READING FILE: {e}>")?;
            }
        }
        writeln!(buffer, "\n")?;
    }

    buffer.flush()?;

    if !cli.stdout {
        println!("✅ Generated 'context.txt'");
    }
    Ok(())
}
