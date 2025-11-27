// src/apply/extractor.rs
use crate::apply::types::FileContent;
use anyhow::Result;
use regex::Regex;
use std::collections::HashMap;

/// Extracts the optional PLAN block.
#[must_use]
pub fn extract_plan(response: &str) -> Option<String> {
    // FIX: Added (?m)^ to ensure PLAN block is at the start of a line.
    // This prevents "Inception" bugs where documentation inside a file triggers the parser.
    let open_re = Regex::new(r"(?m)^∇∇∇\s*PLAN\s*∇∇∇\s*$").ok()?;
    let close_re = Regex::new(r"(?m)^∆∆∆\s*$").ok()?;

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
    let header_re = Regex::new(r"(?m)^∇∇∇\s*(.+?)\s*∇∇∇\s*$")?;
    let footer_re = Regex::new(r"(?m)^∆∆∆\s*$")?;

    let mut current_pos = 0;

    while let Some(header_match) = header_re.find_at(response, current_pos) {
        current_pos = process_block(
            response,
            header_match,
            &footer_re,
            &mut files
        );
    }

    Ok(files)
}

fn process_block(
    response: &str,
    header_match: regex::Match,
    footer_re: &Regex,
    files: &mut HashMap<String, FileContent>,
) -> usize {
    let raw_path = header_match.as_str().replace('∇', "").trim().to_string();

    // Skip MANIFEST and PLAN blocks
    if raw_path == "MANIFEST" || raw_path == "PLAN" {
        return skip_block(response, header_match.end(), footer_re);
    }

    let content_start = header_match.end();

    if let Some(footer_match) = footer_re.find_at(response, content_start) {
        let content_end = footer_match.start();
        let raw_content = &response[content_start..content_end];
        let clean_content = clean_nabla_content(raw_content);
        let line_count = clean_content.lines().count();

        files.insert(
            raw_path,
            FileContent {
                content: clean_content,
                line_count,
            },
        );
        footer_match.end()
    } else {
        // Malformed/Truncated block, skip head
        content_start
    }
}

fn skip_block(response: &str, start_pos: usize, footer_re: &Regex) -> usize {
    if let Some(footer_match) = footer_re.find_at(response, start_pos) {
        footer_match.end()
    } else {
        start_pos
    }
}

fn clean_nabla_content(raw: &str) -> String {
    // We want to remove the single leading newline that usually follows the header
    // and the single trailing newline before the footer, but keep everything else.
    let content = raw.trim_matches('\n');
    content.to_string()
}