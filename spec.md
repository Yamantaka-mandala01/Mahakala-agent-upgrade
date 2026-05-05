# Mahakala Agent Upgrade - 完整规格说明

## 项目概述

将 `hermes-agent-main` (Python AI代理) 和 `hermes-webui-master` (Python/JS WebUI) 的所有功能完整复刻为 **纯 Rust 实现**，所有资源嵌入到单个 `.exe` 文件中。

---

## 源系统功能清单

### 一、hermes-agent-main (AI代理核心)

#### 1. AI Agent 系统
- **Agent 核心**: 对话管理、工具调用、技能执行
- **Prompt Builder**: 动态提示语构建、上下文组装
- **Context Engine**: 上下文管理、参考文献集成
- **Memory Manager**: 对话历史、上下文压缩、记忆保持
- **Memory Provider**: 支持多种记忆存储 (Honcho, Mem0)
- **Title Generator**: 自动生成会话标题
- **Error Classifier**: AI 错误分类和处理
- **Trajectory**: 交互轨迹记录和回溯
- **Prompt Caching**: 提示语缓存优化

#### 2. 工具系统 (32+ 工具)
| 工具 | 功能 |
|------|------|
| shell_exec | 执行系统命令/Shell脚本 |
| file_read | 读取文件内容 |
| file_write | 写入文件内容 |
| file_append | 追加文件内容 |
| file_list | 列出目录内容 |
| file_search | 搜索文件 |
| web_fetch | HTTP请求获取网页内容 |
| web_search | 网络搜索 |
| code_execute | 代码执行 (Python/JS/Bash) |
| browser_tool | 网页浏览和内容提取 |
| memory_tool | 记忆系统交互 |
| skills_tool | 技能调用和执行 |
| tts_tool | 文本转语音 |
| vision_tool | 图像识别和处理 |
| url_safety | URL安全检查 |
| calculator | 数学计算 |
| diff | 文件差异比较 |
| edit | 文件编辑 |
| replace | 文本替换 |
| grep | 文本搜索 |
| head/tail | 文件头尾读取 |
| wc | 统计行数/词数 |
| find | 文件查找 |
| sed | 流编辑器 |
| patch | 补丁应用 |
| git | Git操作 |
| todo_read | 待办事项读取 |
| todo_write | 待办事项写入 |
| task | 任务管理 |
| notification | 通知推送 |

#### 3. 技能系统
- 技能加载器和管理器
- 技能执行引擎
- 技能安装器 (从Git/URL安装)
- 技能配置文件格式

#### 4. 插件系统
- 插件加载器
- 插件生命周期管理
- 动态插件加载
- 记忆插件 (不同存储后端)

#### 5. 平台集成 (网关系统)
| 平台 | 功能 |
|------|------|
| WhatsApp | 消息收发 |
| Telegram | 消息收发、机器人 |
| Discord | 消息收发、机器人 |
| Slack | 消息收发、频道管理 |
| Email | 邮件收发 (IMAP/SMTP) |
| SMS | 短信收发 |
| WeChat/微信 | 消息收发、二维码登录 |
| QQBot | QQ机器人消息收发 |
| API Server | REST API 网关 |

#### 6. 配置系统
- YAML 配置文件
- 环境变量支持
- 多配置文件合并
- 配置验证

#### 7. CLI 接口
- 交互式对话模式
- 单次查询模式
- 配置管理命令
- 会话管理命令
- 工具配置

---

### 二、hermes-webui-master (WebUI系统)

#### 1. 后端 API (Flask)
| API 模块 | 端点 |
|----------|------|
| **认证** | `/api/login`, `/api/logout`, `/api/sessions` |
| **会话操作** | `/api/sessions`, `/api/sessions/:id`, `/api/sessions/:id/messages` |
| **工作区** | `/api/workspaces`, `/api/workspaces/:id` |
| **流式响应** | SSE 流式输出 |
| **配置** | `/api/config`, `/api/providers` |
| **命令** | `/api/commands` |
| **模型** | `/api/models` |
| **文件上传** | `/api/upload` |
| **网关监控** | `/api/gateway/status` |
| **更新检查** | `/api/updates/check` |
| **状态同步** | `/api/state/sync` |
| **澄清** | `/api/clarify` |
| **引导** | `/api/onboarding` |
| **用户配置** | `/api/profiles` |

#### 2. 前端 UI (原生 JavaScript)
| 模块 | 功能 |
|------|------|
| **sessions.js** | 会话列表、创建、删除、切换 |
| **workspace.js** | 工作区管理、切换 |
| **panels.js** | 面板展示和隐藏 |
| **ui.js** | 整体UI交互、状态管理 |
| **messages.js** | 消息渲染、流式更新 |
| **commands.js** | 命令执行、结果显示 |
| **login.js** | 登录表单、认证 |
| **onboarding.js** | 新手引导流程 |
| **boot.js** | 应用启动初始化 |
| **i18n.js** | 多语言支持 |
| **icons.js** | 图标系统 |

