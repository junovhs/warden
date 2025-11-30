// src/roadmap/audit.rs
use crate::roadmap::slugify;
use crate::roadmap::types::{Roadmap, Task, TaskStatus};
use colored::Colorize;
use std::fs;
use std::path::Path;
use walkdir::{DirEntry, WalkDir};

pub struct AuditOptions {
    pub strict: bool,
}

pub fn run(roadmap: &Roadmap, root: &Path, _opts: AuditOptions) {
    println!("{}", "🕵️  Roadmap Traceability Audit".bold().cyan());
    println!("{}", "─────────────────────────────────────".dimmed());

    let tasks = roadmap.all_tasks();
    let completed: Vec<&&Task> = tasks
        .iter()
        .filter(|t| t.status == TaskStatus::Complete)
        .collect();

    if completed.is_empty() {
        println!("{}", "No completed tasks to audit.".yellow());
        return;
    }

    // Heuristic scan for un-anchored tasks
    let scanned_test_files = scan_test_files(root);
    let mut missing_count = 0;

    for task in completed {
        // Skip if marked as [no-test]
        if task.text.contains("[no-test]") {
            continue;
        }

        if !verify_task(task, root, &scanned_test_files) {
            print_missing(task);
            missing_count += 1;
        }
    }

    print_summary(missing_count);
}

fn verify_task(task: &Task, root: &Path, scanned_files: &[String]) -> bool {
    // 1. Priority: Explicit Anchors
    if !task.tests.is_empty() {
        return task.tests.iter().all(|t| verify_anchor(t, root));
    }

    // 2. Fallback: Slug Heuristic
    let slug = slugify(&task.text).replace('-', "_");
    let id_slug = task.id.replace('-', "_");

    scanned_files
        .iter()
        .any(|f| f.contains(&slug) || f.contains(&id_slug))
}

fn verify_anchor(anchor: &str, root: &Path) -> bool {
    // Support "path/to/file.rs::function_name" syntax
    let (file_part, fn_part) = if let Some((f, n)) = anchor.split_once("::") {
        (f, Some(n))
    } else {
        (anchor, None)
    };

    let path = root.join(file_part.trim());
    
    if !path.exists() || !path.is_file() {
        return false;
    }

    // If function name is specified, verify it exists in the file content
    if let Some(func_name) = fn_part {
        if let Ok(content) = fs::read_to_string(&path) {
            return content.contains(func_name.trim());
        }
        return false; // Could not read file
    }

    true
}

fn print_missing(task: &Task) {
    println!(
        "{} {} (id: {})",
        "⚠️  Missing Test:".red(),
        task.text.bold(),
        task.id.dimmed()
    );
}

fn print_summary(missing: usize) {
    println!();
    if missing == 0 {
        println!("{}", "✅ All completed tasks have verified tests!".green().bold());
    } else {
        println!(
            "{}",
            format!("❌ Found {missing} tasks without verified tests.").red().bold()
        );
        println!("   (Tip: Add <!-- test: tests/my_test.rs --> to the task in ROADMAP.md)");
    }
}

fn scan_test_files(root: &Path) -> Vec<String> {
    WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !is_ignored_dir(e))
        .flatten()
        .filter(is_heuristic_match)
        .filter_map(|e| e.path().to_str().map(str::to_lowercase))
        .collect()
}

fn is_ignored_dir(entry: &DirEntry) -> bool {
    let name = entry.file_name().to_str().unwrap_or("");
    name.starts_with('.') || name == "target" || name == "node_modules" || name == "vendor"
}

/// Strict filter for the heuristic scanner.
/// Only picks up files that explicitly look like tests.
fn is_heuristic_match(entry: &DirEntry) -> bool {
    if !entry.file_type().is_file() {
        return false;
    }
    
    if !has_code_extension(entry.path()) {
        return false;
    }

    let Some(name) = entry.file_name().to_str() else {
        return false;
    };

    name.contains("test")
        || name.contains("spec")
        || entry.path().components().any(|c| c.as_os_str() == "tests")
}

fn has_code_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|s| s.to_str())
        .is_some_and(|ext| {
            matches!(
                ext.to_ascii_lowercase().as_str(),
                "rs" | "ts" | "js" | "py" | "go"
            )
        })
}