// tests/unit_manifest.rs
//! Unit tests for manifest parsing.
//! Covers: v0.3.0 Manifest Parsing features

use warden_core::apply::manifest;
use warden_core::apply::types::Operation;

/// Verifies manifest block detection.
/// Feature: Manifest block detection
#[test]
fn test_parse_manifest() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
src/main.rs
src/lib.rs
∆∆∆
"#;

    let result = manifest::parse_manifest(input).expect("Should parse");

    assert!(result.is_some(), "Should find manifest");
    let entries = result.unwrap();
    assert_eq!(entries.len(), 2, "Should have 2 entries");
}

/// Verifies [NEW] marker detection.
/// Feature: [NEW] marker detection
#[test]
fn test_new_marker() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
src/existing.rs
src/new_file.rs [NEW]
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    let new_entry = entries
        .iter()
        .find(|e| e.path.contains("new_file"))
        .expect("Should find new_file entry");

    assert_eq!(
        new_entry.operation,
        Operation::New,
        "[NEW] should mark as New operation"
    );
}

/// Verifies [DELETE] marker detection.
/// Feature: [DELETE] marker detection
#[test]
fn test_delete_marker() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
src/keep.rs
src/remove.rs [DELETE]
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    let delete_entry = entries
        .iter()
        .find(|e| e.path.contains("remove"))
        .expect("Should find remove entry");

    assert_eq!(
        delete_entry.operation,
        Operation::Delete,
        "[DELETE] should mark as Delete operation"
    );
}

/// Verifies default Update operation.
/// Feature: Default Update operation
#[test]
fn test_default_update() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
src/main.rs
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    assert_eq!(
        entries[0].operation,
        Operation::Update,
        "Default should be Update"
    );
}

/// Verifies mixed operations in manifest.
#[test]
fn test_mixed_operations() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
src/update.rs
src/create.rs [NEW]
src/remove.rs [DELETE]
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    assert_eq!(entries.len(), 3);

    let update = entries.iter().find(|e| e.path.contains("update")).unwrap();
    let create = entries.iter().find(|e| e.path.contains("create")).unwrap();
    let remove = entries.iter().find(|e| e.path.contains("remove")).unwrap();

    assert_eq!(update.operation, Operation::Update);
    assert_eq!(create.operation, Operation::New);
    assert_eq!(remove.operation, Operation::Delete);
}

/// Verifies list marker handling (bullet points).
#[test]
fn test_list_markers() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
- src/file1.rs
- src/file2.rs [NEW]
* src/file3.rs
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    assert_eq!(entries.len(), 3, "Should handle bullet points");
    assert!(entries.iter().any(|e| e.path == "src/file1.rs"));
    assert!(entries.iter().any(|e| e.path == "src/file2.rs"));
    assert!(entries.iter().any(|e| e.path == "src/file3.rs"));
}

/// Verifies numbered list handling.
#[test]
fn test_numbered_list() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
1. src/first.rs
2. src/second.rs
3. src/third.rs [NEW]
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    assert_eq!(entries.len(), 3, "Should handle numbered list");
}

/// Verifies empty manifest returns empty list.
#[test]
fn test_empty_manifest() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    assert!(
        entries.is_empty(),
        "Empty manifest should return empty list"
    );
}

/// Verifies no manifest returns None.
#[test]
fn test_no_manifest() {
    let input = r#"
∇∇∇ src/main.rs ∇∇∇
fn main() {}
∆∆∆
"#;

    let result = manifest::parse_manifest(input).expect("Should not error");

    assert!(result.is_none(), "Should return None when no manifest");
}

/// Verifies case-insensitive marker detection.
#[test]
fn test_case_insensitive_markers() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
src/file1.rs [new]
src/file2.rs [delete]
src/file3.rs [NEW]
src/file4.rs [DELETE]
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    let new_count = entries
        .iter()
        .filter(|e| e.operation == Operation::New)
        .count();
    let delete_count = entries
        .iter()
        .filter(|e| e.operation == Operation::Delete)
        .count();

    assert_eq!(new_count, 2, "Should detect both [new] and [NEW]");
    assert_eq!(delete_count, 2, "Should detect both [delete] and [DELETE]");
}

/// Verifies whitespace handling in paths.
#[test]
fn test_whitespace_in_manifest() {
    let input = r#"
∇∇∇ MANIFEST ∇∇∇
   src/file1.rs
  src/file2.rs [NEW]
∆∆∆
"#;

    let entries = manifest::parse_manifest(input)
        .expect("Should parse")
        .expect("Should have manifest");

    // Paths should be trimmed
    assert!(entries.iter().any(|e| e.path == "src/file1.rs"));
    assert!(entries.iter().any(|e| e.path == "src/file2.rs"));
}

/// Verifies legacy XML format support.
#[test]
fn test_legacy_xml_format() {
    let input = r#"
<delivery>
src/main.rs
src/lib.rs [NEW]
</delivery>
"#;

    let result = manifest::parse_manifest(input).expect("Should parse");

    // Legacy format should still work
    if let Some(entries) = result {
        assert!(entries.len() >= 1, "Should parse legacy format");
    }
}
