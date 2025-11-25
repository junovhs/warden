// src/clipboard.rs
use anyhow::Result;
use std::process::Command;

/// Copies text to the system clipboard.
///
/// # Errors
/// Returns error if the system clipboard command fails or is missing.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    perform_copy(text)
}

#[cfg(target_os = "macos")]
fn perform_copy(text: &str) -> Result<()> {
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

#[cfg(target_os = "linux")]
fn perform_copy(text: &str) -> Result<()> {
    use std::io::Write;
    let mut child = Command::new("xclip")
        .args(["-selection", "clipboard"])
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(text.as_bytes())?;
    }
    child.wait()?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn perform_copy(text: &str) -> Result<()> {
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

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn perform_copy(_text: &str) -> Result<()> {
    use anyhow::anyhow;
    Err(anyhow!("Clipboard not supported on this OS"))
}