#### 3. 核心功能
- **认证系统**: JWT/Session 认证
- **会话管理**: 创建、删除、切换、消息历史
- **工作区**: 多工作区支持、切换、配置隔离
- **流式响应**: Server-Sent Events (SSE)
- **文件上传**: 拖拽上传、进度显示
- **国际化**: 多语言支持 (zh/en/es/ru等)
- **定时任务**: Cron 任务管理
- **更新检查**: 版本更新通知
- **状态同步**: 前后端状态同步

---

## Rust 实现架构

### 项目结构

```
Mahakala-agent-upgrade/
├── Cargo.toml                          # 项目配置
├── Cargo.lock
├── src/
│   ├── main.rs                         # 入口文件
│   ├── lib.rs                          # 库入口
│   ├── config.rs                       # 配置管理
│   ├── constants.rs                    # 常量定义
│   ├── logging.rs                      # 日志系统
│   ├── error.rs                        # 错误处理
│   ├── agent/
│   │   ├── mod.rs                      # Agent模块
│   │   ├── core.rs                     # Agent核心
│   │   ├── prompt_builder.rs           # 提示语构建
│   │   ├── context_engine.rs           # 上下文引擎
│   │   ├── memory_manager.rs           # 记忆管理
│   │   ├── title_generator.rs          # 标题生成
│   │   ├── error_classifier.rs         # 错误分类
│   │   └── trajectory.rs               # 轨迹记录
│   ├── tools/
│   │   ├── mod.rs                      # 工具模块
│   │   ├── registry.rs                 # 工具注册
│   │   ├── shell_exec.rs               # Shell执行
│   │   ├── file_read.rs                # 文件读取
│   │   ├── file_write.rs               # 文件写入
│   │   ├── file_list.rs                # 文件列表
│   │   ├── file_search.rs              # 文件搜索
│   │   ├── web_fetch.rs                # 网页获取
│   │   ├── web_search.rs               # 网络搜索
│   │   ├── code_execute.rs             # 代码执行
│   │   ├── memory_tool.rs              # 记忆工具
│   │   ├── calculator.rs               # 计算器
│   │   ├── diff.rs                     # 差异比较
│   │   ├── edit.rs                     # 文件编辑
│   │   ├── replace.rs                  # 文本替换
│   │   ├── grep.rs                     # 文本搜索
│   │   ├── head_tail.rs                # 头尾读取
│   │   ├── wc.rs                       # 统计
│   │   ├── find.rs                     # 文件查找
│   │   ├── sed.rs                      # 流编辑
│   │   ├── patch.rs                    # 补丁
│   │   ├── git.rs                      # Git操作
│   │   ├── todo.rs                     # 待办事项
│   │   ├── notification.rs             # 通知
│   │   ├── tts.rs                      # 文本转语音
│   │   ├── vision.rs                   # 视觉处理
│   │   └── url_safety.rs               # URL安全
│   ├── skills/
│   │   ├── mod.rs                      # 技能模块
│   │   ├── registry.rs                 # 技能注册
│   │   ├── executor.rs                 # 技能执行
│   │   └── installer.rs                # 技能安装
│   ├── plugins/
│   │   ├── mod.rs                      # 插件模块
│   │   ├── registry.rs                 # 插件注册
│   │   └── loader.rs                   # 插件加载
│   ├── gateway/
│   │   ├── mod.rs                      # 网关模块
│   │   ├── session.rs                  # 会话管理
│   │   ├── delivery.rs                 # 消息投递
│   │   └── platforms/
│   │       ├── mod.rs                  # 平台模块
│   │       ├── base.rs                 # 基础平台
│   │       ├── whatsapp.rs             # WhatsApp
│   │       ├── telegram.rs             # Telegram
│   │       ├── discord.rs              # Discord
│   │       ├── slack.rs                # Slack
│   │       ├── email.rs                # Email
│   │       ├── sms.rs                  # SMS
│   │       ├── wechat.rs               # 微信
│   │       ├── qqbot.rs                # QQBot
│   │       └── api_server.rs           # API服务器
│   ├── memory/
│   │   ├── mod.rs                      # 记忆模块
│   │   ├── store.rs                    # 存储
│   │   └── providers/
│   │       ├── mod.rs
│   │       ├── honcho.rs
│   │       └── mem0.rs
│   ├── web_server/
│   │   ├── mod.rs                      # Web服务器
│   │   ├── routes.rs                   # API路由
│   │   ├── auth.rs                     # 认证
│   │   ├── sessions.rs                 # 会话API
│   │   ├── workspace.rs                # 工作区API
│   │   ├── streaming.rs                # 流式响应
│   │   ├── upload.rs                   # 文件上传
│   │   ├── config_api.rs               # 配置API
│   │   ├── commands_api.rs             # 命令API
│   │   ├── models_api.rs               # 模型API
│   │   ├── gateway_api.rs              # 网关API
│   │   ├── updates_api.rs              # 更新API
│   │   ├── state_sync.rs               # 状态同步
│   │   ├── clarify.rs                  # 澄清API
│   │   ── onboarding.rs               # 引导API
│   ├── auth/
│   │   ├── mod.rs                      # 认证模块
│   │   ├── jwt.rs                      # JWT处理
│   │   └── session.rs                  # 会话管理
│   ├── workspace/
│   │   ├── mod.rs                      # 工作区模块
│   │   └── manager.rs                  # 工作区管理
│   ├── cron/
│   │   ├── mod.rs                      # 定时任务模块
│   │   └── scheduler.rs                # 调度器
│   ├── i18n/
│   │   ├── mod.rs                      # 国际化模块
│   │   └── translations.rs             # 翻译数据
│   ├── upload/
│   │   ├── mod.rs                      # 上传模块
│   │   └── handler.rs                  # 上传处理
│   ├── state/
│   │   ├── mod.rs                      # 状态模块
│   │   └── sync.rs                     # 状态同步
│   └── cli/
│       ├── mod.rs                      # CLI模块
│       └── commands.rs                 # CLI命令
├── webui/
│   ├── index.html                      # 主页面
│   ├── styles/
│   │   ├── main.css                    # 主样式
│   │   ├── light.css                   # 亮色主题
│   │   └── dark.css                    # 暗色主题
│   ├── js/
│   │   ├── main.js                     # 主JS
│   │   ├── sessions.js                 # 会话JS
│   │   ├── workspace.js                # 工作区JS
│   │   ├── messages.js                 # 消息JS
│   │   ├── commands.js                 # 命令JS
│   │   ├── login.js                    # 登录JS
│   │   ├── onboarding.js               # 引导JS
│   │   ├── i18n.js                     # 国际化JS
│   │   └── icons.js                    # 图标JS
│   ── assets/
│       ├── logo-dark.png               # 暗色logo
│       ├── logo-light.png              # 亮色logo
│       └── favicon.ico
├── tests/
│   └── integration.rs                  # 集成测试
└── build.rs                            # 构建脚本
```

