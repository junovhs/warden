// src/tui/dashboard/state.rs
use crate::roadmap::Roadmap;
use crate::tui::config::state::ConfigApp;
use anyhow::Result;
use std::path::Path;

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
}

impl Default for DashboardApp {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            active_tab: Tab::Roadmap,
            running: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            roadmap: None,
            scroll: 0,
            config: ConfigApp::new(),
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

        Ok(Self {
            active_tab: Tab::Roadmap,
            running: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            roadmap,
            scroll: 0,
            config: ConfigApp::new(),
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
}