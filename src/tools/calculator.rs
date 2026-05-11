use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "calculator".to_string(),
        description: "Perform mathematical calculations".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "calculator",
                "description": "Perform mathematical calculations",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "expression": {
                            "type": "string",
                            "description": "Mathematical expression to evaluate"
                        }
                    },
                    "required": ["expression"]
                }
            }
        }),
        execute: Arc::new(|args: &str| {
            let args = args.to_string();
            Box::pin(async move {
                let parsed: serde_json::Value = serde_json::from_str(&args)?;
                let expression = parsed.get("expression")
                    .and_then(|e| e.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing 'expression' parameter"))?;
                
                let result = match evalexpr::eval(expression) {
                    Ok(val) => format!("{}", val),
                    Err(e) => format!("Error evaluating expression: {}", e),
                };
                
                Ok(result)
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}
