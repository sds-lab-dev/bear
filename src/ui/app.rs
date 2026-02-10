use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::claude_code_client::{ClaudeCodeClient, ClaudeCodeRequest};
use crate::config::Config;
use super::clarification::{self, ClarificationQuestions, QaRound};
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
    RequirementsInput,
    AgentThinking,
    ClarificationAnswer,
    Done,
}

struct AgentThreadResult {
    client: ClaudeCodeClient,
    outcome: Result<ClarificationQuestions, String>,
}

pub struct App {
    pub messages: Vec<ChatMessage>,
    input_mode: InputMode,
    pub input_buffer: String,
    pub confirmed_workspace: Option<PathBuf>,
    pub confirmed_requirements: Option<String>,
    pub should_quit: bool,
    pub cursor_visible: bool,
    cursor_blink_at: Instant,
    current_directory: PathBuf,
    pub scroll_offset: u16,
    keyboard_enhancement_enabled: bool,
    config: Config,
    claude_client: Option<ClaudeCodeClient>,
    agent_result_receiver: Option<mpsc::Receiver<AgentThreadResult>>,
    qa_log: Vec<QaRound>,
    current_round_questions: Vec<String>,
    thinking_started_at: Instant,
}

impl App {
    pub fn new(config: Config) -> Result<Self, UiError> {
        let current_directory = std::env::current_dir()?;

        let initial_message = format!(
            "워크스페이스: {}\n새로운 워크스페이스 절대 경로를 입력하거나, Enter를 눌러 현재 워크스페이스를 사용하세요.",
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
            confirmed_requirements: None,
            should_quit: false,
            cursor_visible: true,
            cursor_blink_at: Instant::now(),
            current_directory,
            scroll_offset: 0,
            keyboard_enhancement_enabled: false,
            config,
            claude_client: None,
            agent_result_receiver: None,
            qa_log: Vec::new(),
            current_round_questions: Vec::new(),
            thinking_started_at: Instant::now(),
        })
    }

    pub fn handle_key_event(&mut self, key_event: KeyEvent) {
        self.reset_cursor_blink();

        match key_event.code {
            KeyCode::PageUp => {
                self.scroll_up();
                return;
            }
            KeyCode::PageDown => {
                self.scroll_down();
                return;
            }
            _ => {}
        }

        match self.input_mode {
            InputMode::WorkspaceConfirm => self.handle_workspace_confirm(key_event),
            InputMode::RequirementsInput => {
                self.handle_multiline_input(key_event, Self::submit_requirements);
            }
            InputMode::ClarificationAnswer => {
                self.handle_multiline_input(key_event, Self::submit_clarification_answer);
            }
            InputMode::AgentThinking | InputMode::Done => {
                if key_event.code == KeyCode::Esc {
                    self.should_quit = true;
                }
            }
        }
    }

    pub fn handle_paste(&mut self, text: String) {
        self.reset_cursor_blink();

        match self.input_mode {
            InputMode::WorkspaceConfirm => {
                let cleaned = text.replace("\r\n", " ").replace(['\r', '\n'], " ");
                self.input_buffer.push_str(&cleaned);
            }
            InputMode::RequirementsInput | InputMode::ClarificationAnswer => {
                let cleaned = text.replace("\r\n", "\n").replace('\r', "\n");
                self.input_buffer.push_str(&cleaned);
            }
            InputMode::AgentThinking | InputMode::Done => {}
        }
    }

    pub fn tick(&mut self) {
        self.tick_cursor_blink();
        self.tick_agent_result();
    }

    fn tick_cursor_blink(&mut self) {
        if self.cursor_blink_at.elapsed() >= CURSOR_BLINK_INTERVAL {
            self.cursor_visible = !self.cursor_visible;
            self.cursor_blink_at = Instant::now();
        }
    }

    fn tick_agent_result(&mut self) {
        let receiver = match &self.agent_result_receiver {
            Some(r) => r,
            None => return,
        };

        let result = match receiver.try_recv() {
            Ok(r) => r,
            Err(mpsc::TryRecvError::Empty) => return,
            Err(mpsc::TryRecvError::Disconnected) => {
                self.agent_result_receiver = None;
                self.handle_agent_error("에이전트 통신이 중단되었습니다.".to_string());
                return;
            }
        };

        self.agent_result_receiver = None;
        self.claude_client = Some(result.client);

        match result.outcome {
            Ok(response) => self.handle_clarification_response(response),
            Err(error_message) => self.handle_agent_error(error_message),
        }
    }

    fn reset_cursor_blink(&mut self) {
        self.cursor_visible = true;
        self.cursor_blink_at = Instant::now();
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub fn set_keyboard_enhancement_enabled(&mut self, enabled: bool) {
        self.keyboard_enhancement_enabled = enabled;
    }

    pub fn is_waiting_for_input(&self) -> bool {
        matches!(
            self.input_mode,
            InputMode::WorkspaceConfirm | InputMode::RequirementsInput | InputMode::ClarificationAnswer
        )
    }

    pub fn is_thinking(&self) -> bool {
        matches!(self.input_mode, InputMode::AgentThinking)
    }

    pub fn thinking_indicator(&self) -> &'static str {
        let dots = (self.thinking_started_at.elapsed().as_millis() / 500) % 4;
        match dots {
            0 => "Analyzing",
            1 => "Analyzing.",
            2 => "Analyzing..",
            _ => "Analyzing...",
        }
    }

    pub fn help_text(&self) -> &str {
        match self.input_mode {
            InputMode::WorkspaceConfirm => "[Enter] Confirm  [PgUp/PgDn] Scroll  [Esc] Quit",
            InputMode::RequirementsInput | InputMode::ClarificationAnswer => {
                if self.keyboard_enhancement_enabled {
                    "[Enter] Submit  [Shift+Enter] New line  [PgUp/PgDn] Scroll  [Esc] Quit"
                } else {
                    "[Enter] Submit  [Alt+Enter] New line  [PgUp/PgDn] Scroll  [Esc] Quit"
                }
            }
            InputMode::AgentThinking | InputMode::Done => "[PgUp/PgDn] Scroll  [Esc] Quit",
        }
    }

    fn handle_workspace_confirm(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Enter => {
                let trimmed = self.input_buffer.trim().to_string();
                let workspace = if trimmed.is_empty() {
                    self.current_directory.clone()
                } else {
                    let path = PathBuf::from(&trimmed);
                    if let Some(error_message) = validate_workspace_path(&path) {
                        self.add_user_message(&trimmed);
                        self.add_system_message(&error_message);
                        self.input_buffer.clear();
                        return;
                    }
                    path
                };
                self.add_user_message(&workspace.display().to_string());
                self.add_system_message(&format!(
                    "워크스페이스가 설정되었습니다: {}",
                    workspace.display()
                ));
                self.confirmed_workspace = Some(workspace);
                self.input_buffer.clear();
                self.transition_to_requirements_input();
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

    fn handle_multiline_input(
        &mut self,
        key_event: KeyEvent,
        submit_action: fn(&mut Self),
    ) {
        match key_event.code {
            KeyCode::Enter if self.is_newline_modifier(key_event.modifiers) => {
                self.input_buffer.push('\n');
            }
            KeyCode::Enter => {
                submit_action(self);
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

    fn transition_to_requirements_input(&mut self) {
        self.add_system_message("구현할 요구사항을 입력하세요. Shitft + Enter로 여러 줄 입력이 가능합니다.");
        self.input_mode = InputMode::RequirementsInput;
    }

    fn submit_requirements(&mut self) {
        let requirements = self.input_buffer.trim().to_string();
        if requirements.is_empty() {
            return;
        }

        self.add_user_message(&requirements);
        self.confirmed_requirements = Some(requirements);
        self.input_buffer.clear();

        if let Err(error_message) = self.ensure_claude_client() {
            self.add_system_message(&format!("클라이언트 생성 실패: {}", error_message));
            self.input_mode = InputMode::Done;
            return;
        }

        self.add_system_message("요구사항을 분석 중입니다. 잠시만 기다려 주세요.");
        self.start_clarification_query();
    }

    fn submit_clarification_answer(&mut self) {
        let answer = self.input_buffer.trim().to_string();
        if answer.is_empty() {
            return;
        }

        self.add_user_message(&answer);
        self.input_buffer.clear();

        let questions = std::mem::take(&mut self.current_round_questions);
        self.qa_log.push(QaRound { questions, answer });

        self.add_system_message("답변을 분석 중입니다. 잠시만 기다려 주세요.");
        self.start_clarification_query();
    }

    fn ensure_claude_client(&mut self) -> Result<(), String> {
        if self.claude_client.is_some() {
            return Ok(());
        }

        let workspace = self.confirmed_workspace.clone().unwrap();
        let client = ClaudeCodeClient::new(self.config.api_key().to_string(), vec![workspace])
            .map_err(|err| err.to_string())?;

        self.claude_client = Some(client);
        Ok(())
    }

    fn start_clarification_query(&mut self) {
        let mut client = self.claude_client.take().expect("client must be available");
        let original_request = self.confirmed_requirements.clone().unwrap();
        let qa_log = self.qa_log.clone();

        let (sender, receiver) = mpsc::channel();
        self.agent_result_receiver = Some(receiver);
        self.input_mode = InputMode::AgentThinking;
        self.thinking_started_at = Instant::now();

        std::thread::spawn(move || {
            let request = ClaudeCodeRequest {
                system_prompt: Some(clarification::system_prompt().to_string()),
                user_prompt: clarification::build_user_prompt(&original_request, &qa_log),
                model: None,
                output_schema: clarification::clarification_schema(),
            };

            let outcome = client
                .query::<ClarificationQuestions>(&request)
                .map_err(|err| err.to_string());

            let _ = sender.send(AgentThreadResult { client, outcome });
        });
    }

    fn handle_clarification_response(&mut self, response: ClarificationQuestions) {
        if response.questions.is_empty() {
            self.add_system_message("요구사항 분석이 완료되었습니다.");
            self.input_mode = InputMode::Done;
            return;
        }

        let mut message = String::from("스펙 작성을 위해 다음 질문에 답변해 주세요.\n");
        for (i, question) in response.questions.iter().enumerate() {
            message.push_str(&format!("\n{}. {}", i + 1, question));
        }

        self.current_round_questions = response.questions;
        self.add_system_message(&message);
        self.input_mode = InputMode::ClarificationAnswer;
    }

    fn handle_agent_error(&mut self, error_message: String) {
        self.add_system_message(&format!("에이전트 오류: {}", error_message));
        self.input_mode = InputMode::Done;
    }

    fn is_newline_modifier(&self, modifiers: KeyModifiers) -> bool {
        if self.keyboard_enhancement_enabled {
            modifiers.contains(KeyModifiers::SHIFT)
        } else {
            modifiers.contains(KeyModifiers::ALT)
        }
    }

    fn add_system_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::System,
            content: content.to_string(),
        });
        self.scroll_offset = 0;
    }

    fn add_user_message(&mut self, content: &str) {
        self.messages.push(ChatMessage {
            role: MessageRole::User,
            content: content.to_string(),
        });
        self.scroll_offset = 0;
    }
}

/// 워크스페이스 경로 검증. 문제가 있으면 에러 메시지를, 없으면 None을 반환.
fn validate_workspace_path(path: &Path) -> Option<String> {
    if !path.is_absolute() {
        return Some(format!(
            "절대 경로를 입력해야 합니다: {}\n새로운 워크스페이스 절대 경로를 입력하거나, Enter를 눌러 현재 워크스페이스를 사용하세요.",
            path.display()
        ));
    }
    if !path.is_dir() {
        return Some(format!(
            "존재하지 않는 디렉토리입니다: {}\n새로운 워크스페이스 절대 경로를 입력하거나, Enter를 눌러 현재 워크스페이스를 사용하세요.",
            path.display()
        ));
    }
    None
}
