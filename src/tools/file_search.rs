use super::registry::ToolInfo;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub fn create() -> ToolInfo {
    ToolInfo {
        name: "file_search".to_string(),
        description: "Search for files and directories by name or pattern".to_string(),
        schema: serde_json::json!({
            "type": "function",
            "function": {
                "name": "file_search",
                "description": "Search for files and directories by name or pattern",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Directory to search in (default: current directory)"
                        },
                        "pattern": {
                            "type": "string",
                            "description": "File name pattern to search for"
                        },
                        "file_type": {
                            "type": "string",
                            "description": "Filter by file type (file, directory, all)",
                            "enum": ["file", "directory", "all"]
                        },
                        "max_depth": {
                            "type": "integer",
                            "description": "Maximum search depth (default: 3)"
                        }
                    },
                    "required": ["pattern"]
                }
            }
        }),
        execute: Arc::new(|arguments: &str| {
            let arguments = arguments.to_string();
            Box::pin(async move {
                let args: serde_json::Value = serde_json::from_str(&arguments)
                    .map_err(|e| anyhow::anyhow!("Invalid arguments: {}", e))?;

                let search_path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("");
                let file_type = args.get("file_type").and_then(|v| v.as_str()).unwrap_or("all");
                let max_depth = args.get("max_depth").and_then(|v| v.as_u64()).unwrap_or(3) as usize;

                let mut results = Vec::new();
                
                for entry in walkdir::WalkDir::new(search_path)
                    .max_depth(max_depth)
                    .into_iter()
                    .filter_map(|e| e.ok())
                {
                    let file_name = entry.file_name().to_string_lossy().to_string();
                    
                    if !file_name.contains(pattern) {
                        continue;
                    }

                    let is_dir = entry.file_type().is_dir();
                    
                    match file_type {
                        "file" if is_dir => continue,
                        "directory" if !is_dir => continue,
                        _ => {}
                    }

                    let path = entry.path();
                    let size = if is_dir {
                        "N/A".to_string()
                    } else {
                        match tokio::fs::metadata(path).await {
                            Ok(m) => format!("{} bytes", m.len()),
                            Err(_) => "Unknown".to_string(),
                        }
                    };

                    results.push(format!("{} ({}) - {}", path.display(), if is_dir { "dir" } else { "file" }, size));
                }

                if results.is_empty() {
                    Ok(format!("No files found matching pattern: '{}'", pattern))
                } else {
                    Ok(format!("Found {} files:\n{}", results.len(), results.join("\n")))
                }
            }) as Pin<Box<dyn Future<Output = anyhow::Result<String>> + Send>>
        }),
    }
}