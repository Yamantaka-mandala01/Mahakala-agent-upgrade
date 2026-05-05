# Mahakala Agent

<div align="center">

![Mahakala Agent](resources/mahakala_icon.png)

**A powerful AI Agent built with Rust - featuring WebUI, tool execution, skills, plugins, and multi-platform integration**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/axum-0.7-green.svg)](https://github.com/tokio-rs/axum)
[![Contact](https://img.shields.io/badge/contact-mahakala.hum.pate@gmail.com-red.svg)](mailto:mahakala.hum.pate@gmail.com)

</div>

---

## Overview

Mahakala Agent is a comprehensive AI agent system implemented entirely in Rust. It provides a full-featured web interface, tool execution engine, skill system, plugin architecture, and multi-platform messaging integration.

Originally designed as a Rust port of the Hermes Agent ecosystem, Mahakala Agent delivers a production-ready AI assistant with modern web UI, supporting multiple AI providers including OpenAI, Anthropic Claude, DeepSeek, and local Ollama models.

---

## Comparison with Hermes Agent & OpenClaw

### Positioning Overview

| Feature | Mahakala Agent | Hermes Agent | OpenClaw |
|---------|---------------|--------------|----------|
| **Language** | Rust | Python | Python |
| **Architecture** | Web-First (Axum + SPA) | CLI-First + Gateway | Gateway-First |
| **UI** | Modern WebUI (dark/light theme) | TUI (Terminal UI) | CLI + Messaging Platforms |
| **Deployment** | Single binary, zero dependencies | Python env + dependencies | Python env + dependencies |
| **Memory** | SQLite-based persistent storage | FTS5 + LLM summarization | SQLite + session files |
| **Tools** | 8 built-in + extensible registry | 40+ built-in tools | Community-driven skills |
| **Skills** | Pre-built skill templates | Autonomous skill creation | agentskills.io compatible |
| **Plugins** | Dynamic filesystem loading | Plugin-based memory providers | Gateway plugins |
| **Platforms** | WeChat, extensible gateway | 15+ messaging platforms | 50+ messaging platforms |
| **Local AI** | Ollama native support | Any OpenAI-compatible endpoint | Any OpenAI-compatible endpoint |
| **Thinking Mode** | DeepSeek reasoning support | No specific thinking mode | No specific thinking mode |
| **File Upload** | Multipart upload API | Terminal file operations | File tool |
| **Voice Input** | Web Speech API | Voice mode (CLI/Telegram) | Voice mode |
| **Cron** | Built-in tokio-cron-scheduler | Built-in cron delivery | Built-in cron |
| **WebSocket** | Real-time streaming | SSE + WebSocket | Platform-specific |
| **i18n** | Built-in (zh/en) | English only | English + community translations |

### Key Differences

#### vs Hermes Agent

| Aspect | Hermes Agent | Mahakala Agent |
|--------|-------------|----------------|
| **Learning Loop** | Closed learning loop with autonomous skill creation and self-improvement | Manual skill management, no autonomous learning yet |
| **Terminal Backends** | 6 backends (local, Docker, SSH, Daytona, Singularity, Modal) | Local execution only |
| **Memory System** | FTS5 full-text search with LLM summarization, Honcho dialectic user modeling | SQLite-based fact storage, no semantic search |
| **Subagents** | Spawn isolated subagents for parallel workstreams | No subagent support |
| **MCP Integration** | Full MCP server support | No MCP support |
| **Browser Automation** | Interactive browser with vision support | No browser automation |
| **Research Features** | Batch trajectory export, RL training with Atropos | No research features |
| **Serverless** | Daytona/Modal serverless persistence | No serverless support |

#### vs OpenClaw

| Aspect | OpenClaw | Mahakala Agent |
|--------|----------|----------------|
| **Architecture** | Gateway-First centralized control | Web-First with integrated backend |
| **Platform Coverage** | 50+ messaging platforms | WeChat + extensible |
| **Skill Ecosystem** | agentskills.io compatible, community-driven | Pre-built templates, no community hub |
| **Multi-Agent** | Multi-agent collaboration support | Single agent only |
| **Enterprise** | Enterprise-grade deployment | Development/Personal use |
| **Cloud Hosting** | Multiple cloud options | Self-hosted only |

### Mahakala Agent's Unique Innovations

1. **Rust-Native Performance**: Built entirely in Rust with zero Python dependencies, offering superior memory efficiency and startup speed compared to Python-based alternatives.

2. **Single Binary Deployment**: `cargo build --release` produces a single executable with all assets embedded - no virtual environments, no pip installs, no dependency conflicts.

3. **Web-First Design**: Unlike Hermes Agent's TUI or OpenClaw's CLI-first approach, Mahakala Agent provides a modern, responsive single-page web application with real-time WebSocket communication.

4. **DeepSeek Thinking Mode**: Native support for DeepSeek's reasoning/thinking mode with `reasoning_content` passthrough, enabling transparent chain-of-thought debugging.

5. **Ollama Native Integration**: First-class support for local Ollama models without requiring API keys, making it the most accessible option for offline AI usage.

6. **Integrated WeChat SDK**: Built-in WeChat QR code generation and messaging support, targeting the Chinese messaging ecosystem that Hermes Agent and OpenClaw don't natively support.

7. **Real-Time Output & Debug Panels**: Built-in output logging and debug panels in the WebUI for transparent monitoring of agent operations, API calls, and tool execution.

8. **Voice Input via Web Speech API**: Browser-native voice input without additional dependencies or external services.

9. **Multipart File Upload**: Native file upload with MIME type detection and persistent storage, accessible through the WebUI toolbar.

10. **Configurable Tool Iteration Limits**: Adjustable maximum tool call iterations (default 50) to prevent infinite recursion while allowing complex multi-step operations.

---

## Features

### AI Agent Core
- **Multi-Provider Support**: OpenAI, Anthropic Claude, DeepSeek, and local Ollama models
- **Tool Calling**: Native tool calling with OpenAI-compatible schema format
- **Thinking Mode**: Full support for DeepSeek reasoning/thinking mode with `reasoning_content`
- **Conversation History**: Persistent conversation management with SQLite storage
- **Streaming Responses**: Real-time streaming output via Server-Sent Events (SSE)
- **System Prompt**: Customizable system prompt defining agent role and capabilities

### Tool System (8 Built-in Tools)
| Tool | Description |
|------|-------------|
| `file_read` | Read file contents |
| `file_write` | Write content to files |
| `file_list` | List directory contents |
| `web_fetch` | Fetch content from URLs |
| `calculator` | Mathematical calculations |
| `shell_exec` | Execute shell commands |
| `memory` | Store and retrieve information |
| `todo` | Manage todo lists |

### Skill System
- Pre-built skills for common tasks (code review, CI/CD, creative writing, web research, etc.)
- Skill execution engine with parameter validation
- Skill management through WebUI

### Plugin System
- Dynamic plugin loading from filesystem
- Plugin manifest-based discovery
- Built-in plugins: disk cleanup, network monitoring, memory management, log management, scheduler, security scanner

### WebUI Interface
- Modern, responsive single-page application
- Dark/Light theme support
- Multi-language support (i18n)
- Real-time chat with streaming
- Tool management panel
- Skill management panel
- Plugin management panel
- Memory/facts management
- Cron job scheduling
- Settings and configuration
- **Output Panel**: Real-time operation logging
- **Debug Panel**: Detailed API request/response tracing
- **Quick Actions**: Code execution, web search, file management, terminal access

### WeChat Integration
- QR code generation for WeChat login
- PNG image saving for QR codes
- WeChat messaging support

### Additional Features
- **Cron Scheduler**: Automated task scheduling
- **Memory System**: SQLite-based persistent memory storage
- **Session Management**: Multiple conversation sessions
- **Token Usage Tracking**: Monitor API token consumption
- **Development Mode**: Hot-reload support for frontend assets
- **File Upload**: Multipart file upload with MIME type detection
- **Voice Input**: Browser-native speech recognition via Web Speech API

---

## Tech Stack

- **Language**: Rust 2021 Edition
- **Web Framework**: Axum 0.7 with WebSocket support
- **HTTP Client**: reqwest 0.12
- **Database**: SQLite (rusqlite)
- **Serialization**: serde + serde_json
- **Async Runtime**: Tokio
- **Logging**: tracing + tracing-subscriber
- **Cron**: tokio-cron-scheduler
- **Authentication**: JWT (jsonwebtoken) + Argon2

---

## Quick Start

### Prerequisites

- Rust 1.70 or later
- (Optional) Ollama for local AI models
- (Optional) API keys for cloud providers (OpenAI, Anthropic, DeepSeek)

### Build

```bash
cd Mahakala-agent-upgrade
cargo build --release
```

### Run

```bash
cargo run --release
```

The web server will start at `http://127.0.0.1:3000`

### Configuration

Configuration is managed through the WebUI settings panel. Supported AI providers:

| Provider | API Base URL | Notes |
|----------|-------------|-------|
| OpenAI | `https://api.openai.com/v1` | Requires API key |
| Anthropic | `https://api.anthropic.com/v1` | Requires API key |
| DeepSeek | `https://api.deepseek.com/v1` | Requires API key |
| Ollama | `http://localhost:11434/v1` | No API key needed |

---

## Project Structure

```
Mahakala-agent-upgrade/
├── Cargo.toml                  # Project manifest
├── Cargo.lock
├── src/
│   ├── main.rs                 # Application entry point
│   ├── lib.rs                  # Library exports
│   ├── config.rs               # Configuration management
│   ├── constants.rs            # Constants and defaults
│   ├── logging.rs              # Logging setup
│   ├── error.rs                # Error types
│   ├── agent/
│   │   ├── core.rs             # AI agent core logic
│   │   ├── mod.rs
│   │   └── prompt_builder.rs   # System prompt builder
│   ├── tools/
│   │   ├── mod.rs              # Tool module exports
│   │   ├── registry.rs         # Tool registration & schemas
│   │   ├── all_tools.rs        # Bulk tool registration
│   │   ├── calculator.rs       # Math calculator tool
│   │   ├── file_list.rs        # Directory listing tool
│   │   ├── file_read.rs        # File reading tool
│   │   ├── file_write.rs       # File writing tool
│   │   ├── memory_tool.rs      # Memory storage tool
│   │   ├── shell_exec.rs       # Shell command execution
│   │   ├── todo.rs             # Todo list management
│   │   └── web_fetch.rs        # URL content fetching
│   ├── skills/
│   │   ── mod.rs              # Skill management
│   ├── plugins/
│   │   └── mod.rs              # Plugin system
│   ├── memory/
│   │   └── mod.rs              # Memory manager (SQLite)
│   ├── cron/
│   │   └── mod.rs              # Cron scheduler
│   ├── wechat/
│   │   ── mod.rs              # WeChat integration
│   ├── web_server/
│   │   └── mod.rs              # Axum web server & routes
│   ├── auth/
│   │   └── mod.rs              # Authentication
│   ├── gateway/
│   │   └── mod.rs              # Platform gateway
│   ├── workspace/
│   │   └── mod.rs              # Workspace management
│   ├── state/
│   │   ├── mod.rs              # Application state
│   │   └── sync.rs             # State synchronization
│   ├── upload/
│   │   └── mod.rs              # File upload handling
│   ├── i18n/
│   │   └── mod.rs              # Internationalization
│   └── cli/
│       └── mod.rs              # CLI interface
├── webui/
│   ├── index.html              # Main HTML page
│   ├── js/
│   │   └── main.js             # Frontend JavaScript
│   └── styles/
│       ── main.css            # Stylesheet
├── resources/
│   ├── mahakala_icon.png       # App icon (dark)
│   └── mahakala_icon_light.png # App icon (light)
├── .mahakala/
│   ├── soul.md                 # Agent personality
│   ── user.md                 # User preferences
└── data/
    └── memory.db               # SQLite memory database
```

---

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/` | WebUI main page |
| GET | `/api/status` | Server status |
| GET | `/api/tools` | List all tools |
| POST | `/api/tools/:name/execute` | Execute a tool |
| GET | `/api/skills` | List all skills |
| GET | `/api/plugins` | List all plugins |
| POST | `/api/chat` | Send chat message |
| GET | `/api/memory/facts` | List memory facts |
| GET | `/api/cron` | List cron jobs |
| POST | `/api/config` | Update configuration |
| POST | `/api/upload` | Upload file (multipart) |
| GET | `/api/upload/:id` | Get uploaded file info |
| DELETE | `/api/upload/:id` | Delete uploaded file |

---

## Development

### Debug Build

```bash
cargo build
cargo run
```

### Run Tests

```bash
cargo test
```

### Check Code

```bash
cargo check
cargo clippy
```

---

## Roadmap

### Current Limitations

The following features are planned but not yet implemented:

| Feature | Status | Priority |
|---------|--------|----------|
| **Closed Learning Loop** | Not implemented | High |
| **Autonomous Skill Creation** | Not implemented | High |
| **Semantic Memory Search** | Not implemented | Medium |
| **Subagent/Delegation System** | Not implemented | High |
| **MCP Server Integration** | Not implemented | Medium |
| **Browser Automation** | Not implemented | Low |
| **Docker/SSH Terminal Backends** | Not implemented | Medium |
| **Multi-Agent Collaboration** | Not implemented | Low |
| **Voice Mode (beyond Web Speech API)** | Partial | Medium |
| **Image Generation & Vision** | Not implemented | Low |
| **Serverless Deployment** | Not implemented | Low |
| **Community Skill Hub** | Not implemented | Low |
| **RL Training / Trajectory Export** | Not implemented | Low |
| **File Manager (WebUI)** | Stub only | Medium |
| **Terminal (WebUI)** | Stub only | Medium |
| **More Messaging Platforms** | WeChat only | Medium |

### Future Development Plan

#### Phase 1: Core Enhancements (Q3 2026)
- [ ] Implement closed learning loop with autonomous skill creation
- [ ] Add semantic memory search with vector embeddings
- [ ] Implement subagent/delegation system for parallel task execution
- [ ] Add MCP server integration for extended tool capabilities
- [ ] Expand tool library to 20+ built-in tools

#### Phase 2: Platform Expansion (Q4 2026)
- [ ] Add Telegram, Discord, Slack gateway support
- [ ] Implement Docker and SSH terminal backends
- [ ] Add browser automation with Playwright integration
- [ ] Implement voice mode with real-time audio streaming
- [ ] Add image generation and vision analysis tools

#### Phase 3: Enterprise Features (Q1 2027)
- [ ] Multi-agent collaboration framework
- [ ] Role-based access control (RBAC)
- [ ] Audit logging and compliance features
- [ ] Serverless deployment support (Daytona/Modal compatible)
- [ ] Community skill hub with agentskills.io compatibility

#### Phase 4: Research & Advanced Features (Q2 2027)
- [ ] Batch trajectory generation for RL training
- [ ] Atropos-compatible RL environment
- [ ] Honcho dialectic user modeling
- [ ] FTS5 full-text search with LLM summarization
- [ ] Periodic memory nudging system

---

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](.github/CONTRIBUTING.md) for details.

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- Inspired by the Hermes Agent ecosystem (Nous Research)
- Architecture influenced by OpenClaw's gateway design
- Built with the excellent [Axum](https://github.com/tokio-rs/axum) web framework
- Thanks to the Rust community for amazing libraries

---

<br>

---

# 大黑天智能体 (Mahakala Agent)

<div align="center">

**基于 Rust 构建的强大 AI 智能体 - 支持 WebUI、工具执行、技能系统、插件架构和多平台集成**

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/axum-0.7-green.svg)](https://github.com/tokio-rs/axum)
[![Contact](https://img.shields.io/badge/contact-mahakala.hum.pate@gmail.com-red.svg)](mailto:mahakala.hum.pate@gmail.com)

</div>

---

## 概述

大黑天智能体（Mahakala Agent）是一个完全使用 Rust 实现的综合性 AI 智能体系统。它提供功能完整的 Web 界面、工具执行引擎、技能系统、插件架构和多平台消息集成。

最初设计为 Hermes Agent 生态系统的 Rust 移植版本，大黑天智能体提供生产就绪的 AI 助手，配备现代化 Web UI，支持多种 AI 提供商，包括 OpenAI、Anthropic Claude、DeepSeek 和本地 Ollama 模型。

---

## 与 Hermes Agent 和 OpenClaw 的对比

### 定位概览

| 特性 | 大黑天智能体 | Hermes Agent | OpenClaw |
|------|-------------|--------------|----------|
| **编程语言** | Rust | Python | Python |
| **架构设计** | Web 优先 (Axum + SPA) | CLI 优先 + 网关 | 网关优先 |
| **用户界面** | 现代化 WebUI (深色/浅色主题) | TUI (终端界面) | CLI + 消息平台 |
| **部署方式** | 单一二进制文件，零依赖 | Python 环境 + 依赖 | Python 环境 + 依赖 |
| **记忆系统** | 基于 SQLite 的持久化存储 | FTS5 + LLM 摘要 | SQLite + 会话文件 |
| **内置工具** | 8 个内置 + 可扩展注册表 | 40+ 内置工具 | 社区驱动的技能 |
| **技能系统** | 预构建技能模板 | 自主技能创建 | 兼容 agentskills.io |
| **插件系统** | 动态文件系统加载 | 基于插件的记忆提供者 | 网关插件 |
| **消息平台** | 微信，可扩展网关 | 15+ 消息平台 | 50+ 消息平台 |
| **本地 AI** | Ollama 原生支持 | 任意 OpenAI 兼容端点 | 任意 OpenAI 兼容端点 |
| **思考模式** | DeepSeek 推理支持 | 无特定思考模式 | 无特定思考模式 |
| **文件上传** | Multipart 上传 API | 终端文件操作 | 文件工具 |
| **语音输入** | Web Speech API | 语音模式 (CLI/Telegram) | 语音模式 |
| **定时任务** | 内置 tokio-cron-scheduler | 内置 cron 推送 | 内置 cron |
| **实时通信** | WebSocket 实时流 | SSE + WebSocket | 平台特定 |
| **国际化** | 内置 (中文/英文) | 仅英文 | 英文 + 社区翻译 |

### 核心差异

#### 与 Hermes Agent 对比

| 方面 | Hermes Agent | 大黑天智能体 |
|------|-------------|-------------|
| **学习循环** | 闭环学习，自主技能创建和自我改进 | 手动技能管理，暂无自主学习 |
| **终端后端** | 6 种后端 (本地、Docker、SSH、Daytona、Singularity、Modal) | 仅本地执行 |
| **记忆系统** | FTS5 全文搜索 + LLM 摘要，Honcho 辩证用户建模 | 基于 SQLite 的事实存储，无语义搜索 |
| **子智能体** | 支持隔离子智能体并行工作流 | 不支持子智能体 |
| **MCP 集成** | 完整 MCP 服务器支持 | 不支持 MCP |
| **浏览器自动化** | 交互式浏览器 + 视觉支持 | 无浏览器自动化 |
| **研究功能** | 批量轨迹导出，Atropos RL 训练 | 无研究功能 |
| **无服务器** | Daytona/Modal 无服务器持久化 | 不支持无服务器 |

#### 与 OpenClaw 对比

| 方面 | OpenClaw | 大黑天智能体 |
|------|----------|-------------|
| **架构** | 网关优先集中式控制 | Web 优先，集成后端 |
| **平台覆盖** | 50+ 消息平台 | 微信 + 可扩展 |
| **技能生态** | 兼容 agentskills.io，社区驱动 | 预构建模板，无社区中心 |
| **多智能体** | 支持多智能体协作 | 仅单智能体 |
| **企业级** | 企业级部署 | 开发/个人使用 |
| **云托管** | 多种云选项 | 仅自托管 |

### 大黑天智能体的独特创新

1. **Rust 原生性能**：完全使用 Rust 构建，零 Python 依赖，相比 Python 方案提供更优的内存效率和启动速度。

2. **单一二进制部署**：`cargo build --release` 生成包含所有嵌入资源的单一可执行文件 - 无需虚拟环境、无需 pip 安装、无依赖冲突。

3. **Web 优先设计**：不同于 Hermes Agent 的 TUI 或 OpenClaw 的 CLI 优先方案，大黑天智能体提供现代化、响应式的单页 Web 应用，支持实时 WebSocket 通信。

4. **DeepSeek 思考模式**：原生支持 DeepSeek 的推理/思考模式，支持 `reasoning_content` 透传，实现透明的思维链调试。

5. **Ollama 原生集成**：一级支持本地 Ollama 模型，无需 API 密钥，是离线 AI 使用的最便捷选择。

6. **集成微信 SDK**：内置微信二维码生成和消息支持，针对 Hermes Agent 和 OpenClaw 未原生支持的中文消息生态系统。

7. **实时输出与调试面板**：WebUI 内置输出日志和调试面板，透明监控智能体操作、API 调用和工具执行。

8. **Web Speech API 语音输入**：浏览器原生语音输入，无需额外依赖或外部服务。

9. **Multipart 文件上传**：原生文件上传，支持 MIME 类型检测和持久化存储，通过 WebUI 工具栏访问。

10. **可配置工具迭代限制**：可调整的最大工具调用迭代次数（默认 50 次），防止无限递归同时允许复杂的多步操作。

---

## 功能特性

### AI 智能体核心
- **多提供商支持**：OpenAI、Anthropic Claude、DeepSeek 和本地 Ollama 模型
- **工具调用**：原生工具调用，兼容 OpenAI schema 格式
- **思考模式**：完整支持 DeepSeek 推理/思考模式，支持 `reasoning_content`
- **对话历史**：基于 SQLite 的持久化对话管理
- **流式响应**：通过 Server-Sent Events (SSE) 实时流式输出
- **系统提示**：可自定义系统提示，定义智能体角色和能力

### 工具系统（8 个内置工具）
| 工具 | 描述 |
|------|------|
| `file_read` | 读取文件内容 |
| `file_write` | 写入文件内容 |
| `file_list` | 列出目录内容 |
| `web_fetch` | 获取 URL 内容 |
| `calculator` | 数学计算 |
| `shell_exec` | 执行 Shell 命令 |
| `memory` | 存储和检索信息 |
| `todo` | 管理待办事项 |

### 技能系统
- 预构建技能，覆盖常见任务（代码审查、CI/CD、创意写作、网络研究等）
- 带参数验证的技能执行引擎
- 通过 WebUI 管理技能

### 插件系统
- 从文件系统动态加载插件
- 基于插件清单的发现机制
- 内置插件：磁盘清理、网络监控、内存管理、日志管理、调度器、安全扫描器

### WebUI 界面
- 现代化、响应式单页应用
- 深色/浅色主题支持
- 多语言支持（国际化）
- 实时聊天，支持流式输出
- 工具管理面板
- 技能管理面板
- 插件管理面板
- 记忆/事实管理
- 定时任务调度
- 设置和配置
- **输出面板**：实时操作日志
- **调试面板**：详细的 API 请求/响应追踪
- **快捷操作**：代码执行、网页搜索、文件管理、终端访问

### 微信集成
- 微信登录二维码生成
- PNG 图片保存二维码
- 微信消息支持

### 其他功能
- **定时调度器**：自动化任务调度
- **记忆系统**：基于 SQLite 的持久化记忆存储
- **会话管理**：多对话会话
- **Token 使用追踪**：监控 API token 消耗
- **开发模式**：前端资源热重载支持
- **文件上传**：Multipart 文件上传，支持 MIME 类型检测
- **语音输入**：通过 Web Speech API 的浏览器原生语音识别

---

## 技术栈

- **语言**：Rust 2021 Edition
- **Web 框架**：Axum 0.7，支持 WebSocket
- **HTTP 客户端**：reqwest 0.12
- **数据库**：SQLite (rusqlite)
- **序列化**：serde + serde_json
- **异步运行时**：Tokio
- **日志**：tracing + tracing-subscriber
- **定时任务**：tokio-cron-scheduler
- **认证**：JWT (jsonwebtoken) + Argon2

---

## 快速开始

### 前置条件

- Rust 1.70 或更高版本
- （可选）Ollama 用于本地 AI 模型
- （可选）云提供商 API 密钥（OpenAI、Anthropic、DeepSeek）

### 构建

```bash
cd Mahakala-agent-upgrade
cargo build --release
```

### 运行

```bash
cargo run --release
```

Web 服务器将在 `http://127.0.0.1:3000` 启动

### 配置

配置通过 WebUI 设置面板管理。支持的 AI 提供商：

| 提供商 | API 基础 URL | 说明 |
|--------|-------------|------|
| OpenAI | `https://api.openai.com/v1` | 需要 API 密钥 |
| Anthropic | `https://api.anthropic.com/v1` | 需要 API 密钥 |
| DeepSeek | `https://api.deepseek.com/v1` | 需要 API 密钥 |
| Ollama | `http://localhost:11434/v1` | 无需 API 密钥 |

---

## 项目结构

```
Mahakala-agent-upgrade/
├── Cargo.toml                  # 项目清单
├── Cargo.lock
── src/
│   ├── main.rs                 # 应用入口
│   ├── lib.rs                  # 库导出
│   ├── config.rs               # 配置管理
│   ├── constants.rs            # 常量和默认值
│   ├── logging.rs              # 日志设置
│   ├── error.rs                # 错误类型
│   ├── agent/
│   │   ├── core.rs             # AI 智能体核心逻辑
│   │   ├── mod.rs
│   │   └── prompt_builder.rs   # 系统提示构建器
│   ├── tools/
│   │   ├── mod.rs              # 工具模块导出
│   │   ├── registry.rs         # 工具注册和 schema
│   │   ├── all_tools.rs        # 批量工具注册
│   │   ├── calculator.rs       # 数学计算器工具
│   │   ├── file_list.rs        # 目录列表工具
│   │   ├── file_read.rs        # 文件读取工具
│   │   ├── file_write.rs       # 文件写入工具
│   │   ├── memory_tool.rs      # 记忆存储工具
│   │   ├── shell_exec.rs       # Shell 命令执行
│   │   ├── todo.rs             # 待办事项管理
│   │   └── web_fetch.rs        # URL 内容获取
│   ├── skills/
│   │   └── mod.rs              # 技能管理
│   ├── plugins/
│   │   ── mod.rs              # 插件系统
│   ├── memory/
│   │   └── mod.rs              # 记忆管理器 (SQLite)
│   ├── cron/
│   │   └── mod.rs              # 定时调度器
│   ├── wechat/
│   │   └── mod.rs              # 微信集成
│   ├── web_server/
│   │   └── mod.rs              # Axum Web 服务器和路由
│   ├── auth/
│   │   ── mod.rs              # 认证
│   ├── gateway/
│   │   └── mod.rs              # 平台网关
│   ├── workspace/
│   │   └── mod.rs              # 工作区管理
│   ├── state/
│   │   ├── mod.rs              # 应用状态
│   │   └── sync.rs             # 状态同步
│   ├── upload/
│   │   └── mod.rs              # 文件上传处理
│   ├── i18n/
│   │   └── mod.rs              # 国际化
│   └── cli/
│       ── mod.rs              # CLI 接口
├── webui/
│   ├── index.html              # 主 HTML 页面
│   ├── js/
│   │   └── main.js             # 前端 JavaScript
│   └── styles/
│       └── main.css            # 样式表
├── resources/
│   ├── mahakala_icon.png       # 应用图标 (深色)
│   └── mahakala_icon_light.png # 应用图标 (浅色)
├── .mahakala/
│   ├── soul.md                 # 智能体个性
│   └── user.md                 # 用户偏好
└── data/
    └── memory.db               # SQLite 记忆数据库
```

---

## API 端点

| 方法 | 端点 | 描述 |
|------|------|------|
| GET | `/` | WebUI 主页面 |
| GET | `/api/status` | 服务器状态 |
| GET | `/api/tools` | 列出所有工具 |
| POST | `/api/tools/:name/execute` | 执行工具 |
| GET | `/api/skills` | 列出所有技能 |
| GET | `/api/plugins` | 列出所有插件 |
| POST | `/api/chat` | 发送聊天消息 |
| GET | `/api/memory/facts` | 列出记忆事实 |
| GET | `/api/cron` | 列出定时任务 |
| POST | `/api/config` | 更新配置 |
| POST | `/api/upload` | 上传文件 (multipart) |
| GET | `/api/upload/:id` | 获取已上传文件信息 |
| DELETE | `/api/upload/:id` | 删除已上传文件 |

---

## 开发

### 调试构建

```bash
cargo build
cargo run
```

### 运行测试

```bash
cargo test
```

### 代码检查

```bash
cargo check
cargo clippy
```

---

## 开发路线图

### 当前限制

以下功能已规划但尚未实现：

| 功能 | 状态 | 优先级 |
|------|------|--------|
| **闭环学习系统** | 未实现 | 高 |
| **自主技能创建** | 未实现 | 高 |
| **语义记忆搜索** | 未实现 | 中 |
| **子智能体/委托系统** | 未实现 | 高 |
| **MCP 服务器集成** | 未实现 | 中 |
| **浏览器自动化** | 未实现 | 低 |
| **Docker/SSH 终端后端** | 未实现 | 中 |
| **多智能体协作** | 未实现 | 低 |
| **语音模式（超越 Web Speech API）** | 部分实现 | 中 |
| **图像生成与视觉分析** | 未实现 | 低 |
| **无服务器部署** | 未实现 | 低 |
| **社区技能中心** | 未实现 | 低 |
| **RL 训练 / 轨迹导出** | 未实现 | 低 |
| **文件管理器（WebUI）** | 仅存根 | 中 |
| **终端（WebUI）** | 仅存根 | 中 |
| **更多消息平台** | 仅微信 | 中 |

### 未来开发计划

#### 第一阶段：核心增强（2026 Q3）
- [ ] 实现闭环学习系统，支持自主技能创建
- [ ] 添加基于向量嵌入的语义记忆搜索
- [ ] 实现子智能体/委托系统，支持并行任务执行
- [ ] 添加 MCP 服务器集成，扩展工具能力
- [ ] 将工具库扩展到 20+ 内置工具

#### 第二阶段：平台扩展（2026 Q4）
- [ ] 添加 Telegram、Discord、Slack 网关支持
- [ ] 实现 Docker 和 SSH 终端后端
- [ ] 集成 Playwright 实现浏览器自动化
- [ ] 实现支持实时音频流的语音模式
- [ ] 添加图像生成和视觉分析工具

#### 第三阶段：企业功能（2027 Q1）
- [ ] 多智能体协作框架
- [ ] 基于角色的访问控制（RBAC）
- [ ] 审计日志和合规功能
- [ ] 无服务器部署支持（兼容 Daytona/Modal）
- [ ] 兼容 agentskills.io 的社区技能中心

#### 第四阶段：研究与高级功能（2027 Q2）
- [ ] 批量轨迹生成用于 RL 训练
- [ ] 兼容 Atropos 的 RL 环境
- [ ] Honcho 辩证用户建模
- [ ] FTS5 全文搜索 + LLM 摘要
- [ ] 定期记忆提示系统

---

## 贡献

欢迎贡献！详情请参阅 [CONTRIBUTING.md](.github/CONTRIBUTING.md)。

---

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

---

## 致谢

- 灵感来源于 Hermes Agent 生态系统（Nous Research）
- 架构受 OpenClaw 网关设计影响
- 基于优秀的 [Axum](https://github.com/tokio-rs/axum) Web 框架构建
- 感谢 Rust 社区提供的优秀库
