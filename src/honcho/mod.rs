use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
    pub interaction_count: usize,
    pub preferences: HashMap<String, String>,
    pub personality_traits: HashMap<String, f64>,
    pub knowledge_level: HashMap<String, f64>,
    pub communication_style: String,
    pub goals: Vec<String>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialecticalState {
    pub user_id: String,
    pub thesis: String,
    pub antithesis: String,
    pub synthesis: String,
    pub current_position: String,
    pub belief_strength: HashMap<String, f64>,
    pub cognitive_biases: Vec<String>,
    pub emotional_state: String,
    pub last_updated: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteractionRecord {
    pub id: String,
    pub user_id: String,
    pub timestamp: i64,
    pub input: String,
    pub response: String,
    pub sentiment: f64,
    pub complexity: f64,
    pub topics: Vec<String>,
    pub outcomes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserModel {
    pub profile: UserProfile,
    pub dialectical_state: DialecticalState,
    pub interaction_history: Vec<InteractionRecord>,
    pub learning_progress: HashMap<String, f64>,
    pub engagement_score: f64,
    pub trust_level: f64,
}

pub struct HonchoModeling {
    users: Arc<Mutex<HashMap<String, UserModel>>>,
    interactions: Arc<Mutex<Vec<InteractionRecord>>>,
    #[allow(dead_code)]
    analytics: Arc<Mutex<HashMap<String, serde_json::Value>>>,
    config: HonchoConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HonchoConfig {
    pub max_interactions_per_user: usize,
    pub enable_dialectical_analysis: bool,
    pub enable_sentiment_tracking: bool,
    pub enable_learning_progress: bool,
    pub personality_dimensions: Vec<String>,
    pub knowledge_domains: Vec<String>,
}

impl Default for HonchoConfig {
    fn default() -> Self {
        Self {
            max_interactions_per_user: 10000,
            enable_dialectical_analysis: true,
            enable_sentiment_tracking: true,
            enable_learning_progress: true,
            personality_dimensions: vec![
                "openness".to_string(),
                "conscientiousness".to_string(),
                "extraversion".to_string(),
                "agreeableness".to_string(),
                "neuroticism".to_string(),
            ],
            knowledge_domains: vec![
                "technical".to_string(),
                "creative".to_string(),
                "analytical".to_string(),
                "social".to_string(),
            ],
        }
    }
}

impl HonchoModeling {
    pub fn new(config: HonchoConfig) -> Self {
        Self {
            users: Arc::new(Mutex::new(HashMap::new())),
            interactions: Arc::new(Mutex::new(Vec::new())),
            analytics: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn create_user(&self, user_id: String, name: String) -> Result<String, AppError> {
        let now = chrono::Utc::now().timestamp();
        
        let mut personality_traits = HashMap::new();
        for dimension in &self.config.personality_dimensions {
            personality_traits.insert(dimension.clone(), 0.5);
        }

        let mut knowledge_level = HashMap::new();
        for domain in &self.config.knowledge_domains {
            knowledge_level.insert(domain.clone(), 0.0);
        }

        let profile = UserProfile {
            user_id: user_id.clone(),
            name,
            created_at: now,
            updated_at: now,
            interaction_count: 0,
            preferences: HashMap::new(),
            personality_traits,
            knowledge_level,
            communication_style: "neutral".to_string(),
            goals: Vec::new(),
            constraints: Vec::new(),
        };

        let dialectical_state = DialecticalState {
            user_id: user_id.clone(),
            thesis: String::new(),
            antithesis: String::new(),
            synthesis: String::new(),
            current_position: String::new(),
            belief_strength: HashMap::new(),
            cognitive_biases: Vec::new(),
            emotional_state: "neutral".to_string(),
            last_updated: now,
        };

        let user_model = UserModel {
            profile,
            dialectical_state,
            interaction_history: Vec::new(),
            learning_progress: HashMap::new(),
            engagement_score: 0.5,
            trust_level: 0.5,
        };

        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        users.insert(user_id.clone(), user_model);

        Ok(user_id)
    }

    pub fn get_user(&self, user_id: &str) -> Result<UserModel, AppError> {
        let users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        users.get(user_id).cloned().ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))
    }

    pub fn list_users(&self) -> Vec<UserProfile> {
        let users = self.users.lock().unwrap();
        users.values().map(|u| u.profile.clone()).collect()
    }

    pub fn record_interaction(
        &self,
        user_id: &str,
        input: String,
        response: String,
        topics: Vec<String>,
    ) -> Result<String, AppError> {
        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        let user_model = users.get_mut(user_id)
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;

        if user_model.interaction_history.len() >= self.config.max_interactions_per_user {
            return Err(AppError::Internal("Maximum interaction limit reached".to_string()));
        }

        let sentiment = self.analyze_sentiment(&input);
        let complexity = self.estimate_complexity(&input);

        let now = chrono::Utc::now().timestamp();
        let interaction = InteractionRecord {
            id: uuid::Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            timestamp: now,
            input: input.clone(),
            response: response.clone(),
            sentiment,
            complexity,
            topics: topics.clone(),
            outcomes: Vec::new(),
        };

        user_model.interaction_history.push(interaction.clone());
        user_model.profile.interaction_count += 1;
        user_model.profile.updated_at = now;

        self.update_user_model(user_model, &input, &response, &topics)?;

        drop(users);

        let mut interactions = self.interactions.lock().map_err(|e| AppError::Internal(format!("Failed to lock interactions: {}", e)))?;
        interactions.push(interaction.clone());

        Ok(interaction.id)
    }

    fn analyze_sentiment(&self, text: &str) -> f64 {
        let positive_words = ["good", "great", "excellent", "happy", "love", "thanks", "perfect", "wonderful"];
        let negative_words = ["bad", "terrible", "awful", "hate", "angry", "worst", "horrible", "disappointing"];
        
        let text_lower = text.to_lowercase();
        let mut score = 0.0;
        let mut count = 0;

        for word in &positive_words {
            if text_lower.contains(word) {
                score += 1.0;
                count += 1;
            }
        }

        for word in &negative_words {
            if text_lower.contains(word) {
                score -= 1.0;
                count += 1;
            }
        }

        if count > 0 { score / count as f64 } else { 0.0 }
    }

    fn estimate_complexity(&self, text: &str) -> f64 {
        let word_count = text.split_whitespace().count();
        let unique_words = text.split_whitespace().collect::<std::collections::HashSet<_>>().len();
        let _avg_word_length = if word_count > 0 { text.len() as f64 / word_count as f64 } else { 0.0 };
        
        let lexical_diversity = if word_count > 0 { unique_words as f64 / word_count as f64 } else { 0.0 };
        let length_factor = (word_count as f64 / 100.0).min(1.0);
        
        (lexical_diversity * 0.6 + length_factor * 0.4).min(1.0)
    }

    fn update_user_model(
        &self,
        user_model: &mut UserModel,
        input: &str,
        response: &str,
        topics: &[String],
    ) -> Result<(), AppError> {
        if self.config.enable_sentiment_tracking {
            let recent_sentiments: Vec<f64> = user_model.interaction_history
                .iter()
                .rev()
                .take(10)
                .map(|i| i.sentiment)
                .collect();
            
            if !recent_sentiments.is_empty() {
                let avg_sentiment = recent_sentiments.iter().sum::<f64>() / recent_sentiments.len() as f64;
                user_model.engagement_score = (user_model.engagement_score * 0.7 + avg_sentiment * 0.3).clamp(0.0, 1.0);
            }
        }

        if self.config.enable_dialectical_analysis {
            self.update_dialectical_state(&mut user_model.dialectical_state, input, response)?;
        }

        if self.config.enable_learning_progress {
            for topic in topics {
                let progress = user_model.learning_progress.entry(topic.clone()).or_insert(0.0);
                *progress = (*progress + 0.01).min(1.0);
                
                if let Some(kl) = user_model.profile.knowledge_level.get_mut(topic) {
                    *kl = (*kl + 0.005).min(1.0);
                }
            }
        }

        let input_lower = input.to_lowercase();
        if input_lower.contains("please") || input_lower.contains("thank") {
            user_model.profile.communication_style = "polite".to_string();
        } else if input.contains("?") {
            user_model.profile.communication_style = "inquisitive".to_string();
        } else if input.len() > 200 {
            user_model.profile.communication_style = "detailed".to_string();
        }

        Ok(())
    }

    fn update_dialectical_state(
        &self,
        state: &mut DialecticalState,
        input: &str,
        _response: &str,
    ) -> Result<(), AppError> {
        if state.thesis.is_empty() {
            state.thesis = input.to_string();
        } else if state.antithesis.is_empty() && input != state.thesis {
            state.antithesis = input.to_string();
        } else if !state.antithesis.is_empty() && state.synthesis.is_empty() {
            state.synthesis = format!("Synthesis of: {} and {}", state.thesis, state.antithesis);
            state.current_position = state.synthesis.clone();
        }

        state.last_updated = chrono::Utc::now().timestamp();
        Ok(())
    }

    pub fn get_user_profile(&self, user_id: &str) -> Result<UserProfile, AppError> {
        let user = self.get_user(user_id)?;
        Ok(user.profile)
    }

    pub fn get_dialectical_state(&self, user_id: &str) -> Result<DialecticalState, AppError> {
        let user = self.get_user(user_id)?;
        Ok(user.dialectical_state)
    }

    pub fn get_interaction_history(&self, user_id: &str, limit: usize) -> Vec<InteractionRecord> {
        let users = self.users.lock().unwrap();
        if let Some(user) = users.get(user_id) {
            let start = user.interaction_history.len().saturating_sub(limit);
            user.interaction_history[start..].to_vec()
        } else {
            Vec::new()
        }
    }

    pub fn update_user_preference(&self, user_id: &str, key: String, value: String) -> Result<(), AppError> {
        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        let user_model = users.get_mut(user_id)
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        
        user_model.profile.preferences.insert(key, value);
        user_model.profile.updated_at = chrono::Utc::now().timestamp();
        Ok(())
    }

    pub fn add_user_goal(&self, user_id: &str, goal: String) -> Result<(), AppError> {
        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        let user_model = users.get_mut(user_id)
            .ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        
        if !user_model.profile.goals.contains(&goal) {
            user_model.profile.goals.push(goal);
        }
        Ok(())
    }

    pub fn get_user_analytics(&self, user_id: &str) -> Result<serde_json::Value, AppError> {
        let user = self.get_user(user_id)?;
        
        let avg_sentiment = if !user.interaction_history.is_empty() {
            user.interaction_history.iter().map(|i| i.sentiment).sum::<f64>() / user.interaction_history.len() as f64
        } else {
            0.0
        };

        let avg_complexity = if !user.interaction_history.is_empty() {
            user.interaction_history.iter().map(|i| i.complexity).sum::<f64>() / user.interaction_history.len() as f64
        } else {
            0.0
        };

        let top_topics = {
            let mut topic_counts: HashMap<String, usize> = HashMap::new();
            for interaction in &user.interaction_history {
                for topic in &interaction.topics {
                    *topic_counts.entry(topic.clone()).or_insert(0) += 1;
                }
            }
            let mut sorted: Vec<_> = topic_counts.into_iter().collect();
            sorted.sort_by(|a, b| b.1.cmp(&a.1));
            sorted.into_iter().take(5).map(|(t, _)| t).collect::<Vec<_>>()
        };

        Ok(serde_json::json!({
            "user_id": user_id,
            "total_interactions": user.profile.interaction_count,
            "avg_sentiment": avg_sentiment,
            "avg_complexity": avg_complexity,
            "engagement_score": user.engagement_score,
            "trust_level": user.trust_level,
            "top_topics": top_topics,
            "learning_progress": user.learning_progress,
        }))
    }

    pub fn delete_user(&self, user_id: &str) -> Result<(), AppError> {
        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        users.remove(user_id).ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        Ok(())
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let users = self.users.lock().unwrap();
        let interactions = self.interactions.lock().unwrap();

        let total_interactions: usize = users.values().map(|u| u.profile.interaction_count).sum();
        let avg_engagement = if !users.is_empty() {
            users.values().map(|u| u.engagement_score).sum::<f64>() / users.len() as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_users": users.len(),
            "total_interactions": total_interactions,
            "avg_engagement": avg_engagement,
            "interaction_history_length": interactions.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_modeling() -> HonchoModeling {
        HonchoModeling::new(HonchoConfig::default())
    }

    #[test]
    fn test_create_user() {
        let modeling = create_test_modeling();
        let result = modeling.create_user("user1".to_string(), "Test User".to_string());
        assert!(result.is_ok());
        
        let user = modeling.get_user("user1").unwrap();
        assert_eq!(user.profile.name, "Test User");
        assert_eq!(user.profile.interaction_count, 0);
    }

    #[test]
    fn test_list_users() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "User 1".to_string());
        let _ = modeling.create_user("user2".to_string(), "User 2".to_string());

        let users = modeling.list_users();
        assert_eq!(users.len(), 2);
    }

    #[test]
    fn test_record_interaction() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());

        let result = modeling.record_interaction(
            "user1",
            "Hello, how are you?".to_string(),
            "I'm doing well, thank you!".to_string(),
            vec!["greeting".to_string()],
        );
        assert!(result.is_ok());

        let user = modeling.get_user("user1").unwrap();
        assert_eq!(user.profile.interaction_count, 1);
        assert_eq!(user.interaction_history.len(), 1);
    }

    #[test]
    fn test_get_user_profile() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());

        let profile = modeling.get_user_profile("user1").unwrap();
        assert_eq!(profile.name, "Test User");
        assert!(!profile.personality_traits.is_empty());
    }

    #[test]
    fn test_update_user_preference() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());

        let result = modeling.update_user_preference("user1", "theme".to_string(), "dark".to_string());
        assert!(result.is_ok());

        let user = modeling.get_user("user1").unwrap();
        assert_eq!(user.profile.preferences.get("theme"), Some(&"dark".to_string()));
    }

    #[test]
    fn test_add_user_goal() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());

        let result = modeling.add_user_goal("user1", "Learn Rust".to_string());
        assert!(result.is_ok());

        let user = modeling.get_user("user1").unwrap();
        assert!(user.profile.goals.contains(&"Learn Rust".to_string()));
    }

    #[test]
    fn test_get_user_analytics() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());
        
        let _ = modeling.record_interaction(
            "user1",
            "Hello world".to_string(),
            "Hi there!".to_string(),
            vec!["greeting".to_string()],
        );

        let analytics = modeling.get_user_analytics("user1").unwrap();
        assert_eq!(analytics["total_interactions"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_delete_user() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());

        let result = modeling.delete_user("user1");
        assert!(result.is_ok());

        let result = modeling.get_user("user1");
        assert!(result.is_err());
    }

    #[test]
    fn test_get_stats() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());
        let _ = modeling.create_user("user2".to_string(), "Another User".to_string());

        let stats = modeling.get_stats();
        assert_eq!(stats["total_users"].as_u64().unwrap(), 2);
    }

    #[test]
    fn test_dialectical_state_updates() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());

        let _ = modeling.record_interaction(
            "user1",
            "I believe X is true".to_string(),
            "Interesting perspective".to_string(),
            vec!["philosophy".to_string()],
        );

        let _ = modeling.record_interaction(
            "user1",
            "But Y might also be valid".to_string(),
            "Good point".to_string(),
            vec!["philosophy".to_string()],
        );

        let state = modeling.get_dialectical_state("user1").unwrap();
        assert!(!state.thesis.is_empty());
        assert!(!state.antithesis.is_empty());
    }

    #[test]
    fn test_multiple_interactions() {
        let modeling = create_test_modeling();
        let _ = modeling.create_user("user1".to_string(), "Test User".to_string());

        for i in 0..5 {
            let _ = modeling.record_interaction(
                "user1",
                format!("Message {}", i),
                format!("Response {}", i),
                vec![format!("topic-{}", i)],
            );
        }

        let user = modeling.get_user("user1").unwrap();
        assert_eq!(user.profile.interaction_count, 5);
        assert_eq!(user.interaction_history.len(), 5);

        let history = modeling.get_interaction_history("user1", 3);
        assert_eq!(history.len(), 3);
    }
}
