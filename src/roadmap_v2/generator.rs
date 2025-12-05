// src/roadmap_v2/generator.rs
use std::fmt::Write;
use super::types::{Section, Task, TaskStore, SectionStatus, TaskStatus};

impl TaskStore {
    /// Generate ROADMAP.md content from the store
    #[must_use]
    pub fn to_markdown(&self) -> String {
        let mut out = String::new();
        
        let _ = writeln!(out, "# {}\n", self.meta.title);
        
        if !self.meta.description.is_empty() {
            out.push_str(&self.meta.description);
            out.push_str("\n\n");
        }

        out.push_str("---\n\n");

        for section in &self.sections {
            write_section(&mut out, section, &self.tasks);
        }

        out
    }
}

fn write_section(out: &mut String, section: &Section, all_tasks: &[Task]) {
    let status_marker = match section.status {
        SectionStatus::Complete => " ?",
        SectionStatus::Current => " ?? CURRENT",
        SectionStatus::Pending => "",
    };

    let _ = writeln!(out, "## {}{}\n", section.title, status_marker);

    let section_tasks: Vec<_> = all_tasks.iter()
        .filter(|t| t.section == section.id)
        .collect();

    let groups = collect_groups(&section_tasks);

    for group in &groups {
        if let Some(name) = group {
            let _ = writeln!(out, "### {name}");
        }

        for task in section_tasks.iter().filter(|t| &t.group == group) {
            write_task(out, task);
        }

        out.push('\n');
    }

    out.push_str("---\n\n");
}

fn collect_groups(tasks: &[&Task]) -> Vec<Option<String>> {
    let mut groups: Vec<Option<String>> = Vec::new();
    
    for task in tasks {
        if !groups.contains(&task.group) {
            groups.push(task.group.clone());
        }
    }
    
    groups
}

fn write_task(out: &mut String, task: &Task) {
    let checkbox = match task.status {
        TaskStatus::Pending => "[ ]",
        TaskStatus::Done | TaskStatus::NoTest => "[x]",
    };

    let test_anchor = match (&task.test, &task.status) {
        (Some(tst), _) => format!(" <!-- test: {tst} -->"),
        (None, TaskStatus::NoTest) => " [no-test]".to_string(),
        (None, _) => String::new(),
    };

    let _ = writeln!(out, "- {checkbox} **{}**{test_anchor}", task.text);
}