// src/tui/runner.rs
use crate::config::Config;
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub enum CheckEvent {
    Log(String),
    Finished(bool), // success?
}

/// Spawns a background thread to run the configured check commands.
/// Streams output line-by-line to the provided sender.
pub fn spawn_checks(tx: Sender<CheckEvent>) {
    thread::spawn(move || {
        run_check_sequence(&tx);
    });
}

fn run_check_sequence(tx: &Sender<CheckEvent>) {
    let mut config = Config::new();
    config.load_local_config();

    let mut success = true;

    if let Some(commands) = config.commands.get("check") {
        for cmd in commands {
            if !run_single_command(cmd, tx) {
                success = false;
                break;
            }
        }
    } else {
        let _ = tx.send(CheckEvent::Log("No [check] commands configured.".into()));
    }

    // Always run internal scan
    if success && !run_single_command("slopchop", tx) {
        success = false;
    }

    let _ = tx.send(CheckEvent::Finished(success));
}

fn run_single_command(cmd_str: &str, tx: &Sender<CheckEvent>) -> bool {
    let _ = tx.send(CheckEvent::Log(format!("> {cmd_str}")));

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    let Some((prog, args)) = parts.split_first() else {
        return true;
    };

    let child = match Command::new(prog)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            let _ = tx.send(CheckEvent::Log(format!("Failed to start: {e}")));
            return false;
        }
    };

    // Wait for it to finish
    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            let _ = tx.send(CheckEvent::Log(format!("Wait failed: {e}")));
            return false;
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    for line in stdout.lines() {
        let _ = tx.send(CheckEvent::Log(line.to_string()));
    }
    for line in stderr.lines() {
        let _ = tx.send(CheckEvent::Log(line.to_string()));
    }

    if output.status.success() {
        let _ = tx.send(CheckEvent::Log("OK".to_string()));
        true
    } else {
        let _ = tx.send(CheckEvent::Log("FAILED".to_string()));
        false
    }
}