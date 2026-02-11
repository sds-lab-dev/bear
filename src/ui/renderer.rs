use ratatui::Frame;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use unicode_width::UnicodeWidthChar;

use super::app::{App, ChatMessage, MessageRole};

pub const SYSTEM_PREFIX: &str = "Bear> ";
pub const USER_PREFIX: &str = " You> ";

const BEAR_TEXTS: [&str; 7] = [
    "",
    "       () _ _ ()",
    "      / __  __ \\",
    "/@@\\ /  o    o  \\ /@@\\",
    "\\ @ \\|     ^    |/ @ /",
    " \\   \\    ___   /   /",
    "  \\   \\________/   /",
];

const BEAR_COLUMN_WIDTH: usize = 29;
const RIGHT_COLUMN_START: usize = 3;

pub fn render(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let separator_style = Style::default().fg(Color::DarkGray);
    let user_prefix_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let user_text_style = Style::default().fg(Color::Green);

    let mut lines: Vec<Line> = Vec::new();

    lines.extend(build_banner_lines(area.width));

    app.terminal_width = area.width;
    app.terminal_height = area.height;

    for message in &app.messages {
        lines.extend(format_message(message, area.width));
    }

    if app.is_waiting_for_input() {
        let cursor = if app.cursor_visible { "█" } else { " " };
        lines.extend(build_input_lines(
            &app.input_buffer,
            app.cursor_position,
            cursor,
            user_prefix_style,
            user_text_style,
            area.width,
        ));
    } else if app.is_thinking() {
        let system_prefix_style = Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD);
        let thinking_style = Style::default().fg(Color::Yellow);
        lines.push(Line::from(vec![
            Span::styled(SYSTEM_PREFIX, system_prefix_style),
            Span::styled(app.thinking_indicator().to_string(), thinking_style),
        ]));
    } else {
        lines.push(Line::from(""));
    }

    let separator = "─".repeat(area.width as usize);
    lines.push(Line::from(Span::styled(separator, separator_style)));
    lines.push(Line::from(Span::styled(
        app.help_text(),
        separator_style,
    )));

    let total_lines = lines.len() as u16;
    let max_scroll = total_lines.saturating_sub(area.height);
    let scroll = max_scroll.saturating_sub(app.scroll_offset);

    frame.render_widget(
        Paragraph::new(lines).scroll((scroll, 0)),
        area,
    );

    // lines 소비 후 빌림이 해제되어 scroll_offset 수정 가능.
    app.scroll_offset = app.scroll_offset.min(max_scroll);
}

fn build_banner_lines(width: u16) -> Vec<Line<'static>> {
    let bear_style = Style::default().fg(Color::Yellow);
    let separator_style = Style::default().fg(Color::DarkGray);

    let right_column_width = (width as usize).saturating_sub(BEAR_COLUMN_WIDTH);
    let right_column = build_right_column(right_column_width);

    let mut lines: Vec<Line> = Vec::new();

    for (i, bear_text) in BEAR_TEXTS.iter().enumerate() {
        let padded = format!("{:<width$}", bear_text, width = BEAR_COLUMN_WIDTH);
        let mut spans = vec![Span::styled(padded, bear_style)];

        let right_offset = i.wrapping_sub(RIGHT_COLUMN_START);
        if let Some((text, color, bold)) = right_column.get(right_offset) {
            let mut style = Style::default().fg(*color);
            if *bold {
                style = style.add_modifier(Modifier::BOLD);
            }
            spans.push(Span::styled(text.clone(), style));
        }

        lines.push(Line::from(spans));
    }

    let separator = "─".repeat(width as usize);
    lines.push(Line::from(Span::styled(separator, separator_style)));

    lines
}

fn format_message(message: &ChatMessage, max_width: u16) -> Vec<Line<'static>> {
    let system_prefix_style = Style::default()
        .fg(Color::Cyan)
        .add_modifier(Modifier::BOLD);
    let user_prefix_style = Style::default()
        .fg(Color::Green)
        .add_modifier(Modifier::BOLD);
    let user_text_style = Style::default().fg(Color::Green);

    let mut lines: Vec<Line<'static>> = Vec::new();

    let (prefix, prefix_style, text_style) = match message.role {
        MessageRole::System => (SYSTEM_PREFIX, system_prefix_style, Style::default()),
        MessageRole::User => (USER_PREFIX, user_prefix_style, user_text_style),
    };
    let padding = " ".repeat(prefix.len());
    let text_width = (max_width as usize).saturating_sub(prefix.len());
    let mut is_first = true;

    for text_line in message.content.lines() {
        let line_style = if matches!(message.role, MessageRole::System)
            && is_tool_label(text_line)
        {
            text_style.add_modifier(Modifier::BOLD)
        } else {
            text_style
        };

        for visual_line in wrap_text_by_char_width(text_line, text_width) {
            if is_first {
                lines.push(Line::from(vec![
                    Span::styled(prefix, prefix_style),
                    Span::styled(visual_line, line_style),
                ]));
                is_first = false;
            } else {
                lines.push(Line::from(Span::styled(
                    format!("{}{}", padding, visual_line),
                    line_style,
                )));
            }
        }
    }
    lines.push(Line::from(""));

    lines
}

