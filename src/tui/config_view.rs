// src/tui/config_view.rs
use crate::tui::config_state::ConfigApp;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub fn draw(f: &mut Frame, app: &ConfigApp) {
    let area = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),
                Constraint::Min(5),
                Constraint::Length(1),
            ]
            .as_ref(),
        )
        .split(area);

    draw_header(f, app, chunks[0]);
    draw_form(f, app, chunks[1]);
    draw_footer(f, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &ConfigApp, area: Rect) {
    let title = Span::styled(
        " 🧙 WARDEN CONFIGURATION ",
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    let status = if let Some((msg, _)) = &app.saved_message {
        Span::styled(format!(" {msg} "), Style::default().fg(Color::Green))
    } else if app.modified {
        Span::styled(" [Modified] ", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let line = Line::from(vec![title, Span::raw(" |"), status]);

    f.render_widget(
        Paragraph::new(line)
            .block(Block::default().borders(Borders::ALL))
            .alignment(Alignment::Center),
        area,
    );
}

fn draw_form(f: &mut Frame, app: &ConfigApp, area: Rect) {
    let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(60), Constraint::Percentage(20)])
        .split(area);
    
    // Centered column
    let center_area = layout[1];

    let items = vec![
        ("Max File Tokens", app.rules.max_file_tokens.to_string()),
        ("Cyclomatic Complexity", app.rules.max_cyclomatic_complexity.to_string()),
        ("Nesting Depth", app.rules.max_nesting_depth.to_string()),
        ("Function Arguments", app.rules.max_function_args.to_string()),
        ("Function Words", app.rules.max_function_words.to_string()),
    ];

    let mut lines = Vec::new();
    lines.push(Line::from("")); // Spacer

    for (i, (label, value)) in items.into_iter().enumerate() {
        let is_selected = i == app.selected_field;
        let style = if is_selected {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let pointer = if is_selected { " > " } else { "   " };
        let text = format!("{pointer}{label:<25} {value}");
        
        lines.push(Line::from(Span::styled(text, style)));
        lines.push(Line::from("")); // Spacer
    }

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Rules ");

    f.render_widget(
        Paragraph::new(lines).block(block),
        center_area,
    );
}

fn draw_footer(f: &mut Frame, area: Rect) {
    let text = " [↑/↓] Select | [←/→] Adjust Value | [Enter/s] Save | [q] Quit ";
    f.render_widget(
        Paragraph::new(text).style(Style::default().fg(Color::DarkGray).bg(Color::Black)),
        area,
    );
}