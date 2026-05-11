use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub id: String,
    pub name: String,
    pub platform_type: String,
    pub enabled: bool,
    pub connected: bool,
    pub config: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub platform: String,
    pub content: String,
    pub sender: String,
    pub timestamp: i64,
    pub read: bool,
}

pub struct GatewayManager {
    platforms: Arc<Mutex<HashMap<String, PlatformConfig>>>,
    messages: Arc<Mutex<Vec<Message>>>,
}

impl Default for GatewayManager {
    fn default() -> Self {
        Self::new()
    }
}

impl GatewayManager {
    pub fn new() -> Self {
        let mut platforms = HashMap::new();
        let now = chrono::Utc::now().timestamp();

        let default_platforms = vec![
            ("wechat", "WeChat", "wechat"),
            ("qqbot", "QQ Bot", "qq"),
            ("telegram", "Telegram", "telegram"),
            ("feishu", "Feishu", "feishu"),
            ("discord", "Discord", "discord"),
        ];

        for (id, name, ptype) in default_platforms {
            platforms.insert(id.to_string(), PlatformConfig {
                id: id.to_string(),
                name: name.to_string(),
                platform_type: ptype.to_string(),
                enabled: false,
                connected: false,
                config: serde_json::Value::Null,
                created_at: now,
            });
        }

        Self {
            platforms: Arc::new(Mutex::new(platforms)),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn connect_platform(&self, platform_id: &str) -> Result<PlatformConfig, AppError> {
        let mut platforms = self.platforms.lock();
        if let Some(platform) = platforms.get_mut(platform_id) {
            platform.connected = true;
            platform.enabled = true;
            Ok(platform.clone())
        } else {
            Err(AppError::NotFound(format!("Platform {} not found", platform_id)))
        }
    }

    pub fn disconnect_platform(&self, platform_id: &str) -> Result<PlatformConfig, AppError> {
        let mut platforms = self.platforms.lock();
        if let Some(platform) = platforms.get_mut(platform_id) {
            platform.connected = false;
            Ok(platform.clone())
        } else {
            Err(AppError::NotFound(format!("Platform {} not found", platform_id)))
        }
    }

    pub fn get_platform(&self, platform_id: &str) -> Option<PlatformConfig> {
        let platforms = self.platforms.lock();
        platforms.get(platform_id).cloned()
    }

    pub fn list_platforms(&self) -> Vec<PlatformConfig> {
        let platforms = self.platforms.lock();
        platforms.values().cloned().collect()
    }

    pub fn list_connected(&self) -> Vec<PlatformConfig> {
        let platforms = self.platforms.lock();
        platforms.values()
            .filter(|p| p.connected)
            .cloned()
            .collect()
    }

    pub fn update_platform_config(&self, platform_id: &str, config: serde_json::Value) -> Result<PlatformConfig, AppError> {
        let mut platforms = self.platforms.lock();
        if let Some(platform) = platforms.get_mut(platform_id) {
            platform.config = config;
            Ok(platform.clone())
        } else {
            Err(AppError::NotFound(format!("Platform {} not found", platform_id)))
        }
    }

    pub fn add_message(&self, platform: &str, content: &str, sender: &str) -> Message {
        let now = chrono::Utc::now().timestamp();
        let message = Message {
            id: uuid::Uuid::new_v4().to_string(),
            platform: platform.to_string(),
            content: content.to_string(),
            sender: sender.to_string(),
            timestamp: now,
            read: false,
        };

        let mut messages = self.messages.lock();
        messages.push(message.clone());
        message
    }

    pub fn list_messages(&self) -> Vec<Message> {
        let messages = self.messages.lock();
        messages.clone()
    }

    pub fn list_messages_by_platform(&self, platform: &str) -> Vec<Message> {
        let messages = self.messages.lock();
        messages.iter()
            .filter(|m| m.platform == platform)
            .cloned()
            .collect()
    }

    pub fn mark_read(&self, message_id: &str) -> Result<bool, AppError> {
        let mut messages = self.messages.lock();
        if let Some(msg) = messages.iter_mut().find(|m| m.id == message_id) {
            msg.read = true;
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("Message {} not found", message_id)))
        }
    }

    pub fn get_unread_count(&self) -> usize {
        let messages = self.messages.lock();
        messages.iter().filter(|m| !m.read).count()
    }
}

#[derive(Clone)]
pub struct GatewayHandle {
    inner: Arc<GatewayManager>,
}

impl Default for GatewayHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl GatewayHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(GatewayManager::new()),
        }
    }

    pub fn connect_platform(&self, platform_id: &str) -> Result<PlatformConfig, AppError> {
        self.inner.connect_platform(platform_id)
    }

    pub fn disconnect_platform(&self, platform_id: &str) -> Result<PlatformConfig, AppError> {
        self.inner.disconnect_platform(platform_id)
    }

    pub fn get_platform(&self, platform_id: &str) -> Option<PlatformConfig> {
        self.inner.get_platform(platform_id)
    }

    pub fn list_platforms(&self) -> Vec<PlatformConfig> {
        self.inner.list_platforms()
    }

    pub fn list_connected(&self) -> Vec<PlatformConfig> {
        self.inner.list_connected()
    }

    pub fn update_platform_config(&self, platform_id: &str, config: serde_json::Value) -> Result<PlatformConfig, AppError> {
        self.inner.update_platform_config(platform_id, config)
    }

    pub fn add_message(&self, platform: &str, content: &str, sender: &str) -> Message {
        self.inner.add_message(platform, content, sender)
    }

    pub fn list_messages(&self) -> Vec<Message> {
        self.inner.list_messages()
    }

    pub fn list_messages_by_platform(&self, platform: &str) -> Vec<Message> {
        self.inner.list_messages_by_platform(platform)
    }

    pub fn mark_read(&self, message_id: &str) -> Result<bool, AppError> {
        self.inner.mark_read(message_id)
    }

    pub fn get_unread_count(&self) -> usize {
        self.inner.get_unread_count()
    }
}
