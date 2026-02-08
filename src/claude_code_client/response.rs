use serde::Deserialize;

/// Claude Agent SDK의 ResultMessage 타입에 대응하는 구조체.
/// https://github.com/anthropics/claude-agent-sdk-python/blob/ea81d412923c3e4d6b94ba770ea452cdfba3f51a/src/claude_agent_sdk/types.py#L671 참고.
#[derive(Debug, Deserialize)]
pub struct CliResponse {
    pub session_id: String,
    pub is_error: bool,
    #[serde(default)]
    pub result: Option<String>,
    #[serde(default)]
    pub structured_output: Option<serde_json::Value>,
}