// src/tui/dashboard/ui.rs
use super::state::{DashboardApp, Tab};
use crate::tui::config::view as config_view;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Tabs};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &mut DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main Content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let title = Span::styled(
        format!(" SLOPCHOP v{} ", app.version),
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
    );

    let tabs = Tabs::new(vec!["ROADMAP", "CHECKS", "CONTEXT", "CONFIG", "LOGS"])
        .select(app.active_tab as usize)
        .block(Block::default().borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .divider("|");

    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Length(20), Constraint::Min(0)])
        .split(area);

    f.render_widget(
        Paragraph::new(title).block(Block::default().borders(Borders::ALL)),
        layout[0],
    );
    f.render_widget(tabs, layout[1]);
}

fn draw_main(f: &mut Frame, app: &DashboardApp, area: Rect) {
    match app.active_tab {
        Tab::Roadmap => draw_roadmap(f, app, area),
        Tab::Checks => draw_checks(f, app, area),
        Tab::Config => config_view::draw_embed(f, &app.config, area),
        _ => draw_placeholder(f, app, area),
    }
}

fn draw_checks(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let status_color = if app.check_running {
        Color::Yellow
    } else {
        Color::Green
    };
    
    let title = if app.check_running { 
        " [ CHECKS: RUNNING... ] " 
    } else { 
        " [ CHECKS: IDLE ] " 
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(status_color))
        .title(title);

    // Join logs with newlines for now. 
    // Optimization: In a real TUI we'd pass lines directly to List or Paragraph.
    let text = app.check_logs.join("\n");

    let p = Paragraph::new(text)
        .block(block)
        .scroll((app.scroll, 0));

    f.render_widget(p, area);
}

fn draw_roadmap(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let content = if let Some(r) = &app.roadmap {
        r.compact_state()
    } else {
        "No ROADMAP.md found.".to_string()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" [ FLIGHT PLAN ] ");

    let p = Paragraph::new(content)
        .block(block)
        .scroll((app.scroll, 0));

    f.render_widget(p, area);
}

fn draw_placeholder(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let content = format!("{:?} View (Coming Soon)", app.active_tab);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" [{:?}] ", app.active_tab));

    f.render_widget(
        Paragraph::new(content)
            .block(block)
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_footer(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let mut controls = vec![
        Span::raw(" [1-5] Navigate | [j/k] Scroll | "),
        Span::styled(" [q] Quit ", Style::default().add_modifier(Modifier::BOLD)),
    ];

    if app.active_tab == Tab::Checks {
        controls.insert(0, Span::styled(
            " [r] Run Checks | ", 
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        ));
    }

    let text = Line::from(controls);

    f.render_widget(
        Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        area,
    );
}