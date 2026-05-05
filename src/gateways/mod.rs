use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    pub id: String,
    pub platform: String,
    pub name: String,
    pub enabled: bool,
    pub token: String,
    pub webhook_url: Option<String>,
    pub settings: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub platform: String,
    pub chat_id: String,
    pub user_id: String,
    pub content: String,
    pub timestamp: i64,
    pub attachments: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayStatus {
    pub id: String,
    pub platform: String,
    pub name: String,
    pub connected: bool,
    pub message_count: usize,
    pub last_activity: Option<i64>,
}

pub struct GatewayManager {
    gateways: Arc<Mutex<HashMap<String, GatewayConfig>>>,
    message_counts: Arc<Mutex<HashMap<String, usize>>>,
}

impl GatewayManager {
    pub fn new() -> Self {
        Self {
            gateways: Arc::new(Mutex::new(HashMap::new())),
            message_counts: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_gateway(&self, config: GatewayConfig) -> Result<(), AppError> {
        let mut gateways = self.gateways.lock();
        gateways.insert(config.id.clone(), config);
        Ok(())
    }

    pub fn remove_gateway(&self, id: &str) -> Result<bool, AppError> {
        let mut gateways = self.gateways.lock();
        Ok(gateways.remove(id).is_some())
    }

    pub fn enable_gateway(&self, id: &str) -> Result<(), AppError> {
        let mut gateways = self.gateways.lock();
        if let Some(gateway) = gateways.get_mut(id) {
            gateway.enabled = true;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Gateway {} not found", id)))
        }
    }

    pub fn disable_gateway(&self, id: &str) -> Result<(), AppError> {
        let mut gateways = self.gateways.lock();
        if let Some(gateway) = gateways.get_mut(id) {
            gateway.enabled = false;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Gateway {} not found", id)))
        }
    }

    pub fn get_gateway(&self, id: &str) -> Option<GatewayConfig> {
        let gateways = self.gateways.lock();
        gateways.get(id).cloned()
    }

    pub fn list_gateways(&self) -> Vec<GatewayConfig> {
        let gateways = self.gateways.lock();
        gateways.values().cloned().collect()
    }

    pub fn get_status(&self, id: &str) -> Option<GatewayStatus> {
        let gateways = self.gateways.lock();
        let message_counts = self.message_counts.lock();
        
        gateways.get(id).map(|gateway| {
            GatewayStatus {
                id: gateway.id.clone(),
                platform: gateway.platform.clone(),
                name: gateway.name.clone(),
                connected: gateway.enabled,
                message_count: *message_counts.get(&gateway.id).unwrap_or(&0),
                last_activity: None,
            }
        })
    }

    pub fn list_statuses(&self) -> Vec<GatewayStatus> {
        let gateways = self.gateways.lock();
        let message_counts = self.message_counts.lock();
        
        gateways.values().map(|gateway| {
            GatewayStatus {
                id: gateway.id.clone(),
                platform: gateway.platform.clone(),
                name: gateway.name.clone(),
                connected: gateway.enabled,
                message_count: *message_counts.get(&gateway.id).unwrap_or(&0),
                last_activity: None,
            }
        }).collect()
    }

    pub fn send_message(&self, gateway_id: &str, chat_id: &str, content: &str) -> Result<String, AppError> {
        let gateways = self.gateways.lock();
        if let Some(gateway) = gateways.get(gateway_id) {
            if !gateway.enabled {
                return Err(AppError::Internal(format!("Gateway {} is not enabled", gateway_id)));
            }

            let message_id = uuid::Uuid::new_v4().to_string();
            
            let mut message_counts = self.message_counts.lock();
            *message_counts.entry(gateway_id.to_string()).or_insert(0) += 1;

            Ok(format!("Message sent via {}: {}", gateway.platform, message_id))
        } else {
            Err(AppError::NotFound(format!("Gateway {} not found", gateway_id)))
        }
    }

    pub fn receive_message(&self, gateway_id: &str, message: Message) -> Result<(), AppError> {
        let gateways = self.gateways.lock();
        if gateways.contains_key(gateway_id) {
            let mut message_counts = self.message_counts.lock();
            *message_counts.entry(gateway_id.to_string()).or_insert(0) += 1;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Gateway {} not found", gateway_id)))
        }
    }

    pub fn get_message_count(&self, gateway_id: &str) -> usize {
        let message_counts = self.message_counts.lock();
        *message_counts.get(gateway_id).unwrap_or(&0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_gateway() {
        let manager = GatewayManager::new();
        let config = GatewayConfig {
            id: "telegram_1".to_string(),
            platform: "telegram".to_string(),
            name: "Telegram Bot".to_string(),
            enabled: false,
            token: "test_token".to_string(),
            webhook_url: None,
            settings: serde_json::json!({}),
        };

        let result = manager.add_gateway(config);
        assert!(result.is_ok());
        assert_eq!(manager.list_gateways().len(), 1);
    }

    #[test]
    fn test_enable_disable_gateway() {
        let manager = GatewayManager::new();
        let config = GatewayConfig {
            id: "discord_1".to_string(),
            platform: "discord".to_string(),
            name: "Discord Bot".to_string(),
            enabled: false,
            token: "test_token".to_string(),
            webhook_url: None,
            settings: serde_json::json!({}),
        };

        manager.add_gateway(config).unwrap();
        manager.enable_gateway("discord_1").unwrap();
        
        let gateway = manager.get_gateway("discord_1").unwrap();
        assert!(gateway.enabled);

        manager.disable_gateway("discord_1").unwrap();
        let gateway = manager.get_gateway("discord_1").unwrap();
        assert!(!gateway.enabled);
    }

    #[test]
    fn test_send_message() {
        let manager = GatewayManager::new();
        let config = GatewayConfig {
            id: "slack_1".to_string(),
            platform: "slack".to_string(),
            name: "Slack Bot".to_string(),
            enabled: true,
            token: "test_token".to_string(),
            webhook_url: None,
            settings: serde_json::json!({}),
        };

        manager.add_gateway(config).unwrap();
        let result = manager.send_message("slack_1", "channel_1", "Hello!");
        assert!(result.is_ok());
        assert_eq!(manager.get_message_count("slack_1"), 1);
    }
}
