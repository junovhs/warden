// tests/unit_validator.rs
//! Unit tests for content validation.
//! Covers: v0.4.0 Truncation Detection and Path Safety features

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

// =============================================================================
// TRUNCATION DETECTION
// =============================================================================

/// Verifies block comment ellipsis is detected.
/// Feature: Pattern: /* ... */
#[test]
fn test_block_comment_ellipsis() {
    let content = "fn main() {\n    /* ... */\n}";
    let files = make_file_with_content("test.rs", content);

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_truncation = errors
                .iter()
                .any(|e| e.contains("truncation") || e.contains("..."));
            assert!(has_truncation, "Should detect /* ... */ as truncation");
        }
        _ => panic!("Should fail for block comment ellipsis"),
    }
}

/// Verifies hash-style ellipsis is detected.
/// Feature: Pattern: # ...
#[test]
fn test_hash_ellipsis() {
    let content = "def main():\n    # ...\n    pass";
    let files = make_file_with_content("test.py", content);

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_truncation = errors
                .iter()
                .any(|e| e.contains("truncation") || e.contains("..."));
            assert!(has_truncation, "Should detect # ... as truncation");
        }
        _ => panic!("Should fail for hash ellipsis"),
    }
}

/// Verifies "rest of" phrases are detected.
/// Feature: Pattern: "rest of" phrases
#[test]
fn test_lazy_phrase_rest_of() {
    let content = "fn main() {\n    // rest of implementation here\n}";
    let files = make_file_with_content("test.rs", content);

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_lazy = errors
                .iter()
                .any(|e| e.contains("truncation") || e.contains("rest of"));
            assert!(has_lazy, "Should detect 'rest of' phrase: {:?}", errors);
        }
        _ => panic!("Should fail for 'rest of' phrase"),
    }
}

/// Verifies "remaining" phrases are detected.
/// Feature: Pattern: "remaining" phrases
#[test]
fn test_lazy_phrase_remaining() {
    let content = "fn main() {\n    // remaining logic here\n}";
    let files = make_file_with_content("test.rs", content);

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_lazy = errors
                .iter()
                .any(|e| e.contains("truncation") || e.contains("remaining"));
            assert!(has_lazy, "Should detect 'remaining' phrase: {:?}", errors);
        }
        _ => panic!("Should fail for 'remaining' phrase"),
    }
}

/// Verifies line number is reported in error.
/// Feature: Line number in error
#[test]
fn test_line_number_reported() {
    let content = "line1\nline2\nline3\n// ...\nline5";
    let files = make_file_with_content("test.rs", content);

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            // Error should include line number (line 4)
            let has_line_num = errors
                .iter()
                .any(|e| e.contains(":4:") || e.contains("line 4") || e.contains(":4"));
            assert!(
                has_line_num,
                "Should report line number in error: {:?}",
                errors
            );
        }
        _ => panic!("Should fail for truncation marker"),
    }
}

// =============================================================================
// SENSITIVE PATHS - SPECIFIC
// =============================================================================

/// Verifies .gnupg/ is blocked.
/// Feature: Block .gnupg/
#[test]
fn test_gnupg_blocked() {
    let files = make_file_with_content(".gnupg/pubring.gpg", "fake key");
    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_security = errors
                .iter()
                .any(|e| e.contains("SECURITY") || e.contains("sensitive") || e.contains("hidden"));
            assert!(has_security, "Should block .gnupg: {:?}", errors);
        }
        _ => panic!("Should block .gnupg path"),
    }
}

/// Verifies id_rsa is blocked.
/// Feature: Block id_rsa
#[test]
fn test_id_rsa_blocked() {
    let files = make_file_with_content("id_rsa", "fake private key");
    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_security = errors
                .iter()
                .any(|e| e.contains("SECURITY") || e.contains("sensitive") || e.contains("id_rsa"));
            assert!(has_security, "Should block id_rsa: {:?}", errors);
        }
        _ => panic!("Should block id_rsa"),
    }
}

/// Verifies credentials files are blocked.
/// Feature: Block credentials
#[test]
fn test_credentials_blocked() {
    let files = make_file_with_content("credentials", "aws_access_key=xxx");
    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_security = errors.iter().any(|e| {
                e.contains("SECURITY") || e.contains("sensitive") || e.contains("credentials")
            });
            assert!(has_security, "Should block credentials file: {:?}", errors);
        }
        _ => panic!("Should block credentials"),
    }
}

