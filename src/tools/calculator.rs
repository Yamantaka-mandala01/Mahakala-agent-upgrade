use super::registry::ToolInfo;

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
        execute: Box::new(|args: &str| {
            let parsed: serde_json::Value = serde_json::from_str(args)?;
            let expression = parsed.get("expression")
                .and_then(|e| e.as_str())
                .ok_or_else(|| anyhow::anyhow!("Missing 'expression' parameter"))?;
            
            // 简单计算器实现
            let result = match evalexpr::eval(expression) {
                Ok(val) => format!("{}", val),
                Err(e) => format!("Error evaluating expression: {}", e),
            };
            
            Ok(result)
        }),
    }
}
