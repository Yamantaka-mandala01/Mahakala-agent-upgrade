use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "text_tool".to_string(),
        description: "Text processing utilities (count words, characters, lines, transform case, etc.)".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "text_tool",
                "description": "Text processing utilities (count words, characters, lines, transform case, etc.)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Action to perform (count, uppercase, lowercase, reverse, trim)",
                            "enum": ["count", "uppercase", "lowercase", "reverse", "trim"]
                        },
                        "text": {
                            "type": "string",
                            "description": "Text to process"
                        }
                    },
                    "required": ["action", "text"]
                }
            }
        }),
        execute: Arc::new(|arguments: &str| {
            let arguments = arguments.to_string();
            Box::pin(async move {
                let args: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

                let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("count");
                let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("");

                match action {
                    "count" => {
                        let words = text.split_whitespace().count();
                        let chars = text.chars().count();
                        let lines = text.lines().count();
                        Ok(format!("Words: {}, Characters: {}, Lines: {}", words, chars, lines))
                    }
                    "uppercase" => Ok(text.to_uppercase()),
                    "lowercase" => Ok(text.to_lowercase()),
                    "reverse" => Ok(text.chars().rev().collect()),
                    "trim" => Ok(text.trim().to_string()),
                    _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
                }
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}
