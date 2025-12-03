// src/clipboard/windows.rs
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

/// Copies the file at the given path to the clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn copy_file_handle(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    let escaped_path = path_str.replace('\'', "''");
    let cmd = format!("Set-Clipboard -Path '{escaped_path}'");

    Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &cmd])
        .output()
        .context("Failed to set clipboard via PowerShell")?;
    Ok(())
}

/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_copy(text: &str) -> Result<()> {
    let mut child = Command::new("clip").stdin(Stdio::piped()).spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_read() -> Result<String> {
    let output = Command::new("powershell")
        .args(["-command", "Get-Clipboard"])
        .output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
