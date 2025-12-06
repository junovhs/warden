// src/tui/dashboard/state.rs
use crate::types::ScanReport;
use crate::config::Config;
use crate::roadmap_v2::types::TaskStore;
use crate::tui::config::state::ConfigApp;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Dashboard,
    Roadmap,
    Config,
    Logs,
}

pub struct DashboardApp<'a> {
    pub config: &'a mut Config,
    pub active_tab: Tab,
    pub scan_report: Option<ScanReport>,
    pub roadmap: Option<TaskStore>,
    pub config_editor: ConfigApp,
    pub last_scan: Option<Instant>,
    pub logs: Vec<String>,
    pub should_quit: bool,
    pub scroll: u16,
    pub roadmap_scroll: u16,
    pub roadmap_filter: TaskStatusFilter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatusFilter {
    All,
    Pending,
    Done,
}

impl<'a> DashboardApp<'a> {
    pub fn new(config: &'a mut Config) -> Self {
        Self {
            config,
            active_tab: Tab::Dashboard,
            scan_report: None,
            roadmap: None,
            config_editor: ConfigApp::new(),
            last_scan: None,
            logs: vec!["Welcome to SlopChop Dashboard".to_string()],
            should_quit: false,
            scroll: 0,
            roadmap_scroll: 0,
            roadmap_filter: TaskStatusFilter::All,
        }
    }

    pub fn log(&mut self, message: &str) {
        self.logs.push(format!("> {message}"));
        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    pub fn on_tick(&mut self) {
        if self.active_tab == Tab::Dashboard {
            if let Some(last) = self.last_scan {
                if last.elapsed() > Duration::from_secs(5) {
                    self.trigger_scan();
                }
            } else {
                self.trigger_scan();
            }
        }
        self.config_editor.check_message_expiry();
    }

    pub fn trigger_scan(&mut self) {
        self.last_scan = Some(Instant::now());
    }

    pub fn next_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Dashboard => Tab::Roadmap,
            Tab::Roadmap => Tab::Config,
            Tab::Config => Tab::Logs,
            Tab::Logs => Tab::Dashboard,
        };
    }

    pub fn previous_tab(&mut self) {
        self.active_tab = match self.active_tab {
            Tab::Dashboard => Tab::Logs,
            Tab::Logs => Tab::Config,
            Tab::Config => Tab::Roadmap,
            Tab::Roadmap => Tab::Dashboard,
        };
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}