use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
        execute: Arc::new(|args: &str| {
            let args = args.to_string();
            Box::pin(async move {
                let parsed: serde_json::Value = serde_json::from_str(&args)?;
                let path = parsed.get("path")
                    .and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
                let content = parsed.get("content")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'content' parameter"))?;
                
                if let Some(parent) = std::path::Path::new(path).parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                tokio::fs::write(path, content).await?;
                Ok(format!("Successfully wrote to {}", path))
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}
