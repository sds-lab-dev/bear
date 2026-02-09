use std::path::PathBuf;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent};

use super::error::UiError;

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(500);

pub enum MessageRole {
    System,
    User,
}

pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

enum InputMode {
    WorkspaceConfirm,
    WorkspaceInput,
    Done,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    input_mode: InputMode,
    pub input_buffer: String,
    pub confirmed_workspace: Option<PathBuf>,
    pub should_quit: bool,
    pub cursor_visible: bool,
    cursor_blink_at: Instant,
    current_directory: PathBuf,
}

impl App {
    pub fn new() -> Result<Self, UiError> {
        let current_directory = std::env::current_dir()?;

        let initial_message = format!(
            "현재 디렉토리: {}\n이 디렉토리를 워크스페이스로 사용하시겠습니까? (y/n)",
            current_directory.display()
        );

        let messages = vec![ChatMessage {
            role: MessageRole::System,
            content: initial_message,
        }];

        Ok(Self {
            messages,
            input_mode: InputMode::WorkspaceConfirm,
            input_buffer: String::new(),
            confirmed_workspace: None,
            should_quit: false,
            cursor_visible: true,
            cursor_blink_at: Instant::now(),
            current_directory,
        })
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.reset_cursor_blink();

        match self.input_mode {
            InputMode::WorkspaceConfirm => self.handle_workspace_confirm(key_event),
            InputMode::WorkspaceInput => self.handle_workspace_input(key_event),
            InputMode::Done => self.handle_done(key_event),
        }
    }

    pub fn tick(&mut self) {
        if self.cursor_blink_at.elapsed() >= CURSOR_BLINK_INTERVAL {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_at = Instant::now();
        }
    }

    fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_at = Instant::now();
    }

    pub fn is_waiting_for_input(&self) -> bool {
        matches!(
            self.input_mode,
            InputMode::WorkspaceConfirm | InputMode::WorkspaceInput
        )
    }

    pub fn help_text(&self) -> &str {
        match self.input_mode {
            InputMode::WorkspaceConfirm => "[Y] 예  [N] 아니오  [Q/Esc] 종료",
            InputMode::WorkspaceInput => "[Enter] 확인  [Esc] 종료",
            InputMode::Done => "[Q/Esc] 종료",
        }
    }

    fn handle_workspace_confirm(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.add_user_message("Y");
                self.confirmed_workspace = Some(self.current_directory.clone());
                let path = self.current_directory.display().to_string();
                self.add_system_message(&format!("워크스페이스가 설정되었습니다: {}", path));
                self.input_mode = InputMode::Done;
            }
            KeyCode::Char('n') | KeyCode::Char('N') => {
                self.add_user_message("N");
                self.add_system_message("워크스페이스 디렉토리 경로를 입력하세요:");
                self.input_mode = InputMode::WorkspaceInput;
            }
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn handle_workspace_input(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                let trimmed = self.input_buffer.trim().to_string();
                if !trimmed.is_empty() {
                    self.add_user_message(&trimmed);
                    self.confirmed_workspace = Some(PathBuf::from(&trimmed));
                    self.add_system_message(&format!(
                        "워크스페이스가 설정되었습니다: {}",
                        trimmed
                    ));
                    self.input_buffer.clear();
                    self.input_mode = InputMode::Done;
                }
            }
            KeyCode::Backspace => {
                self.input_buffer.pop();
            }
            KeyCode::Esc => {
                self.should_quit = true;
            }
            KeyCode::Char(c) => {
                self.input_buffer.push(c);
            }
            _ => {}
        }
    }

    fn handle_done(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') | KeyCode::Esc => {
                self.should_quit = true;
            }
            _ => {}
        }
    }

    fn add_system_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: content.to_string(),
        });
    }

    fn add_user_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: content.to_string(),
        });
    }
}
