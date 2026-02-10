#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing environment variable: {name}")]
    MissingEnvVar { name: String },
}

pub struct Config {
    api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let api_key = read_required_env("ANTHROPIC_API_KEY")?;
        Ok(Self { api_key })
    }

    pub fn api_key(&self) -> &str {
        &self.api_key
    }
}

fn read_required_env(name: &str) -> Result<String, ConfigError> {
    std::env::var(name).map_err(|_| ConfigError::MissingEnvVar {
        name: name.to_string(),
    })
}
