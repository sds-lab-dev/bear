#[derive(Debug, thiserror::Error)]
pub enum ClaudeCodeClientError {
    #[error("claude binary not found")]
    BinaryNotFound,

    #[error("CLI execution failed: {message}")]
    CommandExecutionFailed { message: String },

    #[error("JSON parsing failed: {source}")]
    JsonParsingFailed {
        #[from]
        source: serde_json::Error,
    },

    #[error("CLI returned an error: {message}")]
    CliReturnedError { message: String },

    #[error("failed to create directory {path}: {source}")]
    DirectoryCreationFailed {
        path: String,
        source: std::io::Error,
    },

    #[error("structured_output field is missing from the response")]
    MissingStructuredOutput,
}
