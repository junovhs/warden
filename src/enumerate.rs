// src/enumerate.rs
use crate::config::{Config, GitMode};
use crate::constants::should_prune;
use crate::error::{Result, WardenError};
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

pub struct FileEnumerator {
    config: Config,
}

impl FileEnumerator {
    #[must_use]
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Enumerates files based on configuration.
    /// # Errors
    /// Returns error if git mode is Yes but not in a git repo.
    pub fn enumerate(&self) -> Result<Vec<PathBuf>> {
        match self.config.git_mode {
            GitMode::Yes => Self::enumerate_git_required(),
            GitMode::No => Ok(self.walk_filesystem()),
            GitMode::Auto => Ok(self.enumerate_auto()),
        }
    }

    fn enumerate_git_required() -> Result<Vec<PathBuf>> {
        if !in_git_repo() {
            return Err(WardenError::NotInGitRepo);
        }
        git_ls_files().map(filter_pruned)
    }

    fn enumerate_auto(&self) -> Vec<PathBuf> {
        if !in_git_repo() {
            return self.walk_filesystem();
        }
        git_ls_files().map_or_else(|_| self.walk_filesystem(), filter_pruned)
    }

    fn walk_filesystem(&self) -> Vec<PathBuf> {
        let walker = WalkDir::new(".")
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| !should_prune(&e.file_name().to_string_lossy()));

        collect_files(walker, self.config.verbose)
    }
}

fn in_git_repo() -> bool {
    Command::new("git")
        .args(["rev-parse", "--is-inside-work-tree"])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn git_ls_files() -> Result<Vec<PathBuf>> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "--exclude-standard", "."])
        .output()?;

    if !out.status.success() {
        return Err(WardenError::Other(format!(
            "git ls-files failed: {}",
            out.status
        )));
    }

    let paths = out
        .stdout
        .split(|&b| b == 0)
        .filter(|chunk| !chunk.is_empty())
        .map(|chunk| PathBuf::from(String::from_utf8_lossy(chunk).as_ref()))
        .collect();

    Ok(paths)
}

fn filter_pruned(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths
        .into_iter()
        .filter(|p| !contains_pruned_component(p))
        .collect()
}

fn contains_pruned_component(path: &Path) -> bool {
    path.components()
        .filter_map(|c| c.as_os_str().to_str())
        .any(should_prune)
}

fn collect_files<I>(walker: I, verbose: bool) -> Vec<PathBuf>
where
    I: Iterator<Item = walkdir::Result<walkdir::DirEntry>>,
{
    let mut paths = Vec::new();
    let mut error_count = 0;

    for item in walker {
        match item {
            Ok(entry) if entry.file_type().is_file() => {
                let p = entry.path().strip_prefix(".").unwrap_or(entry.path());
                paths.push(p.to_path_buf());
            }
            Err(_) => error_count += 1,
            _ => {}
        }
    }

    if error_count > 0 && verbose {
        eprintln!("WARN: Encountered {error_count} errors during file walk");
    }

    paths
}
