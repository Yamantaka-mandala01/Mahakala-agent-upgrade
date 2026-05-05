use super::registry::ToolInfo;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "file_read".to_string(),
        description: "Read the contents of a file".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "file_read",
                "description": "Read the contents of a file",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Path to the file to read"
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
            
            let content = std::fs::read_to_string(path)?;
            Ok(content)
        }),
    }
}
