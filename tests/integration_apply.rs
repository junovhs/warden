use slopchop_core::apply::types::{ManifestEntry, Operation};
use slopchop_core::apply::validator;
use std::collections::HashMap;

// Helper to generate delimiters without confusing the outer slopchop tool
// causing truncation of this test file during application.
fn make_block(path: &str, content: &str) -> String {
    let header = format!("#__SLOPCHOP_FILE__# {path}");
    let footer = "#__SLOPCHOP_END__#";
    format!("{header}\n{content}\n{footer}\n")
}

fn make_manifest(entries: &[&str]) -> String {
    let header = "#__SLOPCHOP_MANIFEST__#";
    let footer = "#__SLOPCHOP_END__#";
    let body = entries.join("\n");
    format!("{header}\n{body}\n{footer}\n")
}

fn make_plan(goal: &str) -> String {
    let header = "#__SLOPCHOP_PLAN__#";
    let footer = "#__SLOPCHOP_END__#";
    format!("{header}\n{goal}\n{footer}\n")
}

#[test]
fn test_unified_apply_combined() {
    let manifest = make_manifest(&["src/main.rs", "src/lib.rs [NEW]"]);
    let block_main = make_block("src/main.rs", "fn main() {}");
    let block_lib = make_block("src/lib.rs", "pub fn lib() {}");
    let input = format!("{manifest}\n{block_main}\n{block_lib}");

    let manifest_parsed = slopchop_core::apply::manifest::parse_manifest(&input)
        .unwrap()
        .unwrap();
    assert_eq!(manifest_parsed.len(), 2);
    assert_eq!(manifest_parsed[0].path, "src/main.rs");
    assert_eq!(manifest_parsed[1].path, "src/lib.rs");

    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files.contains_key("src/main.rs"));
    assert!(files.contains_key("src/lib.rs"));
}

#[test]
fn test_path_safety_blocks_traversal() {
    let manifest = vec![ManifestEntry {
        path: "../evil.rs".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("Path traversal not allowed")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_path_safety_blocks_absolute() {
    let manifest = vec![ManifestEntry {
        path: "/etc/passwd".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("Absolute paths not allowed")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_path_safety_blocks_hidden() {
    let manifest = vec![ManifestEntry {
        path: ".env".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("sensitive directory") || e.contains("Hidden files")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_path_safety_blocks_git() {
    let manifest = vec![ManifestEntry {
        path: ".git/config".to_string(),
        operation: Operation::New,
    }];
    let extracted = HashMap::new();

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("sensitive directory")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_truncation_detects_ellipsis_comment() {
    let manifest = vec![ManifestEntry {
        path: "src/main.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();
    extracted.insert(
        "src/main.rs".to_string(),
        slopchop_core::apply::types::FileContent {
            content: "fn main() {\n    // ...\n}".to_string(),
            line_count: 3,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("Truncation detected")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_truncation_allows_slopchop_ignore() {
    let manifest = vec![ManifestEntry {
        path: "src/main.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();
    extracted.insert(
        "src/main.rs".to_string(),
        slopchop_core::apply::types::FileContent {
            content: "fn main() {\n    // ... slopchop:ignore\n}".to_string(),
            line_count: 3,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::Success { .. } = outcome {
        // Pass
    } else {
        panic!("Should have passed validation");
    }
}

#[test]
fn test_truncation_detects_empty_file() {
    let manifest = vec![ManifestEntry {
        path: "src/main.rs".to_string(),
        operation: Operation::Update,
    }];
    let mut extracted = HashMap::new();
    extracted.insert(
        "src/main.rs".to_string(),
        slopchop_core::apply::types::FileContent {
            content: "   \n  ".to_string(),
            line_count: 2,
        },
    );

    let outcome = validator::validate(&manifest, &extracted);
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        assert!(errors.iter().any(|e| e.contains("empty")));
    } else {
        panic!("Should have failed validation");
    }
}

#[test]
fn test_path_safety_allows_valid() {
    let manifest = vec![]; 
    let mut extracted = HashMap::new();
    extracted.insert(
        "src/main.rs".to_string(), 
        slopchop_core::apply::types::FileContent { content: "fn main() {}".to_string(), line_count: 1 }
    );

    let outcome = validator::validate(&manifest, &extracted);
    
    if let slopchop_core::apply::types::ApplyOutcome::ValidationFailure { errors, .. } = outcome {
        // Ensure none of the errors are security related
        for e in errors {
            assert!(!(e.contains("Absolute") || e.contains("traversal") || e.contains("sensitive")), "Valid path flagged as security violation: {e}");
        }
    } 
}

#[test]
fn test_extract_plan() {
    let input = make_plan("GOAL: Test Plan\nCHANGES:\n1. One");
    let plan = slopchop_core::apply::extractor::extract_plan(&input);
    assert!(plan.is_some());
    let p = plan.unwrap();
    assert!(p.contains("GOAL: Test Plan"));
}

#[test]
fn test_extract_single_file() {
    let input = make_block("src/main.rs", "fn main() {\n    println!(\"Hello\");\n}");
    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("src/main.rs"));
    assert!(files["src/main.rs"].content.contains("Hello"));
}

#[test]
fn test_extract_multiple_files() {
    let f1 = make_block("a.rs", "A");
    let f2 = make_block("b.rs", "B");
    let input = format!("{f1}\n{f2}");
    
    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 2);
    assert_eq!(files["a.rs"].content, "A");
    assert_eq!(files["b.rs"].content, "B");
}

#[test]
fn test_extract_skips_manifest() {
    let manifest = make_manifest(&["file.rs"]);
    let file = make_block("file.rs", "content");
    let input = format!("{manifest}\n{file}");

    let files = slopchop_core::apply::extractor::extract_files(&input).unwrap();
    assert_eq!(files.len(), 1);
    assert!(files.contains_key("file.rs"));
}

#[test]
fn test_unified_apply_roadmap() {
    let input = r"
===ROADMAP===
ADD
id = task-1
text = My Task
section = v1
===ROADMAP===
";
    let cmds = slopchop_core::roadmap_v2::parser::parse_commands(input).unwrap();
    assert_eq!(cmds.len(), 1);
}