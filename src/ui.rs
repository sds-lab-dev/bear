pub mod app;
mod error;
mod event;
mod renderer;

pub use error::UiError;

use std::io::stdout;
use std::time::Duration;

use crossterm::event::{Event, KeyEventKind};
use crossterm::terminal::{self, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use app::App;

pub fn run() -> Result<(), UiError> {
    terminal::enable_raw_mode()?;
    let mut stdout = stdout();
    stdout.execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    loop {
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
    terminal.backend_mut().execute(LeaveAlternateScreen)?;

    Ok(())
}
