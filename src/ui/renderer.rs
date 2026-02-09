use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::app::{App, MessageRole};

pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::vertical([
        Constraint::Length(8),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(frame.area());

    render_banner(frame, chunks[0]);
    render_separator(frame, chunks[1]);
    render_chat(frame, app, chunks[2]);
}

fn render_banner(frame: &mut Frame, area: Rect) {
    let columns = Layout::horizontal([
        Constraint::Length(29),
        Constraint::Min(1),
    ])
    .split(area);

    let bear_lines = vec![
        Line::from(""),
        Line::from("       () _ _ ()"),
        Line::from("      / __  __ \\"),
        Line::from("/@@\\ /  o    o  \\ /@@\\"),
        Line::from("\\ @ \\|     ^    |/ @ /"),
        Line::from(" \\   \\    ___   /   /"),
        Line::from("  \\   \\________/   /"),
    ];
    let bear = Paragraph::new(bear_lines)
        .style(Style::default().fg(Color::Yellow));
    frame.render_widget(bear, columns[0]);

    let slogan_lines = vec![
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled(
            "Bear: The AI developer that saves your time.",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(Span::styled(
            "Bear, your AI developer, does the heavy lifting for you; you just collect your paycheck and don't worry about a thing.",
            Style::default().fg(Color::DarkGray),
        )),
    ];
    let slogan = Paragraph::new(slogan_lines)
        .wrap(Wrap { trim: true });
    frame.render_widget(slogan, columns[1]);
}

fn render_separator(frame: &mut Frame, area: Rect) {
    let line = "─".repeat(area.width as usize);
    let paragraph = Paragraph::new(Span::styled(
        line,
        Style::default().fg(Color::DarkGray),
    ));
    frame.render_widget(paragraph, area);
}

fn render_chat(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines: Vec<Line> = Vec::new();

    let separator_style = Style::default().fg(Color::DarkGray);
    let system_prefix_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let user_prefix_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let user_text_style = Style::default().fg(Color::Green);

    for message in &app.messages {
        match message.role {
            MessageRole::System => {
                for (i, text_line) in message.content.lines().enumerate() {
                    if i == 0 {
                        lines.push(Line::from(vec![
                            Span::styled("Bear> ", system_prefix_style),
                            Span::raw(text_line),
                        ]));
                    } else {
                        lines.push(Line::from(format!("      {}", text_line)));
                    }
                }
            }
            MessageRole::User => {
                lines.push(Line::from(vec![
                    Span::styled("You> ", user_prefix_style),
                    Span::styled(message.content.as_str(), user_text_style),
                ]));
            }
        }
        lines.push(Line::from(""));
    }

    if app.is_waiting_for_input() {
        let cursor = if app.cursor_visible { "█" } else { " " };
        lines.push(Line::from(vec![
            Span::styled("You> ", user_prefix_style),
            Span::styled(app.input_buffer.as_str(), user_text_style),
            Span::styled(cursor, user_text_style),
        ]));
    }

    let separator = "─".repeat(area.width as usize);
    lines.push(Line::from(Span::styled(separator, separator_style)));
    lines.push(Line::from(Span::styled(
        app.help_text(),
        separator_style,
    )));

    let content_height = lines.len() as u16;
    let scroll = content_height.saturating_sub(area.height);

    let paragraph = Paragraph::new(lines).scroll((scroll, 0));
    frame.render_widget(paragraph, area);
}
