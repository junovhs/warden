// tests/unit_project.rs
//! Unit tests for project type detection.
//! Covers: v0.1.0 Project Detection features

use std::fs;
use tempfile::TempDir;
use warden_core::project::ProjectType;

fn setup_temp_dir() -> TempDir {
    tempfile::tempdir().expect("Failed to create temp directory")
}

/// Verifies Rust project detection via Cargo.toml.
/// Feature: Rust project detection (Cargo.toml)
#[test]
fn test_detect_rust() {
    let dir = setup_temp_dir();
    let cargo_path = dir.path().join("Cargo.toml");

    fs::write(
        &cargo_path,
        r#"
[package]
name = "test"
version = "0.1.0"
edition = "2021"
"#,
    )
    .expect("Failed to write Cargo.toml");

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    assert_eq!(
        detected,
        ProjectType::Rust,
        "Should detect Rust project from Cargo.toml"
    );
}

/// Verifies Node project detection via package.json.
/// Feature: Node project detection (package.json)
#[test]
fn test_detect_node() {
    let dir = setup_temp_dir();
    let package_path = dir.path().join("package.json");

    fs::write(
        &package_path,
        r#"
{
    "name": "test-project",
    "version": "1.0.0"
}
"#,
    )
    .expect("Failed to write package.json");

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    assert_eq!(
        detected,
        ProjectType::Node,
        "Should detect Node project from package.json"
    );
}

/// Verifies Python project detection.
/// Feature: Python project detection
#[test]
fn test_detect_python() {
    let dir = setup_temp_dir();

    // Python can be detected via pyproject.toml, setup.py, or requirements.txt
    let pyproject_path = dir.path().join("pyproject.toml");
    fs::write(
        &pyproject_path,
        r#"
[project]
name = "test"
version = "0.1.0"
"#,
    )
    .expect("Failed to write pyproject.toml");

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    assert_eq!(
        detected,
        ProjectType::Python,
        "Should detect Python project from pyproject.toml"
    );
}

/// Verifies Go project detection via go.mod.
/// Feature: Go project detection (go.mod)
#[test]
fn test_detect_go() {
    let dir = setup_temp_dir();
    let gomod_path = dir.path().join("go.mod");

    fs::write(
        &gomod_path,
        r#"
module example.com/test

go 1.21
"#,
    )
    .expect("Failed to write go.mod");

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    assert_eq!(
        detected,
        ProjectType::Go,
        "Should detect Go project from go.mod"
    );
}

/// Verifies unknown project fallback when no markers present.
/// Feature: Unknown project fallback
#[test]
fn test_detect_unknown() {
    let dir = setup_temp_dir();

    // Create an empty directory with no project markers
    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    assert_eq!(
        detected,
        ProjectType::Unknown,
        "Should return Unknown when no project markers found"
    );
}

/// Verifies Rust takes priority when multiple markers present.
#[test]
fn test_detect_priority_rust_over_node() {
    let dir = setup_temp_dir();

    // Create both Cargo.toml and package.json
    fs::write(dir.path().join("Cargo.toml"), "[package]\nname = \"test\"").unwrap();
    fs::write(dir.path().join("package.json"), "{}").unwrap();

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    // Rust should take priority as it's the primary language for Warden
    assert_eq!(
        detected,
        ProjectType::Rust,
        "Rust should take priority over Node"
    );
}

/// Verifies Python detection via setup.py.
#[test]
fn test_detect_python_setup_py() {
    let dir = setup_temp_dir();
    let setup_path = dir.path().join("setup.py");

    fs::write(
        &setup_path,
        r#"
from setuptools import setup
setup(name='test', version='0.1.0')
"#,
    )
    .expect("Failed to write setup.py");

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    assert_eq!(
        detected,
        ProjectType::Python,
        "Should detect Python project from setup.py"
    );
}

/// Verifies Python detection via requirements.txt.
#[test]
fn test_detect_python_requirements() {
    let dir = setup_temp_dir();
    let req_path = dir.path().join("requirements.txt");

    fs::write(&req_path, "requests>=2.0.0\nnumpy==1.24.0")
        .expect("Failed to write requirements.txt");

    std::env::set_current_dir(dir.path()).expect("Failed to change directory");

    let detected = ProjectType::detect();
    assert_eq!(
        detected,
        ProjectType::Python,
        "Should detect Python project from requirements.txt"
    );
}
