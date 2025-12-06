// slopchop:ignore
// src/roadmap/cmd_helpers.rs
use crate::roadmap::types::Roadmap;
use crate::roadmap::str_utils::slugify;
use regex::Regex;

pub fn find_line_idx(roadmap: &Roadmap, path: &str) -> Option<usize> {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    find_line_idx_in_lines(&lines, path)
}

pub fn find_line_idx_in_lines(lines: &[&str], path: &str) -> Option<usize> {
    for (i, line) in lines.iter().enumerate() {
        if is_task(line) {
            let text = extract_task_text(line);
            if slugify(&text) == path {
                return Some(i);
            }
        }
    }
    None
}

pub fn is_task(line: &str) -> bool {
    line.trim().starts_with("- [ ]") || line.trim().starts_with("- [x]")
}

fn extract_task_text(line: &str) -> String {
    let re = Regex::new(r"- \[[x ]\] (.*)").unwrap();
    if let Some(caps) = re.captures(line) {
        return caps.get(1).map_or("", |m| m.as_str()).trim().to_string();
    }
    line.to_string()
}

pub fn replace_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, move |lines| {
        if idx < lines.len() {
            lines[idx] = line;
        }
    });
}

pub fn insert_raw(roadmap: &mut Roadmap, idx: usize, line: String) {
    modify_lines(roadmap, move |lines| {
        if idx <= lines.len() {
            lines.insert(idx, line);
        }
    });
}

pub fn remove_raw(roadmap: &mut Roadmap, idx: usize) {
    modify_lines(roadmap, move |lines| {
        if idx < lines.len() {
            lines.remove(idx);
        }
    });
}

fn modify_lines<F>(roadmap: &mut Roadmap, f: F)
where
    F: FnOnce(&mut Vec<String>),
{
    let mut lines: Vec<String> = roadmap.raw.lines().map(String::from).collect();
    f(&mut lines);
    roadmap.raw = lines.join("\n");
}

pub fn scan_insertion_point(lines: &[&str], parent: &str, after: Option<&str>) -> Option<usize> {
    let section_idx = lines.iter().position(|l| {
        l.starts_with("#") && slugify(l.trim_start_matches('#').trim()) == parent
    })?;

    let mut insert_idx = section_idx + 1;
    
    if let Some(target) = after {
        for (i, line) in lines.iter().enumerate().skip(section_idx + 1) {
            if line.starts_with('#') {
                break; 
            }
            if is_task(line) {
                let text = extract_task_text(line);
                if slugify(&text) == target {
                    return Some(i + 1);
                }
            }
        }
    } else {
        for (i, line) in lines.iter().enumerate().skip(section_idx + 1) {
            if line.starts_with('#') {
                return Some(i);
            }
            insert_idx = i + 1;
        }
    }

    Some(insert_idx)
}