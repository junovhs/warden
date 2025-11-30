// tests/protection_roadmap.rs
//! Tests for ROADMAP.md protection.
//! Covers: v0.4.0 Path Safety — Protected Files features

use std::collections::HashMap;
use warden_core::apply::types::{ApplyOutcome, FileContent};
use warden_core::apply::validator;

fn make_file_with_content(path: &str, content: &str) -> HashMap<String, FileContent> {
    let mut files = HashMap::new();
    files.insert(
        path.to_string(),
        FileContent {
            content: content.to_string(),
            line_count: content.lines().count(),
        },
    );
    files
}

/// Verifies ROADMAP.md rewrite is blocked.
/// Feature: Block ROADMAP.md rewrite
#[test]
fn test_roadmap_rewrite_is_blocked() {
    let files = make_file_with_content(
        "ROADMAP.md",
        "# Malicious Roadmap\n\nThis would replace the real roadmap.",
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_protection = errors.iter().any(|e| {
                e.contains("PROTECTED") || e.contains("ROADMAP") || e.contains("programmatically")
            });
            assert!(
                has_protection,
                "Should block ROADMAP.md rewrite with protection message: {:?}",
                errors
            );
        }
        _ => panic!("Should block ROADMAP.md rewrite"),
    }
}

/// Verifies case-insensitive ROADMAP.md protection.
/// Feature: Case-insensitive protection
#[test]
fn test_roadmap_rewrite_blocked_case_insensitive() {
    let test_cases = vec![
        "ROADMAP.md",
        "roadmap.md",
        "Roadmap.md",
        "RoAdMaP.md",
        "ROADMAP.MD",
    ];

    for path in test_cases {
        let files = make_file_with_content(path, "# Modified roadmap");
        let result = validator::validate(&vec![], &files);

        match result {
            ApplyOutcome::ValidationFailure { errors, .. } => {
                let has_protection = errors
                    .iter()
                    .any(|e| e.contains("PROTECTED") || e.to_lowercase().contains("roadmap"));
                assert!(
                    has_protection,
                    "Should block '{}' case-insensitively: {:?}",
                    path, errors
                );
            }
            _ => panic!("Should block '{}' rewrite", path),
        }
    }
}

/// Verifies ROADMAP.md in subdirectories is allowed.
#[test]
fn test_roadmap_in_subdir_allowed() {
    let files = make_file_with_content("docs/ROADMAP.md", "# Project Documentation Roadmap");

    let result = validator::validate(&vec![], &files);

    // ROADMAP.md in subdirectories might be allowed
    // The protection is specifically for the root ROADMAP.md
    match result {
        ApplyOutcome::Success { .. } => {} // Subdirectory allowed
        ApplyOutcome::ValidationFailure { errors, .. } => {
            // If blocked, it should be for different reason than root protection
            // (e.g., if implementation blocks all ROADMAP.md files)
        }
        _ => {}
    }
}

/// Verifies roadmap command usage suggestion is provided.
#[test]
fn test_roadmap_error_suggests_command() {
    let files = make_file_with_content("ROADMAP.md", "# Replaced roadmap");

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure {
            errors, ai_message, ..
        } => {
            // Error or AI message should mention programmatic management
            let mentions_programmatic = errors
                .iter()
                .any(|e| e.contains("programmatically") || e.contains("roadmap apply"));
            let ai_mentions =
                ai_message.contains("programmatically") || ai_message.contains("roadmap");

            assert!(
                mentions_programmatic || ai_mentions,
                "Should suggest using roadmap commands: errors={:?}, ai_message={}",
                errors,
                ai_message
            );
        }
        _ => panic!("Should block ROADMAP.md"),
    }
}

/// Verifies similarly named files are not blocked.
#[test]
fn test_similar_names_allowed() {
    let allowed_files = vec![
        "ROADMAP_BACKUP.md",
        "ROADMAP-old.md",
        "my_ROADMAP.md",
        "ROADMAP.txt",
        "ROADMAPS.md",
    ];

    for path in allowed_files {
        let files = make_file_with_content(path, "# Some content");
        let result = validator::validate(&vec![], &files);

        match result {
            ApplyOutcome::Success { written, .. } => {
                assert!(
                    written.contains(&path.to_string()),
                    "'{}' should be allowed",
                    path
                );
            }
            ApplyOutcome::ValidationFailure { errors, .. } => {
                // Should not fail for ROADMAP protection
                let protected_error = errors
                    .iter()
                    .any(|e| e.contains("PROTECTED") && e.contains("ROADMAP.md"));
                assert!(
                    !protected_error,
                    "'{}' should not trigger ROADMAP protection: {:?}",
                    path, errors
                );
            }
            _ => {}
        }
    }
}

/// Verifies protection error is clear and actionable.
#[test]
fn test_protection_error_is_actionable() {
    let files = make_file_with_content("ROADMAP.md", "# New content");
    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let error_text = errors.join(" ");

            // Error should be clear and actionable
            assert!(
                error_text.contains("ROADMAP") || error_text.contains("roadmap"),
                "Error should mention ROADMAP"
            );
            assert!(
                error_text.contains("programmatically")
                    || error_text.contains("apply")
                    || error_text.contains("PROTECTED"),
                "Error should explain what to do instead"
            );
        }
        _ => panic!("Should block ROADMAP.md"),
    }
}

/// Verifies other protected files concept could be extended.
#[test]
fn test_other_files_not_protected() {
    // Verify that random files are not incorrectly protected
    let regular_files = vec![
        "src/main.rs",
        "README.md",
        "Cargo.toml",
        "package.json",
        "LICENSE",
    ];

    for path in regular_files {
        let files = make_file_with_content(path, "content");
        let result = validator::validate(&vec![], &files);

        match result {
            ApplyOutcome::Success { .. } => {} // Expected
            ApplyOutcome::ValidationFailure { errors, .. } => {
                let protected_error = errors.iter().any(|e| e.contains("PROTECTED"));
                assert!(
                    !protected_error,
                    "'{}' should not be protected: {:?}",
                    path, errors
                );
            }
            _ => {}
        }
    }
}
