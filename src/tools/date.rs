use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "get_date".to_string(),
        description: "Get current date and time in various formats".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "get_date",
                "description": "Get current date and time in various formats",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "format": {
                            "type": "string",
                            "description": "Date format (e.g., 'iso', 'unix', 'human')",
                            "enum": ["iso", "unix", "human"]
                        },
                        "timezone": {
                            "type": "string",
                            "description": "Timezone offset (e.g., '+08:00')"
                        }
                    },
                    "required": []
                }
            }
        }),
        execute: Arc::new(|arguments: &str| {
            let arguments = arguments.to_string();
            Box::pin(async move {
                let args: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

                let format = args.get("format").and_then(|v| v.as_str()).unwrap_or("human");
                let now = chrono::Utc::now();

                let result = match format {
                    "iso" => now.to_rfc3339(),
                    "unix" => now.timestamp().to_string(),
                    "human" => now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                    _ => now.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
                };

                Ok(result)
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}
