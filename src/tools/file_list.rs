use super::registry::ToolInfo;

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
        execute: Box::new(|args: &str| {
            let parsed: serde_json::Value = serde_json::from_str(args)?;
            let path = parsed.get("path")
                .and_then(|p| p.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'path' parameter"))?;
            
            let entries = std::fs::read_dir(path)?;
            let mut result = Vec::new();
            for entry in entries {
                if let Ok(entry) = entry {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let file_type = entry.file_type().ok();
                    let is_dir = file_type.map(|ft| ft.is_dir()).unwrap_or(false);
                    result.push(format!("{}{}", if is_dir { "[D] " } else { "    " }, name));
                }
            }
            Ok(result.join("\n"))
        }),
    }
}
