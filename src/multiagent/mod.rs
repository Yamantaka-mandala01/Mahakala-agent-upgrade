use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProfile {
    pub id: String,
    pub name: String,
    pub role: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub created_at: i64,
    pub task_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationTask {
    pub id: String,
    pub title: String,
    pub description: String,
    pub assigned_agents: Vec<String>,
    pub status: String,
    pub progress: f64,
    pub created_at: i64,
    pub completed_at: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub from_agent: String,
    pub to_agent: String,
    pub content: String,
    pub timestamp: i64,
    pub message_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollaborationConfig {
    pub max_agents: usize,
    pub default_role: String,
    pub enable_auto_assignment: bool,
}

pub struct MultiAgentFramework {
    agents: Arc<Mutex<HashMap<String, AgentProfile>>>,
    tasks: Arc<Mutex<HashMap<String, CollaborationTask>>>,
    messages: Arc<Mutex<Vec<AgentMessage>>>,
    config: CollaborationConfig,
}

impl MultiAgentFramework {
    pub fn new(config: CollaborationConfig) -> Self {
        Self {
            agents: Arc::new(Mutex::new(HashMap::new())),
            tasks: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    pub fn register_agent(&self, name: String, role: String, capabilities: Vec<String>) -> Result<String, AppError> {
        let agents = self.agents.lock();
        if agents.len() >= self.config.max_agents {
            return Err(AppError::Internal(format!("Maximum agent limit ({}) reached", self.config.max_agents)));
        }
        drop(agents);

        let id = uuid::Uuid::new_v4().to_string();
        let profile = AgentProfile {
            id: id.clone(),
            name,
            role,
            capabilities,
            status: "active".to_string(),
            created_at: chrono::Utc::now().timestamp(),
            task_count: 0,
        };

        let mut agents = self.agents.lock();
        agents.insert(id.clone(), profile);
        Ok(id)
    }

    pub fn unregister_agent(&self, id: &str) -> Result<(), AppError> {
        let mut agents = self.agents.lock();
        agents.remove(id);
        Ok(())
    }

    pub fn list_agents(&self) -> Vec<AgentProfile> {
        let agents = self.agents.lock();
        agents.values().cloned().collect()
    }

    pub fn get_agent(&self, id: &str) -> Option<AgentProfile> {
        let agents = self.agents.lock();
        agents.get(id).cloned()
    }

    pub fn create_collaboration_task(
        &self,
        title: String,
        description: String,
        agent_ids: Vec<String>,
    ) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let task = CollaborationTask {
            id: id.clone(),
            title,
            description,
            assigned_agents: agent_ids.clone(),
            status: "pending".to_string(),
            progress: 0.0,
            created_at: chrono::Utc::now().timestamp(),
            completed_at: None,
        };

        let mut tasks = self.tasks.lock();
        tasks.insert(id.clone(), task);

        for agent_id in &agent_ids {
            let mut agents = self.agents.lock();
            if let Some(agent) = agents.get_mut(agent_id) {
                agent.task_count += 1;
            }
        }

        Ok(id)
    }

    pub fn start_task(&self, task_id: &str) -> Result<(), AppError> {
        let mut tasks = self.tasks.lock();
        if let Some(task) = tasks.get_mut(task_id) {
            task.status = "in_progress".to_string();
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Task {} not found", task_id)))
        }
    }

    pub fn update_task_progress(&self, task_id: &str, progress: f64) -> Result<(), AppError> {
        let mut tasks = self.tasks.lock();
        if let Some(task) = tasks.get_mut(task_id) {
            task.progress = progress.min(100.0);
            if task.progress >= 100.0 {
                task.status = "completed".to_string();
                task.completed_at = Some(chrono::Utc::now().timestamp());
            }
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Task {} not found", task_id)))
        }
    }

    pub fn complete_task(&self, task_id: &str) -> Result<(), AppError> {
        self.update_task_progress(task_id, 100.0)
    }

    pub fn list_tasks(&self) -> Vec<CollaborationTask> {
        let tasks = self.tasks.lock();
        tasks.values().cloned().collect()
    }

    pub fn get_task(&self, task_id: &str) -> Option<CollaborationTask> {
        let tasks = self.tasks.lock();
        tasks.get(task_id).cloned()
    }

    pub fn send_message(&self, from_agent: &str, to_agent: &str, content: &str, message_type: &str) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let message = AgentMessage {
            id: id.clone(),
            from_agent: from_agent.to_string(),
            to_agent: to_agent.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            message_type: message_type.to_string(),
        };

        let mut messages = self.messages.lock();
        messages.push(message);
        Ok(id)
    }

    pub fn get_agent_messages(&self, agent_id: &str) -> Vec<AgentMessage> {
        let messages = self.messages.lock();
        messages.iter()
            .filter(|m| m.from_agent == agent_id || m.to_agent == agent_id)
            .cloned()
            .collect()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let agents = self.agents.lock();
        let tasks = self.tasks.lock();
        let messages = self.messages.lock();

        let active_agents = agents.values().filter(|a| a.status == "active").count();
        let pending_tasks = tasks.values().filter(|t| t.status == "pending").count();
        let in_progress_tasks = tasks.values().filter(|t| t.status == "in_progress").count();
        let completed_tasks = tasks.values().filter(|t| t.status == "completed").count();

        serde_json::json!({
            "total_agents": agents.len(),
            "active_agents": active_agents,
            "total_tasks": tasks.len(),
            "pending_tasks": pending_tasks,
            "in_progress_tasks": in_progress_tasks,
            "completed_tasks": completed_tasks,
            "total_messages": messages.len(),
            "max_agents": self.config.max_agents,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_agent() {
        let config = CollaborationConfig {
            max_agents: 10,
            default_role: "assistant".to_string(),
            enable_auto_assignment: true,
        };
        let framework = MultiAgentFramework::new(config);

        let id = framework.register_agent(
            "Agent1".to_string(),
            "researcher".to_string(),
            vec!["search".to_string(), "analyze".to_string()],
        );
        assert!(id.is_ok());
        assert_eq!(framework.list_agents().len(), 1);
    }

    #[test]
    fn test_create_and_complete_task() {
        let config = CollaborationConfig {
            max_agents: 10,
            default_role: "assistant".to_string(),
            enable_auto_assignment: true,
        };
        let framework = MultiAgentFramework::new(config);

        let agent1 = framework.register_agent("Agent1".to_string(), "researcher".to_string(), vec![]).unwrap();
        let agent2 = framework.register_agent("Agent2".to_string(), "writer".to_string(), vec![]).unwrap();

        let task_id = framework.create_collaboration_task(
            "Research Task".to_string(),
            "Research and write a report".to_string(),
            vec![agent1, agent2],
        ).unwrap();

        framework.start_task(&task_id).unwrap();
        framework.update_task_progress(&task_id, 50.0).unwrap();
        
        let task = framework.get_task(&task_id).unwrap();
        assert_eq!(task.status, "in_progress");
        assert_eq!(task.progress, 50.0);

        framework.complete_task(&task_id).unwrap();
        let task = framework.get_task(&task_id).unwrap();
        assert_eq!(task.status, "completed");
        assert_eq!(task.progress, 100.0);
    }

    #[test]
    fn test_send_message() {
        let config = CollaborationConfig {
            max_agents: 10,
            default_role: "assistant".to_string(),
            enable_auto_assignment: true,
        };
        let framework = MultiAgentFramework::new(config);

        let agent1 = framework.register_agent("Agent1".to_string(), "researcher".to_string(), vec![]).unwrap();
        let agent2 = framework.register_agent("Agent2".to_string(), "writer".to_string(), vec![]).unwrap();

        let msg_id = framework.send_message(&agent1, &agent2, "Hello!", "text").unwrap();
        assert!(!msg_id.is_empty());

        let messages = framework.get_agent_messages(&agent1);
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "Hello!");
    }
}
