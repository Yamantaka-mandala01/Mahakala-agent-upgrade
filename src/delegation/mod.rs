use crate::agent::{AIAgent, AgentConfig};
use crate::error::AppError;
use crate::tools::registry::ToolRegistry;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentConfig {
    pub id: String,
    pub name: String,
    pub role: String,
    pub model: String,
    pub provider: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f64>,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub assigned_to: Option<String>,
    pub status: TaskStatus,
    pub result: Option<String>,
    pub created_at: i64,
    pub completed_at: Option<i64>,
    pub priority: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DelegationResult {
    pub task_id: String,
    pub sub_agent_id: String,
    pub status: TaskStatus,
    pub result: String,
}

#[derive(Clone)]
pub struct SubAgent {
    pub config: SubAgentConfig,
    pub agent: Arc<RwLock<AIAgent>>,
    pub tasks: Arc<Mutex<Vec<Task>>>,
    pub is_active: bool,
}

impl SubAgent {
    pub fn new(config: SubAgentConfig, registry: Arc<ToolRegistry>) -> Self {
        let agent_config = AgentConfig {
            model: config.model.clone(),
            provider: config.provider.clone(),
            api_base_url: config.api_base_url.clone(),
            api_key: config.api_key.clone(),
            temperature: config.temperature,
            max_tokens: config.max_tokens,
        };

        let agent = AIAgent::new(agent_config, registry);

        Self {
            config,
            agent: Arc::new(RwLock::new(agent)),
            tasks: Arc::new(Mutex::new(Vec::new())),
            is_active: false,
        }
    }

    pub async fn execute_task(&self, task: &Task) -> Result<String, AppError> {
        let mut agent = self.agent.write().await;
        let result = agent.process_message(&task.description).await;
        
        match result {
            Ok(response) => Ok(response),
            Err(e) => Err(AppError::Internal(format!("SubAgent task execution failed: {}", e))),
        }
    }

    pub fn get_task_count(&self) -> usize {
        self.tasks.lock().len()
    }

    pub fn get_completed_tasks(&self) -> Vec<Task> {
        self.tasks.lock()
            .iter()
            .filter(|t| t.status == TaskStatus::Completed)
            .cloned()
            .collect()
    }
}

pub struct DelegationSystem {
    pub sub_agents: Arc<Mutex<Vec<SubAgent>>>,
    pub task_queue: Arc<Mutex<Vec<Task>>>,
    pub registry: Arc<ToolRegistry>,
}

impl DelegationSystem {
    pub fn new(registry: Arc<ToolRegistry>) -> Self {
        Self {
            sub_agents: Arc::new(Mutex::new(Vec::new())),
            task_queue: Arc::new(Mutex::new(Vec::new())),
            registry,
        }
    }

    pub fn create_sub_agent(&self, config: SubAgentConfig) -> Result<String, AppError> {
        let sub_agent = SubAgent::new(config, self.registry.clone());
        let id = sub_agent.config.id.clone();
        
        let mut agents = self.sub_agents.lock();
        agents.push(sub_agent);
        
        Ok(id)
    }

    pub fn remove_sub_agent(&self, id: &str) -> Result<bool, AppError> {
        let mut agents = self.sub_agents.lock();
        let initial_len = agents.len();
        agents.retain(|a| a.config.id != id);
        Ok(agents.len() < initial_len)
    }

    pub fn get_sub_agent(&self, id: &str) -> Option<SubAgent> {
        let agents = self.sub_agents.lock();
        agents.iter().find(|a| a.config.id == id).cloned()
    }

    pub fn list_sub_agents(&self) -> Vec<SubAgentConfig> {
        let agents = self.sub_agents.lock();
        agents.iter().map(|a| a.config.clone()).collect()
    }

    pub fn create_task(&self, description: &str, priority: u8) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let task = Task {
            id: id.clone(),
            description: description.to_string(),
            assigned_to: None,
            status: TaskStatus::Pending,
            result: None,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
            priority,
        };

        let mut queue = self.task_queue.lock();
        queue.push(task);
        
        id
    }

    pub async fn delegate_task(&self, task_id: &str, sub_agent_id: &str) -> Result<DelegationResult, AppError> {
        let sub_agent = {
            let agents = self.sub_agents.lock();
            agents.iter().find(|a| a.config.id == sub_agent_id).cloned()
        };

        if let Some(agent) = sub_agent {
            let task = {
                let mut queue = self.task_queue.lock();
                queue.iter_mut().find(|t| t.id == task_id).map(|t| {
                    t.assigned_to = Some(sub_agent_id.to_string());
                    t.status = TaskStatus::Running;
                    t.clone()
                })
            };

            if let Some(task) = task {
                let result = agent.execute_task(&task).await;
                
                let mut queue = self.task_queue.lock();
                if let Some(t) = queue.iter_mut().find(|t| t.id == task_id) {
                    match &result {
                        Ok(response) => {
                            t.status = TaskStatus::Completed;
                            t.result = Some(response.clone());
                            t.completed_at = Some(chrono::Utc::now().timestamp());
                        }
                        Err(_) => {
                            t.status = TaskStatus::Failed;
                            t.result = Some(format!("Task execution failed"));
                            t.completed_at = Some(chrono::Utc::now().timestamp());
                        }
                    }
                }

                match result {
                    Ok(response) => Ok(DelegationResult {
                        task_id: task_id.to_string(),
                        sub_agent_id: sub_agent_id.to_string(),
                        status: TaskStatus::Completed,
                        result: response,
                    }),
                    Err(e) => Ok(DelegationResult {
                        task_id: task_id.to_string(),
                        sub_agent_id: sub_agent_id.to_string(),
                        status: TaskStatus::Failed,
                        result: e.to_string(),
                    }),
                }
            } else {
                Err(AppError::NotFound(format!("Task {} not found", task_id)))
            }
        } else {
            Err(AppError::NotFound(format!("SubAgent {} not found", sub_agent_id)))
        }
    }

    pub async fn delegate_parallel(&self, task_ids: &[String], sub_agent_ids: &[String]) -> Result<Vec<DelegationResult>, AppError> {
        let mut handles = vec![];
        
        for (i, task_id) in task_ids.iter().enumerate() {
            let sub_agent_id = sub_agent_ids[i % sub_agent_ids.len()].clone();
            let task_id = task_id.clone();
            let sub_agent_id_clone = sub_agent_id.clone();
            
            let system = self.clone_for_parallel();
            
            let handle = tokio::spawn(async move {
                system.delegate_task(&task_id, &sub_agent_id_clone).await
            });
            
            handles.push(handle);
        }

        let mut results = vec![];
        for handle in handles {
            match handle.await {
                Ok(result) => {
                    if let Ok(r) = result {
                        results.push(r);
                    }
                }
                Err(e) => {
                    tracing::error!("Parallel task execution failed: {}", e);
                }
            }
        }

        Ok(results)
    }

    fn clone_for_parallel(&self) -> Arc<DelegationSystem> {
        Arc::new(DelegationSystem {
            sub_agents: self.sub_agents.clone(),
            task_queue: self.task_queue.clone(),
            registry: self.registry.clone(),
        })
    }

    pub fn get_task(&self, id: &str) -> Option<Task> {
        let queue = self.task_queue.lock();
        queue.iter().find(|t| t.id == id).cloned()
    }

    pub fn list_tasks(&self) -> Vec<Task> {
        let queue = self.task_queue.lock();
        queue.clone()
    }

    pub fn get_pending_tasks(&self) -> Vec<Task> {
        let queue = self.task_queue.lock();
        queue.iter()
            .filter(|t| t.status == TaskStatus::Pending)
            .cloned()
            .collect()
    }

    pub fn cancel_task(&self, id: &str) -> Result<bool, AppError> {
        let mut queue = self.task_queue.lock();
        if let Some(task) = queue.iter_mut().find(|t| t.id == id) {
            if task.status == TaskStatus::Pending || task.status == TaskStatus::Running {
                task.status = TaskStatus::Cancelled;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Ok(false)
        }
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let queue = self.task_queue.lock();
        let total = queue.len();
        let pending = queue.iter().filter(|t| t.status == TaskStatus::Pending).count();
        let running = queue.iter().filter(|t| t.status == TaskStatus::Running).count();
        let completed = queue.iter().filter(|t| t.status == TaskStatus::Completed).count();
        let failed = queue.iter().filter(|t| t.status == TaskStatus::Failed).count();

        let agents = self.sub_agents.lock();
        let agent_count = agents.len();

        serde_json::json!({
            "total_tasks": total,
            "pending": pending,
            "running": running,
            "completed": completed,
            "failed": failed,
            "sub_agents": agent_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_sub_agent() {
        let registry = Arc::new(ToolRegistry::new());
        let system = DelegationSystem::new(registry);

        let config = SubAgentConfig {
            id: "test_agent".to_string(),
            name: "Test Agent".to_string(),
            role: "Test role".to_string(),
            model: "test-model".to_string(),
            provider: Some("ollama".to_string()),
            api_base_url: Some("http://localhost:11434/v1".to_string()),
            api_key: None,
            max_tokens: Some(1000),
            temperature: Some(0.7),
            tools: vec![],
        };

        let result = system.create_sub_agent(config);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "test_agent");
    }

    #[test]
    fn test_create_task() {
        let registry = Arc::new(ToolRegistry::new());
        let system = DelegationSystem::new(registry);

        let task_id = system.create_task("Test task", 1);
        assert!(!task_id.is_empty());

        let task = system.get_task(&task_id);
        assert!(task.is_some());
        assert_eq!(task.unwrap().description, "Test task");
    }

    #[test]
    fn test_list_sub_agents() {
        let registry = Arc::new(ToolRegistry::new());
        let system = DelegationSystem::new(registry);

        let agents = system.list_sub_agents();
        assert!(agents.is_empty());

        let config = SubAgentConfig {
            id: "agent1".to_string(),
            name: "Agent 1".to_string(),
            role: "Role 1".to_string(),
            model: "model1".to_string(),
            provider: None,
            api_base_url: None,
            api_key: None,
            max_tokens: None,
            temperature: None,
            tools: vec![],
        };

        system.create_sub_agent(config).unwrap();
        let agents = system.list_sub_agents();
        assert_eq!(agents.len(), 1);
    }
}
