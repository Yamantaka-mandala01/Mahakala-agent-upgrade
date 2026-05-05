use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub loaded: bool,
    pub config: serde_json::Value,
    pub created_at: i64,
    pub source: PluginSource,
    pub manifest_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PluginSource {
    Builtin,
    Dynamic,
    UserInstalled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCreate {
    pub name: String,
    pub description: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub entry: Option<String>,
    pub config: Option<serde_json::Value>,
}

pub struct PluginRegistry {
    plugins: Arc<Mutex<HashMap<String, Plugin>>>,
    plugin_dir: PathBuf,
}

impl PluginRegistry {
    pub fn new() -> Self {
        let plugin_dir = crate::constants::get_mahakala_home().join("plugins");
        std::fs::create_dir_all(&plugin_dir).ok();

        let mut plugins = HashMap::new();

        let default_plugins = vec![
            ("disk_cleanup", "Disk Cleanup", "System disk cleanup plugin", "1.0.0"),
            ("memory_plugin", "Memory Plugin", "Memory management plugin", "1.1.0"),
            ("network_monitor", "Network Monitor", "Network traffic monitoring", "0.9.0"),
            ("log_manager", "Log Manager", "Log rotation and management", "1.0.0"),
            ("scheduler_plugin", "Scheduler Plugin", "Advanced task scheduling", "0.8.0"),
            ("security_scanner", "Security Scanner", "Security vulnerability scanning", "1.2.0"),
        ];

        let now = chrono::Utc::now().timestamp();
        for (id, name, desc, version) in default_plugins {
            plugins.insert(id.to_string(), Plugin {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                version: version.to_string(),
                loaded: id == "disk_cleanup" || id == "memory_plugin",
                config: serde_json::Value::Null,
                created_at: now,
                source: PluginSource::Builtin,
                manifest_path: None,
            });
        }

        let registry = Self {
            plugins: Arc::new(Mutex::new(plugins)),
            plugin_dir,
        };

        registry.scan_plugins();
        registry
    }

    pub fn scan_plugins(&self) {
        let entries = match std::fs::read_dir(&self.plugin_dir) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        let now = chrono::Utc::now().timestamp();
        let mut plugins = self.plugins.lock();

        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }

            let manifest_path = path.join("plugin.json");
            if !manifest_path.exists() {
                continue;
            }

            let manifest_content = match std::fs::read_to_string(&manifest_path) {
                Ok(content) => content,
                Err(_) => continue,
            };

            let manifest: PluginManifest = match serde_json::from_str(&manifest_content) {
                Ok(m) => m,
                Err(_) => continue,
            };

            let id = manifest.name.to_lowercase().replace(" ", "_");
            if plugins.contains_key(&id) {
                continue;
            }

            let plugin = Plugin {
                id: id.clone(),
                name: manifest.name,
                description: manifest.description,
                version: manifest.version,
                loaded: false,
                config: manifest.config.unwrap_or(serde_json::Value::Null),
                created_at: now,
                source: PluginSource::Dynamic,
                manifest_path: Some(manifest_path.to_string_lossy().to_string()),
            };

            plugins.insert(id, plugin);
        }
    }

    pub fn load(&self, plugin_id: &str) -> Result<Plugin, AppError> {
        let mut plugins = self.plugins.lock();
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.loaded = true;
            Ok(plugin.clone())
        } else {
            Err(AppError::NotFound(format!("Plugin {} not found", plugin_id)))
        }
    }

    pub fn unload(&self, plugin_id: &str) -> Result<Plugin, AppError> {
        let mut plugins = self.plugins.lock();
        if let Some(plugin) = plugins.get_mut(plugin_id) {
            plugin.loaded = false;
            Ok(plugin.clone())
        } else {
            Err(AppError::NotFound(format!("Plugin {} not found", plugin_id)))
        }
    }

    pub fn get(&self, plugin_id: &str) -> Option<Plugin> {
        let plugins = self.plugins.lock();
        plugins.get(plugin_id).cloned()
    }

    pub fn list(&self) -> Vec<Plugin> {
        let plugins = self.plugins.lock();
        plugins.values().cloned().collect()
    }

    pub fn list_loaded(&self) -> Vec<Plugin> {
        let plugins = self.plugins.lock();
        plugins.values()
            .filter(|p| p.loaded)
            .cloned()
            .collect()
    }

    pub fn add_plugin(&self, create: PluginCreate) -> Result<Plugin, AppError> {
        let now = chrono::Utc::now().timestamp();
        let id = create.name.to_lowercase().replace(" ", "_");
        let plugin = Plugin {
            id: id.clone(),
            name: create.name,
            description: create.description.unwrap_or_default(),
            version: create.version.unwrap_or_else(|| "1.0.0".to_string()),
            loaded: false,
            config: serde_json::Value::Null,
            created_at: now,
            source: PluginSource::UserInstalled,
            manifest_path: None,
        };

        let mut plugins = self.plugins.lock();
        plugins.insert(id, plugin.clone());
        Ok(plugin)
    }

    pub fn remove_plugin(&self, plugin_id: &str) -> Result<bool, AppError> {
        let mut plugins = self.plugins.lock();
        Ok(plugins.remove(plugin_id).is_some())
    }

    pub fn reload_plugins(&self) -> usize {
        self.scan_plugins();
        self.plugins.lock().len()
    }

    pub fn install_from_manifest(&self, manifest_path: &Path) -> Result<Plugin, AppError> {
        let manifest_content = std::fs::read_to_string(manifest_path)
            .map_err(|e| AppError::Io(e))?;
        
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .map_err(|e| AppError::Json(e))?;

        let id = manifest.name.to_lowercase().replace(" ", "_");
        let now = chrono::Utc::now().timestamp();
        let plugin = Plugin {
            id: id.clone(),
            name: manifest.name,
            description: manifest.description,
            version: manifest.version,
            loaded: false,
            config: manifest.config.unwrap_or(serde_json::Value::Null),
            created_at: now,
            source: PluginSource::Dynamic,
            manifest_path: Some(manifest_path.to_string_lossy().to_string()),
        };

        let mut plugins = self.plugins.lock();
        plugins.insert(id, plugin.clone());
        Ok(plugin)
    }
}

