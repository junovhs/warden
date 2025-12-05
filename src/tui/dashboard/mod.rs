// src/tui/dashboard/mod.rs
pub mod state;
mod ui;

use crate::tui::watcher;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::Backend;
use ratatui::Terminal;
use state::DashboardApp;
use std::time::Duration;

/// Run the dashboard TUI.
///
/// # Errors
/// Returns error if terminal operations fail or watcher cannot start.
pub fn run<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = DashboardApp::default();
    watcher::spawn_watcher(app.watch_tx.clone());

    while app.running {
        terminal.draw(|f| ui::draw(f, &mut app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(&mut app, key.code);
                }
            }
        }

        poll_events(&mut app);
    }

    Ok(())
}

fn handle_key(app: &mut DashboardApp, code: KeyCode) {
    if app.show_popup {
        handle_popup_key(app, code);
        return;
    }

    match code {
        KeyCode::Char('q') => app.running = false,
        KeyCode::Char('1') => app.switch_tab(state::Tab::Roadmap),
        KeyCode::Char('2') => app.switch_tab(state::Tab::Checks),
        KeyCode::Char('3') => app.switch_tab(state::Tab::Context),
        KeyCode::Char('4') => app.switch_tab(state::Tab::Config),
        KeyCode::Char('5') => app.switch_tab(state::Tab::Logs),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_up(),
        KeyCode::Down | KeyCode::Char('j') => app.scroll_down(),
        KeyCode::Char('r') => handle_refresh(app),
        _ => {}
    }
}

fn handle_popup_key(app: &mut DashboardApp, code: KeyCode) {
    match code {
        KeyCode::Char('y') | KeyCode::Enter => {
            app.log_system("Applying payload...");
            app.show_popup = false;
            app.pending_payload = None;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.log_system("Payload dismissed.");
            app.show_popup = false;
            app.pending_payload = None;
        }
        _ => {}
    }
}

fn handle_refresh(app: &mut DashboardApp) {
    match app.active_tab {
        state::Tab::Roadmap => {
            app.roadmap = crate::roadmap::Roadmap::from_file(std::path::Path::new("ROADMAP.md")).ok();
            app.refresh_flat_roadmap();
            app.log_system("Roadmap refreshed.");
        }
        state::Tab::Context => {
            app.refresh_context_items();
            app.log_system("Context refreshed.");
        }
        _ => {}
    }
}

fn poll_events(app: &mut DashboardApp) {
    while let Ok(event) = app.watch_rx.try_recv() {
        match event {
            watcher::WatcherEvent::PayloadDetected(payload) => {
                app.pending_payload = Some(payload);
                app.show_popup = true;
                app.log_system("Payload detected in clipboard!");
            }
        }
    }

    while let Ok(event) = app.check_rx.try_recv() {
        match event {
            crate::tui::runner::CheckEvent::Log(line) => {
                app.check_logs.push(line);
            }
            crate::tui::runner::CheckEvent::Finished(success) => {
                app.check_running = false;
                let msg = if success { "Checks passed!" } else { "Checks failed." };
                app.log_system(msg);
            }
        }
    }
}