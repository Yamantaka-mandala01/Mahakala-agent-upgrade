use super::registry::ToolInfo;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "todo".to_string(),
        description: "Manage todo list".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "todo",
                "description": "Manage todo list",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "action": {
                            "type": "string",
                            "enum": ["add", "list", "complete", "remove"],
                            "description": "Action to perform"
                        },
                        "task": {
                            "type": "string",
                            "description": "Task description (for add action)"
                        },
                        "index": {
                            "type": "integer",
                            "description": "Task index (for complete/remove action)"
                        }
                    },
                    "required": ["action"]
                }
            }
        }),
        execute: Box::new(|args: &str| {
            let parsed: serde_json::Value = serde_json::from_str(args)?;
            let action = parsed.get("action")
                .and_then(|a| a.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'action' parameter"))?;
            
            let home = crate::constants::get_mahakala_home();
            let todo_file = home.join("todos.json");
            
            let mut todos: Vec<String> = if todo_file.exists() {
                let content = std::fs::read_to_string(&todo_file)?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                Vec::new()
            };
            
            match action {
                "add" => {
                    let task = parsed.get("task").and_then(|t| t.as_str())
                        .ok_or_else(|| anyhow::anyhow!("Missing 'task' for add action"))?;
                    todos.push(task.to_string());
                    std::fs::write(&todo_file, serde_json::to_string_pretty(&todos)?)?;
                    Ok(format!("Added todo: {}", task))
                }
                "list" => {
                    if todos.is_empty() {
                        Ok("No todos".to_string())
                    } else {
                        let list = todos.iter().enumerate()
                            .map(|(i, t)| format!("{}. {}", i + 1, t))
                            .collect::<Vec<_>>();
                        Ok(list.join("\n"))
                    }
                }
                "complete" => {
                    let index = parsed.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                    if index > 0 && index <= todos.len() {
                        let task = todos.remove(index - 1);
                        std::fs::write(&todo_file, serde_json::to_string_pretty(&todos)?)?;
                        Ok(format!("Completed: {}", task))
                    } else {
                        Ok("Invalid index".to_string())
                    }
                }
                "remove" => {
                    let index = parsed.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                    if index > 0 && index <= todos.len() {
                        let task = todos.remove(index - 1);
                        std::fs::write(&todo_file, serde_json::to_string_pretty(&todos)?)?;
                        Ok(format!("Removed: {}", task))
                    } else {
                        Ok("Invalid index".to_string())
                    }
                }
                _ => Err(anyhow::anyhow!("Unknown action: {}", action)),
            }
        }),
    }
}
