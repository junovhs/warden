// src/project.rs
//! Project type detection and configuration generation.

use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectType {
    Rust,
    Node,
    Python,
    Unknown,
}

impl ProjectType {
    /// Detects the project type from current directory.
    #[must_use]
    pub fn detect() -> Self {
        if Path::new("Cargo.toml").exists() {
            return Self::Rust;
        }
        if Path::new("package.json").exists() {
            return Self::Node;
        }
        if Path::new("pyproject.toml").exists() || Path::new("requirements.txt").exists() {
            return Self::Python;
        }
        Self::Unknown
    }
}

/// Returns the npx command for the current platform.
#[must_use]
pub fn npx_cmd() -> &'static str {
    if cfg!(windows) {
        "npx.cmd"
    } else {
        "npx"
    }
}

/// Returns the npm command for the current platform.
#[must_use]
pub fn npm_cmd() -> &'static str {
    if cfg!(windows) {
        "npm.cmd"
    } else {
        "npm"
    }
}

/// Returns the cargo command for the current platform.
#[must_use]
pub fn cargo_cmd() -> &'static str {
    if cfg!(windows) {
        "cargo.exe"
    } else {
        "cargo"
    }
}

/// Generates warden.toml content based on detected project type.
#[must_use]
pub fn generate_toml() -> String {
    let project = ProjectType::detect();
    let commands = generate_commands_section(project);

    format!(
        r#"# warden.toml
[rules]
max_file_tokens = 2000
max_cyclomatic_complexity = 5
max_nesting_depth = 2
max_function_args = 5
max_function_words = 5
ignore_naming_on = ["tests", "spec"]

{commands}
"#
    )
}

fn generate_commands_section(project: ProjectType) -> String {
    match project {
        ProjectType::Rust => rust_commands(),
        ProjectType::Node => node_commands(),
        ProjectType::Python => python_commands(),
        ProjectType::Unknown => unknown_commands(),
    }
}

fn rust_commands() -> String {
    r#"[commands]
check = "cargo clippy --all-targets -- -D warnings -D clippy::pedantic"
fix = "cargo fmt""#
        .to_string()
}

fn node_commands() -> String {
    let npx = npx_cmd();
    format!(
        r#"[commands]
check = "{npx} @biomejs/biome check src/"
fix = "{npx} @biomejs/biome check --write src/""#
    )
}

fn python_commands() -> String {
    r#"[commands]
check = "ruff check ."
fix = "ruff check --fix .""#
        .to_string()
}

fn unknown_commands() -> String {
    r#"# No project type detected. Configure commands manually:
# [commands]
# check = "your-lint-command"
# fix = "your-fix-command""#
        .to_string()
}
