// src/apply/extractor.rs
use crate::apply::types::FileContent;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

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
        let path = header_match.as_str()
            .replace('∇', "")
            .trim()
            .to_string();

        let content_start = header_match.end();

        // Find the next footer starting from where the header ended
        if let Some(footer_match) = footer_re.find_at(response, content_start) {
            let content_end = footer_match.start();
            
            // Extract and clean content
            // We trim the immediate newline after the header and before the footer
            // but preserve indentation and internal newlines.
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

            // Move past this block
            current_pos = footer_match.end();
        } else {
            // If we found a header but no footer, the file is truncated or malformed.
            // We skip it and try to find the next header (or just stop).
            // In a strict mode, we might error here, but for now we proceed.
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