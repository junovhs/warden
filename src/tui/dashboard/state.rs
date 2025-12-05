// src/tui/dashboard/state.rs
use crate::config::Config;
use crate::discovery;
use crate::roadmap::types::Section;
use crate::roadmap::{Roadmap, TaskStatus};
use crate::tokens::Tokenizer;
use crate::tui::config::state::ConfigApp;
use crate::tui::runner::CheckEvent;
use crate::tui::watcher::WatcherEvent;
use anyhow::Result;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Clone)]
pub struct ContextItem {
    pub path: PathBuf,
    pub tokens: usize,
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

    // Context Tab State
    pub context_items: Vec<ContextItem>,
    pub selected_file: usize,

    // Watcher State
    pub watch_tx: Sender<WatcherEvent>,
    pub watch_rx: Receiver<WatcherEvent>,
    pub pending_payload: Option<String>,
    pub show_popup: bool,
    pub system_logs: Vec<String>,

    // Generic Scroll State (Logs/Checks)
    pub scroll: u16,
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
                context_items: Vec::new(),
                selected_file: 0,
                watch_tx: wtx,
                watch_rx: wrx,
                pending_payload: None,
                show_popup: false,
                system_logs: Vec::new(),
                scroll: 0,
            }
        })
    }
}

impl DashboardApp {
    /// Create a new dashboard application.
    ///
    /// # Errors
    /// Returns error if roadmap file cannot be parsed.
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
            context_items: Vec::new(),
            selected_file: 0,
            watch_tx: wtx,
            watch_rx: wrx,
            pending_payload: None,
            show_popup: false,
            system_logs: vec!["System initialized.".into()],
            scroll: 0,
        };
        app.refresh_flat_roadmap();
        app.refresh_context_items();
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

    pub fn refresh_context_items(&mut self) {
        let mut config = Config::new();
        config.load_local_config();
        
        if let Ok(files) = discovery::discover(&config) {
            self.context_items = files.into_iter().filter_map(|path| {
                let content = std::fs::read_to_string(&path).ok()?;
                let tokens = Tokenizer::count(&content);
                Some(ContextItem { path, tokens })
            }).collect();

            self.context_items.sort_by(|a, b| b.tokens.cmp(&a.tokens));
        }
    }

    pub fn switch_tab(&mut self, tab: Tab) {
        self.active_tab = tab;
        self.scroll = 0;
    }

    pub fn scroll_up(&mut self) {
        match self.active_tab {
            Tab::Roadmap => {
                if self.selected_task > 0 {
                    self.selected_task -= 1;
                }
            }
            Tab::Context => {
                if self.selected_file > 0 {
                    self.selected_file -= 1;
                }
            }
            Tab::Checks | Tab::Logs => {
                self.scroll = self.scroll.saturating_sub(1);
            }
            Tab::Config => {}
        }
    }

    pub fn scroll_down(&mut self) {
        match self.active_tab {
            Tab::Roadmap => {
                if self.selected_task < self.flat_roadmap.len().saturating_sub(1) {
                    self.selected_task += 1;
                }
            }
            Tab::Context => {
                if self.selected_file < self.context_items.len().saturating_sub(1) {
                    self.selected_file += 1;
                }
            }
            Tab::Checks | Tab::Logs => {
                self.scroll = self.scroll.saturating_add(1);
            }
            Tab::Config => {}
        }
    }

    pub fn log_system(&mut self, msg: impl Into<String>) {
        self.system_logs.push(msg.into());
    }
}

fn flatten_section(section: &Section, indent: usize, out: &mut Vec<FlatTask>) {
    if section.tasks.is_empty() && section.subsections.is_empty() {
        return;
    }

    out.push(FlatTask {
        id: section.id.clone(),
        text: section.heading.clone(),
        status: TaskStatus::Pending,
        indent,
        is_header: true,
    });

    for task in &section.tasks {
        out.push(FlatTask {
            id: task.id.clone(),
            text: task.text.clone(),
            status: task.status.clone(),
            indent: indent + 1,
            is_header: false,
        });
    }

    for sub in &section.subsections {
        flatten_section(sub, indent + 1, out);
    }
}