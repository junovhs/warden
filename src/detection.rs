// src/detection.rs
use crate::error::Result;
use std::collections::HashSet;
use std::fmt;
use std::path::Path;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum BuildSystemType {
    Rust,
    Node,
    Python,
    Go,
    CMake,
    Conan,
}

impl fmt::Display for BuildSystemType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Default)]
pub struct Detector;

impl Detector {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Detects build systems.
    /// # Errors
    /// Returns `Ok`.
    pub fn detect_build_systems(
        &self,
        files: &[std::path::PathBuf],
    ) -> Result<Vec<BuildSystemType>> {
        let mut detected = HashSet::new();
        for file in files {
            check_file(file, &mut detected);
        }
        Ok(detected.into_iter().collect())
    }
}

fn check_file(path: &Path, set: &mut HashSet<BuildSystemType>) {
    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
        if check_cmake(path, set) {
            return;
        }
        check_common(name, set);
    }
}

fn check_cmake(path: &Path, set: &mut HashSet<BuildSystemType>) -> bool {
    if path
        .extension()
        .is_some_and(|e| e.eq_ignore_ascii_case("cmake"))
    {
        set.insert(BuildSystemType::CMake);
        return true;
    }
    false
}

fn check_common(name: &str, set: &mut HashSet<BuildSystemType>) {
    match name {
        "Cargo.toml" => {
            set.insert(BuildSystemType::Rust);
        }
        "package.json" => {
            set.insert(BuildSystemType::Node);
        }
        "requirements.txt" | "pyproject.toml" | "Pipfile" => {
            set.insert(BuildSystemType::Python);
        }
        "go.mod" => {
            set.insert(BuildSystemType::Go);
        }
        "CMakeLists.txt" => {
            set.insert(BuildSystemType::CMake);
        }
        "conanfile.txt" | "conanfile.py" => {
            set.insert(BuildSystemType::Conan);
        }
        _ => {}
    }
}
