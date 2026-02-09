use ratatui::Frame;
use ratatui::buffer::Buffer;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::app::{App, ChatMessage, MessageRole};

/// 인라인 뷰포트(입력 영역)만 렌더링.
pub fn render(frame: &mut Frame, app: &App) {
    let area = frame.area();

    let separator_style = Style::default().fg(Color::DarkGray);
    let user_prefix_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let user_text_style = Style::default().fg(Color::Green);

    let mut lines: Vec<Line> = Vec::new();

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

    frame.render_widget(Paragraph::new(lines), area);
}

/// 메시지를 insert_before용 라인으로 포맷.
pub fn render_message_into(message: &ChatMessage, buf: &mut Buffer) {
    let lines = format_message(message);
    Paragraph::new(lines).render(buf.area, buf);
}

/// 메시지의 렌더링 높이(줄 수)를 계산.
pub fn message_height(message: &ChatMessage) -> u16 {
    format_message(message).len() as u16
}

fn format_message(message: &ChatMessage) -> Vec<Line<'static>> {
    let system_prefix_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let user_prefix_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let user_text_style = Style::default().fg(Color::Green);

    let mut lines: Vec<Line<'static>> = Vec::new();

    match message.role {
        MessageRole::System => {
            for (i, text_line) in message.content.lines().enumerate() {
                if i == 0 {
                    lines.push(Line::from(vec![
                        Span::styled("Bear> ", system_prefix_style),
                        Span::styled(text_line.to_string(), Style::default()),
                    ]));
                } else {
                    lines.push(Line::from(format!("      {}", text_line)));
                }
            }
        }
        MessageRole::User => {
            lines.push(Line::from(vec![
                Span::styled("You> ", user_prefix_style),
                Span::styled(message.content.clone(), user_text_style),
            ]));
        }
    }
    lines.push(Line::from(""));

    lines
}
