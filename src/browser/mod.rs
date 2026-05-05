use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserInstance {
    pub id: String,
    pub browser_type: String,
    pub headless: bool,
    pub url: Option<String>,
    pub title: Option<String>,
    pub is_active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserConfig {
    pub browser_type: String,
    pub headless: bool,
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotResult {
    pub path: String,
    pub format: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageContent {
    pub url: String,
    pub title: String,
    pub text_content: String,
    pub html_content: String,
}

pub struct BrowserManager {
    instances: Arc<Mutex<HashMap<String, BrowserInstance>>>,
    config: BrowserConfig,
}

impl BrowserManager {
    pub fn new(config: BrowserConfig) -> Self {
        Self {
            instances: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn launch_browser(&self, browser_type: Option<String>) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let instance = BrowserInstance {
            id: id.clone(),
            browser_type: browser_type.unwrap_or_else(|| self.config.browser_type.clone()),
            headless: self.config.headless,
            url: None,
            title: None,
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
        };

        let mut instances = self.instances.lock();
        instances.insert(id.clone(), instance);
        Ok(id)
    }

    pub fn close_browser(&self, id: &str) -> Result<(), AppError> {
        let mut instances = self.instances.lock();
        if let Some(instance) = instances.get_mut(id) {
            instance.is_active = false;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Browser instance {} not found", id)))
        }
    }

    pub fn navigate(&self, instance_id: &str, url: &str) -> Result<(), AppError> {
        let mut instances = self.instances.lock();
        if let Some(instance) = instances.get_mut(instance_id) {
            if !instance.is_active {
                return Err(AppError::Internal(format!("Browser instance {} is not active", instance_id)));
            }
            instance.url = Some(url.to_string());
            instance.title = Some(format!("Page: {}", url));
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Browser instance {} not found", instance_id)))
        }
    }

    pub fn click(&self, instance_id: &str, selector: &str) -> Result<String, AppError> {
        let instances = self.instances.lock();
        if let Some(instance) = instances.get(instance_id) {
            if !instance.is_active {
                return Err(AppError::Internal(format!("Browser instance {} is not active", instance_id)));
            }
            Ok(format!("Clicked on '{}' in {}", selector, instance.url.as_deref().unwrap_or("unknown")))
        } else {
            Err(AppError::NotFound(format!("Browser instance {} not found", instance_id)))
        }
    }

    pub fn fill(&self, instance_id: &str, selector: &str, value: &str) -> Result<String, AppError> {
        let instances = self.instances.lock();
        if let Some(instance) = instances.get(instance_id) {
            if !instance.is_active {
                return Err(AppError::Internal(format!("Browser instance {} is not active", instance_id)));
            }
            Ok(format!("Filled '{}' with '{}' in {}", selector, value, instance.url.as_deref().unwrap_or("unknown")))
        } else {
            Err(AppError::NotFound(format!("Browser instance {} not found", instance_id)))
        }
    }

    pub fn get_content(&self, instance_id: &str) -> Result<PageContent, AppError> {
        let instances = self.instances.lock();
        if let Some(instance) = instances.get(instance_id) {
            if !instance.is_active {
                return Err(AppError::Internal(format!("Browser instance {} is not active", instance_id)));
            }
            Ok(PageContent {
                url: instance.url.clone().unwrap_or_default(),
                title: instance.title.clone().unwrap_or_default(),
                text_content: "Page text content would be extracted here".to_string(),
                html_content: "<html><body>Page HTML content would be extracted here</body></html>".to_string(),
            })
        } else {
            Err(AppError::NotFound(format!("Browser instance {} not found", instance_id)))
        }
    }

    pub fn screenshot(&self, instance_id: &str, path: &str, format: &str) -> Result<ScreenshotResult, AppError> {
        let instances = self.instances.lock();
        if let Some(instance) = instances.get(instance_id) {
            if !instance.is_active {
                return Err(AppError::Internal(format!("Browser instance {} is not active", instance_id)));
            }
            Ok(ScreenshotResult {
                path: path.to_string(),
                format: format.to_string(),
                width: self.config.viewport_width,
                height: self.config.viewport_height,
            })
        } else {
            Err(AppError::NotFound(format!("Browser instance {} not found", instance_id)))
        }
    }

    pub fn evaluate(&self, instance_id: &str, script: &str) -> Result<String, AppError> {
        let instances = self.instances.lock();
        if let Some(instance) = instances.get(instance_id) {
            if !instance.is_active {
                return Err(AppError::Internal(format!("Browser instance {} is not active", instance_id)));
            }
            Ok(format!("Evaluated script in {}: {}", instance.url.as_deref().unwrap_or("unknown"), script))
        } else {
            Err(AppError::NotFound(format!("Browser instance {} not found", instance_id)))
        }
    }

    pub fn list_instances(&self) -> Vec<BrowserInstance> {
        let instances = self.instances.lock();
        instances.values().cloned().collect()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let instances = self.instances.lock();
        let total = instances.len();
        let active = instances.values().filter(|i| i.is_active).count();

        serde_json::json!({
            "total_instances": total,
            "active_instances": active,
            "browser_type": self.config.browser_type,
            "headless": self.config.headless,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_browser() {
        let config = BrowserConfig {
            browser_type: "chromium".to_string(),
            headless: true,
            viewport_width: 1920,
            viewport_height: 1080,
            user_agent: None,
        };
        let manager = BrowserManager::new(config);

        let id = manager.launch_browser(None);
        assert!(id.is_ok());
        assert_eq!(manager.list_instances().len(), 1);
    }

    #[test]
    fn test_navigate_and_close() {
        let config = BrowserConfig {
            browser_type: "chromium".to_string(),
            headless: true,
            viewport_width: 1920,
            viewport_height: 1080,
            user_agent: None,
        };
        let manager = BrowserManager::new(config);

        let id = manager.launch_browser(None).unwrap();
        manager.navigate(&id, "https://example.com").unwrap();
        
        let instances = manager.list_instances();
        assert_eq!(instances[0].url, Some("https://example.com".to_string()));

        manager.close_browser(&id).unwrap();
        let instances = manager.list_instances();
        assert!(!instances[0].is_active);
    }

    #[test]
    fn test_get_content() {
        let config = BrowserConfig {
            browser_type: "chromium".to_string(),
            headless: true,
            viewport_width: 1920,
            viewport_height: 1080,
            user_agent: None,
        };
        let manager = BrowserManager::new(config);

        let id = manager.launch_browser(None).unwrap();
        manager.navigate(&id, "https://example.com").unwrap();

        let content = manager.get_content(&id);
        assert!(content.is_ok());
        assert_eq!(content.unwrap().url, "https://example.com");
    }
}
