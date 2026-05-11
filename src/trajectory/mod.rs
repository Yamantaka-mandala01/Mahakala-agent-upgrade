use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub step_id: usize,
    pub state: serde_json::Value,
    pub action: serde_json::Value,
    pub reward: f64,
    pub next_state: serde_json::Value,
    pub done: bool,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub id: String,
    pub name: String,
    pub description: String,
    pub environment: String,
    pub agent_id: String,
    pub steps: Vec<TrajectoryStep>,
    pub total_reward: f64,
    pub episode_length: usize,
    pub created_at: i64,
    pub tags: Vec<String>,
    pub is_valid: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryBatch {
    pub id: String,
    pub name: String,
    pub description: String,
    pub environment: String,
    pub agent_id: String,
    pub trajectories: Vec<Trajectory>,
    pub batch_size: usize,
    pub avg_reward: f64,
    pub min_reward: f64,
    pub max_reward: f64,
    pub created_at: i64,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RlTrainingConfig {
    pub environment: String,
    pub agent_id: String,
    pub batch_size: usize,
    pub max_steps_per_episode: usize,
    pub exploration_rate: f64,
    pub learning_rate: f64,
    pub discount_factor: f64,
    pub reward_scale: f64,
    pub save_trajectories: bool,
    pub tags: Vec<String>,
}

impl Default for RlTrainingConfig {
    fn default() -> Self {
        Self {
            environment: "default".to_string(),
            agent_id: "default-agent".to_string(),
            batch_size: 32,
            max_steps_per_episode: 100,
            exploration_rate: 0.1,
            learning_rate: 0.001,
            discount_factor: 0.99,
            reward_scale: 1.0,
            save_trajectories: true,
            tags: vec![],
        }
    }
}

pub struct TrajectoryGenerator {
    batches: Arc<Mutex<HashMap<String, TrajectoryBatch>>>,
    trajectories: Arc<Mutex<HashMap<String, Trajectory>>>,
    configs: Arc<Mutex<HashMap<String, RlTrainingConfig>>>,
    counters: Arc<Mutex<HashMap<String, usize>>>,
}

impl Default for TrajectoryGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TrajectoryGenerator {
    pub fn new() -> Self {
        Self {
            batches: Arc::new(Mutex::new(HashMap::new())),
            trajectories: Arc::new(Mutex::new(HashMap::new())),
            configs: Arc::new(Mutex::new(HashMap::new())),
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn create_config(&self, config: RlTrainingConfig) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let mut configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        configs.insert(id.clone(), config);
        Ok(id)
    }

    pub fn get_config(&self, config_id: &str) -> Result<RlTrainingConfig, AppError> {
        let configs = self.configs.lock().map_err(|e| AppError::Internal(format!("Failed to lock configs: {}", e)))?;
        configs.get(config_id).cloned().ok_or_else(|| AppError::NotFound(format!("Config {} not found", config_id)))
    }

    pub fn generate_batch(&self, config_id: &str, batch_name: String) -> Result<String, AppError> {
        let config = self.get_config(config_id)?;
        
        let batch_id = uuid::Uuid::new_v4().to_string();
        let mut trajectories = Vec::new();
        let mut total_reward = 0.0;
        let mut min_reward = f64::MAX;
        let mut max_reward = f64::MIN;

        for i in 0..config.batch_size {
            let trajectory = self.generate_episode(&config, i)?;
            let episode_reward = trajectory.total_reward;
            total_reward += episode_reward;
            min_reward = min_reward.min(episode_reward);
            max_reward = max_reward.max(episode_reward);
            trajectories.push(trajectory);
        }

        let avg_reward = if config.batch_size > 0 { total_reward / config.batch_size as f64 } else { 0.0 };

        let batch = TrajectoryBatch {
            id: batch_id.clone(),
            name: batch_name,
            description: format!("Batch generated with config {}", config_id),
            environment: config.environment.clone(),
            agent_id: config.agent_id.clone(),
            trajectories: trajectories.clone(),
            batch_size: config.batch_size,
            avg_reward,
            min_reward,
            max_reward,
            created_at: chrono::Utc::now().timestamp(),
            status: "completed".to_string(),
        };

        let mut batches = self.batches.lock().map_err(|e| AppError::Internal(format!("Failed to lock batches: {}", e)))?;
        batches.insert(batch_id.clone(), batch);

        let mut trajectories_map = self.trajectories.lock().map_err(|e| AppError::Internal(format!("Failed to lock trajectories: {}", e)))?;
        for trajectory in trajectories {
            trajectories_map.insert(trajectory.id.clone(), trajectory);
        }

        let mut counters = self.counters.lock().map_err(|e| AppError::Internal(format!("Failed to lock counters: {}", e)))?;
        let counter = counters.entry("total_batches".to_string()).or_insert(0);
        *counter += 1;
        let counter = counters.entry("total_episodes".to_string()).or_insert(0);
        *counter += config.batch_size;

        Ok(batch_id)
    }

    fn generate_episode(&self, config: &RlTrainingConfig, episode_idx: usize) -> Result<Trajectory, AppError> {
        let mut steps = Vec::new();
        let mut total_reward = 0.0;
        let mut current_state = self.initialize_state(config);
        let mut done = false;
        let mut step_count = 0;

        while !done && step_count < config.max_steps_per_episode {
            let action = self.select_action(config, &current_state, step_count);
            let (next_state, reward, episode_done) = self.step_environment(config, &current_state, &action);
            
            let scaled_reward = reward * config.reward_scale;
            total_reward += scaled_reward;

            let step = TrajectoryStep {
                step_id: step_count,
                state: current_state.clone(),
                action,
                reward: scaled_reward,
                next_state: next_state.clone(),
                done: episode_done,
                metadata: HashMap::new(),
            };

            steps.push(step);
            current_state = next_state;
            done = episode_done;
            step_count += 1;
        }

        let trajectory_id = uuid::Uuid::new_v4().to_string();
        let trajectory = Trajectory {
            id: trajectory_id,
            name: format!("Episode {}", episode_idx),
            description: format!("Episode generated with config {}", config.environment),
            environment: config.environment.clone(),
            agent_id: config.agent_id.clone(),
            steps,
            total_reward,
            episode_length: step_count,
            created_at: chrono::Utc::now().timestamp(),
            tags: config.tags.clone(),
            is_valid: true,
        };

        Ok(trajectory)
    }

    fn initialize_state(&self, config: &RlTrainingConfig) -> serde_json::Value {
        serde_json::json!({
            "environment": config.environment,
            "step": 0,
            "state_vector": vec![0.0; 10],
            "metadata": {
                "episode_start": chrono::Utc::now().timestamp(),
            }
        })
    }

    fn select_action(&self, config: &RlTrainingConfig, state: &serde_json::Value, step: usize) -> serde_json::Value {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{}-{}-{}", config.agent_id, step, state).hash(&mut hasher);
        let hash_value = hasher.finish() as f64 / u64::MAX as f64;

        if hash_value < config.exploration_rate {
            serde_json::json!({
                "type": "explore",
                "action_id": (hash_value * 10.0) as usize,
                "parameters": {
                    "random_value": hash_value,
                }
            })
        } else {
            serde_json::json!({
                "type": "exploit",
                "action_id": 0,
                "parameters": {
                    "confidence": 1.0 - config.exploration_rate,
                }
            })
        }
    }

    fn step_environment(&self, config: &RlTrainingConfig, state: &serde_json::Value, action: &serde_json::Value) -> (serde_json::Value, f64, bool) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        format!("{}-{}-{}", config.environment, state, action).hash(&mut hasher);
        let hash_value = hasher.finish() as f64 / u64::MAX as f64;

        let reward = (hash_value - 0.5) * 2.0;
        let done = hash_value > 0.95;

        let next_state = serde_json::json!({
            "environment": config.environment,
            "step": state.get("step").and_then(|v| v.as_u64()).unwrap_or(0) + 1,
            "state_vector": vec![hash_value; 10],
            "metadata": {
                "last_action": action.get("type").and_then(|v| v.as_str()).unwrap_or(""),
            }
        });

        (next_state, reward, done)
    }

    pub fn get_batch(&self, batch_id: &str) -> Result<TrajectoryBatch, AppError> {
        let batches = self.batches.lock().map_err(|e| AppError::Internal(format!("Failed to lock batches: {}", e)))?;
        batches.get(batch_id).cloned().ok_or_else(|| AppError::NotFound(format!("Batch {} not found", batch_id)))
    }

    pub fn list_batches(&self, filters: Option<HashMap<String, String>>) -> Vec<TrajectoryBatch> {
        let batches = self.batches.lock().unwrap();
        let mut result: Vec<TrajectoryBatch> = batches.values().cloned().collect();

        if let Some(f) = filters {
            if let Some(environment) = f.get("environment") {
                result.retain(|b| b.environment == *environment);
            }
            if let Some(agent_id) = f.get("agent_id") {
                result.retain(|b| b.agent_id == *agent_id);
            }
            if let Some(status) = f.get("status") {
                result.retain(|b| b.status == *status);
            }
        }

        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        result
    }

    pub fn get_trajectory(&self, trajectory_id: &str) -> Result<Trajectory, AppError> {
        let trajectories = self.trajectories.lock().map_err(|e| AppError::Internal(format!("Failed to lock trajectories: {}", e)))?;
        trajectories.get(trajectory_id).cloned().ok_or_else(|| AppError::NotFound(format!("Trajectory {} not found", trajectory_id)))
    }

    pub fn export_batch(&self, batch_id: &str, format: &str) -> Result<String, AppError> {
        let batch = self.get_batch(batch_id)?;

        match format {
            "json" => {
                serde_json::to_string_pretty(&batch).map_err(|e| AppError::Internal(format!("Failed to serialize batch: {}", e)))
            }
            "csv" => {
                let mut csv = String::from("trajectory_id,step_id,state,action,reward,next_state,done\n");
                for trajectory in &batch.trajectories {
                    for step in &trajectory.steps {
                        csv.push_str(&format!(
                            "{},{},{},{},{},{},{}\n",
                            trajectory.id,
                            step.step_id,
                            serde_json::to_string(&step.state).unwrap_or_default(),
                            serde_json::to_string(&step.action).unwrap_or_default(),
                            step.reward,
                            serde_json::to_string(&step.next_state).unwrap_or_default(),
                            step.done
                        ));
                    }
                }
                Ok(csv)
            }
            "pickle" => {
                Err(AppError::Internal("Pickle format not supported in Rust implementation".to_string()))
            }
            _ => Err(AppError::Internal(format!("Unsupported export format: {}", format))),
        }
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let batches = self.batches.lock().unwrap();
        let trajectories = self.trajectories.lock().unwrap();
        let counters = self.counters.lock().unwrap();

        let total_batches = batches.len();
        let total_trajectories = trajectories.len();
        let total_steps: usize = trajectories.values().map(|t| t.steps.len()).sum();
        
        let avg_reward = if total_trajectories > 0 {
            trajectories.values().map(|t| t.total_reward).sum::<f64>() / total_trajectories as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_batches": total_batches,
            "total_trajectories": total_trajectories,
            "total_steps": total_steps,
            "avg_reward": avg_reward,
            "counters": *counters
        })
    }

    pub fn delete_batch(&self, batch_id: &str) -> Result<(), AppError> {
        let mut batches = self.batches.lock().map_err(|e| AppError::Internal(format!("Failed to lock batches: {}", e)))?;
        if let Some(batch) = batches.remove(batch_id) {
            let mut trajectories = self.trajectories.lock().map_err(|e| AppError::Internal(format!("Failed to lock trajectories: {}", e)))?;
            for trajectory in &batch.trajectories {
                trajectories.remove(&trajectory.id);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_generator() -> TrajectoryGenerator {
        TrajectoryGenerator::new()
    }

    fn create_test_config() -> RlTrainingConfig {
        RlTrainingConfig {
            environment: "test-env".to_string(),
            agent_id: "test-agent".to_string(),
            batch_size: 5,
            max_steps_per_episode: 10,
            exploration_rate: 0.1,
            learning_rate: 0.001,
            discount_factor: 0.99,
            reward_scale: 1.0,
            save_trajectories: true,
            tags: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_create_config() {
        let generator = create_test_generator();
        let config = create_test_config();
        let result = generator.create_config(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_generate_batch() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        
        let batch_id = generator.generate_batch(&config_id, "test-batch".to_string()).unwrap();
        assert!(!batch_id.is_empty());

        let batch = generator.get_batch(&batch_id).unwrap();
        assert_eq!(batch.name, "test-batch");
        assert_eq!(batch.trajectories.len(), 5);
        assert_eq!(batch.status, "completed");
    }

    #[test]
    fn test_list_batches() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        
        let _ = generator.generate_batch(&config_id, "batch1".to_string());
        let _ = generator.generate_batch(&config_id, "batch2".to_string());

        let batches = generator.list_batches(None);
        assert_eq!(batches.len(), 2);

        let mut filters = HashMap::new();
        filters.insert("environment".to_string(), "test-env".to_string());
        let filtered = generator.list_batches(Some(filters));
        assert_eq!(filtered.len(), 2);
    }

    #[test]
    fn test_get_trajectory() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        let batch_id = generator.generate_batch(&config_id, "test-batch".to_string()).unwrap();
        
        let batch = generator.get_batch(&batch_id).unwrap();
        let trajectory_id = &batch.trajectories[0].id;
        
        let trajectory = generator.get_trajectory(trajectory_id).unwrap();
        assert_eq!(trajectory.id, *trajectory_id);
        assert!(!trajectory.steps.is_empty());
    }

    #[test]
    fn test_export_batch_json() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        let batch_id = generator.generate_batch(&config_id, "test-batch".to_string()).unwrap();
        
        let result = generator.export_batch(&batch_id, "json");
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("test-batch"));
    }

    #[test]
    fn test_export_batch_csv() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        let batch_id = generator.generate_batch(&config_id, "test-batch".to_string()).unwrap();
        
        let result = generator.export_batch(&batch_id, "csv");
        assert!(result.is_ok());
        let csv_str = result.unwrap();
        assert!(csv_str.contains("trajectory_id,step_id,state,action,reward,next_state,done"));
    }

    #[test]
    fn test_get_stats() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        let _ = generator.generate_batch(&config_id, "test-batch".to_string());
        
        let stats = generator.get_stats();
        assert_eq!(stats["total_batches"].as_u64().unwrap(), 1);
        assert_eq!(stats["total_trajectories"].as_u64().unwrap(), 5);
        assert!(stats["total_steps"].as_u64().unwrap() > 0);
    }

    #[test]
    fn test_delete_batch() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        let batch_id = generator.generate_batch(&config_id, "test-batch".to_string()).unwrap();
        
        let result = generator.delete_batch(&batch_id);
        assert!(result.is_ok());

        let batches = generator.list_batches(None);
        assert!(batches.is_empty());
    }

    #[test]
    fn test_trajectory_validation() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        let batch_id = generator.generate_batch(&config_id, "test-batch".to_string()).unwrap();
        
        let batch = generator.get_batch(&batch_id).unwrap();
        for trajectory in &batch.trajectories {
            assert!(trajectory.is_valid);
            assert!(!trajectory.steps.is_empty());
            assert!(trajectory.episode_length > 0);
        }
    }

    #[test]
    fn test_batch_statistics() {
        let generator = create_test_generator();
        let config = create_test_config();
        let config_id = generator.create_config(config).unwrap();
        let batch_id = generator.generate_batch(&config_id, "test-batch".to_string()).unwrap();
        
        let batch = generator.get_batch(&batch_id).unwrap();
        assert!(batch.avg_reward.is_finite());
        assert!(batch.min_reward.is_finite());
        assert!(batch.max_reward.is_finite());
        assert!(batch.min_reward <= batch.max_reward);
        assert!(batch.batch_size > 0);
        assert!(!batch.trajectories.is_empty());
    }
}
