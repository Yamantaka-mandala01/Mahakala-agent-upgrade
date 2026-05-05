pub mod registry;
pub mod all_tools;

// 工具模块
pub mod shell_exec;
pub mod file_read;
pub mod file_write;
pub mod file_list;
pub mod web_fetch;
pub mod calculator;
pub mod memory_tool;
pub mod todo;

// 占位符工具 - 后续实现
pub mod file_append {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "file_append".to_string(), description: "Append to file".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod file_search {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "file_search".to_string(), description: "Search files".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod web_search {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "web_search".to_string(), description: "Search web".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod code_execute {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "code_execute".to_string(), description: "Execute code".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod edit {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "edit".to_string(), description: "Edit file".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod replace {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "replace".to_string(), description: "Replace text".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod diff {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "diff".to_string(), description: "Diff files".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod grep {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "grep".to_string(), description: "Search text".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod head_tail {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "head_tail".to_string(), description: "Head/tail".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod wc {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "wc".to_string(), description: "Word count".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod sed {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "sed".to_string(), description: "Stream editor".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod patch {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "patch".to_string(), description: "Apply patch".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod git {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "git".to_string(), description: "Git operations".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod notification {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "notification".to_string(), description: "Send notification".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod tts {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "tts".to_string(), description: "Text to speech".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod vision {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "vision".to_string(), description: "Vision processing".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod url_safety {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "url_safety".to_string(), description: "URL safety check".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod browser_tool {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "browser_tool".to_string(), description: "Browser automation".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}
pub mod skills_tool {
    use super::registry::ToolInfo;
    pub fn create() -> ToolInfo {
        ToolInfo { name: "skills_tool".to_string(), description: "Skills execution".to_string(), schema: serde_json::json!({}), execute: Box::new(|_| Ok("Not implemented".to_string())) }
    }
}

#[cfg(test)]
mod tests {
    use super::registry::ToolRegistry;
    use super::all_tools;

    #[test]
    fn test_tool_registry_registration() {
        let registry = ToolRegistry::new();
        all_tools::register_all_tools(&registry);
        assert!(registry.count() > 0, "Tools should be registered");
    }

    #[tokio::test]
    async fn test_calculator_tool() {
        let registry = ToolRegistry::new();
        all_tools::register_all_tools(&registry);
        
        let result = registry.execute_tool("calculator", r#"{"expression": "2 + 2"}"#).await;
        assert!(result.is_ok(), "Calculator should execute successfully");
        assert_eq!(result.unwrap(), "4", "2 + 2 should equal 4");
    }

    #[tokio::test]
    async fn test_shell_exec_tool() {
        let registry = ToolRegistry::new();
        all_tools::register_all_tools(&registry);
        
        let result = registry.execute_tool("shell_exec", r#"{"command": "echo hello"}"#).await;
        assert!(result.is_ok(), "Shell exec should execute successfully");
        let output = result.unwrap();
        assert!(output.contains("hello"), "Output should contain 'hello'");
    }

    #[test]
    fn test_tool_schemas() {
        let registry = ToolRegistry::new();
        all_tools::register_all_tools(&registry);
        
        let schemas = registry.get_tool_schemas();
        assert!(!schemas.is_empty(), "Tool schemas should not be empty");
        
        for schema in &schemas {
            // OpenAI-compatible format: { "type": "function", "function": { "name": ..., "description": ..., "parameters": ... } }
            assert!(schema.get("type").is_some(), "Each schema should have a type field");
            let function = schema.get("function").expect("Schema should have a function object");
            assert!(function.get("name").is_some(), "Each function should have a name");
            assert!(function.get("description").is_some(), "Each function should have a description");
            assert!(function.get("parameters").is_some(), "Each function should have parameters");
        }
    }
}
