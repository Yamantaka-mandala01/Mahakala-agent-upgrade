use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServer {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub connected: bool,
    pub tools: Vec<McpTool>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    pub server_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

pub struct McpClient {
    servers: Arc<Mutex<HashMap<String, McpServer>>>,
}

impl McpClient {
    pub fn new() -> Self {
        Self {
            servers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_server(&self, id: String, name: String, url: String, description: String) -> Result<(), AppError> {
        let server = McpServer {
            id: id.clone(),
            name,
            url,
            description,
            connected: false,
            tools: Vec::new(),
            created_at: chrono::Utc::now().timestamp(),
        };

        let mut servers = self.servers.lock();
        servers.insert(id, server);
        Ok(())
    }

    pub fn remove_server(&self, id: &str) -> Result<bool, AppError> {
        let mut servers = self.servers.lock();
        Ok(servers.remove(id).is_some())
    }

    pub fn connect_server(&self, id: &str) -> Result<Vec<McpTool>, AppError> {
        let mut servers = self.servers.lock();
        if let Some(server) = servers.get_mut(id) {
            server.connected = true;
            
            let mock_tools = vec![
                McpTool {
                    name: format!("{}_search", server.id),
                    description: format!("Search using {}", server.name),
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "query": { "type": "string", "description": "Search query" }
                        },
                        "required": ["query"]
                    }),
                },
                McpTool {
                    name: format!("{}_fetch", server.id),
                    description: format!("Fetch content using {}", server.name),
                    schema: serde_json::json!({
                        "type": "object",
                        "properties": {
                            "url": { "type": "string", "description": "URL to fetch" }
                        },
                        "required": ["url"]
                    }),
                },
            ];

            server.tools = mock_tools.clone();
            Ok(mock_tools)
        } else {
            Err(AppError::NotFound(format!("MCP server {} not found", id)))
        }
    }

    pub fn disconnect_server(&self, id: &str) -> Result<(), AppError> {
        let mut servers = self.servers.lock();
        if let Some(server) = servers.get_mut(id) {
            server.connected = false;
            server.tools.clear();
            Ok(())
        } else {
            Err(AppError::NotFound(format!("MCP server {} not found", id)))
        }
    }

    pub fn list_servers(&self) -> Vec<McpServer> {
        let servers = self.servers.lock();
        servers.values().cloned().collect()
    }

    pub fn get_server(&self, id: &str) -> Option<McpServer> {
        let servers = self.servers.lock();
        servers.get(id).cloned()
    }

    pub fn get_all_tools(&self) -> Vec<McpTool> {
        let servers = self.servers.lock();
        servers.values()
            .filter(|s| s.connected)
            .flat_map(|s| s.tools.clone())
            .collect()
    }

    pub async fn execute_tool(&self, call: McpToolCall) -> Result<String, AppError> {
        let servers = self.servers.lock();
        if let Some(server) = servers.get(&call.server_id) {
            if !server.connected {
                return Err(AppError::Internal(format!("MCP server {} is not connected", call.server_id)));
            }

            let tool = server.tools.iter().find(|t| t.name == call.tool_name);
            if tool.is_none() {
                return Err(AppError::NotFound(format!("Tool {} not found in server {}", call.tool_name, call.server_id)));
            }

            let result = format!("MCP tool {} executed with args: {}", call.tool_name, call.arguments);
            Ok(result)
        } else {
            Err(AppError::NotFound(format!("MCP server {} not found", call.server_id)))
        }
    }

    pub fn get_server_count(&self) -> usize {
        self.servers.lock().len()
    }

    pub fn get_connected_count(&self) -> usize {
        self.servers.lock().values().filter(|s| s.connected).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_server() {
        let client = McpClient::new();
        let result = client.add_server(
            "test_server".to_string(),
            "Test Server".to_string(),
            "http://localhost:3000".to_string(),
            "Test MCP server".to_string(),
        );
        assert!(result.is_ok());
        assert_eq!(client.get_server_count(), 1);
    }

    #[test]
    fn test_connect_server() {
        let client = McpClient::new();
        client.add_server(
            "test_server".to_string(),
            "Test Server".to_string(),
            "http://localhost:3000".to_string(),
            "Test MCP server".to_string(),
        ).unwrap();

        let tools = client.connect_server("test_server");
        assert!(tools.is_ok());
        assert!(!tools.unwrap().is_empty());
    }

    #[test]
    fn test_list_servers() {
        let client = McpClient::new();
        let servers = client.list_servers();
        assert!(servers.is_empty());

        client.add_server(
            "server1".to_string(),
            "Server 1".to_string(),
            "http://localhost:3001".to_string(),
            "Server 1".to_string(),
        ).unwrap();

        let servers = client.list_servers();
        assert_eq!(servers.len(), 1);
    }
}
