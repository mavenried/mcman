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

pub fn draw(f: &mut Frame, app: &mut App) {
    let size = f.area();

    let input_height = if app.completions.is_empty() { 3 } else { 4 };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(input_height)])
        .split(size);

    draw_logs(f, app, chunks[0]);
    draw_input(f, app, chunks[1]);
}

pub fn draw_logs(f: &mut Frame, app: &mut App, area: Rect) {
    let content_width = area.width.saturating_sub(4) as usize;
    let viewport_height = area.height.saturating_sub(2) as usize;

    if content_width != app.wrap_width {
        app.rebuild_wrapped_logs(content_width);
    }

    app.max_scroll = app.wrapped_logs.len().saturating_sub(viewport_height);

    let scroll_indicator = if app.auto_scroll {
        " [AUTO-SCROLL] ".to_string()
    } else {
        format!(" [{}/{}] ", app.scroll_offset + 1, app.max_scroll + 1)
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

    let start = app.scroll_offset.min(app.max_scroll);
    let end = (start + viewport_height).min(app.wrapped_logs.len());

    let items: Vec<ListItem> = app.wrapped_logs[start..end]
        .iter()
        .map(|line| {
            let color = classify_line(line);
            ListItem::new(Line::from(vec![Span::styled(
                line.clone(),
                Style::default().fg(color),
            )]))
        })
        .collect();

    let list = List::new(items).block(block);

    f.render_widget(list, area);
}

pub fn draw_input(f: &mut Frame, app: &App, area: Rect) {
    let cursor_pos = app.cursor_pos.min(app.input.len());

    let cursor_pos = if app.input.is_char_boundary(cursor_pos) {
        cursor_pos
    } else {
        app.input
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i < cursor_pos)
            .last()
            .unwrap_or(0)
    };
    let before_cursor = &app.input[..cursor_pos];
    let cursor_char = app.input[cursor_pos..].chars().next().unwrap_or(' ');
    let after_cursor = if cursor_pos < app.input.len() {
        &app.input[cursor_pos + cursor_char.len_utf8()..]
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
        ratatui::text::Text::from(vec![hint_line, content])
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
