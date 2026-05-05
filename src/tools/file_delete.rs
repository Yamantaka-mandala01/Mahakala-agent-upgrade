use super::registry::ToolInfo;

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
        execute: Box::new(|arguments| {
            let args: serde_json::Value = serde_json::from_str(arguments)
                .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

            let path = args.get("path").and_then(|v| v.as_str())
                .ok_or_else(|| anyhow::anyhow!("path is required"))?;
            let recursive = args.get("recursive").and_then(|v| v.as_bool()).unwrap_or(false);

            let path = std::path::Path::new(path);

            if !path.exists() {
                return Err(anyhow::anyhow!("Path not found: {}", path.display()));
            }

            if path.is_dir() && recursive {
                std::fs::remove_dir_all(path)
                    .map_err(|e| anyhow::anyhow!("Failed to delete directory: {}", e))?;
                Ok(format!("Directory deleted: {}", path.display()))
            } else if path.is_dir() {
                std::fs::remove_dir(path)
                    .map_err(|e| anyhow::anyhow!("Failed to delete directory: {}", e))?;
                Ok(format!("Empty directory deleted: {}", path.display()))
            } else {
                std::fs::remove_file(path)
                    .map_err(|e| anyhow::anyhow!("Failed to delete file: {}", e))?;
                Ok(format!("File deleted: {}", path.display()))
            }
        }),
    }
}
