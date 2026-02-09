use std::time::Duration;

use crossterm::event::{self, Event};

use super::error::UiError;

pub fn poll_event(timeout: Duration) -> Result<Option<Event>, UiError> {
    if event::poll(timeout)? {
        Ok(Some(event::read()?))
    } else {
        Ok(None)
    }
}
