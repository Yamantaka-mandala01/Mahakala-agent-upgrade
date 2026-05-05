use super::registry::ToolRegistry;
use crate::tools::{shell_exec, file_read, file_write, file_list, web_fetch, calculator, memory_tool, todo};

pub fn register_all_tools(registry: &ToolRegistry) {
    registry.register(shell_exec::create());
    registry.register(file_read::create());
    registry.register(file_write::create());
    registry.register(file_list::create());
    registry.register(web_fetch::create());
    registry.register(calculator::create());
    registry.register(memory_tool::create());
    registry.register(todo::create());
}
