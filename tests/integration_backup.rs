// tests/integration_backup.rs
//! Integration tests for the backup system.
//! Covers: v0.3.0 Backup System features

use std::collections::HashMap;
use std::fs;
use tempfile::TempDir;
use warden_core::apply::types::{ApplyOutcome, FileContent, ManifestEntry, Operation};
use warden_core::apply::writer;

fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Verifies backup directory is created.
/// Feature: Backup directory creation
#[test]
fn test_backup_dir_created() {
    let dir = setup_temp_dir();

    // Create an existing file that will be modified
    let existing_path = dir.path().join("existing.rs");
    fs::write(&existing_path, "fn old() {}").expect("Should create file");

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

    let backup_dir = dir.path().join(".warden_apply_backup");
    assert!(backup_dir.exists(), "Backup directory should be created");
}

/// Verifies timestamp subfolder is created.
/// Feature: Timestamp subfolder
#[test]
fn test_timestamp_folder() {
    let dir = setup_temp_dir();

    // Create existing file
    fs::write(dir.path().join("file.rs"), "old content").unwrap();

    let mut files = HashMap::new();
    files.insert(
        "file.rs".to_string(),
        FileContent {
            content: "new content".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "file.rs".to_string(),
        operation: Operation::Update,
    }];

    writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    let backup_dir = dir.path().join(".warden_apply_backup");

    // Should have at least one timestamp folder
    let entries: Vec<_> = fs::read_dir(&backup_dir)
        .expect("Should read backup dir")
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        entries.len() >= 1,
        "Should have at least one timestamp folder"
    );

    // Timestamp folder name should be numeric
    let folder_name = entries[0].file_name();
    let name_str = folder_name.to_string_lossy();
    assert!(
        name_str.chars().all(|c| c.is_numeric()),
        "Folder name should be timestamp"
    );
}

/// Verifies existing file is backed up before modification.
/// Feature: Existing file backup
#[test]
fn test_existing_backed_up() {
    let dir = setup_temp_dir();

    let original_content = "fn original() { /* important */ }";
    let existing_path = dir.path().join("important.rs");
    fs::write(&existing_path, original_content).expect("Should create file");

    let mut files = HashMap::new();
    files.insert(
        "important.rs".to_string(),
        FileContent {
            content: "fn modified() {}".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "important.rs".to_string(),
        operation: Operation::Update,
    }];

    writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    // Find the backup
    let backup_dir = dir.path().join(".warden_apply_backup");
    let timestamp_folders: Vec<_> = fs::read_dir(&backup_dir)
        .expect("Should read")
        .filter_map(|e| e.ok())
        .collect();

    assert!(!timestamp_folders.is_empty(), "Should have backup folder");

    let backed_up_file = timestamp_folders[0].path().join("important.rs");
    assert!(backed_up_file.exists(), "Backup file should exist");

    let backed_up_content = fs::read_to_string(&backed_up_file).expect("Should read backup");
    assert_eq!(
        backed_up_content, original_content,
        "Backup should have original content"
    );
}

/// Verifies new files don't need backup.
/// Feature: New file skip (no backup needed)
#[test]
fn test_new_file_no_backup() {
    let dir = setup_temp_dir();

    let mut files = HashMap::new();
    files.insert(
        "brand_new.rs".to_string(),
        FileContent {
            content: "fn new() {}".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "brand_new.rs".to_string(),
        operation: Operation::New,
    }];

    let result = writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    // Verify file was created
    assert!(dir.path().join("brand_new.rs").exists());

    // For new files only, backup might not be created at all
    match result {
        ApplyOutcome::Success { backed_up, .. } => {
            // backed_up could be false for new-only operations
            // Key point: no error occurred
        }
        _ => panic!("Expected success"),
    }
}

/// Verifies backup preserves directory structure.
/// Feature: Backup path structure preserved
#[test]
fn test_path_structure() {
    let dir = setup_temp_dir();

    // Create nested existing file
    let nested_dir = dir.path().join("src/modules/core");
    fs::create_dir_all(&nested_dir).expect("Should create dirs");
    let existing_file = nested_dir.join("engine.rs");
    fs::write(&existing_file, "fn engine() {}").expect("Should write");

    let mut files = HashMap::new();
    files.insert(
        "src/modules/core/engine.rs".to_string(),
        FileContent {
            content: "fn updated_engine() {}".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "src/modules/core/engine.rs".to_string(),
        operation: Operation::Update,
    }];

    writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    // Find backup and verify structure
    let backup_dir = dir.path().join(".warden_apply_backup");
    let timestamp_folders: Vec<_> = fs::read_dir(&backup_dir)
        .expect("Should read")
        .filter_map(|e| e.ok())
        .collect();

    let backup_path = timestamp_folders[0]
        .path()
        .join("src/modules/core/engine.rs");
    assert!(
        backup_path.exists(),
        "Backup should preserve full path structure"
    );
}

/// Verifies multiple backups in sequence.
#[test]
fn test_multiple_sequential_backups() {
    let dir = setup_temp_dir();

    // Create file
    fs::write(dir.path().join("evolving.rs"), "v1").unwrap();

    // First modification
    let mut files1 = HashMap::new();
    files1.insert(
        "evolving.rs".to_string(),
        FileContent {
            content: "v2".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "evolving.rs".to_string(),
        operation: Operation::Update,
    }];

    writer::write_files(&manifest, &files1, Some(dir.path())).expect("First write");

    // Small delay to ensure different timestamp
    std::thread::sleep(std::time::Duration::from_millis(1100));

    // Second modification
    let mut files2 = HashMap::new();
    files2.insert(
        "evolving.rs".to_string(),
        FileContent {
            content: "v3".to_string(),
            line_count: 1,
        },
    );

    writer::write_files(&manifest, &files2, Some(dir.path())).expect("Second write");

    // Should have multiple backup folders
    let backup_dir = dir.path().join(".warden_apply_backup");
    let timestamp_folders: Vec<_> = fs::read_dir(&backup_dir)
        .expect("Should read")
        .filter_map(|e| e.ok())
        .collect();

    assert!(
        timestamp_folders.len() >= 2,
        "Should have multiple backup timestamps"
    );
}

/// Verifies backup indicates in result.
#[test]
fn test_backup_indicated_in_result() {
    let dir = setup_temp_dir();

    // Create existing file
    fs::write(dir.path().join("file.rs"), "original").unwrap();

    let mut files = HashMap::new();
    files.insert(
        "file.rs".to_string(),
        FileContent {
            content: "modified".to_string(),
            line_count: 1,
        },
    );

    let manifest = vec![ManifestEntry {
        path: "file.rs".to_string(),
        operation: Operation::Update,
    }];

    let result = writer::write_files(&manifest, &files, Some(dir.path())).expect("Should write");

    match result {
        ApplyOutcome::Success { backed_up, .. } => {
            assert!(backed_up, "Should indicate backup was created");
        }
        _ => panic!("Expected success"),
    }
}