fn build_right_column(max_width: usize) -> Vec<(String, Color, bool)> {
    let slogan_lines = wrap_words(
        "Bear: The AI developer that saves your time.",
        max_width,
    );
    let description_lines = wrap_words(
        "Bear, your AI developer, does the heavy lifting for you; \
         you just collect your paycheck and don't worry about a thing.",
        max_width,
    );

    let mut lines: Vec<(String, Color, bool)> = Vec::new();
    for line in &slogan_lines {
        lines.push((line.clone(), Color::Cyan, true));
    }
    if !slogan_lines.is_empty() && !description_lines.is_empty() {
        lines.push((String::new(), Color::Reset, false));
    }
    for line in &description_lines {
        lines.push((line.clone(), Color::DarkGray, false));
    }
    lines
}

fn build_input_lines<'a>(
    input_buffer: &str,
    cursor_position: usize,
    cursor_char: &'a str,
    prefix_style: Style,
    text_style: Style,
    max_width: u16,
) -> Vec<Line<'a>> {
    let cursor_reserved = 1;
    let text_width = (max_width as usize).saturating_sub(USER_PREFIX.len() + cursor_reserved);

    let mut lines = Vec::new();
    let logical_lines: Vec<&str> = input_buffer.split('\n').collect();

    let mut global_char_offset = 0;
    let mut is_first_visual_line = true;

    for (logical_idx, logical_line) in logical_lines.iter().enumerate() {
        let visual_lines = wrap_text_by_char_width(logical_line, text_width);
        let visual_line_count = visual_lines.len();
        let mut line_char_offset = 0;

        for (visual_idx, visual_text) in visual_lines.iter().enumerate() {
            let visual_char_count = visual_text.chars().count();
            let visual_start = global_char_offset + line_char_offset;
            let is_last_visual_of_logical = visual_idx == visual_line_count - 1;

            let cursor_col = cursor_column_on_visual_line(
                cursor_position,
                visual_start,
                visual_char_count,
                is_last_visual_of_logical,
            );

            let padding = " ".repeat(USER_PREFIX.len());
            let prefix = if is_first_visual_line {
                USER_PREFIX.to_string()
            } else {
                padding.clone()
            };
            let current_prefix_style = if is_first_visual_line {
                prefix_style
            } else {
                Style::default()
            };

            let mut spans = vec![Span::styled(prefix, current_prefix_style)];

            if let Some(col) = cursor_col {
                let before: String = visual_text.chars().take(col).collect();
                let after: String = visual_text.chars().skip(col).collect();
                spans.push(Span::styled(before, text_style));
                spans.push(Span::styled(cursor_char.to_string(), text_style));
                spans.push(Span::styled(after, text_style));
            } else {
                spans.push(Span::styled(visual_text.to_string(), text_style));
            }

            lines.push(Line::from(spans));

            line_char_offset += visual_char_count;
            is_first_visual_line = false;
        }

        global_char_offset += logical_line.chars().count();
        if logical_idx < logical_lines.len() - 1 {
            global_char_offset += 1; // '\n'
        }
    }

    lines
}

/// 커서가 이 visual line 위에 있으면 해당 컬럼을, 아니면 None을 반환.
fn cursor_column_on_visual_line(
    cursor_position: usize,
    visual_start: usize,
    visual_char_count: usize,
    is_last_visual_of_logical: bool,
) -> Option<usize> {
    let visual_end = visual_start + visual_char_count;

    if cursor_position >= visual_start && cursor_position < visual_end {
        return Some(cursor_position - visual_start);
    }

    if cursor_position == visual_end && is_last_visual_of_logical {
        return Some(visual_char_count);
    }

    None
}

pub(super) fn wrap_text_by_char_width(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![text.to_string()];
    }

    let mut result = Vec::new();
    let mut current_line = String::new();
    let mut current_width: usize = 0;

    for ch in text.chars() {
        let char_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if current_width + char_width > max_width && current_width > 0 {
            result.push(current_line);
            current_line = String::new();
            current_width = 0;
        }
        current_line.push(ch);
        current_width += char_width;
    }

    result.push(current_line);
    result
}

fn is_tool_label(line: &str) -> bool {
    line.starts_with("[Tool Call:") || line.starts_with("[Tool Result]")
}

fn wrap_words(text: &str, max_width: usize) -> Vec<String> {
    if max_width == 0 {
        return vec![];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();

    for word in text.split_whitespace() {
        if current_line.is_empty() {
            current_line.push_str(word);
        } else if current_line.len() + 1 + word.len() <= max_width {
            current_line.push(' ');
            current_line.push_str(word);
        } else {
            lines.push(current_line);
            current_line = word.to_string();
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    lines
}
