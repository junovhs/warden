// src/tui/dashboard/ui.rs
use super::state::{DashboardApp, Tab};
use crate::roadmap::TaskStatus;
use crate::tui::config::view as config_view;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Tabs};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &mut DashboardApp) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, app, chunks[2]);

    if app.show_popup {
        draw_popup(f, f.area());
    }
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

fn draw_main(f: &mut Frame, app: &mut DashboardApp, area: Rect) {
    match app.active_tab {
        Tab::Roadmap => draw_roadmap(f, app, area),
        Tab::Checks => draw_checks(f, app, area),
        Tab::Config => config_view::draw_embed(f, &app.config, area),
        Tab::Logs => draw_logs(f, app, area),
        Tab::Context => draw_placeholder(f, app, area),
    }
}

fn draw_roadmap(f: &mut Frame, app: &mut DashboardApp, area: Rect) {
    let items: Vec<ListItem> = app
        .flat_roadmap
        .iter()
        .map(|item| {
            let indent = "  ".repeat(item.indent);
            if item.is_header {
                ListItem::new(format!("{indent}{}", item.text))
                    .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            } else {
                let icon = match item.status {
                    TaskStatus::Complete => "[x]",
                    TaskStatus::Pending => "[ ]",
                };
                let color = if item.status == TaskStatus::Complete {
                    Color::Green
                } else {
                    Color::White
                };
                ListItem::new(format!("{indent}{icon} {}", item.text)).style(Style::default().fg(color))
            }
        })
        .collect();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" [ FLIGHT PLAN ] ");

    let list = List::new(items)
        .block(block)
        .highlight_style(Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD));

    let mut state = ListState::default();
    state.select(Some(app.selected_task));
    
    f.render_stateful_widget(list, area, &mut state);
}

fn draw_checks(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let status_color = if app.check_running {
        Color::Yellow
    } else {
        Color::Green
    };
    
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(status_color))
        .title(if app.check_running { " [ RUNNING ] " } else { " [ IDLE ] " });

    let text = app.check_logs.join("\n");
    let lines = text.lines().count();
    let height = area.height as usize;
    
    // Allow truncation for TUI display logic
    #[allow(clippy::cast_possible_truncation)]
    let scroll = if lines > height { (lines - height) as u16 } else { 0 };

    let p = Paragraph::new(text).block(block).scroll((scroll, 0));
    f.render_widget(p, area);
}

fn draw_logs(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" [ SYSTEM LOGS ] ");

    let text = app.system_logs.join("\n");
    let p = Paragraph::new(text).block(block);
    f.render_widget(p, area);
}

fn draw_placeholder(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let content = format!("{:?} View (Coming Soon)", app.active_tab);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" [{:?}] ", app.active_tab));

    f.render_widget(
        Paragraph::new(content).block(block).alignment(Alignment::Center),
        area,
    );
}

fn draw_footer(f: &mut Frame, app: &DashboardApp, area: Rect) {
    let mut controls = vec![
        Span::raw(" [1-5] Navigate | "),
        Span::styled(" [q] Quit ", Style::default().add_modifier(Modifier::BOLD)),
    ];

    match app.active_tab {
        Tab::Roadmap => controls.insert(1, Span::raw(" [j/k] Select | [SPACE] Toggle | ")),
        Tab::Checks => controls.insert(1, Span::raw(" [r] Run Checks | ")),
        _ => {}
    }

    f.render_widget(
        Paragraph::new(Line::from(controls))
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_popup(f: &mut Frame, area: Rect) {
    let popup_area = centered_rect(60, 20, area);
    f.render_widget(Clear, popup_area);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" ?? INCOMING PAYLOAD ")
        .style(Style::default().bg(Color::DarkGray).fg(Color::White));

    let content = "SlopChop Protocol detected in clipboard.\n\nApply changes?\n\n[y] Apply & Verify\n[n] Discard";
    
    let p = Paragraph::new(content).block(block).alignment(Alignment::Center);
    f.render_widget(p, popup_area);
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}