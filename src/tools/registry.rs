use std::collections::HashMap;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;
use parking_lot::RwLock;

pub type ToolFn = Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>> + Send + Sync>;

pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub schema: serde_json::Value,
    pub execute: ToolFn,
}

pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, ToolInfo>>>,
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, tool: ToolInfo) {
        let name = tool.name.clone();
        self.tools.write().insert(name, tool);
    }

    pub async fn execute_tool(&self, name: &str, arguments: &str) -> anyhow::Result<String> {
        let tool_fn = {
            let tools = self.tools.read();
            let tool = tools.get(name)
                .ok_or_else(|| anyhow::anyhow!("Tool not found: {}", name))?;
            // Clone the Arc<ToolFn> to avoid holding the lock across await
            tool.execute.clone()
        };
        tool_fn(arguments).await
    }

    pub fn get_tool_schemas(&self) -> Vec<serde_json::Value> {
        let tools = self.tools.read();
        tools.values().map(|t| {
            // Return the schema directly in OpenAI-compatible format
            // The schema already contains: { "type": "function", "function": { "name": ..., "description": ..., "parameters": ... } }
            t.schema.clone()
        }).collect()
    }

    pub fn count(&self) -> usize {
        self.tools.read().len()
    }
}

impl Clone for ToolRegistry {
    fn clone(&self) -> Self {
        Self {
            tools: Arc::clone(&self.tools),
        }
    }
}