---

## 技术栈

| 组件 | Rust Crate |
|------|-----------|
| Web框架 | Axum + Tokio |
| HTTP客户端 | reqwest |
| JSON处理 | serde, serde_json |
| 配置解析 | serde_yaml, dotenv |
| 命令行 | clap |
| 日志 | tracing, tracing-subscriber |
| JWT认证 | jsonwebtoken |
| 定时任务 | tokio-cron-scheduler |
| SSE流式 | axum-extra |
| 文件上传 | multer |
| WebSocket | tokio-tungstenite |
| SQLite存储 | rusqlite |
| 正则表达式 | regex |
| 文件操作 | tokio::fs |
| 路径处理 | pathdiff |
| 国际化 | fluent |
| 模板渲染 | askama |
| 嵌入资源 | rust-embed |

---

## API 端点完整列表

### 认证 API
- `POST /api/login` - 用户登录
- `POST /api/logout` - 用户登出
- `GET /api/sessions` - 获取会话列表
- `POST /api/sessions` - 创建新会话
- `GET /api/sessions/:id` - 获取会话详情
- `DELETE /api/sessions/:id` - 删除会话
- `GET /api/sessions/:id/messages` - 获取会话消息
- `POST /api/sessions/:id/messages` - 发送消息
- `POST /api/sessions/:id/stream` - 流式消息 (SSE)
- `GET /api/workspaces` - 获取工作区列表
- `POST /api/workspaces` - 创建工作区
- `GET /api/workspaces/:id` - 获取工作区详情
- `PUT /api/workspaces/:id` - 更新工作区
- `DELETE /api/workspaces/:id` - 删除工作区
- `POST /api/commands` - 执行命令
- `GET /api/models` - 获取模型列表
- `POST /api/config` - 保存配置
- `GET /api/config` - 获取配置
- `GET /api/providers` - 获取提供商列表
- `POST /api/upload` - 文件上传
- `GET /api/gateway/status` - 获取网关状态
- `GET /api/updates/check` - 检查更新
- `POST /api/state/sync` - 状态同步
- `POST /api/clarify` - 澄清请求
- `GET /api/onboarding/status` - 获取引导状态
- `POST /api/onboarding/complete` - 完成引导
- `GET /api/profiles` - 获取用户配置
- `PUT /api/profiles` - 更新用户配置

---

## 主题系统

### 亮色主题
- 浅色背景 (#f5f5f5)
- 深色文字
- 品牌色高亮

### 暗色主题
- 深色背景 (#1a1a2e)
- 浅色文字
- 品牌色高亮

主题切换通过 CSS 变量和 `data-theme` 属性实现。

---

## 嵌入资源

所有前端资源（HTML、CSS、JS、图片）通过 `rust-embed` 嵌入到可执行文件中，实现单个 `.exe` 文件分发。

图标文件作为程序 logo 嵌入 Windows 资源。

---

## 编译目标

- 目标平台: `x86_64-pc-windows-msvc`
- 输出: 单个 `.exe` 文件
- 图标: 使用提供的像素风格图标
- 大小优化: release profile, lto, strip
