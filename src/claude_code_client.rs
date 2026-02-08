mod binary_finder;
mod error;
mod response;

pub use error::ClaudeCodeClientError;
pub use response::CliResponse;

use std::path::PathBuf;
use std::process::Command;

use serde::de::DeserializeOwned;

pub struct ClaudeCodeRequest {
    pub system_prompt: Option<String>,
    pub user_prompt: String,
    pub model: Option<String>,
    pub output_schema: serde_json::Value,
}

pub struct ClaudeCodeClient {
    binary_path: PathBuf,
    api_key: String,
    additional_work_directories: Vec<PathBuf>,
    session_id: Option<String>,
}

impl ClaudeCodeClient {
    pub fn new(
        api_key: String,
        additional_work_directories: Vec<PathBuf>,
    ) -> Result<Self, ClaudeCodeClientError> {
        let binary_path = binary_finder::find_claude_binary()?;

        for directory in &additional_work_directories {
            if !directory.exists() {
                std::fs::create_dir_all(directory).map_err(|err| {
                    ClaudeCodeClientError::DirectoryCreationFailed {
                        path: directory.display().to_string(),
                        source: err,
                    }
                })?;
            }
        }

        Ok(Self {
            binary_path,
            api_key,
            additional_work_directories,
            session_id: None,
        })
    }

    pub fn query<T: DeserializeOwned>(
        &mut self,
        request: &ClaudeCodeRequest,
    ) -> Result<T, ClaudeCodeClientError> {
        let model_effort_level = "high";
        let disable_auto_memory = "0";  // 0 = force enable.
        let disable_feedback_survey = "1";

        let mut command = Command::new(&self.binary_path);
        command
            .env("ANTHROPIC_API_KEY", &self.api_key)
            .env("CLAUDE_CODE_EFFORT_LEVEL", model_effort_level)
            .env("CLAUDE_CODE_DISABLE_AUTO_MEMORY", disable_auto_memory)
            .env("CLAUDE_CODE_DISABLE_FEEDBACK_SURVEY", disable_feedback_survey)
            .arg("-p")
            .arg("--output-format").arg("json")
            .arg("--allow-dangerously-skip-permissions")
            .arg("--permission-mode").arg("bypassPermissions")
            .arg("--tools").arg("AskUserQuestion,Bash,TaskOutput,Edit,ExitPlanMode,Glob,Grep,KillShell,MCPSearch,Read,Skill,Task,TaskCreate,TaskGet,TaskList,TaskUpdate,WebFetch,WebSearch,Write,LSP");

        // 최초 실행이면 새 세션 ID를 생성하고, 후속 실행이면 기존 세션을 재개한다.
        let new_session_id = match &self.session_id {
            Some(existing_id) => {
                command.arg("--resume").arg(existing_id);
                None
            }
            None => {
                let id = uuid::Uuid::new_v4().to_string();
                command.arg("--session-id").arg(&id);
                Some(id)
            }
        };

        if !self.additional_work_directories.is_empty() {
            command.arg("--add-dir");
            for directory in &self.additional_work_directories {
                command.arg(directory);
            }
        }

        if let Some(model) = &request.model {
            command.arg("--model").arg(model);
        }

        if let Some(system_prompt) = &request.system_prompt {
            command.arg("--append-system-prompt").arg(system_prompt);
        }

        let output_schema_string = request.output_schema.to_string();
        command.arg("--json-schema").arg(&output_schema_string);
        command.arg(&request.user_prompt);

        let output = command.output().map_err(|err| {
            ClaudeCodeClientError::CommandExecutionFailed {
                message: err.to_string(),
            }
        })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ClaudeCodeClientError::CommandExecutionFailed {
                message: stderr.to_string(),
            });
        }

        let response: CliResponse = serde_json::from_slice(&output.stdout)?;
        if response.is_error {
            return Err(ClaudeCodeClientError::CliReturnedError {
                message: response.result.unwrap_or_default(),
            });
        }

        // 최초 실행이었다면 세션 ID를 저장해둔다.
        if new_session_id.is_some() {
            self.session_id = Some(response.session_id);
        }

        let structured_output = response
            .structured_output
            .ok_or(ClaudeCodeClientError::MissingStructuredOutput)?;

        let result: T = serde_json::from_value(structured_output)?;

        Ok(result)
    }
}
