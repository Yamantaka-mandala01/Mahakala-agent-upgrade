use super::registry::ToolInfo;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "memory".to_string(),
        description: "Store or retrieve information from memory".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "memory",
                "description": "Store or retrieve information from memory",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["store", "retrieve", "list"],
                            "description": "Action to perform"
                        },
                        "key": {
                            "type": "string",
                            "description": "Key for the memory entry"
                        },
                        "value": {
                            "type": "string",
                            "description": "Value to store (for store action)"
                        }
                    },
                    "required": ["action"]
                }
            }
        }),
        execute: Box::new(|args: &str| {
            // 简化实现 - 使用文件存储
            let parsed: serde_json::Value = serde_json::from_str(args)?;
            let action = parsed.get("action")
                .and_then(|a| a.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))?;
            
            let home = crate::constants::get_mahakala_home();
            let memory_file = home.join("memory.json");
            
            let mut memories: std::collections::HashMap<String, String> = if memory_file.exists() {
                let content = std::fs::read_to_string(&memory_file)?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                std::collections::HashMap::new()
            };
            
            match action {
                "store" => {
                    let key = parsed.get("key").and_then(|k| k.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing 'key' for store action"))?;
                    let value = parsed.get("value").and_then(|v| v.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing 'value' for store action"))?;
                    memories.insert(key.to_string(), value.to_string());
                    std::fs::write(&memory_file, serde_json::to_string_pretty(&memories)?)?;
                    Ok(format!("Stored memory: {}", key))
                }
                "retrieve" => {
                    let key = parsed.get("key").and_then(|k| k.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing 'key' for retrieve action"))?;
                    match memories.get(key) {
                        Some(value) => Ok(value.clone()),
                        None => Ok(format!("No memory found for key: {}", key)),
                    }
                }
                "list" => {
                    let keys: Vec<&String> = memories.keys().collect();
                    Ok(format!("Memory keys: {:?}", keys))
                }
                _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
            }
        }),
    }
}
