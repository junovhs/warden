// src/reporting.rs
use crate::types::{FileReport, ScanReport, Violation};
use anyhow::Result;
use colored::Colorize;

/// Prints the scan report to stdout.
///
/// # Errors
/// Returns Ok(()) normally.
pub fn print_report(report: &ScanReport) -> Result<()> {
    let failures = count_failures(report);

    // Filter and print only violating files
    report
        .files
        .iter()
        .filter(|f| !f.is_clean())
        .for_each(print_file_report);

    print_summary(report, failures);
    Ok(())
}

fn count_failures(report: &ScanReport) -> usize {
    report
        .files
        .iter()
        .filter(|f| !f.is_clean())
        .map(|f| f.violations.len())
        .sum()
}

fn print_file_report(file: &FileReport) {
    for v in &file.violations {
        print_violation(&file.path, v);
    }
}

fn print_violation(path: &std::path::Path, v: &Violation) {
    let filename = path.to_string_lossy();
    let line_num = v.row + 1;

    println!("{}: {}", "error".red().bold(), v.message.bold());
    println!("  {} {}:{}:1", "-->".blue(), filename, line_num);
    println!("   {}", "|".blue());
    println!(
        "   {} {}: Action required",
        "=".blue().bold(),
        v.law.white().bold()
    );
    println!();
}

fn print_summary(report: &ScanReport, failures: usize) {
    if failures > 0 {
        let msg = format!(
            "❌ Warden found {failures} violations in {}ms.",
            report.duration_ms
        );
        println!("{}", msg.red().bold());
    } else {
        let msg = format!(
            "✅ All Clear. Scanned {} tokens in {}ms.",
            report.total_tokens, report.duration_ms
        );
        println!("{}", msg.green().bold());
    }
}
