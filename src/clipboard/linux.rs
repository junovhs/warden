// src/clipboard/linux.rs
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

fn is_wsl() -> bool {
    if std::env::var("WSL_DISTRO_NAME").is_ok() {
        return true;
    }
    if let Ok(version) = std::fs::read_to_string("/proc/version") {
        let v = version.to_lowercase();
        return v.contains("microsoft") || v.contains("wsl");
    }
    false
}

/// Copies the file at the given path to the clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn copy_file_handle(path: &Path) -> Result<()> {
    if is_wsl() {
        return copy_file_handle_wsl(path);
    }
    copy_file_handle_native(path)
}

fn copy_file_handle_native(path: &Path) -> Result<()> {
    let uri = format!("file://{}", path.to_string_lossy());

    if try_wl_copy_uri(&uri).is_ok() {
        return Ok(());
    }
    copy_uri_xclip(&uri)
}

fn try_wl_copy_uri(uri: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .args(["--type", "text/uri-list"])
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        write!(stdin, "{uri}")?;
    }
    child.wait()?;
    Ok(())
}

fn copy_uri_xclip(uri: &str) -> Result<()> {
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-t", "text/uri-list", "-i"])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn xclip")?;

    if let Some(mut stdin) = child.stdin.take() {
        write!(stdin, "{uri}").context("Failed to write to xclip")?;
    }
    child.wait().context("Failed to wait for xclip")?;
    Ok(())
}

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

/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_copy(text: &str) -> Result<()> {
    if is_wsl() {
        return perform_copy_wsl(text);
    }
    perform_copy_native(text)
}

fn perform_copy_wsl(text: &str) -> Result<()> {
    if try_wsl_clip(text).is_ok() {
        return Ok(());
    }
    try_wsl_powershell(text)
}

fn try_wsl_clip(text: &str) -> Result<()> {
    let mut child = Command::new("clip.exe")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn clip.exe")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .context("Failed to write to clip.exe")?;
    }
    child.wait().context("Failed to wait for clip.exe")?;
    Ok(())
}

fn try_wsl_powershell(text: &str) -> Result<()> {
    let mut child = Command::new("powershell.exe")
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-Command",
            "$Input | Set-Clipboard",
        ])
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn powershell.exe")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .context("Failed to write to powershell.exe")?;
    }
    child.wait().context("Failed to wait for powershell.exe")?;
    Ok(())
}

fn perform_copy_native(text: &str) -> Result<()> {
    if try_xclip_copy(text).is_ok() {
        return Ok(());
    }
    try_wl_copy(text)
}

fn try_xclip_copy(text: &str) -> Result<()> {
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard", "-in"])
        .stdin(Stdio::piped())
        .spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

fn try_wl_copy(text: &str) -> Result<()> {
    let mut child = Command::new("wl-copy")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to spawn wl-copy")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(text.as_bytes())
            .context("Failed to write to wl-copy")?;
    }
    child.wait().context("Failed to wait for wl-copy")?;
    Ok(())
}

/// Reads text from the system clipboard.
///
/// # Errors
/// Returns error if the external clipboard command fails.
pub fn perform_read() -> Result<String> {
    if is_wsl() {
        return perform_read_wsl();
    }
    perform_read_native()
}

fn perform_read_wsl() -> Result<String> {
    let output = Command::new("powershell.exe")
        .args(["-NoProfile", "-NonInteractive", "-Command", "Get-Clipboard"])
        .output()
        .context("Failed to run Get-Clipboard via powershell.exe")?;
    Ok(String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string())
}

fn perform_read_native() -> Result<String> {
    if let Ok(output) = Command::new("xclip")
        .args(["-selection", "clipboard", "-out"])
        .output()
    {
        return Ok(String::from_utf8_lossy(&output.stdout).to_string());
    }
    let output = Command::new("wl-paste").output()?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
