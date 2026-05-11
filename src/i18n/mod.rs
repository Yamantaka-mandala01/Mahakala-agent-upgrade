use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    pub current_locale: String,
    pub available_locales: Vec<String>,
}

pub struct I18nManager {
    translations: Arc<Mutex<HashMap<String, HashMap<String, String>>>>,
    current_locale: Arc<Mutex<String>>,
}

impl Default for I18nManager {
    fn default() -> Self {
        Self::new()
    }
}

impl I18nManager {
    pub fn new() -> Self {
        let mut translations: HashMap<String, HashMap<String, String>> = HashMap::new();

        let mut zh = HashMap::new();
        zh.insert("app.title".to_string(), "Mahakala Agent".to_string());
        zh.insert("nav.chat".to_string(), "对话".to_string());
        zh.insert("nav.memory".to_string(), "记忆".to_string());
        zh.insert("nav.skills".to_string(), "技能".to_string());
        zh.insert("nav.tools".to_string(), "工具".to_string());
        zh.insert("nav.plugins".to_string(), "插件".to_string());
        zh.insert("nav.messages".to_string(), "消息".to_string());
        zh.insert("nav.platforms".to_string(), "平台".to_string());
        zh.insert("nav.cron".to_string(), "定时任务".to_string());
        zh.insert("nav.settings".to_string(), "设置".to_string());
        zh.insert("common.save".to_string(), "保存".to_string());
        zh.insert("common.cancel".to_string(), "取消".to_string());
        zh.insert("common.delete".to_string(), "删除".to_string());
        zh.insert("common.edit".to_string(), "编辑".to_string());
        zh.insert("common.add".to_string(), "添加".to_string());
        zh.insert("common.search".to_string(), "搜索".to_string());
        zh.insert("common.loading".to_string(), "加载中...".to_string());
        zh.insert("common.success".to_string(), "成功".to_string());
        zh.insert("common.error".to_string(), "错误".to_string());
        zh.insert("chat.placeholder".to_string(), "输入消息...".to_string());
        zh.insert("chat.send".to_string(), "发送".to_string());
        zh.insert("chat.thinking".to_string(), "思考中...".to_string());
        zh.insert("memory.facts".to_string(), "事实".to_string());
        zh.insert("memory.entities".to_string(), "实体".to_string());
        zh.insert("memory.dimensions".to_string(), "维度".to_string());
        zh.insert("settings.model".to_string(), "模型设置".to_string());
        zh.insert("settings.theme".to_string(), "主题".to_string());
        zh.insert("settings.language".to_string(), "语言".to_string());
        zh.insert("settings.api_key".to_string(), "API密钥".to_string());

        let mut en = HashMap::new();
        en.insert("app.title".to_string(), "Mahakala Agent".to_string());
        en.insert("nav.chat".to_string(), "Chat".to_string());
        en.insert("nav.memory".to_string(), "Memory".to_string());
        en.insert("nav.skills".to_string(), "Skills".to_string());
        en.insert("nav.tools".to_string(), "Tools".to_string());
        en.insert("nav.plugins".to_string(), "Plugins".to_string());
        en.insert("nav.messages".to_string(), "Messages".to_string());
        en.insert("nav.platforms".to_string(), "Platforms".to_string());
        en.insert("nav.cron".to_string(), "Cron Jobs".to_string());
        en.insert("nav.settings".to_string(), "Settings".to_string());
        en.insert("common.save".to_string(), "Save".to_string());
        en.insert("common.cancel".to_string(), "Cancel".to_string());
        en.insert("common.delete".to_string(), "Delete".to_string());
        en.insert("common.edit".to_string(), "Edit".to_string());
        en.insert("common.add".to_string(), "Add".to_string());
        en.insert("common.search".to_string(), "Search".to_string());
        en.insert("common.loading".to_string(), "Loading...".to_string());
        en.insert("common.success".to_string(), "Success".to_string());
        en.insert("common.error".to_string(), "Error".to_string());
        en.insert("chat.placeholder".to_string(), "Type a message...".to_string());
        en.insert("chat.send".to_string(), "Send".to_string());
        en.insert("chat.thinking".to_string(), "Thinking...".to_string());
        en.insert("memory.facts".to_string(), "Facts".to_string());
        en.insert("memory.entities".to_string(), "Entities".to_string());
        en.insert("memory.dimensions".to_string(), "Dimensions".to_string());
        en.insert("settings.model".to_string(), "Model Settings".to_string());
        en.insert("settings.theme".to_string(), "Theme".to_string());
        en.insert("settings.language".to_string(), "Language".to_string());
        en.insert("settings.api_key".to_string(), "API Key".to_string());

        translations.insert("zh".to_string(), zh);
        translations.insert("en".to_string(), en);

        Self {
            translations: Arc::new(Mutex::new(translations)),
            current_locale: Arc::new(Mutex::new("zh".to_string())),
        }
    }

    pub fn set_locale(&self, locale: &str) -> Result<(), AppError> {
        let translations = self.translations.lock();
        if !translations.contains_key(locale) {
            return Err(AppError::InvalidInput(format!("Locale {} not supported", locale)));
        }
        let mut current = self.current_locale.lock();
        *current = locale.to_string();
        Ok(())
    }

    pub fn get_locale(&self) -> String {
        let current = self.current_locale.lock();
        current.clone()
    }

    pub fn t(&self, key: &str) -> String {
        let locale = self.get_locale();
        let translations = self.translations.lock();
        translations
            .get(&locale)
            .and_then(|map| map.get(key))
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    pub fn add_translation(&self, locale: &str, key: &str, value: &str) {
        let mut translations = self.translations.lock();
        translations
            .entry(locale.to_string())
            .or_default()
            .insert(key.to_string(), value.to_string());
    }

    pub fn get_config(&self) -> I18nConfig {
        let translations = self.translations.lock();
        I18nConfig {
            current_locale: self.get_locale(),
            available_locales: translations.keys().cloned().collect(),
        }
    }
}

#[derive(Clone)]
pub struct I18nHandle {
    inner: Arc<I18nManager>,
}

impl Default for I18nHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl I18nHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(I18nManager::new()),
        }
    }

    pub fn set_locale(&self, locale: &str) -> Result<(), AppError> {
        self.inner.set_locale(locale)
    }

    pub fn get_locale(&self) -> String {
        self.inner.get_locale()
    }

    pub fn t(&self, key: &str) -> String {
        self.inner.t(key)
    }

    pub fn add_translation(&self, locale: &str, key: &str, value: &str) {
        self.inner.add_translation(locale, key, value);
    }

    pub fn get_config(&self) -> I18nConfig {
        self.inner.get_config()
    }
}
