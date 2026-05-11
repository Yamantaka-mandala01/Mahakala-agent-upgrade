use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "file_list".to_string(),
        description: "List files in a directory".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "file_list",
                "description": "List files in a directory",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory path to list"
                        }
                    },
                    "required": ["path"]
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
                
                let mut entries = tokio::fs::read_dir(path).await?;
                let mut result = Vec::new();
                while let Some(entry) = entries.next_entry().await? {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let file_type = entry.file_type().await.ok();
                    let is_dir = file_type.map(|ft| ft.is_dir()).unwrap_or(false);
                    result.push(format!("{}{}", if is_dir { "[D] " } else { "    " }, name));
                }
                Ok(result.join("\n"))
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}
