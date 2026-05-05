use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliCommand {
    pub name: String,
    pub description: String,
    pub usage: String,
}

pub struct CliManager {
    commands: Vec<CliCommand>,
}

impl CliManager {
    pub fn new() -> Self {
        let commands = vec![
            CliCommand {
                name: "help".to_string(),
                description: "Show help information".to_string(),
                usage: "help [command]".to_string(),
            },
            CliCommand {
                name: "status".to_string(),
                description: "Show system status".to_string(),
                usage: "status".to_string(),
            },
            CliCommand {
                name: "model".to_string(),
                description: "Switch AI model".to_string(),
                usage: "model <model_name>".to_string(),
            },
            CliCommand {
                name: "memory".to_string(),
                description: "Memory management commands".to_string(),
                usage: "memory <list|search|clear>".to_string(),
            },
            CliCommand {
                name: "skill".to_string(),
                description: "Skill management commands".to_string(),
                usage: "skill <list|install|uninstall> <skill_name>".to_string(),
            },
            CliCommand {
                name: "plugin".to_string(),
                description: "Plugin management commands".to_string(),
                usage: "plugin <list|load|unload> <plugin_name>".to_string(),
            },
            CliCommand {
                name: "config".to_string(),
                description: "Configuration commands".to_string(),
                usage: "config <get|set> <key> [value]".to_string(),
            },
            CliCommand {
                name: "workspace".to_string(),
                description: "Workspace management commands".to_string(),
                usage: "workspace <list|create|switch> [name]".to_string(),
            },
            CliCommand {
                name: "cron".to_string(),
                description: "Cron job management commands".to_string(),
                usage: "cron <list|add|remove|run> [args...]".to_string(),
            },
            CliCommand {
                name: "platform".to_string(),
                description: "Platform gateway commands".to_string(),
                usage: "platform <list|connect|disconnect> [platform_id]".to_string(),
            },
            CliCommand {
                name: "exit".to_string(),
                description: "Exit the application".to_string(),
                usage: "exit".to_string(),
            },
        ];

        Self { commands }
    }

    pub fn list_commands(&self) -> Vec<CliCommand> {
        self.commands.clone()
    }

    pub fn get_command(&self, name: &str) -> Option<CliCommand> {
        self.commands.iter().find(|c| c.name == name).cloned()
    }

    pub fn execute(&self, input: &str) -> Result<String, AppError> {
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(String::new());
        }

        let cmd = parts[0].to_lowercase();
        let args = &parts[1..];

        match cmd.as_str() {
            "help" => Ok(self.help(args)),
            "status" => Ok(self.status()),
            "exit" => Ok("Goodbye!".to_string()),
            _ => {
                if self.get_command(&cmd).is_some() {
                    Ok(format!("Command '{}' is recognized but requires runtime context to execute.", cmd))
                } else {
                    Err(AppError::InvalidInput(format!("Unknown command: {}", cmd)))
                }
            }
        }
    }

    fn help(&self, args: &[&str]) -> String {
        if let Some(cmd_name) = args.first() {
            if let Some(cmd) = self.get_command(cmd_name) {
                format!("{} - {}\nUsage: {}", cmd.name, cmd.description, cmd.usage)
            } else {
                format!("Unknown command: {}", cmd_name)
            }
        } else {
            let mut output = "Available commands:\n".to_string();
            for cmd in &self.commands {
                output.push_str(&format!("  {:12} - {}\n", cmd.name, cmd.description));
            }
            output.push_str("\nUse 'help <command>' for more information.");
            output
        }
    }

    fn status(&self) -> String {
        format!(
            "Mahakala Agent Status:\n\
            - Version: {}\n\
            - Platform: {}\n\
            - Uptime: {} seconds\n\
            - Status: Running",
            env!("CARGO_PKG_VERSION"),
            std::env::consts::OS,
            0
        )
    }
}

#[derive(Clone)]
pub struct CliHandle {
    inner: std::sync::Arc<CliManager>,
}

impl CliHandle {
    pub fn new() -> Self {
        Self {
            inner: std::sync::Arc::new(CliManager::new()),
        }
    }

    pub fn list_commands(&self) -> Vec<CliCommand> {
        self.inner.list_commands()
    }

    pub fn get_command(&self, name: &str) -> Option<CliCommand> {
        self.inner.get_command(name)
    }

    pub fn execute(&self, input: &str) -> Result<String, AppError> {
        self.inner.execute(input)
    }
}