#[derive(Clone)]
pub struct PluginManager {
    registry: Arc<PluginRegistry>,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
        }
    }

    pub fn load(&self, plugin_id: &str) -> Result<Plugin, AppError> {
        self.registry.load(plugin_id)
    }

    pub fn unload(&self, plugin_id: &str) -> Result<Plugin, AppError> {
        self.registry.unload(plugin_id)
    }

    pub fn get(&self, plugin_id: &str) -> Option<Plugin> {
        self.registry.get(plugin_id)
    }

    pub fn list(&self) -> Vec<Plugin> {
        self.registry.list()
    }

    pub fn list_loaded(&self) -> Vec<Plugin> {
        self.registry.list_loaded()
    }

    pub fn add_plugin(&self, create: PluginCreate) -> Result<Plugin, AppError> {
        self.registry.add_plugin(create)
    }

    pub fn remove_plugin(&self, plugin_id: &str) -> Result<bool, AppError> {
        self.registry.remove_plugin(plugin_id)
    }

    pub fn reload_plugins(&self) -> usize {
        self.registry.reload_plugins()
    }

    pub fn install_from_manifest(&self, manifest_path: &Path) -> Result<Plugin, AppError> {
        self.registry.install_from_manifest(manifest_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_load_unload() {
        let manager = PluginManager::new();
        
        let result = manager.load("disk_cleanup");
        assert!(result.is_ok(), "Plugin load should succeed");
        assert!(result.unwrap().loaded, "Plugin should be loaded");
        
        let result = manager.unload("disk_cleanup");
        assert!(result.is_ok(), "Plugin unload should succeed");
        assert!(!result.unwrap().loaded, "Plugin should be unloaded");
    }

    #[test]
    fn test_plugin_list() {
        let manager = PluginManager::new();
        let plugins = manager.list();
        assert!(!plugins.is_empty(), "Should have default plugins");
    }

    #[test]
    fn test_plugin_add_remove() {
        let manager = PluginManager::new();
        
        let create = PluginCreate {
            name: "test_plugin".to_string(),
            description: Some("Test plugin".to_string()),
            version: Some("1.0.0".to_string()),
        };
        
        let result = manager.add_plugin(create);
        assert!(result.is_ok(), "Plugin add should succeed");
        
        let plugin = result.unwrap();
        assert_eq!(plugin.name, "test_plugin");
        
        let result = manager.remove_plugin(&plugin.id);
        assert!(result.is_ok(), "Plugin remove should succeed");
        assert!(result.unwrap(), "Plugin should be removed");
    }

    #[test]
    fn test_plugin_loaded_list() {
        let manager = PluginManager::new();
        manager.load("disk_cleanup").unwrap();
        
        let loaded = manager.list_loaded();
        assert!(!loaded.is_empty(), "Should have loaded plugins");
    }
}
