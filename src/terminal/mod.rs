use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerContainer {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub ports: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub host: String,
    pub port: u16,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshConnection {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub connected: bool,
    pub last_activity: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSession {
    pub id: String,
    pub session_type: String,
    pub target: String,
    pub output: Vec<String>,
    pub is_active: bool,
    pub created_at: i64,
}

pub struct DockerManager {
    containers: Arc<Mutex<HashMap<String, DockerContainer>>>,
    config: DockerConfig,
}

impl DockerManager {
    pub fn new(config: DockerConfig) -> Self {
        Self {
            containers: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn list_containers(&self) -> Vec<DockerContainer> {
        let containers = self.containers.lock();
        containers.values().cloned().collect()
    }

    pub fn create_container(&self, name: String, image: String) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let container = DockerContainer {
            id: id.clone(),
            name,
            image,
            status: "created".to_string(),
            ports: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
        };

        let mut containers = self.containers.lock();
        containers.insert(id.clone(), container);
        Ok(id)
    }

    pub fn start_container(&self, id: &str) -> Result<(), AppError> {
        let mut containers = self.containers.lock();
        if let Some(container) = containers.get_mut(id) {
            container.status = "running".to_string();
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Container {} not found", id)))
        }
    }

    pub fn stop_container(&self, id: &str) -> Result<(), AppError> {
        let mut containers = self.containers.lock();
        if let Some(container) = containers.get_mut(id) {
            container.status = "stopped".to_string();
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Container {} not found", id)))
        }
    }

    pub fn remove_container(&self, id: &str) -> Result<(), AppError> {
        let mut containers = self.containers.lock();
        containers.remove(id);
        Ok(())
    }

    pub fn exec_in_container(&self, container_id: &str, command: &str) -> Result<String, AppError> {
        let containers = self.containers.lock();
        if containers.contains_key(container_id) {
            Ok(format!("Executed '{}' in container {}", command, container_id))
        } else {
            Err(AppError::NotFound(format!("Container {} not found", container_id)))
        }
    }

    pub fn get_container_logs(&self, container_id: &str) -> Result<String, AppError> {
        let containers = self.containers.lock();
        if containers.contains_key(container_id) {
            Ok(format!("Logs for container {}", container_id))
        } else {
            Err(AppError::NotFound(format!("Container {} not found", container_id)))
        }
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let containers = self.containers.lock();
        let total = containers.len();
        let running = containers.values().filter(|c| c.status == "running").count();
        let stopped = containers.values().filter(|c| c.status == "stopped").count();

        serde_json::json!({
            "total_containers": total,
            "running": running,
            "stopped": stopped,
            "host": self.config.host,
            "port": self.config.port,
        })
    }
}

pub struct SshManager {
    connections: Arc<Mutex<HashMap<String, SshConnection>>>,
    sessions: Arc<Mutex<HashMap<String, TerminalSession>>>,
}

impl SshManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_connection(&self, host: String, port: u16, username: String) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let connection = SshConnection {
            id: id.clone(),
            host,
            port,
            username,
            connected: false,
            last_activity: None,
        };

        let mut connections = self.connections.lock();
        connections.insert(id.clone(), connection);
        Ok(id)
    }

    pub fn remove_connection(&self, id: &str) -> Result<bool, AppError> {
        let mut connections = self.connections.lock();
        Ok(connections.remove(id).is_some())
    }

    pub fn connect(&self, id: &str) -> Result<(), AppError> {
        let mut connections = self.connections.lock();
        if let Some(conn) = connections.get_mut(id) {
            conn.connected = true;
            conn.last_activity = Some(chrono::Utc::now().timestamp());
            Ok(())
        } else {
            Err(AppError::NotFound(format!("SSH connection {} not found", id)))
        }
    }

    pub fn disconnect(&self, id: &str) -> Result<(), AppError> {
        let mut connections = self.connections.lock();
        if let Some(conn) = connections.get_mut(id) {
            conn.connected = false;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("SSH connection {} not found", id)))
        }
    }

    pub fn list_connections(&self) -> Vec<SshConnection> {
        let connections = self.connections.lock();
        connections.values().cloned().collect()
    }

    pub fn execute_command(&self, connection_id: &str, command: &str) -> Result<String, AppError> {
        let connections = self.connections.lock();
        if let Some(conn) = connections.get(connection_id) {
            if !conn.connected {
                return Err(AppError::Internal(format!("SSH connection {} is not connected", connection_id)));
            }

            let session_id = uuid::Uuid::new_v4().to_string();
            let session = TerminalSession {
                id: session_id.clone(),
                session_type: "ssh".to_string(),
                target: format!("{}@{}:{}", conn.username, conn.host, conn.port),
                output: vec![format!("$ {}", command), "Command executed successfully".to_string()],
                is_active: false,
                created_at: chrono::Utc::now().timestamp(),
            };

            let mut sessions = self.sessions.lock();
            sessions.insert(session_id.clone(), session);

            Ok(format!("Command '{}' executed on {}@{}:{}", command, conn.username, conn.host, conn.port))
        } else {
            Err(AppError::NotFound(format!("SSH connection {} not found", connection_id)))
        }
    }

    pub fn get_session(&self, session_id: &str) -> Option<TerminalSession> {
        let sessions = self.sessions.lock();
        sessions.get(session_id).cloned()
    }

    pub fn list_sessions(&self) -> Vec<TerminalSession> {
        let sessions = self.sessions.lock();
        sessions.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_create_container() {
        let config = DockerConfig {
            host: "localhost".to_string(),
            port: 2375,
            api_version: "1.41".to_string(),
        };
        let manager = DockerManager::new(config);

        let id = manager.create_container("test_container".to_string(), "nginx:latest".to_string());
        assert!(id.is_ok());
        assert_eq!(manager.list_containers().len(), 1);
    }

    #[test]
    fn test_docker_start_stop_container() {
        let config = DockerConfig {
            host: "localhost".to_string(),
            port: 2375,
            api_version: "1.41".to_string(),
        };
        let manager = DockerManager::new(config);

        let id = manager.create_container("test_container".to_string(), "nginx:latest".to_string()).unwrap();
        
        manager.start_container(&id).unwrap();
        let containers = manager.list_containers();
        assert_eq!(containers[0].status, "running");

        manager.stop_container(&id).unwrap();
        let containers = manager.list_containers();
        assert_eq!(containers[0].status, "stopped");
    }

    #[test]
    fn test_ssh_add_connection() {
        let manager = SshManager::new();

        let id = manager.add_connection("192.168.1.100".to_string(), 22, "user".to_string());
        assert!(id.is_ok());
        assert_eq!(manager.list_connections().len(), 1);
    }

    #[test]
    fn test_ssh_connect_disconnect() {
        let manager = SshManager::new();

        let id = manager.add_connection("192.168.1.100".to_string(), 22, "user".to_string()).unwrap();
        
        manager.connect(&id).unwrap();
        let connections = manager.list_connections();
        assert!(connections[0].connected);

        manager.disconnect(&id).unwrap();
        let connections = manager.list_connections();
        assert!(!connections[0].connected);
    }
}
