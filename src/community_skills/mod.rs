use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunitySkill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub category: String,
    pub tags: Vec<String>,
    pub code: String,
    pub dependencies: Vec<String>,
    pub readme: String,
    pub license: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub downloads: u64,
    pub rating: f64,
    pub rating_count: u64,
    pub is_published: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillReview {
    pub id: String,
    pub skill_id: String,
    pub user_id: String,
    pub rating: u8,
    pub comment: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDownload {
    pub id: String,
    pub skill_id: String,
    pub user_id: String,
    pub downloaded_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunitySkillConfig {
    pub max_skills_per_user: usize,
    pub enable_reviews: bool,
    pub enable_ratings: bool,
    pub require_approval: bool,
}

impl Default for CommunitySkillConfig {
    fn default() -> Self {
        Self {
            max_skills_per_user: 50,
            enable_reviews: true,
            enable_ratings: true,
            require_approval: false,
        }
    }
}

pub struct CommunitySkillCenter {
    skills: Arc<Mutex<HashMap<String, CommunitySkill>>>,
    reviews: Arc<Mutex<HashMap<String, Vec<SkillReview>>>>,
    downloads: Arc<Mutex<HashMap<String, Vec<SkillDownload>>>>,
    user_skills: Arc<Mutex<HashMap<String, Vec<String>>>>,
    config: CommunitySkillConfig,
}

impl CommunitySkillCenter {
    pub fn new(config: CommunitySkillConfig) -> Self {
        Self {
            skills: Arc::new(Mutex::new(HashMap::new())),
            reviews: Arc::new(Mutex::new(HashMap::new())),
            downloads: Arc::new(Mutex::new(HashMap::new())),
            user_skills: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn publish_skill(
        &self,
        user_id: String,
        name: String,
        description: String,
        version: String,
        category: String,
        tags: Vec<String>,
        code: String,
        dependencies: Vec<String>,
        readme: String,
        license: String,
    ) -> Result<String, AppError> {
        let user_skills = self.user_skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock user skills: {}", e)))?;
        let user_skill_count = user_skills.get(&user_id).map(|v| v.len()).unwrap_or(0);
        if user_skill_count >= self.config.max_skills_per_user {
            return Err(AppError::Internal(format!("Maximum skill limit ({}) reached for user", self.config.max_skills_per_user)));
        }
        drop(user_skills);

        let skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        for skill in skills.values() {
            if skill.name == name && skill.author == user_id {
                return Err(AppError::Internal(format!("Skill with name '{}' already exists", name)));
            }
        }
        drop(skills);

        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let skill = CommunitySkill {
            id: id.clone(),
            name,
            description,
            version,
            author: user_id.clone(),
            category,
            tags,
            code,
            dependencies,
            readme,
            license,
            created_at: now,
            updated_at: now,
            downloads: 0,
            rating: 0.0,
            rating_count: 0,
            is_published: !self.config.require_approval,
        };

        let mut skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        skills.insert(id.clone(), skill);

        let mut user_skills = self.user_skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock user skills: {}", e)))?;
        user_skills.entry(user_id).or_insert_with(Vec::new).push(id.clone());

        Ok(id)
    }

    pub fn get_skill(&self, skill_id: &str) -> Result<CommunitySkill, AppError> {
        let skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        skills.get(skill_id).cloned().ok_or_else(|| AppError::NotFound(format!("Skill {} not found", skill_id)))
    }

    pub fn list_skills(&self, filters: Option<HashMap<String, String>>) -> Vec<CommunitySkill> {
        let skills = self.skills.lock().unwrap();
        let mut result: Vec<CommunitySkill> = skills.values()
            .filter(|s| s.is_published)
            .cloned()
            .collect();

        if let Some(f) = filters {
            if let Some(category) = f.get("category") {
                result.retain(|s| s.category == *category);
            }
            if let Some(author) = f.get("author") {
                result.retain(|s| s.author == *author);
            }
            if let Some(tag) = f.get("tag") {
                result.retain(|s| s.tags.contains(tag));
            }
            if let Some(search) = f.get("search") {
                let search_lower = search.to_lowercase();
                result.retain(|s| 
                    s.name.to_lowercase().contains(&search_lower) ||
                    s.description.to_lowercase().contains(&search_lower) ||
                    s.tags.iter().any(|t| t.to_lowercase().contains(&search_lower))
                );
            }
        }

        result.sort_by(|a, b| b.downloads.cmp(&a.downloads));
        result
    }

    pub fn update_skill(
        &self,
        skill_id: &str,
        user_id: &str,
        name: Option<String>,
        description: Option<String>,
        version: Option<String>,
        code: Option<String>,
        dependencies: Option<Vec<String>>,
        readme: Option<String>,
    ) -> Result<(), AppError> {
        let mut skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        let skill = skills.get_mut(skill_id)
            .ok_or_else(|| AppError::NotFound(format!("Skill {} not found", skill_id)))?;

        if skill.author != user_id {
            return Err(AppError::Internal("Only the author can update the skill".to_string()));
        }

        if let Some(n) = name {
            skill.name = n;
        }
        if let Some(d) = description {
            skill.description = d;
        }
        if let Some(v) = version {
            skill.version = v;
        }
        if let Some(c) = code {
            skill.code = c;
        }
        if let Some(deps) = dependencies {
            skill.dependencies = deps;
        }
        if let Some(r) = readme {
            skill.readme = r;
        }
        skill.updated_at = chrono::Utc::now().timestamp();

        Ok(())
    }

    pub fn delete_skill(&self, skill_id: &str, user_id: &str) -> Result<(), AppError> {
        let mut skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        let skill = skills.get(skill_id)
            .ok_or_else(|| AppError::NotFound(format!("Skill {} not found", skill_id)))?;

        if skill.author != user_id {
            return Err(AppError::Internal("Only the author can delete the skill".to_string()));
        }

        skills.remove(skill_id);

        let mut user_skills = self.user_skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock user skills: {}", e)))?;
        if let Some(user_skill_list) = user_skills.get_mut(user_id) {
            user_skill_list.retain(|id| id != skill_id);
        }

        Ok(())
    }

    pub fn download_skill(&self, skill_id: &str, user_id: &str) -> Result<String, AppError> {
        let mut skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        let skill = skills.get_mut(skill_id)
            .ok_or_else(|| AppError::NotFound(format!("Skill {} not found", skill_id)))?;

        if !skill.is_published {
            return Err(AppError::Internal("Skill is not published".to_string()));
        }

        skill.downloads += 1;
        drop(skills);

        let id = uuid::Uuid::new_v4().to_string();
        let download = SkillDownload {
            id: id.clone(),
            skill_id: skill_id.to_string(),
            user_id: user_id.to_string(),
            downloaded_at: chrono::Utc::now().timestamp(),
        };

        let mut downloads = self.downloads.lock().map_err(|e| AppError::Internal(format!("Failed to lock downloads: {}", e)))?;
        downloads.entry(skill_id.to_string()).or_insert_with(Vec::new).push(download);

        Ok(id)
    }

    pub fn add_review(
        &self,
        skill_id: &str,
        user_id: &str,
        rating: u8,
        comment: String,
    ) -> Result<String, AppError> {
        if !self.config.enable_reviews {
            return Err(AppError::Internal("Reviews are disabled".to_string()));
        }

        if rating < 1 || rating > 5 {
            return Err(AppError::Internal("Rating must be between 1 and 5".to_string()));
        }

        let skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        if !skills.contains_key(skill_id) {
            return Err(AppError::NotFound(format!("Skill {} not found", skill_id)));
        }
        drop(skills);

        let id = uuid::Uuid::new_v4().to_string();
        let review = SkillReview {
            id: id.clone(),
            skill_id: skill_id.to_string(),
            user_id: user_id.to_string(),
            rating,
            comment,
            created_at: chrono::Utc::now().timestamp(),
        };

        {
            let mut reviews = self.reviews.lock().map_err(|e| AppError::Internal(format!("Failed to lock reviews: {}", e)))?;
            reviews.entry(skill_id.to_string()).or_insert_with(Vec::new).push(review);
        }

        if self.config.enable_ratings {
            self.update_skill_rating(skill_id)?;
        }

        Ok(id)
    }

    fn update_skill_rating(&self, skill_id: &str) -> Result<(), AppError> {
        let reviews = self.reviews.lock().map_err(|e| AppError::Internal(format!("Failed to lock reviews: {}", e)))?;
        let skill_reviews = reviews.get(skill_id).cloned().unwrap_or_default();
        drop(reviews);

        if skill_reviews.is_empty() {
            return Ok(());
        }

        let total_rating: u64 = skill_reviews.iter().map(|r| r.rating as u64).sum();
        let rating_count = skill_reviews.len() as u64;
        let avg_rating = total_rating as f64 / rating_count as f64;

        let mut skills = self.skills.lock().map_err(|e| AppError::Internal(format!("Failed to lock skills: {}", e)))?;
        if let Some(skill) = skills.get_mut(skill_id) {
            skill.rating = avg_rating;
            skill.rating_count = rating_count;
        }

        Ok(())
    }

    pub fn get_reviews(&self, skill_id: &str) -> Vec<SkillReview> {
        let reviews = self.reviews.lock().unwrap();
        reviews.get(skill_id).cloned().unwrap_or_default()
    }

    pub fn get_user_skills(&self, user_id: &str) -> Vec<CommunitySkill> {
        let user_skills = self.user_skills.lock().unwrap();
        let skills = self.skills.lock().unwrap();
        
        user_skills.get(user_id)
            .map(|skill_ids| {
                skill_ids.iter()
                    .filter_map(|id| skills.get(id))
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_categories(&self) -> Vec<String> {
        let skills = self.skills.lock().unwrap();
        let mut categories: Vec<String> = skills.values()
            .map(|s| s.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        categories.sort();
        categories
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let skills = self.skills.lock().unwrap();
        let reviews = self.reviews.lock().unwrap();
        let _downloads = self.downloads.lock().unwrap();

        let total_downloads: u64 = skills.values().map(|s| s.downloads).sum();
        let total_reviews: usize = reviews.values().map(|v| v.len()).sum();
        let total_skills = skills.len();
        let published_skills = skills.values().filter(|s| s.is_published).count();

        let category_stats: HashMap<String, usize> = skills.values()
            .fold(HashMap::new(), |mut acc, s| {
                *acc.entry(s.category.clone()).or_insert(0) += 1;
                acc
            });

        serde_json::json!({
            "total_skills": total_skills,
            "published_skills": published_skills,
            "total_downloads": total_downloads,
            "total_reviews": total_reviews,
            "categories": category_stats
        })
    }

    pub fn install_skill(&self, skill_id: &str, user_id: &str) -> Result<String, AppError> {
        self.download_skill(skill_id, user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_center() -> CommunitySkillCenter {
        CommunitySkillCenter::new(CommunitySkillConfig::default())
    }

    #[test]
    fn test_publish_skill() {
        let center = create_test_center();
        let result = center.publish_skill(
            "user1".to_string(),
            "test-skill".to_string(),
            "A test skill".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["test".to_string()],
            "print('hello')".to_string(),
            vec![],
            "# Test Skill".to_string(),
            "MIT".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_skills() {
        let center = create_test_center();
        let _ = center.publish_skill(
            "user1".to_string(),
            "skill1".to_string(),
            "Skill 1".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["test".to_string()],
            "code1".to_string(),
            vec![],
            "readme1".to_string(),
            "MIT".to_string(),
        );
        let _ = center.publish_skill(
            "user2".to_string(),
            "skill2".to_string(),
            "Skill 2".to_string(),
            "1.0.0".to_string(),
            "ai".to_string(),
            vec!["ml".to_string()],
            "code2".to_string(),
            vec![],
            "readme2".to_string(),
            "MIT".to_string(),
        );

        let skills = center.list_skills(None);
        assert_eq!(skills.len(), 2);

        let mut filters = HashMap::new();
        filters.insert("category".to_string(), "utilities".to_string());
        let filtered = center.list_skills(Some(filters));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].category, "utilities");
    }

    #[test]
    fn test_download_skill() {
        let center = create_test_center();
        let skill_id = center.publish_skill(
            "user1".to_string(),
            "test-skill".to_string(),
            "A test skill".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["test".to_string()],
            "code".to_string(),
            vec![],
            "readme".to_string(),
            "MIT".to_string(),
        ).unwrap();

        let result = center.download_skill(&skill_id, "user2");
        assert!(result.is_ok());

        let skill = center.get_skill(&skill_id).unwrap();
        assert_eq!(skill.downloads, 1);
    }

    #[test]
    fn test_add_review() {
        let center = create_test_center();
        let skill_id = center.publish_skill(
            "user1".to_string(),
            "test-skill".to_string(),
            "A test skill".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["test".to_string()],
            "code".to_string(),
            vec![],
            "readme".to_string(),
            "MIT".to_string(),
        ).unwrap();

        let review_id = center.add_review(&skill_id, "user2", 5, "Great skill!".to_string()).unwrap();
        assert!(!review_id.is_empty());

        let reviews = center.get_reviews(&skill_id);
        assert_eq!(reviews.len(), 1);
        assert_eq!(reviews[0].rating, 5);

        let skill = center.get_skill(&skill_id).unwrap();
        assert_eq!(skill.rating, 5.0);
        assert_eq!(skill.rating_count, 1);
    }

    #[test]
    fn test_update_skill() {
        let center = create_test_center();
        let skill_id = center.publish_skill(
            "user1".to_string(),
            "test-skill".to_string(),
            "A test skill".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["test".to_string()],
            "code".to_string(),
            vec![],
            "readme".to_string(),
            "MIT".to_string(),
        ).unwrap();

        let result = center.update_skill(
            &skill_id,
            "user1",
            Some("updated-skill".to_string()),
            Some("Updated description".to_string()),
            Some("2.0.0".to_string()),
            None,
            None,
            None,
        );
        assert!(result.is_ok());

        let skill = center.get_skill(&skill_id).unwrap();
        assert_eq!(skill.name, "updated-skill");
        assert_eq!(skill.version, "2.0.0");
    }

    #[test]
    fn test_delete_skill() {
        let center = create_test_center();
        let skill_id = center.publish_skill(
            "user1".to_string(),
            "test-skill".to_string(),
            "A test skill".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["test".to_string()],
            "code".to_string(),
            vec![],
            "readme".to_string(),
            "MIT".to_string(),
        ).unwrap();

        let result = center.delete_skill(&skill_id, "user1");
        assert!(result.is_ok());

        let skills = center.list_skills(None);
        assert!(skills.is_empty());
    }

    #[test]
    fn test_get_user_skills() {
        let center = create_test_center();
        let _ = center.publish_skill(
            "user1".to_string(),
            "skill1".to_string(),
            "Skill 1".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec![],
            "code1".to_string(),
            vec![],
            "readme1".to_string(),
            "MIT".to_string(),
        );
        let _ = center.publish_skill(
            "user1".to_string(),
            "skill2".to_string(),
            "Skill 2".to_string(),
            "1.0.0".to_string(),
            "ai".to_string(),
            vec![],
            "code2".to_string(),
            vec![],
            "readme2".to_string(),
            "MIT".to_string(),
        );

        let user_skills = center.get_user_skills("user1");
        assert_eq!(user_skills.len(), 2);
    }

    #[test]
    fn test_get_categories() {
        let center = create_test_center();
        let _ = center.publish_skill(
            "user1".to_string(),
            "skill1".to_string(),
            "Skill 1".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec![],
            "code1".to_string(),
            vec![],
            "readme1".to_string(),
            "MIT".to_string(),
        );
        let _ = center.publish_skill(
            "user2".to_string(),
            "skill2".to_string(),
            "Skill 2".to_string(),
            "1.0.0".to_string(),
            "ai".to_string(),
            vec![],
            "code2".to_string(),
            vec![],
            "readme2".to_string(),
            "MIT".to_string(),
        );

        let categories = center.get_categories();
        assert_eq!(categories.len(), 2);
        assert!(categories.contains(&"utilities".to_string()));
        assert!(categories.contains(&"ai".to_string()));
    }

    #[test]
    fn test_get_stats() {
        let center = create_test_center();
        let skill_id = center.publish_skill(
            "user1".to_string(),
            "test-skill".to_string(),
            "A test skill".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec![],
            "code".to_string(),
            vec![],
            "readme".to_string(),
            "MIT".to_string(),
        ).unwrap();

        let _ = center.download_skill(&skill_id, "user2");
        let _ = center.add_review(&skill_id, "user2", 4, "Good".to_string());

        let stats = center.get_stats();
        assert_eq!(stats["total_skills"].as_u64().unwrap(), 1);
        assert_eq!(stats["published_skills"].as_u64().unwrap(), 1);
        assert_eq!(stats["total_downloads"].as_u64().unwrap(), 1);
        assert_eq!(stats["total_reviews"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_search_skills() {
        let center = create_test_center();
        let _ = center.publish_skill(
            "user1".to_string(),
            "python-helper".to_string(),
            "A Python helper skill".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["python".to_string(), "helper".to_string()],
            "code1".to_string(),
            vec![],
            "readme1".to_string(),
            "MIT".to_string(),
        );
        let _ = center.publish_skill(
            "user2".to_string(),
            "javascript-tool".to_string(),
            "A JavaScript tool".to_string(),
            "1.0.0".to_string(),
            "utilities".to_string(),
            vec!["javascript".to_string()],
            "code2".to_string(),
            vec![],
            "readme2".to_string(),
            "MIT".to_string(),
        );

        let mut filters = HashMap::new();
        filters.insert("search".to_string(), "python".to_string());
        let results = center.list_skills(Some(filters));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "python-helper");
    }
}
