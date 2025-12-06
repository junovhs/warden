use crate::roadmap::cmd_helpers;
use crate::roadmap::types::{MovePosition, Roadmap};
use anyhow::{anyhow, Result};

pub fn handle_check(roadmap: &mut Roadmap, path: &str) -> Result<()> {
    let idx = cmd_helpers::find_line_idx(roadmap, path)
        .ok_or_else(|| anyhow!("Task not found: {}", path))?;

    let line = roadmap.raw.lines().nth(idx).unwrap().to_string();
    let new_line = line.replace("- [ ]", "- [x]");
    cmd_helpers::replace_raw(roadmap, idx, new_line);
    Ok(())
}

pub fn handle_uncheck(roadmap: &mut Roadmap, path: &str) -> Result<()> {
    let idx = cmd_helpers::find_line_idx(roadmap, path)
        .ok_or_else(|| anyhow!("Task not found: {}", path))?;

    let line = roadmap.raw.lines().nth(idx).unwrap().to_string();
    let new_line = line.replace("- [x]", "- [ ]");
    cmd_helpers::replace_raw(roadmap, idx, new_line);
    Ok(())
}

pub fn handle_delete(roadmap: &mut Roadmap, path: &str) -> Result<()> {
    let idx = cmd_helpers::find_line_idx(roadmap, path)
        .ok_or_else(|| anyhow!("Task not found: {}", path))?;
    
    cmd_helpers::remove_raw(roadmap, idx);
    Ok(())
}

pub fn handle_add(roadmap: &mut Roadmap, parent: &str, text: &str, after: Option<&str>) -> Result<()> {
    let lines: Vec<&str> = roadmap.raw.lines().collect();
    let idx = cmd_helpers::scan_insertion_point(&lines, parent, after)
        .ok_or_else(|| anyhow!("Could not determine insertion point for parent '{}'", parent))?;

    let new_line = format!("- [ ] {}", text);
    cmd_helpers::insert_raw(roadmap, idx, new_line);
    Ok(())
}

pub fn handle_update(roadmap: &mut Roadmap, path: &str, text: &str) -> Result<()> {
    let idx = cmd_helpers::find_line_idx(roadmap, path)
        .ok_or_else(|| anyhow!("Task not found: {}", path))?;

    let line = roadmap.raw.lines().nth(idx).unwrap();
    let parts: Vec<&str> = line.splitn(3, ']').collect();
    if parts.len() < 2 {
         return Err(anyhow!("Malformed task line"));
    }
    let prefix = format!("{}]", parts[0]); 
    let new_line = format!("{} {}", prefix, text);
    
    cmd_helpers::replace_raw(roadmap, idx, new_line);
    Ok(())
}

pub fn handle_note(roadmap: &mut Roadmap, path: &str, note: &str) -> Result<()> {
    let idx = cmd_helpers::find_line_idx(roadmap, path)
        .ok_or_else(|| anyhow!("Task not found: {}", path))?;
    
    let task_line = roadmap.raw.lines().nth(idx).unwrap();
    let indent = task_line.chars().take_while(|c| c.is_whitespace()).collect::<String>();
    let note_line = format!("{}  > {}", indent, note);
    
    cmd_helpers::insert_raw(roadmap, idx + 1, note_line);
    Ok(())
}

pub fn handle_move(_roadmap: &mut Roadmap, _path: &str, _pos: MovePosition) -> Result<()> {
    // slopchop:ignore Legacy move not implemented for text-based roadmap manipulation
    Err(anyhow!("MOVE command not supported in legacy mode"))
}

pub fn handle_add_section(roadmap: &mut Roadmap, heading: &str) -> Result<()> {
    let new_line = format!("\n## {}\n", heading);
    let mut raw = roadmap.raw.clone();
    raw.push_str(&new_line);
    roadmap.raw = raw;
    Ok(())
}

pub fn handle_add_subsection(roadmap: &mut Roadmap, _parent: &str, heading: &str) -> Result<()> {
    let new_line = format!("\n### {}\n", heading);
    let mut raw = roadmap.raw.clone();
    raw.push_str(&new_line);
    roadmap.raw = raw;
    Ok(())
}

pub fn handle_chain(_roadmap: &mut Roadmap, _parent: &str, _items: Vec<String>) -> Result<()> {
    Err(anyhow!("CHAIN command not supported in legacy mode"))
}