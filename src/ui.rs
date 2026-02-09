pub mod app;
mod error;
mod event;
mod renderer;

pub use error::UiError;

use std::io::{Stdout, stdout};
use std::time::Duration;

use crossterm::event::{Event, KeyEventKind};
use crossterm::style::{Attribute, Color, Print, ResetColor, SetAttribute, SetForegroundColor};
use crossterm::terminal;
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::Viewport;

use app::App;

const INLINE_HEIGHT: u16 = 3;

pub fn run() -> Result<(), UiError> {
    print_banner()?;

    terminal::enable_raw_mode()?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::with_options(
        backend,
        ratatui::TerminalOptions {
            viewport: Viewport::Inline(INLINE_HEIGHT),
        },
    )?;

    let mut app = App::new()?;

    loop {
        flush_messages(&mut terminal, &mut app)?;

        app.tick();
        terminal.draw(|frame| renderer::render(frame, &app))?;

        if let Some(Event::Key(key_event)) = event::poll_event(Duration::from_millis(100))?
            && key_event.kind == KeyEventKind::Press
        {
            app.handle_key_event(key_event);
        }

        if app.should_quit {
            break;
        }
    }

    terminal::disable_raw_mode()?;

    Ok(())
}

fn flush_messages(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &mut App,
) -> Result<(), UiError> {
    while app.has_unprinted_messages() {
        let message = &app.messages[app.printed_message_count];
        let height = renderer::message_height(message);
        terminal.insert_before(height, |buf| {
            renderer::render_message_into(message, buf);
        })?;
        app.printed_message_count += 1;
    }
    Ok(())
}

fn print_banner() -> Result<(), UiError> {
    let mut stdout = stdout();

    let bear_texts = [
        "",
        "       () _ _ ()",
        "      / __  __ \\",
        "/@@\\ /  o    o  \\ /@@\\",
        "\\ @ \\|     ^    |/ @ /",
        " \\   \\    ___   /   /",
        "  \\   \\________/   /",
    ];

    for (i, bear_text) in bear_texts.iter().enumerate() {
        let padded = format!("{:<29}", bear_text);
        crossterm::execute!(stdout, SetForegroundColor(Color::Yellow), Print(padded))?;

        match i {
            3 => {
                crossterm::execute!(
                    stdout,
                    SetForegroundColor(Color::Cyan),
                    SetAttribute(Attribute::Bold),
                    Print("Bear: The AI developer that saves your time."),
                    SetAttribute(Attribute::Reset),
                    ResetColor,
                )?;
            }
            5 => {
                crossterm::execute!(
                    stdout,
                    SetForegroundColor(Color::DarkGrey),
                    Print("Bear, your AI developer, does the heavy lifting for you; \
                           you just collect your paycheck and don't worry about a thing."),
                    ResetColor,
                )?;
            }
            _ => {
                crossterm::execute!(stdout, ResetColor)?;
            }
        }

        crossterm::execute!(stdout, Print("\n"))?;
    }

    let (width, _) = terminal::size()?;
    let separator = "─".repeat(width as usize);
    crossterm::execute!(
        stdout,
        SetForegroundColor(Color::DarkGrey),
        Print(separator),
        ResetColor,
        Print("\n"),
    )?;

    Ok(())
}
