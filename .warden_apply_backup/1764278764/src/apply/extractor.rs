// src/apply/extractor.rs
use crate::apply::types::FileContent;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Extracts the optional PLAN block.
#[must_use]
pub fn extract_plan(response: &str) -> Option<String> {
    let open_re = Regex::new(r"∇∇∇\s*PLAN\s*∇∇∇").ok()?;
    let close_re = Regex::new(r"∆∆∆").ok()?;

    let start_match = open_re.find(response)?;
    let end_match = close_re.find_at(response, start_match.end())?;

    let content = &response[start_match.end()..end_match.start()];
    Some(content.trim().to_string())
}

/// Extracts file blocks using the Robust Delimiter Protocol (Nabla Format).
///
/// Format:
/// ∇∇∇ path/to/file.rs ∇∇∇
/// [content]
/// ∆∆∆
///
/// # Errors
/// Returns error if regex compilation fails.
pub fn extract_files(response: &str) -> Result<HashMap<String, FileContent>> {
    let mut files = HashMap::new();
    
    // ∇∇∇ path ∇∇∇
    // We capture the path loosely to allow for whitespace variance
    let header_re = Regex::new(r"(?m)^∇∇∇\s*(.+?)\s*∇∇∇\s*$")?;
    
    // ∆∆∆
    let footer_re = Regex::new(r"(?m)^∆∆∆\s*$")?;

    let mut current_pos = 0;

    while let Some(header_match) = header_re.find_at(response, current_pos) {
        let raw_path = header_match.as_str().replace('∇', "").trim().to_string();
        
        // Skip MANIFEST and PLAN blocks
        if raw_path == "MANIFEST" || raw_path == "PLAN" {
             if let Some(footer_match) = footer_re.find_at(response, header_match.end()) {
                 current_pos = footer_match.end();
                 continue;
             }
             // Malformed special block, skip head
             current_pos = header_match.end();
             continue;
        }

        let path = raw_path;
        let content_start = header_match.end();

        if let Some(footer_match) = footer_re.find_at(response, content_start) {
            let content_end = footer_match.start();
            let raw_content = &response[content_start..content_end];
            let clean_content = clean_nabla_content(raw_content);
            let line_count = clean_content.lines().count();

            files.insert(
                path,
                FileContent {
                    content: clean_content,
                    line_count,
                },
            );

            current_pos = footer_match.end();
        } else {
            current_pos = content_start;
        }
    }

    Ok(files)
}

fn clean_nabla_content(raw: &str) -> String {
    // We want to remove the single leading newline that usually follows the header
    // and the single trailing newline before the footer, but keep everything else.
    let content = raw.trim_matches('\n');
    
    // We do NOT strip markdown fences anymore. 
    // The Nabla format is designed to hold markdown safely.
    content.to_string()
}