use std::collections::HashMap;
use warden_core::apply::types::{ApplyOutcome, ExtractedFiles, FileContent, ManifestEntry, Operation};
use warden_core::apply::validator;

#[test]
fn test_roadmap_rewrite_is_blocked() {
    // 1. Setup a manifest trying to overwrite ROADMAP.md
    let path = "ROADMAP.md".to_string();
    let mut files = HashMap::new();
    files.insert(
        path.clone(),
        FileContent {
            content: "# Hacked Roadmap".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: path.clone(),
        operation: Operation::Update,
    }];

    // 2. Validate
    let outcome = validator::validate(&manifest, &files);

    // 3. Assert failure
    match outcome {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            assert!(
                errors.iter().any(|e| e.contains("PROTECTED: ROADMAP.md")),
                "Expected protection error, got: {errors:?}"
            );
        }
        _ => panic!("ROADMAP.md should have been blocked, got: {outcome:?}"),
    }
}

#[test]
fn test_roadmap_rewrite_blocked_case_insensitive() {
    // 1. Setup a manifest trying to overwrite roadmap.md (lowercase)
    let path = "roadmap.md".to_string();
    let mut files = HashMap::new();
    files.insert(
        path.clone(),
        FileContent {
            content: "# lowercase map".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: path.clone(),
        operation: Operation::Update,
    }];

    // 2. Validate
    let outcome = validator::validate(&manifest, &files);

    // 3. Assert failure
    match outcome {
        ApplyOutcome::ValidationFailure { errors, .. } => {
            assert!(
                errors.iter().any(|e| e.contains("PROTECTED: ROADMAP.md")),
                "Expected protection error for lowercase path, got: {errors:?}"
            );
        }
        _ => panic!("roadmap.md should have been blocked, got: {outcome:?}"),
    }
}

#[test]
fn test_other_files_allowed() {
    // 1. Setup a manifest for a normal file
    let path = "src/main.rs".to_string();
    let mut files = HashMap::new();
    files.insert(
        path.clone(),
        FileContent {
            content: "fn main() {}".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: path.clone(),
        operation: Operation::Update,
    }];

    // 2. Validate
    let outcome = validator::validate(&manifest, &files);

    // 3. Assert success
    match outcome {
        ApplyOutcome::Success { .. } => {} // OK
        _ => panic!("Normal file should be allowed, got: {outcome:?}"),
    }
}