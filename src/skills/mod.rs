use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub installed: bool,
    pub enabled: bool,
    pub version: String,
    pub config: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillCreate {
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

pub struct SkillRegistry {
    skills: Arc<Mutex<HashMap<String, Skill>>>,
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillRegistry {
    pub fn new() -> Self {
        let mut skills = HashMap::new();

        let default_skills = vec![
            ("creative_writing", "Creative Writing", "AI creative writing skills", "creative"),
            ("code_review", "Code Review", "Automated code review", "devops"),
            ("web_research", "Web Research", "Deep web research assistant", "research"),
            ("document_summary", "Document Summary", "Summarize documents", "productivity"),
            ("data_analysis", "Data Analysis", "Data analysis and visualization", "research"),
            ("email_assistant", "Email Assistant", "Email drafting and management", "productivity"),
            ("ci_cd_pipeline", "CI/CD Pipeline", "CI/CD automation skills", "devops"),
            ("image_generation", "Image Generation", "AI image generation skills", "creative"),
        ];

        let now = chrono::Utc::now().timestamp();
        for (id, name, desc, category) in default_skills {
            skills.insert(id.to_string(), Skill {
                id: id.to_string(),
                name: name.to_string(),
                description: desc.to_string(),
                category: category.to_string(),
                installed: false,
                enabled: true,
                version: "1.0.0".to_string(),
                config: serde_json::Value::Null,
                created_at: now,
            });
        }

        Self {
            skills: Arc::new(Mutex::new(skills)),
        }
    }

    pub fn install(&self, skill_id: &str) -> Result<Skill, AppError> {
        let mut skills = self.skills.lock();
        if let Some(skill) = skills.get_mut(skill_id) {
            skill.installed = true;
            skill.enabled = true;
            Ok(skill.clone())
        } else {
            Err(AppError::NotFound(format!("Skill {} not found", skill_id)))
        }
    }

    pub fn uninstall(&self, skill_id: &str) -> Result<Skill, AppError> {
        let mut skills = self.skills.lock();
        if let Some(skill) = skills.get_mut(skill_id) {
            skill.installed = false;
            skill.enabled = false;
            Ok(skill.clone())
        } else {
            Err(AppError::NotFound(format!("Skill {} not found", skill_id)))
        }
    }

    pub fn enable(&self, skill_id: &str) -> Result<Skill, AppError> {
        let mut skills = self.skills.lock();
        if let Some(skill) = skills.get_mut(skill_id) {
            if !skill.installed {
                return Err(AppError::InvalidInput(format!("Skill {} is not installed", skill_id)));
            }
            skill.enabled = true;
            Ok(skill.clone())
        } else {
            Err(AppError::NotFound(format!("Skill {} not found", skill_id)))
        }
    }

    pub fn disable(&self, skill_id: &str) -> Result<Skill, AppError> {
        let mut skills = self.skills.lock();
        if let Some(skill) = skills.get_mut(skill_id) {
            skill.enabled = false;
            Ok(skill.clone())
        } else {
            Err(AppError::NotFound(format!("Skill {} not found", skill_id)))
        }
    }

    pub fn get(&self, skill_id: &str) -> Option<Skill> {
        let skills = self.skills.lock();
        skills.get(skill_id).cloned()
    }

    pub fn list(&self) -> Vec<Skill> {
        let skills = self.skills.lock();
        skills.values().cloned().collect()
    }

    pub fn list_by_category(&self, category: &str) -> Vec<Skill> {
        let skills = self.skills.lock();
        skills.values()
            .filter(|s| s.category == category)
            .cloned()
            .collect()
    }

    pub fn list_installed(&self) -> Vec<Skill> {
        let skills = self.skills.lock();
        skills.values()
            .filter(|s| s.installed)
            .cloned()
            .collect()
    }

    pub fn list_enabled(&self) -> Vec<Skill> {
        let skills = self.skills.lock();
        skills.values()
            .filter(|s| s.installed && s.enabled)
            .cloned()
            .collect()
    }

    pub fn add_custom_skill(&self, create: SkillCreate) -> Result<Skill, AppError> {
        let now = chrono::Utc::now().timestamp();
        let id = create.name.to_lowercase().replace(" ", "_");
        let skill = Skill {
            id: id.clone(),
            name: create.name,
            description: create.description.unwrap_or_default(),
            category: create.category.unwrap_or_else(|| "general".to_string()),
            installed: true,
            enabled: true,
            version: "1.0.0".to_string(),
            config: serde_json::Value::Null,
            created_at: now,
        };

        let mut skills = self.skills.lock();
        skills.insert(id, skill.clone());
        Ok(skill)
    }
}

#[derive(Clone)]
pub struct SkillManager {
    registry: Arc<SkillRegistry>,
}

impl Default for SkillManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SkillManager {
    pub fn new() -> Self {
        Self {
            registry: Arc::new(SkillRegistry::new()),
        }
    }

    pub fn install(&self, skill_id: &str) -> Result<Skill, AppError> {
        self.registry.install(skill_id)
    }

    pub fn uninstall(&self, skill_id: &str) -> Result<Skill, AppError> {
        self.registry.uninstall(skill_id)
    }

    pub fn enable(&self, skill_id: &str) -> Result<Skill, AppError> {
        self.registry.enable(skill_id)
    }

    pub fn disable(&self, skill_id: &str) -> Result<Skill, AppError> {
        self.registry.disable(skill_id)
    }

    pub fn get(&self, skill_id: &str) -> Option<Skill> {
        self.registry.get(skill_id)
    }

    pub fn list(&self) -> Vec<Skill> {
        self.registry.list()
    }

    pub fn list_by_category(&self, category: &str) -> Vec<Skill> {
        self.registry.list_by_category(category)
    }

    pub fn list_installed(&self) -> Vec<Skill> {
        self.registry.list_installed()
    }

    pub fn list_enabled(&self) -> Vec<Skill> {
        self.registry.list_enabled()
    }

    pub fn add_custom_skill(&self, create: SkillCreate) -> Result<Skill, AppError> {
        self.registry.add_custom_skill(create)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_skill_install() {
        let manager = SkillManager::new();
        let result = manager.install("creative_writing");
        assert!(result.is_ok(), "Skill install should succeed");
        let skill = result.unwrap();
        assert!(skill.installed, "Skill should be installed");
    }

    #[test]
    fn test_skill_enable_disable() {
        let manager = SkillManager::new();
        manager.install("code_review").unwrap();
        
        let result = manager.disable("code_review");
        assert!(result.is_ok(), "Skill disable should succeed");
        assert!(!result.unwrap().enabled, "Skill should be disabled");
        
        let result = manager.enable("code_review");
        assert!(result.is_ok(), "Skill enable should succeed");
        assert!(result.unwrap().enabled, "Skill should be enabled");
    }

    #[test]
    fn test_skill_list() {
        let manager = SkillManager::new();
        let skills = manager.list();
        assert!(!skills.is_empty(), "Should have default skills");
    }

    #[test]
    fn test_skill_categories() {
        let manager = SkillManager::new();
        let creative = manager.list_by_category("creative");
        assert!(!creative.is_empty(), "Should have creative skills");
    }
}
