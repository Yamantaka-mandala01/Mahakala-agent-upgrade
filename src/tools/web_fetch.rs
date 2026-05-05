use super::registry::ToolInfo;

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
        execute: Box::new(|args: &str| {
            let parsed: serde_json::Value = serde_json::from_str(args)?;
            let url = parsed.get("url")
                .and_then(|u| u.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'url' parameter"))?;
            
            let rt = tokio::runtime::Runtime::new()?;
            let content = rt.block_on(async {
                let client = reqwest::Client::new();
                let response = client.get(url).send().await?;
                response.text().await
            })?;
            
            Ok(content)
        }),
    }
}