/// Verifies backup directory writes are blocked.
/// Feature: Block backup directory
#[test]
fn test_backup_dir_blocked() {
    let files = make_file_with_content(".warden_apply_backup/123/file.rs", "malicious");
    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let has_security = errors
                .iter()
                .any(|e| e.contains("SECURITY") || e.contains("backup") || e.contains("hidden"));
            assert!(has_security, "Should block backup directory: {:?}", errors);
        }
        _ => panic!("Should block backup directory writes"),
    }
}

// =============================================================================
// VALID CONTENT
// =============================================================================

/// Verifies normal code passes validation.
#[test]
fn test_valid_code_passes() {
    let content = r#"
fn main() {
    let x = 42;
    println!("The answer is {}", x);
}
"#;
    let files = make_file_with_content("src/main.rs", content);

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::Success { .. } => {} // Expected
        ApplyOutcome::ValidationFailure { errors, .. } => {
            panic!("Valid code should pass: {:?}", errors);
        }
        _ => panic!("Unexpected result"),
    }
}

/// Verifies actual ellipsis in strings is allowed.
#[test]
fn test_ellipsis_in_string_allowed() {
    let content = r#"
fn main() {
    let message = "Loading... please wait";
    println!("{}", message);
}
"#;
    let files = make_file_with_content("src/main.rs", content);

    let result = validator::validate(&vec![], &files);

    // Ellipsis in string literals should be allowed
    match result {
        ApplyOutcome::Success { .. } => {} // Expected
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let truncation_errors: Vec<_> =
                errors.iter().filter(|e| e.contains("truncation")).collect();
            // If there are errors, they shouldn't be truncation errors for string content
            assert!(
                truncation_errors.is_empty() || errors.iter().any(|e| e.contains("warden:ignore")),
                "Ellipsis in string should be allowed: {:?}",
                errors
            );
        }
        _ => {}
    }
}

/// Verifies comments about implementation are allowed when not lazy patterns.
#[test]
fn test_normal_comments_allowed() {
    let content = r#"
fn main() {
    // This function does X
    // It was implemented on 2024-01-01
    println!("Hello");
}
"#;
    let files = make_file_with_content("src/main.rs", content);

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::Success { .. } => {} // Expected
        ApplyOutcome::ValidationFailure { errors, .. } => {
            panic!("Normal comments should pass: {:?}", errors);
        }
        _ => panic!("Unexpected result"),
    }
}

/// Verifies warden:ignore inline bypass.
#[test]
fn test_warden_ignore_inline() {
    let content = "fn main() {\n    // ... warden:ignore (intentional placeholder)\n}";
    let files = make_file_with_content("src/main.rs", content);

    let result = validator::validate(&vec![], &files);

    // Should pass due to warden:ignore
    match result {
        ApplyOutcome::Success { .. } => {} // Expected
        ApplyOutcome::ValidationFailure { errors, .. } => {
            let truncation_errors: Vec<_> = errors
                .iter()
                .filter(|e| e.contains("truncation") && !e.contains("warden:ignore"))
                .collect();
            assert!(
                truncation_errors.is_empty(),
                "warden:ignore should bypass: {:?}",
                errors
            );
        }
        _ => {}
    }
}

/// Verifies multiple files are all validated.
#[test]
fn test_multiple_files_validated() {
    let mut files = HashMap::new();
    files.insert(
        "good.rs".to_string(),
        FileContent {
            content: "fn good() {}".to_string(),
            line_count: 1,
        },
    );
    files.insert(
        "bad.rs".to_string(),
        FileContent {
            content: "fn bad() {\n    // ...\n}".to_string(),
            line_count: 3,
        },
    );

    let result = validator::validate(&vec![], &files);

    match result {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            // Should catch the bad file
            let has_bad_error = errors.iter().any(|e| e.contains("bad.rs"));
            assert!(
                has_bad_error,
                "Should report error for bad.rs: {:?}",
                errors
            );
        }
        _ => panic!("Should fail due to bad.rs"),
    }
}
