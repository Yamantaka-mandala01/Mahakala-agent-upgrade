use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

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
        execute: Arc::new(|args: &str| {
            let args = args.to_string();
            Box::pin(async move {
                let parsed: serde_json::Value = serde_json::from_str(&args)?;
                let command = parsed.get("command")
                    .and_then(|c| c.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'command' parameter"))?;
                
                let output = tokio::process::Command::new(if cfg!(windows) { "cmd" } else { "sh" })
                    .arg(if cfg!(windows) { "/C" } else { "-c" })
                    .arg(command)
                    .output()
                    .await?;
                
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                
                if output.status.success() {
                    Ok(stdout.to_string())
                } else {
                    Ok(format!("Error: {}\n{}", stderr, stdout))
                }
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}