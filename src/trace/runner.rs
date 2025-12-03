// src/trace/runner.rs
//! Trace command runner.

use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;

use super::options::TraceOptions;
use super::output;
use crate::config::Config;
use crate::discovery;
use crate::graph::rank::RepoGraph;

/// Result of tracing dependencies.
pub struct TraceResult {
    pub anchor: PathBuf,
    pub direct: Vec<PathBuf>,
    pub indirect: Vec<PathBuf>,
    pub output: String,
    pub tokens: usize,
}

/// Runs the trace command.
///
/// # Errors
/// Returns error if anchor doesn't exist or file operations fail.
pub fn run(opts: &TraceOptions) -> Result<String> {
    if !opts.anchor.exists() {
        anyhow::bail!("Anchor file not found: {}", opts.anchor.display());
    }

    let config = load_config();
    let files = discovery::discover(&config)?;
    let contents = read_all_files(&files);

    let file_vec: Vec<_> = contents
        .iter()
        .map(|(p, c)| (p.clone(), c.clone()))
        .collect();

    let mut graph = RepoGraph::build(&file_vec);
    graph.focus_on(&opts.anchor);

    let direct = graph.neighbors(&opts.anchor);
    let indirect = collect_indirect(&graph, &opts.anchor, &direct);

    Ok(output::render(&opts.anchor, &direct, &indirect, &contents))
}

/// Shows repository structure map.
///
/// # Errors
/// Returns error if discovery fails.
pub fn map() -> Result<String> {
    let config = load_config();
    let files = discovery::discover(&config)?;

    let mut out = String::from("# Repository Map\n\n");
    for (dir, dir_files) in &group_by_directory(&files) {
        write_dir_section(&mut out, dir, dir_files);
    }

    Ok(out)
}

fn load_config() -> Config {
    let mut config = Config::new();
    config.load_local_config();
    config
}

fn read_all_files(files: &[PathBuf]) -> HashMap<PathBuf, String> {
    files
        .iter()
        .filter_map(|p| fs::read_to_string(p).ok().map(|c| (p.clone(), c)))
        .collect()
}

fn collect_indirect(graph: &RepoGraph, anchor: &Path, direct: &[PathBuf]) -> Vec<PathBuf> {
    graph
        .ranked_files()
        .into_iter()
        .filter(|(p, _)| p != anchor && !direct.contains(p))
        .take(10)
        .map(|(p, _)| p)
        .collect()
}

fn group_by_directory(files: &[PathBuf]) -> BTreeMap<PathBuf, Vec<PathBuf>> {
    let mut dirs: BTreeMap<PathBuf, Vec<PathBuf>> = BTreeMap::new();
    for file in files {
        let dir = file.parent().unwrap_or(Path::new(".")).to_path_buf();
        dirs.entry(dir).or_default().push(file.clone());
    }
    dirs
}

fn write_dir_section(out: &mut String, dir: &Path, files: &[PathBuf]) {
    let _ = writeln!(out, "{}/ ({} files)", dir.display(), files.len());
    for f in files.iter().take(5) {
        let name = f.file_name().unwrap_or_default().to_string_lossy();
        let _ = writeln!(out, "  └── {name}");
    }
    if files.len() > 5 {
        let _ = writeln!(out, "  └── ... and {} more", files.len() - 5);
    }
}

