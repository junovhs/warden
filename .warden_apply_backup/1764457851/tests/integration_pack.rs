use std::fs;
use tempfile::tempdir;
use warden_core::config::Config;
use warden_core::pack::{self, PackOptions};

#[test]
fn test_nabla_delimiters_are_unique() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("test.rs");
    fs::write(&file_path, "fn main() {}").unwrap();

    let files = vec![file_path];
    let config = Config::default();
    let opts = PackOptions::default();

    let content = pack::generate_content(&files, &opts, &config).unwrap();

    assert!(content.contains("∇∇∇"));
    assert!(content.contains("∆∆∆"));
}

#[test]
fn test_nabla_format_structure() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("src/lib.rs");
    fs::create_dir_all(file_path.parent().unwrap()).unwrap();
    fs::write(&file_path, "pub fn test() {}").unwrap();

    let files = vec![file_path];
    let config = Config::default();
    let opts = PackOptions::default();

    let content = pack::generate_content(&files, &opts, &config).unwrap();

    // Fix path separators for test assertion on Windows if needed, though lib handles it
    // The library explicitly does .replace('\\', "/") so we expect forward slashes.
    let path_str = if cfg!(windows) {
        let root_str = temp.path().to_string_lossy().replace('\\', "/");
        format!("{root_str}/src/lib.rs")
    } else {
        temp.path().join("src/lib.rs").to_string_lossy().to_string()
    };

    let expected_header = format!("∇∇∇ {path_str} ∇∇∇");
    assert!(content.contains(&expected_header), "Header not found: {expected_header}");
}

#[test]
fn test_pack_skeleton_integration() {
    let temp = tempdir().unwrap();
    let file_path = temp.path().join("complex.rs");
    let code = r"
        fn complex_logic() {
            let a = 1;
            let b = 2;
            println!({}, a + b);
        }
    ";
    fs::write(&file_path, code).unwrap();

    let files = vec![file_path];
    let config = Config::default();
    let opts = PackOptions {
        skeleton: true,
        ..PackOptions::default()
    };

    let content = pack::generate_content(&files, &opts, &config).unwrap();

    assert!(content.contains("fn complex_logic() { ... }"));
    assert!(!content.contains("println!"));
}

#[test]
fn test_prompt_includes_limits() {
    let config = Config::default();
    let opts = PackOptions {
        prompt: true,
        ..PackOptions::default()
    };

    let content = pack::generate_content(&[], &opts, &config).unwrap();

    assert!(content.contains("Files: MUST be < 2000 tokens"));
    assert!(content.contains("Cyclomatic Complexity: MUST be ≤ 5")); // Default
}

#[test]
fn test_prompt_includes_laws() {
    let config = Config::default();
    let opts = PackOptions {
        prompt: true,
        ..PackOptions::default()
    };
    let content = pack::generate_content(&[], &opts, &config).unwrap();

    assert!(content.contains("LAW OF ATOMICITY"));
    assert!(content.contains("LAW OF COMPLEXITY"));
    assert!(content.contains("LAW OF PARANOIA"));
}

#[test]
fn test_reminder_is_concise() {
    let config = Config::default();
    let opts = PackOptions {
        prompt: true,
        ..PackOptions::default()
    };
    let content = pack::generate_content(&[], &opts, &config).unwrap();

    // Check footer
    assert!(content.contains("END CODEBASE"));
    assert!(content.contains("WARDEN CONSTRAINTS:"));
    assert!(content.contains("□ No .unwrap()"));
}

#[test]
fn test_prompt_includes_nabla_instructions() {
    let config = Config::default();
    let opts = PackOptions {
        prompt: true,
        ..PackOptions::default()
    };
    let content = pack::generate_content(&[], &opts, &config).unwrap();

    assert!(content.contains("∇∇∇ MANIFEST ∇∇∇"));
    assert!(content.contains("∇∇∇ PLAN ∇∇∇"));
}