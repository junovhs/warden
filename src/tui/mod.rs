// src/tui/mod.rs
pub mod config;
pub mod dashboard;
pub mod runner;
pub mod state;
pub mod view;
pub mod watcher;

use crate::config::Config;
use crate::error::Result;

/// Runs the TUI application.
///
/// # Errors
/// Returns error if TUI execution fails or IO error occurs.
pub fn run(config: &mut Config) -> Result<()> {
    // Map anyhow::Result to crate::error::Result
    dashboard::run(config).map_err(|e| crate::error::SlopChopError::from(std::io::Error::other(
        e.to_string(),
    )))
}

/// Runs the configuration TUI.
///
/// # Errors
/// Returns error if TUI setup or execution fails.
pub fn run_config() -> Result<()> {
    runner::setup_terminal().map_err(|e| crate::error::SlopChopError::Other(e.to_string()))?;
    
    let mut terminal = ratatui::Terminal::new(ratatui::backend::CrosstermBackend::new(std::io::stdout()))
        .map_err(|e| crate::error::SlopChopError::Other(e.to_string()))?;
        
    let mut app = config::state::ConfigApp::new();
    let res = app.run(&mut terminal);
    
    runner::restore_terminal().map_err(|e| crate::error::SlopChopError::Other(e.to_string()))?;
    
    res.map_err(|e| crate::error::SlopChopError::Other(e.to_string()))
}