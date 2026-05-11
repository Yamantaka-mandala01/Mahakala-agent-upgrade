use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "env_tool".to_string(),
        description: "Get system environment information (OS, architecture, hostname, etc.)".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "env_tool",
                "description": "Get system environment information (OS, architecture, hostname, etc.)",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "info_type": {
                            "type": "string",
                            "description": "Type of information to get (os, arch, hostname, cwd, env_var)",
                            "enum": ["os", "arch", "hostname", "cwd", "env_var"]
                        },
                        "var_name": {
                            "type": "string",
                            "description": "Environment variable name (for env_var action)"
                        }
                    },
                    "required": ["info_type"]
                }
            }
        }),
        execute: Arc::new(|arguments: &str| {
            let arguments = arguments.to_string();
            Box::pin(async move {
                let args: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

                let info_type = args.get("info_type").and_then(|v| v.as_str()).unwrap_or("os");
                let var_name = args.get("var_name").and_then(|v| v.as_str());

                match info_type {
                    "os" => {
                        let os = std::env::consts::OS;
                        Ok(format!("OS: {}", os))
                    }
                    "arch" => Ok(format!("Architecture: {}", std::env::consts::ARCH)),
                    "hostname" => {
                        let hostname = std::env::var("COMPUTERNAME")
                            .unwrap_or_else(|_| "Unknown".to_string());
                        Ok(format!("Hostname: {}", hostname))
                    }
                    "cwd" => {
                        let cwd = std::env::current_dir()
                            .map_err(|e| anyhow::anyhow!("Failed to get current directory: {}", e))?;
                        Ok(format!("Current directory: {}", cwd.display()))
                    }
                    "env_var" => {
                        if let Some(name) = var_name {
                            match std::env::var(name) {
                                Ok(value) => Ok(format!("{} = {}", name, value)),
                                Err(_) => Ok(format!("Environment variable '{}' not set", name)),
                            }
                        } else {
                            Err(anyhow::anyhow!("var_name is required for env_var action"))
                        }
                    }
                    _ => Err(anyhow::anyhow!("Unknown info_type: {}", info_type)),
                }
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}
