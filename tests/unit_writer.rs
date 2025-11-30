// tests/unit_writer.rs
//! Unit tests for file writing functionality.
//! Covers: v0.3.0 File Writing features

use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use warden_core::apply::types::{ApplyOutcome, FileContent, ManifestEntry, Operation};
use warden_core::apply::writer;

fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Verifies parent directories are created automatically.
/// Feature: Parent directory creation
#[test]
fn test_creates_parent_dirs() {
    let dir = setup_temp_dir();

    let mut files = HashMap::new();
    files.insert(
        "src/deep/nested/file.rs".to_string(),
        FileContent {
            content: "fn test() {}".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "src/deep/nested/file.rs".to_string(),
        operation: Operation::New,
    }];

    let result = writer::write_files(&manifest, &files, Some(dir.path()));

    assert!(result.is_ok(), "Should succeed");

    let file_path = dir.path().join("src/deep/nested/file.rs");
    assert!(
        file_path.exists(),
        "Should create nested directory structure"
    );
}

/// Verifies file content is written correctly.
/// Feature: File content writing
#[test]
fn test_writes_content() {
    let dir = setup_temp_dir();

    let expected_content = "fn hello() {\n    println!(\"world\");\n}";

    let mut files = HashMap::new();
    files.insert(
        "test.rs".to_string(),
        FileContent {
            content: expected_content.to_string(),
            line_count: 3,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "test.rs".to_string(),
        operation: Operation::New,
    }];

    writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    let written = fs::read_to_string(dir.path().join("test.rs")).expect("Should read");
    assert_eq!(written, expected_content, "Content should match exactly");
}

/// Verifies delete operation removes files.
/// Feature: Delete operation
#[test]
fn test_delete_file() {
    let dir = setup_temp_dir();

    // Create a file to delete
    let file_path = dir.path().join("to_delete.rs");
    fs::write(&file_path, "fn old() {}").expect("Should create file");
    assert!(file_path.exists(), "File should exist before delete");

    let files = HashMap::new();
    let manifest = vec![ManifestEntry {
        path: "to_delete.rs".to_string(),
        operation: Operation::Delete,
    }];

    writer::write_files(&manifest, &files, Some(dir.path())).expect("Should succeed");

    assert!(!file_path.exists(), "File should be deleted");
}

/// Verifies written files are tracked in result.
/// Feature: Written files tracking
#[test]
fn test_tracks_written() {
    let dir = setup_temp_dir();

    let mut files = HashMap::new();
    files.insert(
        "file1.rs".to_string(),
        FileContent {
            content: "fn one() {}".to_string(),
            line_count: 1,
        },
    );
    files.insert(
        "file2.rs".to_string(),
        FileContent {
            content: "fn two() {}".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![
        ManifestEntry {
            path: "file1.rs".to_string(),
            operation: Operation::New,
        },
        ManifestEntry {
            path: "file2.rs".to_string(),
            operation: Operation::New,
        },
    ];

    let result = writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    match result {
        ApplyOutcome::Success { written, .. } => {
            assert!(written.contains(&"file1.rs".to_string()));
            assert!(written.contains(&"file2.rs".to_string()));
        }
        _ => panic!("Expected success"),
    }
}

/// Verifies deleted files are tracked in result.
#[test]
fn test_tracks_deleted() {
    let dir = setup_temp_dir();

    // Create files to delete
    fs::write(dir.path().join("delete1.rs"), "old").unwrap();
    fs::write(dir.path().join("delete2.rs"), "old").unwrap();

    let files = HashMap::new();
    let manifest = vec![
        ManifestEntry {
            path: "delete1.rs".to_string(),
            operation: Operation::Delete,
        },
        ManifestEntry {
            path: "delete2.rs".to_string(),
            operation: Operation::Delete,
        },
    ];

    let result = writer::write_files(&manifest, &files, Some(dir.path())).expect("Should succeed");

    match result {
        ApplyOutcome::Success { deleted, .. } => {
            assert!(deleted.contains(&"delete1.rs".to_string()));
            assert!(deleted.contains(&"delete2.rs".to_string()));
        }
        _ => panic!("Expected success"),
    }
}

/// Verifies update operation overwrites existing file.
#[test]
fn test_update_overwrites() {
    let dir = setup_temp_dir();

    // Create existing file
    let file_path = dir.path().join("existing.rs");
    fs::write(&file_path, "fn old() {}").unwrap();

    let mut files = HashMap::new();
    files.insert(
        "existing.rs".to_string(),
        FileContent {
            content: "fn new() {}".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "existing.rs".to_string(),
        operation: Operation::Update,
    }];

    writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    let content = fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("fn new()"), "Should have new content");
    assert!(!content.contains("fn old()"), "Should not have old content");
}

/// Verifies mixed operations work together.
#[test]
fn test_mixed_operations() {
    let dir = setup_temp_dir();

    // Create file to update and delete
    fs::write(dir.path().join("update.rs"), "old").unwrap();
    fs::write(dir.path().join("delete.rs"), "to remove").unwrap();

    let mut files = HashMap::new();
    files.insert(
        "update.rs".to_string(),
        FileContent {
            content: "updated".to_string(),
            line_count: 1,
        },
    );
    files.insert(
        "create.rs".to_string(),
        FileContent {
            content: "new file".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![
        ManifestEntry {
            path: "update.rs".to_string(),
            operation: Operation::Update,
        },
        ManifestEntry {
            path: "create.rs".to_string(),
            operation: Operation::New,
        },
        ManifestEntry {
            path: "delete.rs".to_string(),
            operation: Operation::Delete,
        },
    ];

    let result = writer::write_files(&manifest, &files, Some(dir.path())).expect("Should succeed");

    // Verify all operations
    assert!(dir.path().join("update.rs").exists());
    assert!(dir.path().join("create.rs").exists());
    assert!(!dir.path().join("delete.rs").exists());

    match result {
        ApplyOutcome::Success {
            written, deleted, ..
        } => {
            assert_eq!(written.len(), 2);
            assert_eq!(deleted.len(), 1);
        }
        _ => panic!("Expected success"),
    }
}

/// Verifies deleting non-existent file doesn't error.
#[test]
fn test_delete_nonexistent_ok() {
    let dir = setup_temp_dir();

    let files = HashMap::new();
    let manifest = vec![ManifestEntry {
        path: "nonexistent.rs".to_string(),
        operation: Operation::Delete,
    }];

    // Should not error
    let result = writer::write_files(&manifest, &files, Some(dir.path()));
    assert!(result.is_ok(), "Deleting nonexistent file should not error");
}
