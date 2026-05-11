use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "json_tool".to_string(),
        description: "Parse, format, validate, and transform JSON data".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "json_tool",
                "description": "Parse, format, validate, and transform JSON data",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "description": "Action to perform (parse, format, validate, keys)",
                            "enum": ["parse", "format", "validate", "keys"]
                        },
                        "data": {
                            "type": "string",
                            "description": "JSON data to process"
                        },
                        "path": {
                            "type": "string",
                            "description": "JSON path to extract (e.g., 'foo.bar.baz')"
                        }
                    },
                    "required": ["action", "data"]
                }
            }
        }),
        execute: Arc::new(|arguments: &str| {
            let arguments = arguments.to_string();
            Box::pin(async move {
                let args: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

                let action = args.get("action").and_then(|v| v.as_str()).unwrap_or("parse");
                let data = args.get("data").and_then(|v| v.as_str()).unwrap_or("");

                let json: serde_json::Value = serde_json::from_str(data)
                    .map_err(|e| anyhow::anyhow!("Invalid JSON: {}", e))?;

                match action {
                    "parse" => {
                        Ok(serde_json::to_string_pretty(&json)
                            .map_err(|e| anyhow::anyhow!("Failed to format JSON: {}", e))?)
                    }
                    "format" => {
                        Ok(serde_json::to_string_pretty(&json)
                            .map_err(|e| anyhow::anyhow!("Failed to format JSON: {}", e))?)
                    }
                    "validate" => {
                        Ok("Valid JSON".to_string())
                    }
                    "keys" => {
                        if let Some(obj) = json.as_object() {
                            let keys: Vec<String> = obj.keys().cloned().collect();
                            Ok(format!("Keys: {}", keys.join(", ")))
                        } else {
                            Ok("Not a JSON object".to_string())
                        }
                    }
                    _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
                }
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}
