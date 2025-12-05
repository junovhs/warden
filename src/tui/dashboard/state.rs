// src/tui/dashboard/state.rs
use crate::roadmap::types::Section;
use crate::roadmap::{Roadmap, TaskStatus};
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

#[derive(Debug, Clone)]
pub struct FlatTask {
    pub id: String,
    pub text: String,
    pub status: TaskStatus,
    pub indent: usize,
    pub is_header: bool,
}

pub struct DashboardApp {
    pub active_tab: Tab,
    pub running: bool,
    pub version: String,
    pub roadmap: Option<Roadmap>,
    pub flat_roadmap: Vec<FlatTask>,
    pub selected_task: usize,
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
                flat_roadmap: Vec::new(),
                selected_task: 0,
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
    pub fn new() -> Result<Self> {
        let roadmap = Roadmap::from_file(Path::new("ROADMAP.md")).ok();
        let (ctx, crx) = mpsc::channel();
        let (wtx, wrx) = mpsc::channel();

        let mut app = Self {
            active_tab: Tab::Roadmap,
            running: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            roadmap,
            flat_roadmap: Vec::new(),
            selected_task: 0,
            config: ConfigApp::new(),
            check_logs: Vec::new(),
            check_running: false,
            check_tx: ctx,
            check_rx: crx,
            watch_tx: wtx,
            watch_rx: wrx,
            pending_payload: None,
            show_popup: false,
            system_logs: vec!["System initialized.".into()],
        };
        app.refresh_flat_roadmap();
        Ok(app)
    }

    pub fn refresh_flat_roadmap(&mut self) {
        self.flat_roadmap.clear();
        if let Some(r) = &self.roadmap {
            for section in &r.sections {
                flatten_section(section, 0, &mut self.flat_roadmap);
            }
        }
    }

    pub fn switch_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
    }

    pub fn scroll_up(&mut self) {
        if self.active_tab == Tab::Roadmap && self.selected_task > 0 {
            self.selected_task -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        if self.active_tab == Tab::Roadmap
            && self.selected_task < self.flat_roadmap.len().saturating_sub(1)
        {
            self.selected_task += 1;
        }
    }

    pub fn log_system(&mut self, msg: impl Into<String>) {
        self.system_logs.push(msg.into());
    }
}

fn flatten_section(section: &Section, indent: usize, out: &mut Vec<FlatTask>) {
    // Only show headers if they have content or we want to show empty sections
    if section.tasks.is_empty() && section.subsections.is_empty() {
        return;
    }

    out.push(FlatTask {
        id: section.id.clone(),
        text: section.heading.clone(),
        status: TaskStatus::Pending, // Headers aren't tasks
        indent,
        is_header: true,
    });

    for task in &section.tasks {
        out.push(FlatTask {
            id: task.path.clone(),
            text: task.text.clone(),
            status: task.status,
            indent: indent + 1,
            is_header: false,
        });
    }

    for sub in &section.subsections {
        flatten_section(sub, indent + 1, out);
    }
}