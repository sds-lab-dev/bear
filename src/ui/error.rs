#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("Agent error: {message}")]
    AgentError { message: String },
}
