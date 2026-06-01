use crate::App;
use crate::prelude::*;

pub fn classify_line(line: &str) -> Color {
    let l = line.to_lowercase();
    if l.contains("[error]") || l.contains("exception") || l.contains("stacktrace") {
        Color::Red
    } else if l.contains("[warn]") || l.contains("[mcman]") {
        Color::Yellow
    } else if l.contains("joined the game") || l.contains("left the game") {
        Color::Green
    } else {
        Color::White
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(3)])
        .split(size);

    draw_logs(f, app, chunks[0]);
    draw_input(f, app, chunks[1]);
}

pub fn draw_logs(f: &mut Frame, app: &App, area: Rect) {
    let visible = area.height.saturating_sub(2) as usize;
    let total = app.logs.len();
    let start = app.scroll_offset.min(total.saturating_sub(visible));
    let end = total;

    let scroll_indicator = if app.auto_scroll {
        " [AUTO-SCROLL] ".to_string()
    } else {
        let max_offset = total.saturating_sub(visible);
        format!(" [{}/{}] ", app.scroll_offset + 1, max_offset + 1)
    };

    let running = *app.server_running.lock().unwrap();
    let status = if running {
        "● RUNNING"
    } else {
        "○ STOPPED"
    };
    let status_color = if running { Color::Green } else { Color::Red };

    let title = Line::from(vec![
        Span::styled(
            " Minecraft Console ",
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(status, Style::default().fg(status_color)),
        Span::styled(scroll_indicator, Style::default().fg(Color::DarkGray)),
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::DarkGray))
        .border_type(BorderType::Rounded);

    let text = ratatui::text::Text::from(
        app.logs[start..end]
            .iter()
            .map(|line| {
                let color = classify_line(line);
                Line::from(vec![Span::styled(line.clone(), Style::default().fg(color))])
            })
            .collect::<Vec<_>>(),
    );

    let widget = Paragraph::new(text).block(block).wrap(Wrap { trim: false });

    f.render_widget(widget, area);
}

pub fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let before_cursor = &app.input[..app.cursor_pos];
    let cursor_char = app.input[app.cursor_pos..].chars().next().unwrap_or(' ');
    let after_cursor = if app.cursor_pos < app.input.len() {
        &app.input[app.cursor_pos + cursor_char.len_utf8()..]
    } else {
        ""
    };

    let spans = vec![
        Span::styled(before_cursor, Style::default().fg(Color::White)),
        Span::styled(
            cursor_char.to_string(),
            Style::default().bg(Color::White).fg(Color::Black),
        ),
        Span::styled(after_cursor, Style::default().fg(Color::White)),
    ];

    let completion_hint = if !app.completions.is_empty() {
        let shown: Vec<&str> = app
            .completions
            .iter()
            .take(6)
            .map(|s: &String| s.as_str())
            .collect();

        let extra = if app.completions.len() > 6 {
            format!(" (+{})", app.completions.len() - 6)
        } else {
            String::new()
        };
        format!("  [{}{}]", shown.join("  "), extra)
    } else {
        String::new()
    };

    let title = Line::from(vec![
        Span::styled(" Command", Style::default().fg(Color::Cyan)),
        Span::styled(
            "  Tab=complete  ↑↓=history  PgUp/PgDn=scroll  Ctrl+C=quit",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let content = Line::from(spans);
    let hint_line = Line::from(Span::styled(
        completion_hint,
        Style::default().fg(Color::Yellow),
    ));

    let full_text = if app.completions.is_empty() {
        ratatui::text::Text::from(content)
    } else {
        ratatui::text::Text::from(vec![content, hint_line])
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(Style::default().fg(Color::Cyan))
        .border_type(BorderType::Rounded);

    let widget = Paragraph::new(full_text)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(widget, area);
}
