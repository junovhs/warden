// slopchop:ignore
use crate::roadmap::types::{Roadmap, Section, Task, TaskStatus};
use crate::roadmap::str_utils::{self, slugify};
use anyhow::Result;
use regex::Regex;

/// Parses a ROADMAP.md file into a Roadmap struct.
pub fn parse(content: &str) -> Result<Roadmap> {
    let mut sections = Vec::new();
    let mut current_section_tasks = Vec::new();
    let mut current_section: Option<Section> = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || str_utils::is_ignorable(line) {
            continue;
        }

        if let Some(heading) = parse_heading(line) {
            // Push previous section
            if let Some(mut section) = current_section.take() {
                section.tasks = current_section_tasks;
                sections.push(section);
                current_section_tasks = Vec::new();
            }

            let id = slugify(&heading);
            current_section = Some(Section {
                id,
                heading,
                level: 1, // Simplified
                theme: None,
                tasks: Vec::new(),
                subsections: Vec::new(),
                raw_content: String::new(),
                line_start: 0,
                line_end: 0,
            });
        } else if let Some(task) = parse_task(line) {
            if current_section.is_some() {
                current_section_tasks.push(task);
            }
        }
    }

    // Push last section
    if let Some(mut section) = current_section.take() {
        section.tasks = current_section_tasks;
        sections.push(section);
    }

    Ok(Roadmap {
        path: None,
        title: "Parsed Roadmap".to_string(),
        sections,
        raw: content.to_string(),
    })
}

fn parse_heading(line: &str) -> Option<String> {
    if line.starts_with("# ") || line.starts_with("## ") || line.starts_with("### ") {
        Some(line.trim_start_matches('#').trim().to_string())
    } else {
        None
    }
}

fn parse_task(line: &str) -> Option<Task> {
    // Matches "- [x] Task text" or "- [ ] Task text"
    let re = Regex::new(r"^- \[(x| )\] (.*)").ok()?;
    
    if let Some(caps) = re.captures(line) {
        let status_char = caps.get(1)?.as_str();
        let text_raw = caps.get(2)?.as_str();
        
        let status = if status_char == "x" {
            TaskStatus::Complete
        } else {
            TaskStatus::Pending
        };

        // Legacy Task doesn't store anchors explicitly, just text/path
        let text = text_raw.trim().to_string();
        let id = slugify(&text);

        Some(Task {
            id: id.clone(),
            path: id, // Mapping ID to path for legacy
            text,
            status,
            indent: 0,
            line: 0,
            children: Vec::new(),
            tests: Vec::new(),
        })
    } else {
        None
    }
}

// Public helper for generating IDs
pub fn generate_id(text: &str) -> String {
    slugify(text)
}