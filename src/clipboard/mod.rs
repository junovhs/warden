// src/clipboard/mod.rs
pub mod platform;
pub mod temp;

use crate::tokens::Tokenizer;
use anyhow::Result;
use std::path::Path;

/// Smartly copies text or file handles based on size.
///
/// # Errors
/// Returns error if clipboard access fails or temp file creation fails.
pub fn smart_copy(text: &str) -> Result<String> {
    // 1. The Garbage Man: Clean up old artifacts first
    temp::cleanup_temp_files();

    // 2. Check Size
    let token_count = Tokenizer::count(text);

    if token_count < 1500 {
        // Small? Text Copy.
        platform::perform_copy(text)?;
        Ok("Text copied to clipboard".to_string())
    } else {
        // Huge? File Copy.
        let file_path = temp::write_to_temp(text)?;
        platform::copy_file_handle(&file_path)?;

        let filename = file_path
            .file_name()
            .map_or_else(|| "temp_file".into(), |n| n.to_string_lossy());

        Ok(format!(
            "Large content ({token_count} tokens). Copied as file attachment: {filename}"
        ))
    }
}

/// Copies a file path to clipboard so it can be pasted as a file attachment.
///
/// # Errors
/// Returns error if clipboard access fails.
pub fn copy_file_path(path: &Path) -> Result<()> {
    platform::copy_file_handle(path)
}

/// Wrapper for backward compatibility.
///
/// # Errors
/// Returns error if clipboard access fails.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let _ = smart_copy(text)?;
    Ok(())
}

/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if clipboard access fails.
pub fn read_clipboard() -> Result<String> {
    platform::perform_read()
}
