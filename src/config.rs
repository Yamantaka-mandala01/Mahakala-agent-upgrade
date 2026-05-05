use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::constants;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub model: String,
    pub provider: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub platform: Option<String>,
    pub language: String,
    pub theme: String,
    pub port: u16,
    pub host: String,
    pub workspace: Option<String>,
    pub models: HashMap<String, ModelConfig>,
    pub platforms: HashMap<String, PlatformConfig>,
    pub tools: HashMap<String, ToolConfig>,
    pub skills: HashMap<String, SkillConfig>,
    pub plugins: HashMap<String, PluginConfig>,
    pub cron_jobs: Vec<CronJobConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub enabled: bool,
    pub model: String,
    pub url: Option<String>,
    pub key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub enabled: bool,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolConfig {
    pub enabled: bool,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillConfig {
    pub enabled: bool,
    pub path: Option<String>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub enabled: bool,
    pub path: Option<String>,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobConfig {
    pub id: String,
    pub name: String,
    pub schedule: String,
    pub command: String,
    pub enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        let mut models = HashMap::new();
        // 默认启用 Ollama 本地模型
        models.insert("ollama".to_string(), ModelConfig {
            enabled: true,
            model: "llama3.2".to_string(),
            url: Some("http://localhost:11434".to_string()),
            key: None,
        });
        models.insert("openai".to_string(), ModelConfig {
            enabled: false,
            model: "gpt-4o".to_string(),
            url: Some("https://api.openai.com/v1".to_string()),
            key: None,
        });
        models.insert("anthropic".to_string(), ModelConfig {
            enabled: false,
            model: "claude-sonnet-4-20250514".to_string(),
            url: Some("https://api.anthropic.com/v1".to_string()),
            key: None,
        });
        models.insert("deepseek".to_string(), ModelConfig {
            enabled: false,
            model: "deepseek-chat".to_string(),
            url: Some("https://api.deepseek.com".to_string()),
            key: None,
        });

        let mut platforms = HashMap::new();
        platforms.insert("wechat".to_string(), PlatformConfig {
            enabled: false,
            config: HashMap::new(),
        });

        Self {
            model: "llama3.2".to_string(),
            provider: Some("ollama".to_string()),
            api_base_url: Some("http://localhost:11434/v1".to_string()),
            api_key: None,
            temperature: Some(0.7),
            max_tokens: Some(2048),
            platform: Some("web".to_string()),
            language: "zh".to_string(),
            theme: "dark".to_string(),
            port: constants::DEFAULT_PORT,
            host: constants::DEFAULT_HOST.to_string(),
            workspace: None,
            models,
            platforms,
            tools: HashMap::new(),
            skills: HashMap::new(),
            plugins: HashMap::new(),
            cron_jobs: Vec::new(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Self {
        let config_path = constants::get_config_path();
        if config_path.exists() {
            match std::fs::read_to_string(&config_path) {
                Ok(content) => {
                    match serde_yaml::from_str(&content) {
                        Ok(config) => return config,
                        Err(e) => {
                            tracing::warn!("Failed to parse config file: {}", e);
                        }
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to read config file: {}", e);
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = constants::get_config_path();
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_yaml::to_string(self)?;
        std::fs::write(&config_path, content)?;
        Ok(())
    }

    pub fn get_env_var(key: &str) -> Option<String> {
        std::env::var(key).ok()
    }
}
