use std::path::PathBuf;

use serde::Deserialize;

use workspace::claude_code_client::{ClaudeCodeClient, ClaudeCodeRequest};

#[derive(Debug, Deserialize)]
struct Greeting {
    language: String,
    message: String,
}

fn main() {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable is not set");

    let additional_work_directories = vec![
        PathBuf::from("/var/tmp/bear"),
    ];

    let mut client = ClaudeCodeClient::new(api_key, additional_work_directories)
        .expect("failed to create ClaudeCodeClient");

    let request = ClaudeCodeRequest {
        system_prompt: Some("You are a friendly assistant.".to_string()),
        user_prompt: "Say hello in Korean.".to_string(),
        model: Some("sonnet".to_string()),
        output_schema: serde_json::json!({
            "type": "object",
            "properties": {
                "language": { "type": "string" },
                "message": { "type": "string" }
            },
            "required": ["language", "message"]
        }),
    };

    let result: Greeting = client.query(&request)
        .expect("failed to execute query");

    println!("language: {}", result.language);
    println!("message: {}", result.message);
}
