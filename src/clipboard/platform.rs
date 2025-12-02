// src/clipboard/platform.rs
#![allow(unused_imports)] // OS-specific imports vary

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

// --- Linux / WSL Helpers ---

#[cfg(target_os = "linux")]
fn is_wsl() -> bool {
    if let Ok(version) = std::fs::read_to_string("/proc/version") {
        let v = version.to_lowercase();
        return v.contains("microsoft") || v.contains("wsl");
    }
    false
}

// --- File Handle Operations ---

#[cfg(target_os = "windows")]
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

#[cfg(target_os = "macos")]
/// Copies the file at the given path to the clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn copy_file_handle(path: &Path) -> Result<()> {
    let path_str = path.to_string_lossy();
    let script = format!("set the clipboard to POSIX file \"{path_str}\"");

    Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output()
        .context("Failed to set clipboard via osascript")?;
    Ok(())
}

#[cfg(target_os = "linux")]
/// Copies the file at the given path to the clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn copy_file_handle(path: &Path) -> Result<()> {
    if is_wsl() {
        return copy_file_handle_wsl(path);
    }

    let path_str = path.to_string_lossy();
    let uri = format!("file://{path_str}");

    // Try wl-copy (Wayland)
    if let Ok(mut child) = Command::new("wl-copy")
        .args(["--type", "text/uri-list"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            use std::io::Write;
            let _ = write!(stdin, "{uri}");
        }
        if child.wait().is_ok() {
            return Ok(());
        }
    }

    // Fallback to xclip (X11)
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "text/uri-list", "-i"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        write!(stdin, "{uri}")?;
    }
    child.wait()?;
    Ok(())
}

#[cfg(target_os = "linux")]
fn copy_file_handle_wsl(path: &Path) -> Result<()> {
    let output = Command::new("wslpath")
        .arg("-w")
        .arg(path)
        .output()
        .context("Failed to run wslpath")?;

    let win_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if win_path.is_empty() {
        return Err(anyhow::anyhow!("wslpath returned empty string"));
    }

    let escaped = win_path.replace('\'', "''");
    let cmd = format!("Set-Clipboard -Path '{escaped}'");

    Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", &cmd])
        .output()
        .context("Failed to set clipboard via powershell.exe in WSL")?;

    Ok(())
}

// --- Text Operations ---

#[cfg(target_os = "macos")]
/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_copy(text: &str) -> Result<()> {
    use std::io::Write;
    let mut child = Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

#[cfg(target_os = "macos")]
/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_read() -> Result<String> {
    let output = Command::new("pbpaste").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "linux")]
/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_copy(text: &str) -> Result<()> {
    use std::io::Write;

    if is_wsl() {
        let mut child = Command::new("clip.exe")
            .stdin(std::process::Stdio::piped())
            .spawn()
            .context("Failed to spawn clip.exe")?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
        return Ok(());
    }

    if let Ok(mut child) = Command::new("xclip")
        .args(["-selection", "clipboard", "-in"])
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes())?;
        }
        child.wait()?;
        return Ok(());
    }

    let mut child = Command::new("wl-copy")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

#[cfg(target_os = "linux")]
/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_read() -> Result<String> {
    if is_wsl() {
        let output = Command::new("powershell.exe")
            .args(["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard"])
            .output()
            .context("Failed to run Get-Clipboard via powershell.exe")?;
        return Ok(String::from_utf8_lossy(&output.stdout)
            .trim_end()
            .to_string());
    }

    if let Ok(output) = Command::new("xclip")
        .args(["-selection", "clipboard", "-out"])
        .output()
    {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    let output = Command::new("wl-paste").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[cfg(target_os = "windows")]
/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_copy(text: &str) -> Result<()> {
    use std::io::Write;
    let mut child = Command::new("clip")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

#[cfg(target_os = "windows")]
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
