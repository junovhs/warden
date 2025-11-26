// src/apply/manifest.rs
use crate::apply::types::{ManifestEntry, Operation};
use anyhow::Result;
use regex::Regex;

/// Parses the delivery manifest block.
///
/// # Robustness Features
/// - Uses Regex to strictly identify and strip Markdown list markers.
/// - Distinguishes between "1. file" (list) and ".gitignore" (file).
/// - Case-insensitive tag matching.
///
/// # Errors
/// Returns an error if the regular expressions fail to compile.
pub fn parse_manifest(response: &str) -> Result<Option<Vec<ManifestEntry>>> {
    // Regex for the container tags
    let open_re = Regex::new(r"(?i)<delivery>")?;
    let close_re = Regex::new(r"(?i)</delivery>")?;

    // Regex to strip markdown list markers (e.g. "- ", "* ", "1. ")
    // Matches start-of-line, optional indent, marker, AND mandatory trailing space.
    let list_marker_re = Regex::new(r"^\s*(?:[-*]|\d+\.)\s+")?;

    let start_match = open_re.find(response);
    let end_match = close_re.find(response);

    if let (Some(start), Some(end)) = (start_match, end_match) {
        let block = &response[start.end()..end.start()];
        let mut entries = Vec::new();

        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            // Surgical removal of list markers
            let clean_line = list_marker_re.replace(trimmed, "");
            let clean_line = clean_line.trim();

            if clean_line.is_empty() {
                continue;
            }

            // Parse Operation
            let upper = clean_line.to_uppercase();
            let (path_raw, op) = if upper.contains("[NEW]") {
                (
                    clean_line.replace("[NEW]", "").replace("[new]", ""),
                    Operation::New,
                )
            } else if upper.contains("[DELETE]") {
                (
                    clean_line.replace("[DELETE]", "").replace("[delete]", ""),
                    Operation::Delete,
                )
            } else {
                (clean_line.to_string(), Operation::Update)
            };

            // Extract the path (stop at first whitespace to handle inline comments)
            // e.g. "src/main.rs - The main file" -> "src/main.rs"
            let final_path = path_raw
                .split_whitespace()
                .next()
                .unwrap_or(&path_raw)
                .to_string();

            if !final_path.is_empty() {
                entries.push(ManifestEntry {
                    path: final_path,
                    operation: op,
                });
            }
        }

        Ok(Some(entries))
    } else {
        Ok(None)
    }
}
