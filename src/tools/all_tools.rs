use super::registry::ToolRegistry;
use crate::tools::{shell_exec, file_read, file_write, file_list, web_fetch, calculator, memory_tool, todo};
use crate::tools::{date, json_tool, text_tool, env_tool, file_search, file_delete, http_tool};

pub fn register_all_tools(registry: &ToolRegistry) {
    registry.register(shell_exec::create());
    registry.register(file_read::create());
    registry.register(file_write::create());
    registry.register(file_list::create());
    registry.register(web_fetch::create());
    registry.register(calculator::create());
    registry.register(memory_tool::create());
    registry.register(todo::create());
    
    registry.register(date::create());
    registry.register(json_tool::create());
    registry.register(text_tool::create());
    registry.register(env_tool::create());
    registry.register(file_search::create());
    registry.register(file_delete::create());
    registry.register(http_tool::create());
}
