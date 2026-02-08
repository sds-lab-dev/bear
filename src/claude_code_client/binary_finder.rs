use std::path::PathBuf;

use super::error::ClaudeCodeClientError;

const ABSOLUTE_FALLBACK_PATHS: &[&str] = &[
    "/usr/local/bin/claude",
    "/usr/bin/claude",
];

const HOME_RELATIVE_FALLBACK_PATHS: &[&str] = &[
    ".local/bin/claude",
    ".npm-global/bin/claude",    
    "node_modules/.bin/claude",
    ".yarn/bin/claude",
    ".claude/local/claude"
];

pub fn find_claude_binary() -> Result<PathBuf, ClaudeCodeClientError> {
    // PATH에서 먼저 찾아본다.
    if let Ok(path) = which::which("claude") {
        return Ok(path);
    }

    // 없으면 HOME 디렉토리 밑에서 폴백 바이너리 경로를 찾아본다.
    if let Some(home_directory) = std::env::var_os("HOME") {
        let home = PathBuf::from(home_directory);
        for relative_path in HOME_RELATIVE_FALLBACK_PATHS {
            let candidate = home.join(relative_path);
            if candidate.exists() {
                return Ok(candidate);
            }
        }
    }

    // 없으면 절대경로로 폴백 바이너리 경로를 찾아본다.
    for absolute_path in ABSOLUTE_FALLBACK_PATHS {
        let candidate = PathBuf::from(absolute_path);
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(ClaudeCodeClientError::BinaryNotFound)
}
