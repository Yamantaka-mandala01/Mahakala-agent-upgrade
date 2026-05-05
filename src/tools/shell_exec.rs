use super::registry::ToolInfo;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "shell_exec".to_string(),
        description: "Execute a shell command and return the output".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "shell_exec",
                "description": "Execute a shell command and return the output",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string",
                            "description": "The shell command to execute"
                        }
                    },
                    "required": ["command"]
                }
            }
        }),
        execute: Box::new(|args: &str| {
            let parsed: serde_json::Value = serde_json::from_str(args)?;
            let command = parsed.get("command")
                .and_then(|c| c.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;
            
            let output = std::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
                .arg(if cfg!(windows) { "/C" } else { "-c" })
                .arg(command)
                .output()?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            if output.status.success() {
                Ok(stdout.to_string())
            } else {
                Ok(format!("Error: {}\n{}", stderr, stdout))
            }
        }),
    }
}
