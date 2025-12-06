// slopchop:ignore
// src/tui/dashboard/mod.rs
pub mod state;
pub mod ui;

use crate::config::Config;
use crate::roadmap_v2::types::TaskStore;
use crate::tui::runner;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{backend::CrosstermBackend, Terminal};
use state::DashboardApp;
use std::io;
use std::time::Duration;

/// Runs the dashboard TUI.
///
/// # Errors
/// Returns error if IO or terminal operations fail.
pub fn run(config: &mut Config) -> Result<()> {
    // Setup terminal
    runner::setup_terminal()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stdout()))?;

    let mut app = DashboardApp::new(config);

    // Initial load
    app.trigger_scan();
    
    // Attempt to load slopchop.toml (which contains tasks in v2)
    match TaskStore::load(None) {
         Ok(r) => app.roadmap = Some(r),
         Err(e) => app.log(&format!("Failed to load roadmap: {e}")),
    }

    loop {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Global exit
                if key.code == KeyCode::Char('q') {
                    break;
                }
                
                // Route input
                match app.active_tab {
                    state::Tab::Config => {
                        app.config_editor.handle_input(key.code);
                    }
                    _ => handle_input(&mut app, key.code),
                }
            }
        }

        app.on_tick();
        if app.should_quit {
            break;
        }
    }

    runner::restore_terminal()?;
    Ok(())
}

fn handle_input(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Tab => app.next_tab(),
        KeyCode::BackTab => app.previous_tab(),
        KeyCode::Char('r') => {
            app.trigger_scan();
            app.log("Manual scan triggered");
        },
        _ => {}
    }
}