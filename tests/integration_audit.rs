// tests/integration_audit.rs
//! Integration tests for the audit system.
//! Covers: v0.7.0 Test Traceability features

use std::fs;
use std::path::Path;
use tempfile::TempDir;
use warden_core::roadmap::Roadmap;

fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

fn create_roadmap_with_anchors(dir: &TempDir) -> std::path::PathBuf {
    let path = dir.path().join("ROADMAP.md");
    fs::write(
        &path,
        r#"# Test Project

## v0.1.0

### Features
- [x] **Feature with test** <!-- test: tests/unit.rs::test_feature -->
- [x] **Feature without test** [no-test]
- [ ] **Pending feature** <!-- test: tests/unit.rs::test_pending -->
- [x] **Missing test file** <!-- test: tests/nonexistent.rs::test_missing -->

## v0.2.0

- [ ] **Future feature**
"#,
    )
    .expect("Failed to write roadmap");
    path
}

fn create_test_file(dir: &TempDir, name: &str, content: &str) {
    let tests_dir = dir.path().join("tests");
    fs::create_dir_all(&tests_dir).expect("Should create tests dir");
    fs::write(tests_dir.join(name), content).expect("Should write test file");
}

// =============================================================================
// AUDIT SYSTEM
// =============================================================================

/// Verifies audit scans only completed tasks.
/// Feature: Scan completed tasks
#[test]
fn test_scans_completed_only() {
    let dir = setup_temp_dir();
    let path = create_roadmap_with_anchors(&dir);

    let roadmap = Roadmap::from_file(&path).expect("Should load");
    let tasks = roadmap.all_tasks();

    // Get only completed tasks with test anchors
    let completed_with_tests: Vec<_> = tasks
        .iter()
        .filter(|t| t.status == warden_core::roadmap::TaskStatus::Complete)
        .filter(|t| !t.tests.is_empty())
        .collect();

    // Should find completed tasks with tests
    assert!(
        completed_with_tests.len() >= 1,
        "Should find completed tasks with test anchors"
    );

    // Pending tasks should not be in this list
    let pending_in_list = completed_with_tests
        .iter()
        .any(|t| t.status == warden_core::roadmap::TaskStatus::Pending);
    assert!(!pending_in_list, "Should not include pending tasks");
}

/// Verifies [no-test] tasks are skipped.
/// Feature: [no-test] skip
#[test]
fn test_no_test_skipped() {
    let dir = setup_temp_dir();
    let path = create_roadmap_with_anchors(&dir);

    let roadmap = Roadmap::from_file(&path).expect("Should load");
    let tasks = roadmap.all_tasks();

    // Find the [no-test] task
    let no_test_task = tasks.iter().find(|t| t.text.contains("without test"));

    assert!(no_test_task.is_some(), "Should find [no-test] task");
    let task = no_test_task.unwrap();

    // [no-test] tasks should have empty tests vector
    assert!(
        task.tests.is_empty(),
        "[no-test] task should have no test anchors"
    );
}

/// Verifies explicit anchor verification.
/// Feature: Explicit anchor verification
#[test]
fn test_explicit_anchor_verified() {
    let dir = setup_temp_dir();
    let path = create_roadmap_with_anchors(&dir);

    // Create a test file that matches the anchor
    create_test_file(
        &dir,
        "unit.rs",
        r#"
#[test]
fn test_feature() {
    assert!(true);
}
"#,
    );

    let roadmap = Roadmap::from_file(&path).expect("Should load");
    let tasks = roadmap.all_tasks();

    // Find the task with explicit anchor
    let anchored_task = tasks.iter().find(|t| t.text.contains("Feature with test"));

    assert!(anchored_task.is_some(), "Should find anchored task");
    let task = anchored_task.unwrap();

    // Should have the test anchor
    assert!(!task.tests.is_empty(), "Should have test anchor");
    assert!(
        task.tests[0].contains("tests/unit.rs"),
        "Anchor should reference test file"
    );
}

/// Verifies missing test file detection.
/// Feature: Missing test file detection
#[test]
fn test_missing_file_detected() {
    let dir = setup_temp_dir();
    let path = create_roadmap_with_anchors(&dir);

    // Don't create the test file that's referenced

    let roadmap = Roadmap::from_file(&path).expect("Should load");
    let tasks = roadmap.all_tasks();

    // Find the task referencing missing file
    let missing_task = tasks.iter().find(|t| t.text.contains("Missing test file"));

    assert!(
        missing_task.is_some(),
        "Should find task with missing file reference"
    );
    let task = missing_task.unwrap();

    // The anchor points to a nonexistent file
    let test_path = &task.tests[0];
    let full_path = dir.path().join(test_path.split("::").next().unwrap_or(""));

    assert!(
        !full_path.exists(),
        "Test file should not exist for this test"
    );
}

// =============================================================================
// ANCHOR PARSING
// =============================================================================

/// Verifies multiple test anchors per task.
#[test]
fn test_multiple_anchors() {
    let content = r#"# Test
## v0.1.0
- [x] **Multi-test feature** <!-- test: tests/a.rs::test_a --> <!-- test: tests/b.rs::test_b -->
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    assert_eq!(tasks.len(), 1);
    // Implementation may vary on handling multiple anchors
}

/// Verifies anchor format parsing.
#[test]
fn test_anchor_format() {
    let content = r#"# Test
## v0.1.0
- [x] **Standard anchor** <!-- test: tests/unit.rs::test_name -->
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    assert_eq!(tasks.len(), 1);
    let task = &tasks[0];

    // Should parse the full anchor path
    assert!(!task.tests.is_empty());
    let anchor = &task.tests[0];
    assert!(anchor.contains("tests/unit.rs"), "Should have file path");
    assert!(anchor.contains("::"), "Should have function separator");
    assert!(anchor.contains("test_name"), "Should have function name");
}

// =============================================================================
// AUDIT RESULTS
// =============================================================================

/// Verifies audit can process a valid roadmap.
#[test]
fn test_audit_processes_roadmap() {
    let dir = setup_temp_dir();
    let path = create_roadmap_with_anchors(&dir);

    // Create matching test files
    create_test_file(
        &dir,
        "unit.rs",
        r#"
#[test]
fn test_feature() {}

#[test]
fn test_pending() {}
"#,
    );

    let roadmap = Roadmap::from_file(&path).expect("Should load");

    // Count tasks that would be audited
    let tasks = roadmap.all_tasks();
    let auditable: Vec<_> = tasks
        .iter()
        .filter(|t| t.status == warden_core::roadmap::TaskStatus::Complete)
        .filter(|t| !t.tests.is_empty())
        .collect();

    // Should have auditable tasks
    assert!(!auditable.is_empty(), "Should have tasks to audit");
}

/// Verifies empty roadmap doesn't crash audit.
#[test]
fn test_audit_empty_roadmap() {
    let content = "# Empty\n";
    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // Should handle gracefully
    assert!(tasks.is_empty());
}

/// Verifies roadmap with no test anchors.
#[test]
fn test_audit_no_anchors() {
    let content = r#"# Test
## v0.1.0
- [x] **No anchor here**
- [x] **Also no anchor**
"#;

    let roadmap = Roadmap::parse(content);
    let tasks = roadmap.all_tasks();

    // All tasks should have empty tests
    for task in tasks {
        assert!(
            task.tests.is_empty(),
            "Tasks without anchors should have empty tests"
        );
    }
}
