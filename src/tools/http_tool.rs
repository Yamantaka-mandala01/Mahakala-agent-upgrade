use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "http_request".to_string(),
        description: "Make HTTP requests (GET, POST, PUT, DELETE) to external APIs".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "http_request",
                "description": "Make HTTP requests (GET, POST, PUT, DELETE) to external APIs",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "method": {
                            "type": "string",
                            "description": "HTTP method (GET, POST, PUT, DELETE)",
                            "enum": ["GET", "POST", "PUT", "DELETE"]
                        },
                        "url": {
                            "type": "string",
                            "description": "URL to send request to"
                        },
                        "headers": {
                            "type": "object",
                            "description": "HTTP headers to include"
                        },
                        "body": {
                            "type": "string",
                            "description": "Request body (for POST/PUT)"
                        }
                    },
                    "required": ["method", "url"]
                }
            }
        }),
        execute: Arc::new(|arguments: &str| {
            let arguments = arguments.to_string();
            Box::pin(async move {
                let args: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

                let method = args.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
                let url = args.get("url").and_then(|v| v.as_str())
                    .ok_or_else(|| anyhow::anyhow!("url is required"))?;

                let client = reqwest::Client::new();
                let mut request = match method {
                    "GET" => client.get(url),
                    "POST" => client.post(url),
                    "PUT" => client.put(url),
                    "DELETE" => client.delete(url),
                    _ => return Err(anyhow::anyhow!("Unsupported HTTP method: {}", method)),
                };

                if let Some(headers) = args.get("headers") {
                    if let Some(obj) = headers.as_object() {
                        for (key, value) in obj {
                            if let Some(val_str) = value.as_str() {
                                request = request.header(key, val_str);
                            }
                        }
                    }
                }

                if let Some(body) = args.get("body") {
                    if let Some(body_str) = body.as_str() {
                        request = request.body(body_str.to_string());
                    }
                }

                let response = request.send().await
                    .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

                let status = response.status();
                let body = response.text().await
                    .map_err(|e| anyhow::anyhow!("Failed to read response body: {}", e))?;

                Ok(format!("Status: {}\n\n{}", status, body))
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}