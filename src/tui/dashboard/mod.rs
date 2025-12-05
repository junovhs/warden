// src/tui/dashboard/mod.rs
pub mod state;
pub mod ui;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use state::DashboardApp;
use std::time::Duration;
use crate::apply::{self, types::ApplyContext};
use crate::tui::runner::{spawn_checks, CheckEvent};
use crate::tui::watcher::{spawn_watcher, WatcherEvent};

/// Runs the dashboard main loop.
///
/// # Errors
/// Returns error if drawing fails or event polling errors.
pub fn run<B: ratatui::backend::Backend>(terminal: &mut ratatui::Terminal<B>) -> Result<()> {
    let mut app = DashboardApp::new()?;
    spawn_watcher(app.watch_tx.clone());

    while app.running {
        terminal.draw(|f| ui::draw(f, &mut app))?;
        handle_event(&mut app)?;
        process_worker_messages(&mut app);
        process_watcher_messages(&mut app);
    }
    Ok(())
}

fn handle_event(app: &mut DashboardApp) -> Result<()> {
    if event::poll(Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            process_key(app, key.code);
        }
    }
    app.config.check_message_expiry();
    Ok(())
}

fn process_worker_messages(app: &mut DashboardApp) {
    while let Ok(msg) = app.check_rx.try_recv() {
        match msg {
            CheckEvent::Log(line) => {
                app.check_logs.push(line);
                if app.active_tab == state::Tab::Checks {
                    let target_idx = app.check_logs.len().saturating_sub(10);
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        app.scroll = target_idx as u16;
                    }
                }
            }
            CheckEvent::Finished(_) => {
                app.check_running = false;
                app.check_logs.push("--- Finished ---".into());
            }
        }
    }
}

fn process_watcher_messages(app: &mut DashboardApp) {
    while let Ok(msg) = app.watch_rx.try_recv() {
        match msg {
            WatcherEvent::PayloadDetected(content) => {
                app.pending_payload = Some(content);
                app.show_popup = true;
                app.log_system("Payload detected in clipboard.");
            }
        }
    }
}

fn process_key(app: &mut DashboardApp, key: KeyCode) {
    if app.show_popup {
        handle_popup_input(app, key);
        return;
    }

    if handle_system(app, key) {
        return;
    }
    if handle_navigation(app, key) {
        return;
    }
    route_tab_input(app, key);
}

fn handle_popup_input(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char('y') | KeyCode::Enter => {
            if let Some(content) = app.pending_payload.clone() {
                apply_payload(app, &content);
            }
            app.show_popup = false;
            app.pending_payload = None;
        }
        KeyCode::Char('n') | KeyCode::Esc => {
            app.show_popup = false;
            app.pending_payload = None;
            app.log_system("Payload discarded.");
        }
        _ => {}
    }
}

fn apply_payload(app: &mut DashboardApp, content: &str) {
    app.log_system("Applying payload...");
    
    // We need a Config to create context. app.config holds it but inside TUI structs.
    // Let's just load a fresh one for the Apply logic to ensure safety/freshness.
    let mut config = crate::config::Config::new();
    config.load_local_config();
    
    let ctx = ApplyContext {
        config: &config,
        force: true, // Force true because user said 'y' in TUI
        dry_run: false,
    };

    match apply::process_input(content, &ctx) {
        Ok(outcome) => {
            app.log_system(format!("Apply complete: {outcome:?}"));
            // Trigger checks automatically after apply
            app.switch_tab(state::Tab::Checks);
            app.check_running = true;
            app.check_logs.clear();
            app.check_logs.push("Running post-apply verification...".into());
            spawn_checks(app.check_tx.clone());
        }
        Err(e) => app.log_system(format!("Apply failed: {e}")),
    }
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

fn route_tab_input(app: &mut DashboardApp, key: KeyCode) {
    match app.active_tab {
        state::Tab::Config => app.config.handle_input(key),
        state::Tab::Checks => handle_checks_input(app, key),
        state::Tab::Roadmap => handle_scroll(app, key),
        _ => {}
    }
}

fn handle_checks_input(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char('r') => {
            if !app.check_running {
                app.check_running = true;
                app.check_logs.clear();
                app.check_logs.push("Starting checks...".into());
                spawn_checks(app.check_tx.clone());
            }
        }
        _ => handle_scroll(app, key),
    }
}

fn handle_scroll(app: &mut DashboardApp, key: KeyCode) {
    match key {
        KeyCode::Char('j') | KeyCode::Down => app.scroll_down(),
        KeyCode::Char('k') | KeyCode::Up => app.scroll_up(),
        _ => {}
    }
}