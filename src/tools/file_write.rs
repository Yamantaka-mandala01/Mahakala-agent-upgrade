use super::registry::ToolInfo;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "file_write".to_string(),
        description: "Write content to a file".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "file_write",
                "description": "Write content to a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file"
                        },
                        "content": {
                            "type": "string",
                            "description": "Content to write"
                        }
                    },
                    "required": ["path", "content"]
                }
            }
        }),
        execute: Box::new(|args: &str| {
            let parsed: serde_json::Value = serde_json::from_str(args)?;
            let path = parsed.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
            let content = parsed.get("content")
                .and_then(|c| c.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
            
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(path, content)?;
            Ok(format!("Successfully wrote to {}", path))
        }),
    }
}
