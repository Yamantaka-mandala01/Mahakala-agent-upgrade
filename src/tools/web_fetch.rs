use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "web_fetch".to_string(),
        description: "Fetch content from a URL".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "web_fetch",
                "description": "Fetch content from a URL",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {
                            "type": "string",
                            "description": "URL to fetch"
                        }
                    },
                    "required": ["url"]
                }
            }
        }),
        execute: Arc::new(|args: &str| {
            let args = args.to_string();
            Box::pin(async move {
                let parsed: serde_json::Value = serde_json::from_str(&args)?;
                let url = parsed.get("url")
                    .and_then(|u| u.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
                
                let client = reqwest::Client::new();
                let response = client.get(url).send().await?;
                let content = response.text().await?;
                
                Ok(content)
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}