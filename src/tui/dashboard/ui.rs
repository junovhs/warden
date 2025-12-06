// src/tui/dashboard/ui.rs
use crate::types::FileReport;
use crate::roadmap_v2::types::TaskStatus;
use crate::tui::dashboard::state::{DashboardApp, Tab, TaskStatusFilter};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Tabs},
    Frame,
};

pub fn draw(f: &mut Frame, app: &mut DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header/Tabs
            Constraint::Min(0),    // Content
            Constraint::Length(1), // Footer
        ])
        .split(f.area());

    draw_tabs(f, app, chunks[0]);
    
    match app.active_tab {
        Tab::Dashboard => draw_dashboard(f, app, chunks[1]),
        Tab::Roadmap => draw_roadmap(f, app, chunks[1]),
        Tab::Config => draw_config(f, app, chunks[1]),
        Tab::Logs => draw_logs(f, app, chunks[1]),
    }

    draw_footer(f, chunks[2]);
}

fn draw_tabs(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let titles: Vec<_> = ["Dashboard", "Roadmap", "Config", "Logs"]
        .iter()
        .map(|t| Line::from(Span::styled(*t, Style::default().fg(Color::Green))))
        .collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("SlopChop"))
        .select(app.active_tab as usize)
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::DarkGray));
    
    f.render_widget(tabs, area);
}

fn draw_dashboard(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    // Left: Status
    let status_text = if let Some(report) = &app.scan_report {
        format!(
            "Files: {}\nViolations: {}\nClean: {}",
            report.files.len(),
            report.files.iter().map(FileReport::violation_count).sum::<usize>(),
            report.clean_file_count()
        )
    } else {
        "Scanning...".to_string()
    };

    let status = Paragraph::new(status_text)
        .block(Block::default().borders(Borders::ALL).title("Status"));
    f.render_widget(status, chunks[0]);

    // Right: Recent logs
    draw_logs_mini(f, app, chunks[1]);
}

fn draw_roadmap(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let Some(store) = &app.roadmap else {
        let p = Paragraph::new("No roadmap loaded (slopchop.toml)")
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(p, area);
        return;
    };

    let tasks: Vec<ListItem> = store.tasks.iter()
        .filter(|t| match app.roadmap_filter {
            TaskStatusFilter::All => true,
            TaskStatusFilter::Pending => t.status == TaskStatus::Pending,
            TaskStatusFilter::Done => matches!(t.status, TaskStatus::Done | TaskStatus::NoTest),
        })
        .map(|t| {
            let style = if t.status == TaskStatus::Done {
                Style::default().fg(Color::Green)
            } else {
                Style::default()
            };
            let prefix = match t.status {
                TaskStatus::Done | TaskStatus::NoTest => "[x]",
                TaskStatus::Pending => "[ ]",
            };
            ListItem::new(format!("{} {}", prefix, t.text)).style(style)
        })
        .collect();

    let list = List::new(tasks)
        .block(Block::default().borders(Borders::ALL).title("Roadmap Tasks"))
        .highlight_style(Style::default().bg(Color::DarkGray));

    f.render_widget(list, area);
}

fn draw_config(f: &mut Frame, app: &mut DashboardApp, area: Rect) {
    crate::tui::config::view::draw_embed(f, &app.config_editor, area);
}

fn draw_logs(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let logs: Vec<ListItem> = app.logs.iter()
        .rev()
        .map(|s| ListItem::new(Line::from(s.as_str())))
        .collect();

    let list = List::new(logs)
        .block(Block::default().borders(Borders::ALL).title("System Logs"));
    f.render_widget(list, area);
}

fn draw_logs_mini(f: &mut Frame, app: &DashboardApp, area: Rect) {
     let logs: Vec<ListItem> = app.logs.iter()
        .rev()
        .take(10)
        .map(|s| ListItem::new(Line::from(s.as_str())))
        .collect();

    let list = List::new(logs)
        .block(Block::default().borders(Borders::ALL).title("Recent Activity"));
    f.render_widget(list, area);
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let text = "q: Quit | TAB: Switch View | r: Reload";
    let p = Paragraph::new(text).style(Style::default().fg(Color::DarkGray));
    f.render_widget(p, area);
}