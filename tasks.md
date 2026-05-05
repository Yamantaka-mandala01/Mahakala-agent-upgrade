# Mahakala Agent Upgrade - 任务分解

## 阶段 1: 基础架构 (核心)

### 1.1 项目初始化
- [ ] 创建 Cargo.toml
- [ ] 创建目录结构
- [ ] 配置 rust-embed 嵌入资源

### 1.2 核心模块
- [ ] src/main.rs - 入口文件
- [ ] src/lib.rs - 库入口
- [ ] src/constants.rs - 常量定义
- [ ] src/config.rs - 配置管理 (YAML + 环境变量)
- [ ] src/logging.rs - 日志系统 (tracing)
- [ ] src/error.rs - 错误处理

### 1.3 AI Agent 核心
- [ ] src/agent/mod.rs
- [ ] src/agent/core.rs - Agent 核心 (对话管理、工具调用)
- [ ] src/agent/prompt_builder.rs - 提示语构建
- [ ] src/agent/context_engine.rs - 上下文引擎
- [ ] src/agent/memory_manager.rs - 记忆管理
- [ ] src/agent/title_generator.rs - 标题生成
- [ ] src/agent/error_classifier.rs - 错误分类
- [ ] src/agent/trajectory.rs - 轨迹记录

## 阶段 2: 工具系统

### 2.1 工具框架
- [ ] src/tools/mod.rs
- [ ] src/tools/registry.rs - 工具注册中心

### 2.2 文件工具 (8个)
- [ ] src/tools/file_read.rs
- [ ] src/tools/file_write.rs
- [ ] src/tools/file_append.rs
- [ ] src/tools/file_list.rs
- [ ] src/tools/file_search.rs
- [ ] src/tools/edit.rs
- [ ] src/tools/replace.rs
- [ ] src/tools/diff.rs

### 2.3 Shell/终端工具 (6个)
- [ ] src/tools/shell_exec.rs
- [ ] src/tools/head_tail.rs
- [ ] src/tools/wc.rs
- [ ] src/tools/grep.rs
- [ ] src/tools/sed.rs
- [ ] src/tools/patch.rs

### 2.4 网络工具 (4个)
- [ ] src/tools/web_fetch.rs
- [ ] src/tools/web_search.rs
- [ ] src/tools/url_safety.rs
- [ ] src/tools/browser_tool.rs

### 2.5 其他工具 (8个)
- [ ] src/tools/calculator.rs
- [ ] src/tools/code_execute.rs
- [ ] src/tools/memory_tool.rs
- [ ] src/tools/skills_tool.rs
- [ ] src/tools/tts.rs
- [ ] src/tools/vision.rs
- [ ] src/tools/todo.rs
- [ ] src/tools/notification.rs

## 阶段 3: 技能系统

- [ ] src/skills/mod.rs
- [ ] src/skills/registry.rs
- [ ] src/skills/executor.rs
- [ ] src/skills/installer.rs

## 阶段 4: 插件系统

- [ ] src/plugins/mod.rs
- [ ] src/plugins/registry.rs
- [ ] src/plugins/loader.rs

## 阶段 5: 记忆系统

- [ ] src/memory/mod.rs
- [ ] src/memory/store.rs
- [ ] src/memory/providers/mod.rs
- [ ] src/memory/providers/honcho.rs
- [ ] src/memory/providers/mem0.rs

## 阶段 6: 平台集成 (网关)

### 6.1 网关核心
- [ ] src/gateway/mod.rs
- [ ] src/gateway/session.rs
- [ ] src/gateway/delivery.rs

### 6.2 平台适配器 (8个)
- [ ] src/gateway/platforms/mod.rs
- [ ] src/gateway/platforms/base.rs
- [ ] src/gateway/platforms/whatsapp.rs
- [ ] src/gateway/platforms/telegram.rs
- [ ] src/gateway/platforms/discord.rs
- [ ] src/gateway/platforms/slack.rs
- [ ] src/gateway/platforms/email.rs
- [ ] src/gateway/platforms/sms.rs
- [ ] src/gateway/platforms/wechat.rs
- [ ] src/gateway/platforms/qqbot.rs
- [ ] src/gateway/platforms/api_server.rs

## 阶段 7: Web 服务器

### 7.1 服务器框架
- [ ] src/web_server/mod.rs
- [ ] src/web_server/routes.rs

### 7.2 API 端点
- [ ] src/web_server/auth.rs - 认证 API
- [ ] src/web_server/sessions.rs - 会话 API
- [ ] src/web_server/workspace.rs - 工作区 API
- [ ] src/web_server/streaming.rs - 流式响应 (SSE)
- [ ] src/web_server/upload.rs - 文件上传
- [ ] src/web_server/config_api.rs - 配置 API
- [ ] src/web_server/commands_api.rs - 命令 API
- [ ] src/web_server/models_api.rs - 模型 API
- [ ] src/web_server/gateway_api.rs - 网关 API
- [ ] src/web_server/updates_api.rs - 更新 API
- [ ] src/web_server/state_sync.rs - 状态同步
- [ ] src/web_server/clarify.rs - 澄清 API
- [ ] src/web_server/onboarding.rs - 引导 API

## 阶段 8: 认证系统

- [ ] src/auth/mod.rs
- [ ] src/auth/jwt.rs
- [ ] src/auth/session.rs

## 阶段 9: 工作区管理

- [ ] src/workspace/mod.rs
- [ ] src/workspace/manager.rs

## 阶段 10: 定时任务

- [ ] src/cron/mod.rs
- [ ] src/cron/scheduler.rs

## 阶段 11: 国际化

- [ ] src/i18n/mod.rs
- [ ] src/i18n/translations.rs

## 阶段 12: 文件上传

- [ ] src/upload/mod.rs
- [ ] src/upload/handler.rs

## 阶段 13: 状态同步

- [ ] src/state/mod.rs
- [ ] src/state/sync.rs

## 阶段 14: CLI 接口

- [ ] src/cli/mod.rs
- [ ] src/cli/commands.rs

## 阶段 15: 前端 UI

### 15.1 HTML/CSS
- [ ] webui/index.html
- [ ] webui/styles/main.css
- [ ] webui/styles/light.css
- [ ] webui/styles/dark.css

### 15.2 JavaScript
- [ ] webui/js/main.js
- [ ] webui/js/sessions.js
- [ ] webui/js/workspace.js
- [ ] webui/js/messages.js
- [ ] webui/js/commands.js
- [ ] webui/js/login.js
- [ ] webui/js/onboarding.js
- [ ] webui/js/i18n.js
- [ ] webui/js/icons.js

### 15.3 资源
- [ ] webui/assets/logo-dark.png
- [ ] webui/assets/logo-light.png
- [ ] webui/assets/favicon.ico

## 阶段 16: 构建和测试

- [ ] 配置 Cargo.toml (release profile, lto)
- [ ] 配置 build.rs (嵌入资源)
- [ ] 嵌入 Windows 图标
- [ ] 编译为 .exe
- [ ] 集成测试
- [ ] 功能验证
