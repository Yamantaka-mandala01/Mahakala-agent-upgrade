use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::tools::registry::ToolRegistry;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub model: String,
    pub provider: Option<String>,
    pub api_base_url: Option<String>,
    pub api_key: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub call_type: String,
    pub function: ToolFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_call_id: String,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

// 最大工具调用迭代次数，防止无限递归导致栈溢出
const MAX_TOOL_ITERATIONS: usize = 50;

pub struct AIAgent {
    pub config: AgentConfig,
    pub conversation_history: Vec<Message>,
    pub token_usage: TokenUsage,
    pub registry: Arc<ToolRegistry>,
}

impl AIAgent {
    pub fn new(config: AgentConfig, registry: Arc<ToolRegistry>) -> Self {
        Self {
            config,
            conversation_history: Vec::new(),
            token_usage: TokenUsage {
                prompt_tokens: 0,
                completion_tokens: 0,
                total_tokens: 0,
            },
            registry,
        }
    }

    pub async fn process_message(&mut self, user_message: &str) -> anyhow::Result<String> {
        tracing::info!("Processing message: '{}'", user_message);
        tracing::info!("Message bytes: {:?}", user_message.as_bytes());
        
        // Add user message to history
        self.conversation_history.push(Message {
            role: "user".to_string(),
            content: user_message.to_string(),
            tool_calls: None,
            tool_call_id: None,
            reasoning: None,
        });

        // 使用迭代循环替代递归，防止栈溢出
        let mut iteration = 0;
        loop {
            if iteration >= MAX_TOOL_ITERATIONS {
                tracing::warn!("Max tool iterations ({}) reached, stopping", MAX_TOOL_ITERATIONS);
                return Ok("已达到最大工具调用次数限制".to_string());
            }

            tracing::info!("Iteration {}/{}", iteration + 1, MAX_TOOL_ITERATIONS);

            // Build API request
            let tool_schemas = self.registry.get_tool_schemas();
            let body = self.build_api_request(&tool_schemas);

            // Send request
            let client = reqwest::Client::new();
            let response_text = match self.send_api_request(&client, &body).await {
                Ok(text) => text,
                Err(e) => {
                    tracing::error!("API request failed: {}", e);
                    return Err(anyhow::anyhow!("API request failed: {}", e));
                }
            };

            // Parse response
            let response: serde_json::Value = match serde_json::from_str(&response_text) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to parse response: {}", e);
                    return Err(anyhow::anyhow!("Failed to parse response: {}", e));
                }
            };

            // Handle both OpenAI format (choices) and Anthropic format (content)
            let (content, reasoning, tool_calls_value) = if self.is_anthropic_provider() {
                // Anthropic response format
                let content_blocks = response.get("content").and_then(|c| c.as_array());
                let mut text_content = String::new();
                let mut tool_calls = None;
                
                if let Some(blocks) = content_blocks {
                    for block in blocks {
                        let block_type = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        if block_type == "text" {
                            text_content = block.get("text").and_then(|t| t.as_str()).unwrap_or("").to_string();
                        } else if block_type == "tool_use" {
                            let id = block.get("id").and_then(|i| i.as_str()).unwrap_or("").to_string();
                            let name = block.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string();
                            let input = block.get("input").cloned().unwrap_or(serde_json::json!({}));
                            
                            let tool_call = serde_json::json!([{
                                "id": id,
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "arguments": input.to_string()
                                }
                            }]);
                            tool_calls = Some(tool_call);
                        }
                    }
                }
                
