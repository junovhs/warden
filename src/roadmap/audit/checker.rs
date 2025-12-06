// slopchop:ignore
use crate::roadmap::audit::types::{AuditViolation, ViolationReason};
use crate::roadmap::types::Task;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn check_roadmap(tasks: &[Task], root: &Path) -> Vec<AuditViolation> {
    let test_files = scan_test_files(root);
    
    tasks.iter()
        .flat_map(|task| audit_task(task, &test_files, root))
        .collect()
}

fn scan_test_files(root: &Path) -> HashSet<PathBuf> {
    let mut files = HashSet::new();
    let tests_dir = root.join("tests");

    if tests_dir.exists() {
        for entry in WalkDir::new(tests_dir)
            .into_iter()
            .filter_map(Result::ok) 
        {
            if entry.path().extension().map_or(false, |ext| ext == "rs") {
                files.insert(entry.path().to_path_buf());
            }
        }
    }
    files
}

fn audit_task(task: &Task, _known_tests: &HashSet<PathBuf>, root: &Path) -> Vec<AuditViolation> {
    let mut violations = Vec::new();

    let (_, anchor_opt) = extract_anchor(&task.text);

    if let Some(anchor) = anchor_opt {
        let parts: Vec<&str> = anchor.split("::").collect();
        let file_part = parts[0];
        let file_path = root.join(file_part);

        if !file_path.exists() {
            violations.push(AuditViolation {
                task_id: task.id.clone(),
                task_text: task.text.clone(),
                reason: ViolationReason::MissingTestFile(file_part.to_string()),
            });
        } else if parts.len() > 1 {
            let func_name = parts[1];
            if let Ok(content) = std::fs::read_to_string(&file_path) {
                if !content.contains(&format!("fn {func_name}")) {
                    violations.push(AuditViolation {
                        task_id: task.id.clone(),
                        task_text: task.text.clone(),
                        reason: ViolationReason::MissingTestFunction { 
                            file: file_part.to_string(), 
                            function: func_name.to_string() 
                        },
                    });
                }
            }
        }
    } else {
        violations.push(AuditViolation {
             task_id: task.id.clone(),
             task_text: task.text.clone(),
             reason: ViolationReason::NoTraceability,
        });
    }

    violations
}

fn extract_anchor(text: &str) -> (String, Option<String>) {
    if let Some(start) = text.find("<!-- test:") {
        if let Some(end) = text[start..].find("-->") {
            let anchor = text[start + 10 .. start + end].trim().to_string();
            let clean = format!("{}{}", &text[..start], &text[start + end + 3..]).trim().to_string();
            return (clean, Some(anchor));
        }
    }
    (text.to_string(), None)
}