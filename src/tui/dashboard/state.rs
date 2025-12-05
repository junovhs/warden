// src/tui/dashboard/state.rs
use crate::roadmap::Roadmap;
use crate::tui::config::state::ConfigApp;
use crate::tui::runner::CheckEvent;
use crate::tui::watcher::WatcherEvent;
use anyhow::Result;
use std::path::Path;
use std::sync::mpsc::{self, Receiver, Sender};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tab {
    Roadmap,
    Checks,
    Context,
    Config,
    Logs,
}

pub struct DashboardApp {
    pub active_tab: Tab,
    pub running: bool,
    pub version: String,
    pub roadmap: Option<Roadmap>,
    pub scroll: u16,
    pub config: ConfigApp,
    
    // Checks Tab State
    pub check_logs: Vec<String>,
    pub check_running: bool,
    pub check_tx: Sender<CheckEvent>,
    pub check_rx: Receiver<CheckEvent>,

    // Watcher State
    pub watch_tx: Sender<WatcherEvent>,
    pub watch_rx: Receiver<WatcherEvent>,
    pub pending_payload: Option<String>,
    pub show_popup: bool,
    pub system_logs: Vec<String>,
}

impl Default for DashboardApp {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| {
            let (ctx, crx) = mpsc::channel();
            let (wtx, wrx) = mpsc::channel();
            Self {
                active_tab: Tab::Roadmap,
                running: true,
                version: env!("CARGO_PKG_VERSION").to_string(),
                roadmap: None,
                scroll: 0,
                config: ConfigApp::new(),
                check_logs: Vec::new(),
                check_running: false,
                check_tx: ctx,
                check_rx: crx,
                watch_tx: wtx,
                watch_rx: wrx,
                pending_payload: None,
                show_popup: false,
                system_logs: Vec::new(),
            }
        })
    }
}

impl DashboardApp {
    /// Initialize the dashboard state.
    ///
    /// # Errors
    /// Returns error if roadmap loading fails (though we handle it gracefully).
    pub fn new() -> Result<Self> {
        let roadmap = Roadmap::from_file(Path::new("ROADMAP.md")).ok();
        let (ctx, crx) = mpsc::channel();
        let (wtx, wrx) = mpsc::channel();

        Ok(Self {
            active_tab: Tab::Roadmap,
            running: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            roadmap,
            scroll: 0,
            config: ConfigApp::new(),
            check_logs: Vec::new(),
            check_running: false,
            check_tx: ctx,
            check_rx: crx,
            watch_tx: wtx,
            watch_rx: wrx,
            pending_payload: None,
            show_popup: false,
            system_logs: vec!["System initialized. Watching clipboard...".into()],
        })
    }

    pub fn switch_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        self.scroll = 0;
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn log_system(&mut self, msg: impl Into<String>) {
        self.system_logs.push(msg.into());
    }
}