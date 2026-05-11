use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentState {
    pub state_id: String,
    pub environment_type: String,
    pub step: usize,
    pub observation: serde_json::Value,
    pub reward: f64,
    pub done: bool,
    pub info: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentAction {
    pub action_id: String,
    pub action_type: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub name: String,
    pub environment_type: String,
    pub max_steps: usize,
    pub observation_space: serde_json::Value,
    pub action_space: serde_json::Value,
    pub reward_range: (f64, f64),
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtroposEnvironment {
    pub id: String,
    pub config: EnvironmentConfig,
    pub current_state: EnvironmentState,
    pub episode_count: usize,
    pub total_steps: usize,
    pub total_reward: f64,
    pub is_active: bool,
    pub created_at: i64,
}

pub struct RlEnvironment {
    environments: Arc<Mutex<HashMap<String, AtroposEnvironment>>>,
    configs: Arc<Mutex<HashMap<String, EnvironmentConfig>>>,
    episode_history: Arc<Mutex<Vec<serde_json::Value>>>,
    counters: Arc<Mutex<HashMap<String, usize>>>,
}

impl Default for RlEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

impl RlEnvironment {
    pub fn new() -> Self {
        Self {
            environments: Arc::new(Mutex::new(HashMap::new())),
            configs: Arc::new(Mutex::new(HashMap::new())),
            episode_history: Arc::new(Mutex::new(Vec::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn register_config(&self, config: EnvironmentConfig) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        configs.insert(id.clone(), config);
        Ok(id)
    }

    pub fn get_config(&self, config_id: &str) -> Result<EnvironmentConfig, AppError> {
        let configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        configs.get(config_id).cloned().ok_or_else(|| AppError::NotFound(format!("Config {} not found", config_id)))
    }

    pub fn create_environment(&self, config_id: &str) -> Result<String, AppError> {
        let config = self.get_config(config_id)?;
        let id = uuid::Uuid::new_v4().to_string();
        
        let initial_state = self.initialize_state(&config);
        
        let env = AtroposEnvironment {
            id: id.clone(),
            config,
            current_state: initial_state,
            episode_count: 0,
            total_steps: 0,
            total_reward: 0.0,
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
        };

        let mut environments = self.environments.lock().map_err(|e| AppError::Internal(format!("Failed to lock environments: {}", e)))?;
        environments.insert(id.clone(), env);

        let mut counters = self.counters.lock().map_err(|e| AppError::Internal(format!("Failed to lock counters: {}", e)))?;
        let counter = counters.entry("total_environments".to_string()).or_insert(0);
        *counter += 1;

        Ok(id)
    }

    fn initialize_state(&self, config: &EnvironmentConfig) -> EnvironmentState {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{}-init-{}", config.name, chrono::Utc::now().timestamp()).hash(&mut hasher);
        let hash_value = hasher.finish() as f64 / u64::MAX as f64;

        let observation = match config.environment_type.as_str() {
            "cartpole" => serde_json::json!({
                "position": (hash_value - 0.5) * 0.1,
                "velocity": 0.0,
                "angle": (hash_value - 0.5) * 0.05,
                "angular_velocity": 0.0,
            }),
            "mountain_car" => serde_json::json!({
                "position": -0.5,
                "velocity": 0.0,
            }),
            "acrobot" => serde_json::json!({
                "link1_angle": 0.0,
                "link2_angle": 0.0,
                "link1_velocity": 0.0,
                "link2_velocity": 0.0,
            }),
            _ => serde_json::json!({
                "vector": vec![hash_value; 10],
            }),
        };

        EnvironmentState {
            state_id: uuid::Uuid::new_v4().to_string(),
            environment_type: config.environment_type.clone(),
            step: 0,
            observation,
            reward: 0.0,
            done: false,
            info: HashMap::new(),
        }
    }

    pub fn step(&self, env_id: &str, action: EnvironmentAction) -> Result<EnvironmentState, AppError> {
        let mut environments = self.environments.lock().map_err(|e| AppError::Internal(format!("Failed to lock environments: {}", e)))?;
        let env = environments.get_mut(env_id)
            .ok_or_else(|| AppError::NotFound(format!("Environment {} not found", env_id)))?;

        if !env.is_active {
            return Err(AppError::Internal("Environment is not active".to_string()));
        }

        if env.current_state.done {
            return Err(AppError::Internal("Episode is done, please reset".to_string()));
        }

        if env.current_state.step >= env.config.max_steps {
            env.current_state.done = true;
            return Ok(env.current_state.clone());
        }

        let (observation, reward, done) = self.compute_next_state(env, &action);
        
        env.current_state = EnvironmentState {
            state_id: uuid::Uuid::new_v4().to_string(),
            environment_type: env.config.environment_type.clone(),
            step: env.current_state.step + 1,
            observation,
            reward,
            done,
            info: HashMap::new(),
        };

        env.total_steps += 1;
        env.total_reward += reward;

        if done {
            env.episode_count += 1;
            self.record_episode(env);
        }

        Ok(env.current_state.clone())
    }

    fn compute_next_state(&self, env: &mut AtroposEnvironment, action: &EnvironmentAction) -> (serde_json::Value, f64, bool) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{}-{}-{}", env.config.name, env.current_state.step, action.action_type).hash(&mut hasher);
        let hash_value = hasher.finish() as f64 / u64::MAX as f64;

        match env.config.environment_type.as_str() {
            "cartpole" => {
                let angle = (hash_value - 0.5) * 0.1;
                let reward = if angle.abs() < 0.2 { 1.0 } else { -1.0 };
                let done = angle.abs() > 0.5;
                (serde_json::json!({ "angle": angle, "velocity": hash_value }), reward, done)
            }
            "mountain_car" => {
                let position = -0.5 + hash_value * 1.5;
                let reward = if position >= 0.5 { 100.0 } else { -1.0 };
                let done = position >= 0.5;
                (serde_json::json!({ "position": position, "velocity": hash_value }), reward, done)
            }
            _ => {
                let reward = (hash_value - 0.5) * 2.0;
                let done = hash_value > 0.95;
                (serde_json::json!({ "value": hash_value }), reward, done)
            }
        }
    }

    fn record_episode(&self, env: &AtroposEnvironment) {
        let episode_record = serde_json::json!({
            "environment_id": env.id,
            "episode": env.episode_count,
            "total_steps": env.total_steps,
            "total_reward": env.total_reward,
            "timestamp": chrono::Utc::now().timestamp(),
        });

        if let Ok(mut history) = self.episode_history.lock() {
            history.push(episode_record);
        }
    }

    pub fn reset(&self, env_id: &str) -> Result<EnvironmentState, AppError> {
        let mut environments = self.environments.lock().map_err(|e| AppError::Internal(format!("Failed to lock environments: {}", e)))?;
        let env = environments.get_mut(env_id)
            .ok_or_else(|| AppError::NotFound(format!("Environment {} not found", env_id)))?;

        env.current_state = self.initialize_state(&env.config);
        env.total_steps = 0;
        env.total_reward = 0.0;

        Ok(env.current_state.clone())
    }

    pub fn get_environment(&self, env_id: &str) -> Result<AtroposEnvironment, AppError> {
        let environments = self.environments.lock().map_err(|e| AppError::Internal(format!("Failed to lock environments: {}", e)))?;
        environments.get(env_id).cloned().ok_or_else(|| AppError::NotFound(format!("Environment {} not found", env_id)))
    }

    pub fn list_environments(&self, filters: Option<HashMap<String, String>>) -> Vec<AtroposEnvironment> {
        let environments = self.environments.lock().unwrap();
        let mut result: Vec<AtroposEnvironment> = environments.values().cloned().collect();

        if let Some(f) = filters {
            if let Some(env_type) = f.get("environment_type") {
                result.retain(|e| e.config.environment_type == *env_type);
            }
            if let Some(active) = f.get("is_active") {
                let active_bool = active == "true";
                result.retain(|e| e.is_active == active_bool);
            }
        }

        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }

    pub fn close_environment(&self, env_id: &str) -> Result<(), AppError> {
        let mut environments = self.environments.lock().map_err(|e| AppError::Internal(format!("Failed to lock environments: {}", e)))?;
        if let Some(env) = environments.get_mut(env_id) {
            env.is_active = false;
        }
        Ok(())
    }

    pub fn get_episode_history(&self, env_id: Option<&str>, limit: usize) -> Vec<serde_json::Value> {
        let history = self.episode_history.lock().unwrap();
        let filtered: Vec<serde_json::Value> = match env_id {
            Some(id) => history.iter()
                .filter(|e| e.get("environment_id").and_then(|v| v.as_str()) == Some(id))
                .cloned()
                .collect(),
            None => history.clone(),
        };

        let start = filtered.len().saturating_sub(limit);
        filtered[start..].to_vec()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let environments = self.environments.lock().unwrap();
        let history = self.episode_history.lock().unwrap();

        let total_episodes: usize = environments.values().map(|e| e.episode_count).sum();
        let total_steps: usize = environments.values().map(|e| e.total_steps).sum();
        let avg_reward = if total_episodes > 0 {
            environments.values().map(|e| e.total_reward).sum::<f64>() / total_episodes as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_environments": environments.len(),
            "active_environments": environments.values().filter(|e| e.is_active).count(),
            "total_episodes": total_episodes,
            "total_steps": total_steps,
            "avg_reward": avg_reward,
            "history_length": history.len(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_environment() -> RlEnvironment {
        RlEnvironment::new()
    }

    fn create_test_config() -> EnvironmentConfig {
        EnvironmentConfig {
            name: "test-cartpole".to_string(),
            environment_type: "cartpole".to_string(),
            max_steps: 100,
            observation_space: serde_json::json!({
                "type": "Box",
                "shape": [4],
                "dtype": "float32",
            }),
            action_space: serde_json::json!({
                "type": "Discrete",
                "n": 2,
            }),
            reward_range: (-1.0, 1.0),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_register_config() {
        let env = create_test_environment();
        let config = create_test_config();
        let result = env.register_config(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_create_environment() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        
        let env_id = env.create_environment(&config_id).unwrap();
        assert!(!env_id.is_empty());

        let environment = env.get_environment(&env_id).unwrap();
        assert!(environment.is_active);
        assert_eq!(environment.episode_count, 0);
    }

    #[test]
    fn test_step_environment() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        let env_id = env.create_environment(&config_id).unwrap();

        let action = EnvironmentAction {
            action_id: "test-action".to_string(),
            action_type: "move_left".to_string(),
            parameters: serde_json::json!({}),
        };

        let state = env.step(&env_id, action).unwrap();
        assert!(!state.done || state.step > 0);
    }

    #[test]
    fn test_reset_environment() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        let env_id = env.create_environment(&config_id).unwrap();

        let state = env.reset(&env_id).unwrap();
        assert_eq!(state.step, 0);
        assert!(!state.done);
    }

    #[test]
    fn test_list_environments() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        
        let _ = env.create_environment(&config_id);
        let _ = env.create_environment(&config_id);

        let environments = env.list_environments(None);
        assert_eq!(environments.len(), 2);

        let mut filters = HashMap::new();
        filters.insert("environment_type".to_string(), "cartpole".to_string());
        let filtered = env.list_environments(Some(filters));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_close_environment() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        let env_id = env.create_environment(&config_id).unwrap();

        let result = env.close_environment(&env_id);
        assert!(result.is_ok());

        let environment = env.get_environment(&env_id).unwrap();
        assert!(!environment.is_active);
    }

    #[test]
    fn test_get_episode_history() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        let env_id = env.create_environment(&config_id).unwrap();

        let history = env.get_episode_history(Some(&env_id), 10);
        assert!(history.is_empty());
    }

    #[test]
    fn test_get_stats() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        let _ = env.create_environment(&config_id);

        let stats = env.get_stats();
        assert_eq!(stats["total_environments"].as_u64().unwrap(), 1);
        assert_eq!(stats["active_environments"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_multiple_environments() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        
        let env_id1 = env.create_environment(&config_id).unwrap();
        let env_id2 = env.create_environment(&config_id).unwrap();

        let action1 = EnvironmentAction {
            action_id: "action1".to_string(),
            action_type: "move_left".to_string(),
            parameters: serde_json::json!({}),
        };
        let action2 = EnvironmentAction {
            action_id: "action2".to_string(),
            action_type: "move_right".to_string(),
            parameters: serde_json::json!({}),
        };

        let _ = env.step(&env_id1, action1);
        let _ = env.step(&env_id2, action2);

        let environments = env.list_environments(None);
        assert_eq!(environments.len(), 2);
    }

    #[test]
    fn test_environment_state_transitions() {
        let env = create_test_environment();
        let config = create_test_config();
        let config_id = env.register_config(config).unwrap();
        let env_id = env.create_environment(&config_id).unwrap();

        let mut states = Vec::new();
        for i in 0..10 {
            let action = EnvironmentAction {
                action_id: format!("action-{}", i),
                action_type: if i % 2 == 0 { "move_left" } else { "move_right" }.to_string(),
                parameters: serde_json::json!({}),
            };
            let state = env.step(&env_id, action);
            if let Ok(s) = state {
                let done = s.done;
                states.push(s);
                if done {
                    break;
                }
            }
        }

        assert!(!states.is_empty());
    }
}
