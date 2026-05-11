use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "file_delete".to_string(),
        description: "Delete files and directories safely".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "file_delete",
                "description": "Delete files and directories safely",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File or directory path to delete"
                        },
                        "recursive": {
                            "type": "boolean",
                            "description": "Delete directory recursively (default: false)"
                        }
                    },
                    "required": ["path"]
                }
            }
        }),
        execute: Arc::new(|arguments: &str| {
            let arguments = arguments.to_string();
            Box::pin(async move {
                let args: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

                let path = args.get("path").and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("path is required"))?;
                let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);

                let path = std::path::Path::new(path);

                if !path.exists() {
                    return Err(anyhow::anyhow!("Path not found: {}", path.display()));
                }

                if path.is_dir() && recursive {
                    tokio::fs::remove_dir_all(path).await
                        .map_err(|e| anyhow::anyhow!("Failed to delete directory: {}", e))?;
                    Ok(format!("Directory deleted: {}", path.display()))
                } else if path.is_dir() {
                    tokio::fs::remove_dir(path).await
                        .map_err(|e| anyhow::anyhow!("Failed to delete directory: {}", e))?;
                    Ok(format!("Empty directory deleted: {}", path.display()))
                } else {
                    tokio::fs::remove_file(path).await
                        .map_err(|e| anyhow::anyhow!("Failed to delete file: {}", e))?;
                    Ok(format!("File deleted: {}", path.display()))
                }
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}