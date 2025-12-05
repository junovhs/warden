// src/tui/dashboard/mod.rs
pub mod state;
pub mod ui;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use state::DashboardApp;
use std::time::Duration;

/// Runs the dashboard main loop.
///
/// # Errors
/// Returns error if drawing fails or event polling errors.
pub fn run<B: ratatui::backend::Backend>(terminal: &mut ratatui::Terminal<B>) -> Result<()> {
    let mut app = DashboardApp::new()?;

    while app.running {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        handle_event(&mut app)?;
    }
    Ok(())
}

fn handle_event(app: &mut DashboardApp) -> Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            process_key(app, key.code);
        }
    }
    Ok(())
}

fn process_key(app: &mut DashboardApp, key: KeyCode) {
    if handle_system(app, key) {
        return;
    }
    if handle_navigation(app, key) {
        return;
    }
    handle_scroll(app, key);
}

fn handle_system(app: &mut DashboardApp, key: KeyCode) -> bool {
    if matches!(key, KeyCode::Char('q') | KeyCode::Esc) {
        app.running = false;
        return true;
    }
    false
}

fn handle_navigation(app: &mut DashboardApp, key: KeyCode) -> bool {
    match key {
        KeyCode::Char('1') => app.switch_tab(state::Tab::Roadmap),
        KeyCode::Char('2') => app.switch_tab(state::Tab::Checks),
        KeyCode::Char('3') => app.switch_tab(state::Tab::Context),
        KeyCode::Char('4') => app.switch_tab(state::Tab::Config),
        KeyCode::Char('5') => app.switch_tab(state::Tab::Logs),
        _ => return false,
    }
    true
}

fn handle_scroll(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
        _ => {}
    }
}