                (text_content, None, tool_calls)
            } else {
                // OpenAI-compatible response format
                let choices = response.get("choices").and_then(|c| c.as_array());
                let mut content = String::new();
                let mut reasoning = None;
                let mut tool_calls = None;
                
                if let Some(choices) = choices {
                    if let Some(choice) = choices.first() {
                        if let Some(message) = choice.get("message") {
                            content = message.get("content").and_then(|c| c.as_str()).unwrap_or("").to_string();
                            reasoning = message.get("reasoning_content")
                                .or_else(|| message.get("reasoning"))
                                .and_then(|r| r.as_str())
                                .map(String::from);
                            tool_calls = message.get("tool_calls").cloned();
                        }
                    }
                }
                
                (content, reasoning, tool_calls)
            };

            // Handle tool calls if any
            if let Some(tool_calls) = &tool_calls_value {
                if !tool_calls.is_null() && tool_calls.as_array().is_some_and(|a| !a.is_empty()) {
                    let tool_results = self.execute_tool_calls(tool_calls).await?;

                    // Add assistant message with tool calls
                    if let Ok(calls) = serde_json::from_value::<Vec<ToolCall>>(tool_calls.clone()) {
                        self.conversation_history.push(Message {
                            role: "assistant".to_string(),
                            content: content.clone(),
                            tool_calls: Some(calls),
                            tool_call_id: None,
                            reasoning: reasoning.clone(),
                        });
                    }

                    // Add tool results to conversation
                    for result in tool_results {
                        self.conversation_history.push(Message {
                            role: "tool".to_string(),
                            content: result.content.clone(),
                            tool_calls: None,
                            tool_call_id: Some(result.tool_call_id.clone()),
                            reasoning: None,
                        });
                    }

                    // Continue to next iteration (don't return, let the loop continue)
                    iteration += 1;
                    continue;
                }
            }

            // Check for inline tool calls in content (for Ollama and models that don't support native tool calling)
            let is_ollama_provider = self.is_ollama_provider();
            let processed_content = if is_ollama_provider {
                self.process_inline_tool_calls(&content).await?
            } else {
                content.clone()
            };

            // Add assistant message to history
            self.conversation_history.push(Message {
                role: "assistant".to_string(),
                content: processed_content.clone(),
                tool_calls: None,
                tool_call_id: None,
                reasoning: reasoning.clone(),
            });

            // Update token usage
            if self.is_anthropic_provider() {
                // Anthropic format
                if let Some(usage) = response.get("usage") {
                    self.token_usage.prompt_tokens = usage.get("input_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                    self.token_usage.completion_tokens = usage.get("output_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                    self.token_usage.total_tokens = self.token_usage.prompt_tokens + self.token_usage.completion_tokens;
                }
            } else {
                // OpenAI-compatible format
                if let Some(usage) = response.get("usage") {
                    self.token_usage.prompt_tokens = usage.get("prompt_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                    self.token_usage.completion_tokens = usage.get("completion_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                    self.token_usage.total_tokens = usage.get("total_tokens").and_then(|t| t.as_u64()).unwrap_or(0) as u32;
                }
            }

            return Ok(processed_content);
        }
    }

    fn is_ollama_provider(&self) -> bool {
        self.config.provider.as_deref() == Some("ollama")
    }

    fn is_anthropic_provider(&self) -> bool {
        self.config.provider.as_deref().is_some_and(|p| p.contains("anthropic"))
    }

    fn is_deepseek_provider(&self) -> bool {
        self.config.provider.as_deref().is_some_and(|p| p.contains("deepseek"))
    }

    fn resolve_model_name(&self) -> &str {
        if let Some(ref provider) = self.config.provider {
            if provider.contains('/') {
                return provider;
            }
        }
        if let Some(pos) = self.config.model.find('/') {
            &self.config.model[pos + 1..]
        } else {
            &self.config.model
        }
    }

    fn build_system_prompt(&self) -> String {
        let tools_desc = self.registry.get_tool_schemas();
        let tools_list: Vec<String> = tools_desc.iter()
            .map(|t| format!("- {}: {}", 
                t.get("name").and_then(|n| n.as_str()).unwrap_or(""),
                t.get("description").and_then(|d| d.as_str()).unwrap_or("")
            ))
            .collect();

        format!("You are Mahakala Agent, an intelligent AI assistant. Your role is to help users complete various tasks including code writing, file operations, system management, data analysis, etc.\n\n## Core Capabilities\nYou can use the following tools to help users:\n{}\n\n## Skill System\nYou can call installed skills to perform specific tasks:\n- code_review: Code review\n- ci_cd_pipeline: CI/CD automation\n- creative_writing: Creative writing\n- web_research: Web research\n- document_summary: Document summary\n- email_assistant: Email assistant\n- image_generation: Image generation\n- data_analysis: Data analysis\n\n## Plugin System\nYou can call loaded plugins to extend functionality:\n- disk_cleanup: Disk cleanup\n- network_monitor: Network monitoring\n- memory_plugin: Memory management\n- log_manager: Log management\n- scheduler_plugin: Task scheduling\n- security_scanner: Security scanning\n\n## Working Principles\n1. When user requests require tools, proactively call the appropriate tools\n2. Ensure parameters are correct when using tools\n3. Confirm safety before executing system commands\n4. Confirm path correctness before file operations\n5. Maintain a friendly and professional attitude\n6. Reply to users in Chinese\n\n## Tool Call Format\nWhen you need to call a tool, use the following format:\n<tool_call>\n{{{{\"name\": \"tool_name\", \"arguments\": {{{{\"param\": \"value\"}}}}}}}}\n</tool_call>\n\nCurrent time: {}",
            tools_list.join("\n"),
            chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
        )
    }

    fn build_api_request(&self, tool_schemas: &[serde_json::Value]) -> serde_json::Value {
        let is_ollama = self.is_ollama_provider();
        let is_anthropic = self.is_anthropic_provider();
        let _is_deepseek = self.is_deepseek_provider();

        // Build system prompt with Mahakala Agent role
        let system_prompt = self.build_system_prompt();

        // For Anthropic, we use OpenAI-compatible format (Anthropic supports this)
        let mut messages: Vec<serde_json::Value> = self.conversation_history.iter().map(|m| {
            let mut msg = serde_json::json!({
                "role": m.role,
                "content": m.content,
            });
            if let Some(ref tc) = m.tool_calls {
                msg["tool_calls"] = serde_json::to_value(tc).unwrap_or(serde_json::json!([]));
            }
            if let Some(ref tid) = m.tool_call_id {
                msg["tool_call_id"] = serde_json::Value::String(tid.clone());
            }
            // For DeepSeek, we MUST send reasoning_content back to the API
            if let Some(ref reasoning) = m.reasoning {
                msg["reasoning_content"] = serde_json::Value::String(reasoning.clone());
            }
            msg
        }).collect::<Vec<_>>();

        // Insert system prompt at the beginning if not already present
        if !messages.iter().any(|m| m.get("role").and_then(|r| r.as_str()) == Some("system")) {
            messages.insert(0, serde_json::json!({
                "role": "system",
                "content": system_prompt,
            }));
        }

        // Anthropic requires 'system' role to be different
        if is_anthropic {
            // Convert 'system' role messages to system parameter
            for msg in &mut messages {
                if msg["role"] == "system" {
                    msg["role"] = serde_json::Value::String("user".to_string());
                }
                if msg["role"] == "tool" {
                    msg["role"] = serde_json::Value::String("user".to_string());
                }
            }
        }

        let model_name = self.resolve_model_name();

        let mut body = serde_json::json!({
            "model": model_name,
            "messages": messages,
        });

        // Ollama 的 OpenAI 兼容 API 不支持 tools 参数
        if !tool_schemas.is_empty() && !is_ollama {
            body["tools"] = serde_json::to_value(tool_schemas).unwrap_or(serde_json::json!([]));
        }

        if let Some(temp) = self.config.temperature {
            body["temperature"] = serde_json::json!(temp);
        }

        if let Some(max_tokens) = self.config.max_tokens {
            body["max_tokens"] = serde_json::json!(max_tokens);
        }

        body
    }

    async fn send_api_request(&self, client: &reqwest::Client, body: &serde_json::Value) -> anyhow::Result<String> {
        let is_ollama = self.is_ollama_provider();
        let is_anthropic = self.is_anthropic_provider();

        // Anthropic uses a different endpoint and format
        let (url, request_body) = if is_anthropic {
            let base_url = match &self.config.api_base_url {
                Some(url) => url.trim_end_matches('/').to_string(),
                None => "https://api.anthropic.com/v1".to_string(),
            };
            let url = format!("{}/messages", base_url);
            
            // Convert OpenAI format to Anthropic format
            let messages: Vec<serde_json::Value> = body["messages"]
                .as_array().cloned()
                .unwrap_or_default();
            
            // Extract system message
            let system_content = messages.iter()
                .find(|m| m.get("role").and_then(|r| r.as_str()) == Some("system"))
                .and_then(|m| m.get("content").and_then(|c| c.as_str()))
                .unwrap_or("");
            
            // Filter out system messages from the messages array
            let anthropic_messages: Vec<serde_json::Value> = messages.iter()
                .filter(|m| m.get("role").and_then(|r| r.as_str()) != Some("system"))
                .map(|m| {
                    let role = m.get("role").and_then(|r| r.as_str()).unwrap_or("user");
                    let content = m.get("content").and_then(|c| c.as_str()).unwrap_or("");
                    serde_json::json!({
                        "role": role,
                        "content": content
                    })
                })
                .collect();
            
            let model_name = self.resolve_model_name();
            let mut anthropic_body = serde_json::json!({
                "model": model_name,
                "messages": anthropic_messages,
                "system": system_content,
            });
            
            if let Some(temp) = self.config.temperature {
                anthropic_body["temperature"] = serde_json::json!(temp);
            }
            if let Some(max_tokens) = self.config.max_tokens {
                anthropic_body["max_tokens"] = serde_json::json!(max_tokens);
            }
            
            (url, anthropic_body)
        } else {
            let base_url = match &self.config.api_base_url {
                Some(url) => {
                    if is_ollama && !url.contains("/v1") {
                        format!("{}/v1", url.trim_end_matches('/'))
                    } else {
                        url.clone()
                    }
                }
                None => {
                    if let Some(provider) = &self.config.provider {
                        if provider.starts_with("openai") || provider == "openai" {
                            "https://api.openai.com/v1".to_string()
                        } else if provider.starts_with("ollama") || provider == "ollama" {
                            "http://localhost:11434/v1".to_string()
                        } else if provider.starts_with("deepseek") || provider == "deepseek" {
                            "https://api.deepseek.com/v1".to_string()
                        } else {
                            // 默认使用本地 Ollama
                            "http://localhost:11434/v1".to_string()
                        }
                    } else {
                        // 默认使用本地 Ollama
                        "http://localhost:11434/v1".to_string()
                    }
                }
            };
            (format!("{}/chat/completions", base_url), body.clone())
        };

        tracing::info!("Sending API request to {} with model {}", url, self.config.model);

        let mut request = client.post(&url)
            .header("Content-Type", "application/json");

        // Authentication headers (云端 API 需要 key，Ollama 本地不需要)
        if is_anthropic {
            if let Some(ref api_key) = self.config.api_key {
                request = request
                    .header("x-api-key", api_key)
                    .header("anthropic-version", "2023-06-01");
            }
        } else if !is_ollama {
            if let Some(ref api_key) = self.config.api_key {
                request = request.header("Authorization", format!("Bearer {}", api_key));
            }
        }

        // Additional Anthropic beta header for tool use
        if is_anthropic {
            request = request.header("anthropic-beta", "tools-2024-05-16");
        }

        let body_str = serde_json::to_string(&request_body).unwrap_or_default();
        let truncated = if body_str.len() > 2000 {
            let mut end = 2000;
            while end > 0 && !body_str.is_char_boundary(end) {
                end -= 1;
            }
            &body_str[..end]
        } else {
            &body_str
        };
        tracing::info!("Request body: {}", truncated);

        let response = request
            .json(&request_body)
            .send()
            .await?;

        let status = response.status();
        let text = response.text().await?;

        if !status.is_success() {
            tracing::error!("API error ({}): {}", status, text);
            return Err(anyhow::anyhow!("API error {}: {}", status, text));
        }

        tracing::debug!("Response: {}", &text[..text.len().min(500)]);

        Ok(text)
    }

    async fn execute_tool_calls(&self, tool_calls: &serde_json::Value) -> anyhow::Result<Vec<ToolCallResult>> {
        let mut results = Vec::new();

        if let Some(calls) = tool_calls.as_array() {
            for call in calls {
                if let Some(function) = call.get("function") {
                    let name = function.get("name").and_then(|n| n.as_str()).unwrap_or("");
                    let arguments = function.get("arguments").and_then(|a| a.as_str()).unwrap_or("{}");
                    let call_id = call.get("id").and_then(|id| id.as_str()).unwrap_or("").to_string();

                    tracing::info!("Executing tool: {} with args: {}", name, arguments);

                    match self.registry.execute_tool(name, arguments).await {
                        Ok(result) => {
                            tracing::info!("Tool {} executed successfully", name);
                            results.push(ToolCallResult {
                                tool_call_id: call_id,
                                content: result,
                            });
                        }
                        Err(e) => {
                            tracing::error!("Tool execution failed: {}", e);
                            results.push(ToolCallResult {
                                tool_call_id: call_id,
                                content: format!("Error: {}", e),
                            });
                        }
                    }
                }
            }
        }

        Ok(results)
    }

    /// Process inline tool calls in content (for models that don't support native tool calling)
    async fn process_inline_tool_calls(&self, content: &str) -> anyhow::Result<String> {
        let tool_call_regex = regex::Regex::new(r"<tool_call>\s*(\{.*?\})\s*</tool_call>").unwrap();
        let mut result = content.to_string();
        let mut tool_results = Vec::new();

        for cap in tool_call_regex.captures_iter(content) {
            if let Some(json_str) = cap.get(1) {
                let json_str = json_str.as_str();
                match serde_json::from_str::<serde_json::Value>(json_str) {
                    Ok(tool_call) => {
                        let name = tool_call.get("name").and_then(|n| n.as_str()).unwrap_or("");
                        let arguments = tool_call.get("arguments")
                            .map(|a| a.to_string())
                            .unwrap_or_else(|| "{}".to_string());

                        tracing::info!("Executing inline tool: {} with args: {}", name, arguments);

                        match self.registry.execute_tool(name, &arguments).await {
                            Ok(tool_result) => {
                                tracing::info!("Inline tool {} executed successfully", name);
                                tool_results.push(format!("工具 '{}' 执行结果:\n{}", name, tool_result));
                            }
                            Err(e) => {
                                tracing::error!("Inline tool execution failed: {}", e);
                                tool_results.push(format!("工具 '{}' 执行失败: {}", name, e));
                            }
                        }

                        // Remove the tool call from content
                        result = result.replace(cap.get(0).unwrap().as_str(), "");
                    }
                    Err(e) => {
                        tracing::error!("Failed to parse inline tool call: {}", e);
                    }
                }
            }
        }

        // If tools were executed, add results to the response
        if !tool_results.is_empty() {
            result = format!("{}\n\n{}", result.trim(), tool_results.join("\n\n"));
        }

        Ok(result)
    }
}
