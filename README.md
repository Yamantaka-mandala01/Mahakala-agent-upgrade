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
│   │   └── mod.rs              # Skill management
│   ├── plugins/
│   │   └── mod.rs              # Plugin system
│   ├── memory/
│   │   └── mod.rs              # Memory manager (SQLite)
│   ├── cron/
│   │   └── mod.rs              # Cron scheduler
│   ├── wechat/
│   │   └── mod.rs              # WeChat integration
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
│       └── main.css            # Stylesheet
├── resources/
│   ├── mahakala_icon.png       # App icon (dark)
│   └── mahakala_icon_light.png # App icon (light)
├── .mahakala/
│   ├── soul.md                 # Agent personality
│   └── user.md                 # User preferences
└── data/
    └── memory.db               # SQLite memory database
```

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

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](.github/CONTRIBUTING.md) for details.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the Hermes Agent ecosystem
- Built with the excellent [Axum](https://github.com/tokio-rs/axum) web framework
- Thanks to the Rust community for amazing libraries
