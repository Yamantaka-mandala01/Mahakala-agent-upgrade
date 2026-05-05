use crate::agent::Message;
use std::collections::HashMap;

pub struct PromptBuilder {
    system_prompt: String,
    conversation: Vec<Message>,
    tools: Vec<serde_json::Value>,
    context: HashMap<String, String>,
}

impl PromptBuilder {
    pub fn new(system_prompt: &str) -> Self {
        Self {
            system_prompt: system_prompt.to_string(),
            conversation: Vec::new(),
            tools: Vec::new(),
            context: HashMap::new(),
        }
    }

    pub fn add_message(&mut self, message: Message) {
        self.conversation.push(message);
    }

    pub fn add_tool(&mut self, tool_schema: serde_json::Value) {
        self.tools.push(tool_schema);
    }

    pub fn set_context(&mut self, key: &str, value: &str) {
        self.context.insert(key.to_string(), value.to_string());
    }

    pub fn build_messages(&self) -> Vec<Message> {
        let mut messages = Vec::new();

        // Add system prompt
        if !self.system_prompt.is_empty() {
            let mut system_content = self.system_prompt.clone();
            
            // Add context information
            if !self.context.is_empty() {
                system_content.push_str("\n\n## Context Information\n");
                for (key, value) in &self.context {
                    system_content.push_str(&format!("- {}: {}\n", key, value));
                }
            }

            messages.push(Message {
                role: "system".to_string(),
                content: system_content,
                tool_calls: None,
                tool_call_id: None,
                reasoning: None,
            });
        }

        // Add conversation messages
        messages.extend(self.conversation.clone());

        messages
    }

    pub fn build_tool_schemas(&self) -> Vec<serde_json::Value> {
        self.tools.clone()
    }
}
