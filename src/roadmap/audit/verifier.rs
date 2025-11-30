// src/roadmap/audit/verifier.rs
//! Checks that test anchors resolve to actual test functions.

use super::function_finder;
use super::types::{AuditResult, TaskAnchor, VerificationStatus};
use std::path::Path;

/// Verifies a single task anchor.
#[must_use]
pub fn verify_anchor(anchor: &TaskAnchor, project_root: &Path) -> AuditResult {
    let file_path = project_root.join(&anchor.file);

    if !file_path.exists() {
        return AuditResult {
            task_path: anchor.task_path.clone(),
            anchor: anchor.clone(),
            status: VerificationStatus::MissingFile,
            detail: Some(format!("File not found: {}", anchor.file)),
        };
    }

    match verify_function(&file_path, &anchor.function) {
        Ok(true) => AuditResult {
            task_path: anchor.task_path.clone(),
            anchor: anchor.clone(),
            status: VerificationStatus::Verified,
            detail: None,
        },
        Ok(false) => AuditResult {
            task_path: anchor.task_path.clone(),
            anchor: anchor.clone(),
            status: VerificationStatus::MissingFunction,
            detail: Some(format!(
                "Function '{}' not found in {}",
                anchor.function, anchor.file
            )),
        },
        Err(e) => AuditResult {
            task_path: anchor.task_path.clone(),
            anchor: anchor.clone(),
            status: VerificationStatus::ParseError,
            detail: Some(e),
        },
    }
}

fn verify_function(file_path: &Path, function_name: &str) -> Result<bool, String> {
    function_finder::function_exists(file_path, function_name)
}

/// Verifies multiple anchors and returns all results.
#[must_use]
pub fn verify_all(anchors: &[TaskAnchor], project_root: &Path) -> Vec<AuditResult> {
    anchors
        .iter()
        .map(|a| verify_anchor(a, project_root))
        .collect()
}

/// Returns only the failed verifications.
#[must_use]
pub fn filter_failures(results: &[AuditResult]) -> Vec<&AuditResult> {
    results
        .iter()
        .filter(|r| !matches!(r.status, VerificationStatus::Verified))
        .collect()
}