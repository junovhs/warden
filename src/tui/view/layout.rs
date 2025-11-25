// src/tui/view/layout.rs
use crate::tui::state::{App, SortMode};
use crate::tui::view::components;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn render_dashboard(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(10),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_main(f, app, chunks[1]);
    draw_footer(f, chunks[2]);
}

#[allow(clippy::cast_precision_loss)]
fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let (clean_count, total) = count_stats(app);
    let health = if total > 0 {
        (clean_count as f64 / total as f64) * 100.0
    } else {
        100.0
    };
    let health_color = get_health_color(health);

    let info = build_info_string(app, total);
    let line = build_header_line(health, health_color, &info);

    f.render_widget(
        Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        area,
    );
}

fn count_stats(app: &App) -> (usize, usize) {
    (
        app.report.files.iter().filter(|f| f.is_clean()).count(),
        app.report.files.len(),
    )
}

fn get_health_color(health: f64) -> Color {
    if health > 90.0 {
        Color::Green
    } else if health > 70.0 {
        Color::Yellow
    } else {
        Color::Red
    }
}

fn build_info_string(app: &App, total: usize) -> String {
    let sort_str = match app.sort_mode {
        SortMode::Path => "NAME",
        SortMode::Tokens => "SIZE",
        SortMode::Violations => "ERRORS",
    };
    let filter_str = if app.only_violations {
        " | FILTER: ERRORS"
    } else {
        ""
    };
    format!(" FILES: {total} | SORT: {sort_str}{filter_str} ")
}

fn build_header_line(health: f64, color: Color, info: &str) -> Line<'_> {
    Line::from(vec![
        Span::styled(
            " 🛡️ WARDEN PROTOCOL ",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" | "),
        Span::styled(format!("HEALTH: {health:.1}%"), Style::default().fg(color)),
        Span::raw(" |"),
        Span::raw(info),
    ])
}

fn draw_main(f: &mut Frame, app: &App, area: Rect) {
    let chunks = get_main_chunks(area);
    components::draw_file_list(f, app, chunks[0]);
    components::draw_inspector(f, app, chunks[1]);
}

fn get_main_chunks(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(area)
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let text = " [s] Sort Mode | [f] Filter Errors | [j/k] Navigate | [q] Quit ";
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray).bg(Color::Black)),
        area,
    );
}
