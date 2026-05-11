use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPrompt {
    pub id: String,
    pub user_id: String,
    pub prompt_type: String,
    pub content: String,
    pub context: HashMap<String, String>,
    pub priority: u8,
    pub created_at: i64,
    pub scheduled_at: i64,
    pub triggered_at: Option<i64>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSchedule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub cron_expression: String,
    pub prompt_type: String,
    pub target_users: Vec<String>,
    pub is_active: bool,
    pub last_triggered: Option<i64>,
    pub trigger_count: usize,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptResponse {
    pub id: String,
    pub prompt_id: String,
    pub user_id: String,
    pub response: String,
    pub sentiment: f64,
    pub timestamp: i64,
    pub follow_up_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryPromptConfig {
    pub user_id: String,
    pub prompt_frequency_hours: u64,
    pub preferred_prompt_types: Vec<String>,
    pub max_prompts_per_day: usize,
    pub quiet_hours_start: u8,
    pub quiet_hours_end: u8,
    pub enable_adaptive_scheduling: bool,
}

pub struct MemoryPromptingSystem {
    prompts: Arc<Mutex<HashMap<String, MemoryPrompt>>>,
    schedules: Arc<Mutex<HashMap<String, PromptSchedule>>>,
    responses: Arc<Mutex<HashMap<String, Vec<PromptResponse>>>>,
    user_configs: Arc<Mutex<HashMap<String, MemoryPromptConfig>>>,
    counters: Arc<Mutex<HashMap<String, usize>>>,
}

impl Default for MemoryPromptingSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryPromptingSystem {
    pub fn new() -> Self {
        Self {
            prompts: Arc::new(Mutex::new(HashMap::new())),
            schedules: Arc::new(Mutex::new(HashMap::new())),
            responses: Arc::new(Mutex::new(HashMap::new())),
            user_configs: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_schedule(&self, schedule: PromptSchedule) -> Result<String, AppError> {
        let id = schedule.id.clone();
        let mut schedules = self.schedules.lock().map_err(|e| AppError::Internal(format!("Failed to lock schedules: {}", e)))?;
        schedules.insert(id.clone(), schedule);
        
        let mut counters = self.counters.lock().map_err(|e| AppError::Internal(format!("Failed to lock counters: {}", e)))?;
        let counter = counters.entry("total_schedules".to_string()).or_insert(0);
        *counter += 1;
        
        Ok(id)
    }

    pub fn get_schedule(&self, schedule_id: &str) -> Result<PromptSchedule, AppError> {
        let schedules = self.schedules.lock().map_err(|e| AppError::Internal(format!("Failed to lock schedules: {}", e)))?;
        schedules.get(schedule_id).cloned().ok_or_else(|| AppError::NotFound(format!("Schedule {} not found", schedule_id)))
    }

    pub fn list_schedules(&self, active_only: bool) -> Vec<PromptSchedule> {
        let schedules = self.schedules.lock().unwrap();
        let mut result: Vec<PromptSchedule> = if active_only {
            schedules.values().filter(|s| s.is_active).cloned().collect()
        } else {
            schedules.values().cloned().collect()
        };
        
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }

    pub fn create_prompt(&self, prompt: MemoryPrompt) -> Result<String, AppError> {
        let id = prompt.id.clone();
        let mut prompts = self.prompts.lock().map_err(|e| AppError::Internal(format!("Failed to lock prompts: {}", e)))?;
        prompts.insert(id.clone(), prompt);
        
        let mut counters = self.counters.lock().map_err(|e| AppError::Internal(format!("Failed to lock counters: {}", e)))?;
        let counter = counters.entry("total_prompts".to_string()).or_insert(0);
        *counter += 1;
        
        Ok(id)
    }

    pub fn get_prompt(&self, prompt_id: &str) -> Result<MemoryPrompt, AppError> {
        let prompts = self.prompts.lock().map_err(|e| AppError::Internal(format!("Failed to lock prompts: {}", e)))?;
        prompts.get(prompt_id).cloned().ok_or_else(|| AppError::NotFound(format!("Prompt {} not found", prompt_id)))
    }

    pub fn list_prompts(&self, user_id: Option<&str>, active_only: bool) -> Vec<MemoryPrompt> {
        let prompts = self.prompts.lock().unwrap();
        let mut result: Vec<MemoryPrompt> = prompts.values()
            .filter(|p| {
                if let Some(uid) = user_id {
                    p.user_id == uid
                } else {
                    true
                }
            })
            .filter(|p| {
                if active_only {
                    p.is_active
                } else {
                    true
                }
            })
            .cloned()
            .collect();
        
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }

    pub fn trigger_prompt(&self, prompt_id: &str) -> Result<String, AppError> {
        let mut prompts = self.prompts.lock().map_err(|e| AppError::Internal(format!("Failed to lock prompts: {}", e)))?;
        let prompt = prompts.get_mut(prompt_id)
            .ok_or_else(|| AppError::NotFound(format!("Prompt {} not found", prompt_id)))?;
        
        if !prompt.is_active {
            return Err(AppError::Internal("Prompt is not active".to_string()));
        }
        
        prompt.triggered_at = Some(chrono::Utc::now().timestamp());
        
        Ok(prompt.id.clone())
    }

    pub fn submit_response(&self, response: PromptResponse) -> Result<String, AppError> {
        let mut responses = self.responses.lock().map_err(|e| AppError::Internal(format!("Failed to lock responses: {}", e)))?;
        responses.entry(response.prompt_id.clone()).or_insert_with(Vec::new).push(response.clone());
        
        let mut counters = self.counters.lock().map_err(|e| AppError::Internal(format!("Failed to lock counters: {}", e)))?;
        let counter = counters.entry("total_responses".to_string()).or_insert(0);
        *counter += 1;
        
        Ok(response.id)
    }

    pub fn get_responses(&self, prompt_id: &str) -> Vec<PromptResponse> {
        let responses = self.responses.lock().unwrap();
        responses.get(prompt_id).cloned().unwrap_or_default()
    }

    pub fn set_user_config(&self, config: MemoryPromptConfig) -> Result<(), AppError> {
        let mut user_configs = self.user_configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock user configs: {}", e)))?;
        user_configs.insert(config.user_id.clone(), config);
        Ok(())
    }

    pub fn get_user_config(&self, user_id: &str) -> Result<MemoryPromptConfig, AppError> {
        let user_configs = self.user_configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock user configs: {}", e)))?;
        user_configs.get(user_id).cloned().ok_or_else(|| AppError::NotFound(format!("Config for user {} not found", user_id)))
    }

    pub fn get_due_prompts(&self) -> Vec<MemoryPrompt> {
        let now = chrono::Utc::now().timestamp();
        let prompts = self.prompts.lock().unwrap();
        
        prompts.values()
            .filter(|p| {
                p.is_active && 
                p.scheduled_at <= now && 
                p.triggered_at.is_none()
            })
            .cloned()
            .collect()
    }

    pub fn deactivate_prompt(&self, prompt_id: &str) -> Result<(), AppError> {
        let mut prompts = self.prompts.lock().map_err(|e| AppError::Internal(format!("Failed to lock prompts: {}", e)))?;
        if let Some(prompt) = prompts.get_mut(prompt_id) {
            prompt.is_active = false;
        }
        Ok(())
    }

    pub fn activate_prompt(&self, prompt_id: &str) -> Result<(), AppError> {
        let mut prompts = self.prompts.lock().map_err(|e| AppError::Internal(format!("Failed to lock prompts: {}", e)))?;
        if let Some(prompt) = prompts.get_mut(prompt_id) {
            prompt.is_active = true;
        }
        Ok(())
    }

    pub fn get_user_prompt_stats(&self, user_id: &str) -> serde_json::Value {
        let prompts = self.prompts.lock().unwrap();
        let responses = self.responses.lock().unwrap();
        
        let user_prompts: Vec<&MemoryPrompt> = prompts.values()
            .filter(|p| p.user_id == user_id)
            .collect();
        
        let total_prompts = user_prompts.len();
        let active_prompts = user_prompts.iter().filter(|p| p.is_active).count();
        let triggered_prompts = user_prompts.iter().filter(|p| p.triggered_at.is_some()).count();
        
        let total_responses: usize = user_prompts.iter()
            .map(|p| responses.get(&p.id).map(|r| r.len()).unwrap_or(0))
            .sum();
        
        serde_json::json!({
            "user_id": user_id,
            "total_prompts": total_prompts,
            "active_prompts": active_prompts,
            "triggered_prompts": triggered_prompts,
            "total_responses": total_responses,
        })
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let prompts = self.prompts.lock().unwrap();
        let schedules = self.schedules.lock().unwrap();
        let responses = self.responses.lock().unwrap();
        let user_configs = self.user_configs.lock().unwrap();
        
        let total_responses: usize = responses.values().map(|v| v.len()).sum();
        
        serde_json::json!({
            "total_prompts": prompts.len(),
            "total_schedules": schedules.len(),
            "active_schedules": schedules.values().filter(|s| s.is_active).count(),
            "total_responses": total_responses,
            "total_user_configs": user_configs.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_system() -> MemoryPromptingSystem {
        MemoryPromptingSystem::new()
    }

    fn create_test_schedule() -> PromptSchedule {
        PromptSchedule {
            id: "schedule1".to_string(),
            name: "Daily Check-in".to_string(),
            description: "Daily memory check-in".to_string(),
            cron_expression: "0 9 * * *".to_string(),
            prompt_type: "check_in".to_string(),
            target_users: vec!["user1".to_string()],
            is_active: true,
            last_triggered: None,
            trigger_count: 0,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    fn create_test_prompt() -> MemoryPrompt {
        MemoryPrompt {
            id: "prompt1".to_string(),
            user_id: "user1".to_string(),
            prompt_type: "check_in".to_string(),
            content: "How are you feeling today?".to_string(),
            context: HashMap::new(),
            priority: 5,
            created_at: chrono::Utc::now().timestamp(),
            scheduled_at: chrono::Utc::now().timestamp(),
            triggered_at: None,
            is_active: true,
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_create_schedule() {
        let system = create_test_system();
        let schedule = create_test_schedule();
        let result = system.create_schedule(schedule);
        assert!(result.is_ok());
        
        let retrieved = system.get_schedule("schedule1").unwrap();
        assert_eq!(retrieved.name, "Daily Check-in");
    }

    #[test]
    fn test_list_schedules() {
        let system = create_test_system();
        let schedule1 = create_test_schedule();
        let mut schedule2 = create_test_schedule();
        schedule2.id = "schedule2".to_string();
        
        let _ = system.create_schedule(schedule1);
        let _ = system.create_schedule(schedule2);

        let schedules = system.list_schedules(false);
        assert_eq!(schedules.len(), 2);

        let active = system.list_schedules(true);
        assert_eq!(active.len(), 2);
    }

    #[test]
    fn test_create_prompt() {
        let system = create_test_system();
        let prompt = create_test_prompt();
        let result = system.create_prompt(prompt);
        assert!(result.is_ok());
        
        let retrieved = system.get_prompt("prompt1").unwrap();
        assert_eq!(retrieved.content, "How are you feeling today?");
    }

    #[test]
    fn test_trigger_prompt() {
        let system = create_test_system();
        let prompt = create_test_prompt();
        let _ = system.create_prompt(prompt);

        let result = system.trigger_prompt("prompt1");
        assert!(result.is_ok());

        let prompt = system.get_prompt("prompt1").unwrap();
        assert!(prompt.triggered_at.is_some());
    }

    #[test]
    fn test_submit_response() {
        let system = create_test_system();
        let prompt = create_test_prompt();
        let _ = system.create_prompt(prompt);

        let response = PromptResponse {
            id: "response1".to_string(),
            prompt_id: "prompt1".to_string(),
            user_id: "user1".to_string(),
            response: "I'm feeling great!".to_string(),
            sentiment: 0.8,
            timestamp: chrono::Utc::now().timestamp(),
            follow_up_required: false,
        };

        let result = system.submit_response(response);
        assert!(result.is_ok());

        let responses = system.get_responses("prompt1");
        assert_eq!(responses.len(), 1);
    }

    #[test]
    fn test_set_user_config() {
        let system = create_test_system();
        let config = MemoryPromptConfig {
            user_id: "user1".to_string(),
            prompt_frequency_hours: 24,
            preferred_prompt_types: vec!["check_in".to_string()],
            max_prompts_per_day: 5,
            quiet_hours_start: 22,
            quiet_hours_end: 7,
            enable_adaptive_scheduling: true,
        };

        let result = system.set_user_config(config);
        assert!(result.is_ok());

        let retrieved = system.get_user_config("user1").unwrap();
        assert_eq!(retrieved.prompt_frequency_hours, 24);
    }

    #[test]
    fn test_get_due_prompts() {
        let system = create_test_system();
        let mut prompt = create_test_prompt();
        prompt.scheduled_at = chrono::Utc::now().timestamp() - 3600;
        let _ = system.create_prompt(prompt);

        let due = system.get_due_prompts();
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn test_deactivate_prompt() {
        let system = create_test_system();
        let prompt = create_test_prompt();
        let _ = system.create_prompt(prompt);

        let result = system.deactivate_prompt("prompt1");
        assert!(result.is_ok());

        let retrieved = system.get_prompt("prompt1").unwrap();
        assert!(!retrieved.is_active);
    }

    #[test]
    fn test_activate_prompt() {
        let system = create_test_system();
        let mut prompt = create_test_prompt();
        prompt.is_active = false;
        let _ = system.create_prompt(prompt);

        let result = system.activate_prompt("prompt1");
        assert!(result.is_ok());

        let retrieved = system.get_prompt("prompt1").unwrap();
        assert!(retrieved.is_active);
    }

    #[test]
    fn test_get_user_prompt_stats() {
        let system = create_test_system();
        let prompt = create_test_prompt();
        let _ = system.create_prompt(prompt);

        let stats = system.get_user_prompt_stats("user1");
        assert_eq!(stats["total_prompts"].as_u64().unwrap(), 1);
        assert_eq!(stats["active_prompts"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_get_stats() {
        let system = create_test_system();
        let schedule = create_test_schedule();
        let prompt = create_test_prompt();
        
        let _ = system.create_schedule(schedule);
        let _ = system.create_prompt(prompt);

        let stats = system.get_stats();
        assert_eq!(stats["total_prompts"].as_u64().unwrap(), 1);
        assert_eq!(stats["total_schedules"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_list_prompts_with_filters() {
        let system = create_test_system();
        let prompt1 = create_test_prompt();
        let mut prompt2 = create_test_prompt();
        prompt2.id = "prompt2".to_string();
        prompt2.user_id = "user2".to_string();
        
        let _ = system.create_prompt(prompt1);
        let _ = system.create_prompt(prompt2);

        let all_prompts = system.list_prompts(None, false);
        assert_eq!(all_prompts.len(), 2);

        let user1_prompts = system.list_prompts(Some("user1"), false);
        assert_eq!(user1_prompts.len(), 1);

        let active_prompts = system.list_prompts(None, true);
        assert_eq!(active_prompts.len(), 2);
    }
}
