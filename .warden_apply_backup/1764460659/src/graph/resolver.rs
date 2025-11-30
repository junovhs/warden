// src/graph/resolver.rs
use std::path::{Path, PathBuf};

/// Resolves an import string to a likely file path on disk.
///
/// # Arguments
/// * `project_root` - The root of the repository.
/// * `current_file` - The path of the file containing the import.
/// * `import_str` - The raw import string (e.g., "`crate::foo`", "./utils").
///
/// # Returns
/// `Option<PathBuf>` if a matching local file is found.
#[must_use]
pub fn resolve(project_root: &Path, current_file: &Path, import_str: &str) -> Option<PathBuf> {
    let ext = current_file.extension().and_then(|s| s.to_str())?;
    
    match ext {
        "rs" => resolve_rust(project_root, current_file, import_str),
        "ts" | "tsx" | "js" | "jsx" => resolve_js(project_root, current_file, import_str),
        "py" => resolve_python(project_root, current_file, import_str),
        _ => None,
    }
}

fn resolve_rust(root: &Path, current: &Path, import: &str) -> Option<PathBuf> {
    // 1. Handle "crate::" (Absolute from src/)
    if let Some(rest) = import.strip_prefix("crate::") {
        let parts: Vec<&str> = rest.split("::").collect();
        let base = root.join("src");
        return check_variations(&base, &parts, "rs");
    }

    // 2. Handle "super::" (Parent directory)
    if import.starts_with("super::") {
        return None; // TODO: complex super chain resolution
    }

    // 3. Handle relative `mod foo;` or `use foo;`
    if !import.contains("::") && !import.starts_with("crate") {
        let parent = current.parent()?;
        let parts = vec![import];
        return check_variations(parent, &parts, "rs");
    }

    None
}

fn resolve_js(_root: &Path, current: &Path, import: &str) -> Option<PathBuf> {
    if !import.starts_with('.') {
        return None;
    }

    let parent = current.parent()?;
    let path = parent.join(import);
    
    if let Some(p) = check_js_file(&path) {
        return Some(p);
    }
    check_js_directory(&path)
}

fn check_js_file(path: &Path) -> Option<PathBuf> {
    if path.exists() && path.is_file() {
        return Some(path.to_path_buf());
    }

    let extensions = ["ts", "tsx", "js", "jsx", "json"];
    for ext in extensions {
        let p = path.with_extension(ext);
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn check_js_directory(path: &Path) -> Option<PathBuf> {
    if !path.is_dir() {
        return None;
    }

    let extensions = ["ts", "tsx", "js", "jsx", "json"];
    for ext in extensions {
        let p = path.join(format!("index.{ext}"));
        if p.exists() {
            return Some(p);
        }
    }
    None
}

fn resolve_python(root: &Path, _current: &Path, import: &str) -> Option<PathBuf> {
    // 1. Handle Relative "from . import foo" -> "."
    if import.starts_with('.') {
        return None; // Simplified: assuming simple relative import for now
    }

    // 2. Absolute (from root)
    let parts: Vec<&str> = import.split('.').collect();
    check_variations(root, &parts, "py")
}

fn check_variations(base: &Path, parts: &[&str], ext: &str) -> Option<PathBuf> {
    let mut current = base.to_path_buf();
    for part in parts {
        current.push(part);
    }

    // Variation A: path.ext
    let file_path = current.with_extension(ext);
    if file_path.exists() {
        return Some(file_path);
    }

    // Variation B: path/mod.rs or path/__init__.py
    let index_name = match ext {
        "rs" => "mod.rs",
        "py" => "__init__.py",
        _ => return None,
    };
    
    let index_path = current.join(index_name);
    if index_path.exists() {
        return Some(index_path);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_resolve_rust_mod_relative() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        
        let src = root.join("src");
        fs::create_dir_all(&src).unwrap();
        
        let main = src.join("main.rs");
        let util = src.join("util.rs");
        fs::write(&main, "mod util;").unwrap();
        fs::write(&util, "// util").unwrap();

        let resolved = resolve(root, &main, "util");
        assert_eq!(resolved, Some(util));
    }

    #[test]
    fn test_resolve_js_relative_extension() {
        let temp = tempdir().unwrap();
        let root = temp.path();
        
        let app = root.join("app.ts");
        let cmp = root.join("cmp.tsx");
        fs::write(&app, "").unwrap();
        fs::write(&cmp, "").unwrap();

        let resolved = resolve(root, &app, "./cmp");
        assert_eq!(resolved, Some(cmp));
    }
}