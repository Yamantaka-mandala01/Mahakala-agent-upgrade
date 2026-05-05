use axum::{
    routing::{get, post, delete},
    Router, Json,
    extract::{State, Path, WebSocketUpgrade},
    response::Response,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

use crate::agent::AIAgent;
use crate::config::AppConfig;
use crate::state::SessionDB;
use crate::tools::registry::ToolRegistry;
use crate::memory::MemoryManager;
use crate::skills::SkillManager;
use crate::plugins::PluginManager;
use crate::gateway::GatewayHandle;
use crate::cron::CronManagerHandle;
use crate::workspace::WorkspaceHandle;
use crate::auth::AuthHandle;
use crate::i18n::I18nHandle;
use crate::upload::UploadHandle;
use crate::cli::CliHandle;
use crate::wechat::WechatHandle;
use serde_json::Value;

pub struct AppState {
    pub agent: RwLock<AIAgent>,
    pub config: RwLock<AppConfig>,
    pub session_db: Arc<SessionDB>,
    pub tool_registry: Arc<ToolRegistry>,
    pub memory: MemoryManager,
    pub skills: SkillManager,
    pub plugins: PluginManager,
    pub gateway: GatewayHandle,
    pub cron: CronManagerHandle,
    pub workspace: WorkspaceHandle,
    pub auth: AuthHandle,
    pub i18n: I18nHandle,
    pub upload: UploadHandle,
    pub cli: CliHandle,
    pub wechat: WechatHandle,
}

pub async fn run_web_server(state: Arc<AppState>) -> anyhow::Result<()> {
    let addr = {
        let config = state.config.read().await;
        format!("{}:{}", config.host, config.port)
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        // API routes
        .route("/api/status", get(api_status))
        .route("/api/chat", post(api_chat))
        .route("/api/config", get(api_get_config))
        .route("/api/config", post(api_save_config))
        .route("/api/config/role", get(api_get_role_config))
        .route("/api/config/role", post(api_save_role_config))
        .route("/api/sessions", get(api_list_sessions))
        .route("/api/sessions", post(api_create_session))
        .route("/api/sessions/:id", get(api_get_session))
        .route("/api/sessions/:id", delete(api_delete_session))
        .route("/api/sessions/:id/messages", get(api_get_messages))
        .route("/api/sessions/:id/messages", post(api_send_message))
        .route("/api/providers", get(api_list_providers))
        .route("/api/models", get(api_list_models))
        .route("/api/tools", get(api_list_tools))
        .route("/api/tools/:name", post(api_toggle_tool))
        .route("/api/tools/:name/execute", post(api_execute_tool))
        .route("/api/memory", get(api_get_memory))
        .route("/api/memory", post(api_store_memory))
        .route("/api/memory/facts", get(api_get_facts))
        .route("/api/memory/facts", post(api_add_fact))
        .route("/api/memory/facts/:id", delete(api_delete_fact))
        .route("/api/memory/search", post(api_search_memory))
        .route("/api/memory/stats", get(api_memory_stats))
        .route("/api/todo", get(api_list_todos))
        .route("/api/todo", post(api_manage_todo))
        .route("/api/workspaces", get(api_list_workspaces))
        .route("/api/workspaces", post(api_create_workspace))
        .route("/api/workspaces/:id", get(api_get_workspace))
        .route("/api/workspaces/:id/active", post(api_set_active_workspace))
        .route("/api/workspaces/:id/files", get(api_list_workspace_files))
        .route("/api/skills", get(api_list_skills))
        .route("/api/skills", post(api_install_skill))
        .route("/api/skills/:name", post(api_manage_skill))
        .route("/api/plugins", get(api_list_plugins))
        .route("/api/plugins", post(api_add_plugin))
        .route("/api/plugins/:name", post(api_manage_plugin))
        .route("/api/platforms", get(api_list_platforms))
        .route("/api/platforms/:id", post(api_toggle_platform))
        .route("/api/platforms/:id/messages", get(api_platform_messages))
        .route("/api/cron", get(api_list_cron))
        .route("/api/cron", post(api_add_cron))
        .route("/api/cron/:id/toggle", post(api_toggle_cron))
        .route("/api/cron/:id", delete(api_delete_cron))
        .route("/api/cron/:id/run", post(api_run_cron_now))
        .route("/api/gateway/status", get(api_gateway_status))
        .route("/api/gateway/messages", get(api_gateway_messages))
        .route("/api/gateway/messages/:id/read", post(api_mark_message_read))
        .route("/api/wechat/qr", post(api_wechat_qr))
        .route("/api/wechat/qr/:id", get(api_wechat_qr_image))
        .route("/api/wechat/qr/:id/status", get(api_wechat_qr_status))
        .route("/api/wechat/qr/:id/scan", post(api_wechat_qr_scan))
        .route("/api/wechat/qr/:id/confirm", post(api_wechat_qr_confirm))
        .route("/api/auth/login", post(api_login))
        .route("/api/auth/logout", post(api_logout))
        .route("/api/auth/users", get(api_list_users))
        .route("/api/i18n/config", get(api_i18n_config))
        .route("/api/i18n/locale", post(api_set_locale))
        .route("/api/upload", post(api_upload_file))
        .route("/api/upload/:id", get(api_get_upload))
        .route("/api/upload/:id", delete(api_delete_upload))
        .route("/api/cli/execute", post(api_cli_execute))
        .route("/api/cli/commands", get(api_cli_commands))
        // WebSocket route
        .route("/ws", get(ws_handler))
        // Static file routes
        .route("/webui-js", get(webui_js_handler))
        .route("/webui-css", get(webui_css_handler))
        .route("/fa.css", get(fa_css_handler))
        .route("/webfonts/*path", get(webfonts_handler))
        .route("/tauri-adapter", get(tauri_adapter_handler))
        // Default route
        .fallback(webui_handler)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    tracing::info!("Web server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn webui_handler() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        [
            ("Content-Type", "text/html; charset=utf-8"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
        ],
        axum::response::Html(include_str!("../../webui/index.html")),
    )
}

async fn webui_js_handler() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        [
            ("Content-Type", "application/javascript; charset=utf-8"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
        ],
        include_str!("../../webui/js/main.js"),
    )
}

async fn webui_css_handler() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        [
            ("Content-Type", "text/css"),
            ("Cache-Control", "no-cache, no-store, must-revalidate"),
        ],
        include_str!("../../webui/styles/main.css"),
    )
}

async fn fa_css_handler() -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::OK,
        [("Content-Type", "text/css; charset=utf-8")],
        include_str!("../../webui/fa.css"),
    )
}

async fn webfonts_handler(axum::extract::Path(path): axum::extract::Path<String>) -> impl axum::response::IntoResponse {
    let webfont_path = format!("../../webui/webfonts/{}", path);
    match std::fs::read(&webfont_path) {
        Ok(content) => {
            let mime_type = mime_guess::from_path(&webfont_path).first_or_octet_stream();
            let mime_str = mime_type.to_string();
            (axum::http::StatusCode::OK, [(axum::http::header::CONTENT_TYPE, mime_str)], content)
        }
        Err(_) => (axum::http::StatusCode::NOT_FOUND, [(axum::http::header::CONTENT_TYPE, "text/plain".to_string())], b"Not found".to_vec()),
    }
}

async fn tauri_adapter_handler() -> impl axum::response::IntoResponse {
    (axum::http::StatusCode::OK, [("Content-Type", "application/javascript")], "window.__TAURI__ = {};")
}

// ==================== API Handlers ====================

async fn api_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let config = state.config.read().await;
    let agent = state.agent.read().await;
    let memory_stats = state.memory.get_stats().unwrap_or_default();
    Json(serde_json::json!({
        "status": "running",
        "model": agent.config.model,
        "platform": config.platform,
        "active_sessions": state.session_db.list_sessions().map(|s| s.len()).unwrap_or(0),
        "available_tools": state.tool_registry.count(),
        "version": env!("CARGO_PKG_VERSION"),
        "mode": "standalone",
        "memory": {
            "facts": memory_stats.facts,
            "entities": memory_stats.entities,
            "dimensions": memory_stats.dimensions,
        }
    }))
}

async fn api_chat(
    State(state): State<Arc<AppState>>,
    Json(request): Json<Value>,
) -> Json<Value> {
    let message = request.get("message").and_then(|m| m.as_str()).unwrap_or("").to_string();
    let session_id = request.get("session_id").and_then(|s| s.as_str()).unwrap_or("").to_string();
    let model = request.get("model").and_then(|m| m.as_str());
    let api_key = request.get("apiKey").and_then(|k| k.as_str());
    let api_url = request.get("apiUrl").and_then(|u| u.as_str());
    let provider = request.get("provider").and_then(|p| p.as_str());

    tracing::info!("Received chat request: '{}' (session: {})", message, session_id);
    tracing::info!("Message bytes: {:?}", message.as_bytes());

    {
        let mut agent = state.agent.write().await;
        if let Some(key) = api_key {
            agent.config.api_key = Some(key.to_string());
        }
        if let Some(url) = api_url {
            agent.config.api_base_url = Some(url.to_string());
        }
        if let Some(m) = model {
            agent.config.model = m.to_string();
        }
        if let Some(p) = provider {
            agent.config.provider = Some(p.to_string());
        }
    }

    // 检查是否为 Ollama 本地模型（不需要 API Key）
    let is_local = {
        let agent = state.agent.read().await;
        let p = agent.config.provider.as_deref().unwrap_or("");
        p == "ollama" || p.is_empty()
    };

    // 非本地模型才需要检查 API Key
    if !is_local {
        let has_key = {
            let agent = state.agent.read().await;
            agent.config.api_key.as_ref().map(|k| !k.is_empty()).unwrap_or(false)
        };

        if !has_key {
            return Json(serde_json::json!({
                "error": "API Key not configured. Please configure your AI model in settings or pass apiKey in request.",
                "response": ""
            }));
        }
    }

    if !session_id.is_empty() {
        if let Err(e) = state.session_db.add_message(&session_id, "user", &message) {
            tracing::warn!("Failed to save user message: {}", e);
        }
        let _ = state.memory.log_interaction(Some(&session_id), "user", &message);
    }

    let result = {
        let mut agent = state.agent.write().await;
        agent.process_message(&message).await
    };

    match result {
        Ok(response) => {
            if !session_id.is_empty() {
                if let Err(e) = state.session_db.add_message(&session_id, "assistant", &response) {
                    tracing::warn!("Failed to save assistant response: {}", e);
                }
                let _ = state.memory.log_interaction(Some(&session_id), "assistant", &response);
            }
            Json(serde_json::json!({
                "response": response
            }))
        }
        Err(e) => {
            tracing::error!("Chat processing failed: {}", e);
            Json(serde_json::json!({
                "error": e.to_string(),
                "response": ""
            }))
        }
    }
}

async fn api_get_config(State(state): State<Arc<AppState>>) -> Json<Value> {
    let config = state.config.read().await;
    let agent = state.agent.read().await;

    let mut models = serde_json::Map::new();
    for (name, model_cfg) in &config.models {
        models.insert(name.clone(), serde_json::json!({
            "enabled": model_cfg.enabled,
            "model": model_cfg.model,
            "url": model_cfg.url,
            "key": if model_cfg.key.is_some() { "***" } else { "" },
        }));
    }

    Json(serde_json::json!({
        "models": models,
        "provider": agent.config.provider,
        "model": agent.config.model,
        "language": config.language,
        "theme": config.theme,
        "platforms": config.platforms,
    }))
}

async fn api_save_config(
    State(state): State<Arc<AppState>>,
    Json(config): Json<Value>,
) -> Json<Value> {
    let mut app_config = state.config.write().await;
    let current_agent = state.agent.read().await;
    let existing_api_key = current_agent.config.api_key.clone();
    drop(current_agent);

    if let Some(models) = config.get("models").and_then(|v| v.as_object()) {
        for (provider_name, provider_config) in models {
            if let Some(obj) = provider_config.as_object() {
                let enabled = obj.get("enabled").and_then(|e| e.as_bool()).unwrap_or(false);
                if enabled {
                    let model = obj.get("model").and_then(|m| m.as_str()).unwrap_or("").to_string();
                    let url = obj.get("url").and_then(|u| u.as_str()).unwrap_or("").to_string();
                    let key = obj.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();

                    if !model.is_empty() {
                        let mut agent_config = crate::agent::core::AgentConfig {
                            model: model.clone(),
                            provider: Some(provider_name.clone()),
                            api_base_url: None,
                            api_key: None,
                            temperature: app_config.temperature,
                            max_tokens: app_config.max_tokens,
                        };

                        if !url.is_empty() {
                            agent_config.api_base_url = Some(url.clone());
                        } else {
                            agent_config.api_base_url = match provider_name.as_str() {
                                "openai" => Some("https://api.openai.com/v1".to_string()),
                                "anthropic" => Some("https://api.anthropic.com/v1".to_string()),
                                "deepseek" => Some("https://api.deepseek.com/v1".to_string()),
                                "ollama" => Some("http://localhost:11434/v1".to_string()),
                                _ => None,
                            };
                        }

                        if !key.is_empty() {
                            agent_config.api_key = Some(key.clone());
                        } else if existing_api_key.is_some() {
                            agent_config.api_key = existing_api_key.clone();
                        }

                        let new_agent = crate::agent::AIAgent::new(
                            agent_config,
                            state.tool_registry.clone(),
                        );

                        let mut agent = state.agent.write().await;
                        *agent = new_agent;
                        drop(agent);

                        tracing::info!("Agent config updated: model={}, provider={}", model, provider_name);
                        break;
                    }
                }
            }
        }
    }

    if let Some(lang) = config.get("language").and_then(|v| v.as_str()) {
        app_config.language = lang.to_string();
    }
    if let Some(theme) = config.get("theme").and_then(|v| v.as_str()) {
        app_config.theme = theme.to_string();
    }

    if let Err(e) = app_config.save() {
        tracing::warn!("Failed to save config: {}", e);
    }

    Json(serde_json::json!({
        "success": true,
        "message": "Configuration saved"
    }))
}

async fn api_list_sessions(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.session_db.list_sessions().unwrap_or_default();
    let session_list: Vec<Value> = sessions.iter().map(|s| {
        serde_json::json!({
            "id": s.id,
            "title": s.title,
            "created_at": s.created_at,
            "updated_at": s.updated_at,
        })
    }).collect();
    Json(serde_json::json!({ "sessions": session_list }))
}

async fn api_create_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let id = uuid::Uuid::new_v4().to_string();
    let title = req.get("title").and_then(|t| t.as_str()).unwrap_or("New Conversation");

    if let Err(e) = state.session_db.create_session(&id, title) {
        return Json(serde_json::json!({ "error": e.to_string() }));
    }

    Json(serde_json::json!({ "id": id, "title": title }))
}

async fn api_get_session(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Value> {
    match state.session_db.get_session(&id) {
        Ok(Some(session)) => Json(serde_json::json!({
            "id": session.id,
            "title": session.title,
            "created_at": session.created_at,
            "updated_at": session.updated_at,
        })),
        _ => Json(serde_json::json!({ "error": "Session not found" })),
    }
}

async fn api_delete_session(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Value> {
    match state.session_db.delete_session(&id) {
        Ok(_) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_messages(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Json<Value> {
    let messages = state.session_db.get_messages(&id).unwrap_or_default();
    Json(serde_json::json!({ "messages": messages }))
}

async fn api_send_message(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let message = req.get("message").and_then(|m| m.as_str()).unwrap_or("");

    let _ = state.session_db.add_message(&id, "user", message);
    let _ = state.memory.log_interaction(Some(&id), "user", message);

    let mut agent = state.agent.write().await;
    match agent.process_message(message).await {
        Ok(response) => {
            let _ = state.session_db.add_message(&id, "assistant", &response);
            let _ = state.memory.log_interaction(Some(&id), "assistant", &response);
            Json(serde_json::json!({
                "response": response,
                "session_id": id,
            }))
        }
        Err(e) => Json(serde_json::json!({
            "error": e.to_string(),
            "response": ""
        })),
    }
}

async fn api_list_providers() -> Json<Value> {
    Json(serde_json::json!({
        "providers": [
            { "name": "openai", "display_name": "OpenAI", "url": "https://api.openai.com/v1" },
            { "name": "anthropic", "display_name": "Anthropic", "url": "https://api.anthropic.com/v1" },
            { "name": "deepseek", "display_name": "DeepSeek", "url": "https://api.deepseek.com/v1" },
            { "name": "ollama", "display_name": "Ollama (Local)", "url": "http://localhost:11434/v1" },
        ]
    }))
}

async fn api_list_models() -> Json<Value> {
    Json(serde_json::json!({
        "models": [
            { "provider": "openai", "name": "gpt-4o" },
            { "provider": "anthropic", "name": "claude-sonnet-4-20250514" },
            { "provider": "deepseek", "name": "deepseek-chat" },
        ]
    }))
}

async fn api_list_tools(State(state): State<Arc<AppState>>) -> Json<Value> {
    let schemas = state.tool_registry.get_tool_schemas();
    Json(serde_json::json!({ "tools": schemas }))
}

// Memory handlers
async fn api_get_memory(State(state): State<Arc<AppState>>) -> Json<Value> {
    match state.memory.list_facts(None) {
        Ok(facts) => Json(serde_json::json!({ "memories": facts })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_store_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let category = req.get("category").and_then(|v| v.as_str());

    match state.memory.add_fact(content, category) {
        Ok(fact) => Json(serde_json::json!({
            "success": true,
            "fact": fact
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_facts(State(state): State<Arc<AppState>>) -> Json<Value> {
    match state.memory.list_facts(None) {
        Ok(facts) => Json(serde_json::json!({ "facts": facts })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_add_fact(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let category = req.get("category").and_then(|v| v.as_str());

    match state.memory.add_fact(content, category) {
        Ok(fact) => Json(serde_json::json!({
            "success": true,
            "fact": fact
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_fact(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.memory.delete_fact(&id) {
        Ok(deleted) => Json(serde_json::json!({ "success": deleted, "id": id })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_search_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let query = req.get("query").and_then(|v| v.as_str()).unwrap_or("");
    match state.memory.search_facts(query) {
        Ok(results) => Json(serde_json::json!({
            "results": results,
            "query": query
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_memory_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    match state.memory.get_stats() {
        Ok(stats) => Json(serde_json::json!({
            "facts": stats.facts,
            "entities": stats.entities,
            "dimensions": stats.dimensions
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Todo handlers
async fn api_list_todos() -> Json<Value> {
    Json(serde_json::json!({ "todos": [] }))
}

async fn api_manage_todo(Json(_req): Json<Value>) -> Json<Value> {
    Json(serde_json::json!({ "success": true }))
}

// Workspace handlers
async fn api_list_workspaces(State(state): State<Arc<AppState>>) -> Json<Value> {
    let workspaces = state.workspace.list_workspaces();
    Json(serde_json::json!({ "workspaces": workspaces }))
}

async fn api_create_workspace(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let root_path = req.get("root_path").and_then(|v| v.as_str()).unwrap_or("./workspace");

    match state.workspace.create_workspace(name, root_path) {
        Ok(ws) => Json(serde_json::json!({ "success": true, "workspace": ws })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_workspace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.workspace.get_workspace(&id) {
        Some(ws) => Json(serde_json::json!({ "workspace": ws })),
        None => Json(serde_json::json!({ "error": "Workspace not found" })),
    }
}

async fn api_set_active_workspace(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.workspace.set_active(&id) {
        Ok(ws) => Json(serde_json::json!({ "success": true, "workspace": ws })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_workspace_files(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.workspace.list_files(&id, None) {
        Ok(files) => Json(serde_json::json!({ "files": files })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Skills handlers
async fn api_list_skills(State(state): State<Arc<AppState>>) -> Json<Value> {
    let skills = state.skills.list();
    Json(serde_json::json!({ "skills": skills }))
}

async fn api_install_skill(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("");
    match state.skills.install(name) {
        Ok(skill) => Json(serde_json::json!({
            "success": true,
            "skill": skill
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_manage_skill(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let action = req.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let result = match action {
        "enable" => state.skills.enable(&name),
        "disable" => state.skills.disable(&name),
        "uninstall" => state.skills.uninstall(&name),
        _ => Err(crate::error::AppError::InvalidInput(format!("Unknown action: {}", action))),
    };

    match result {
        Ok(skill) => Json(serde_json::json!({
            "success": true,
            "skill": skill
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Plugins handlers
async fn api_list_plugins(State(state): State<Arc<AppState>>) -> Json<Value> {
    let plugins = state.plugins.list();
    Json(serde_json::json!({ "plugins": plugins }))
}

async fn api_add_plugin(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = req.get("description").and_then(|v| v.as_str()).map(|s| s.to_string());
    let version = req.get("version").and_then(|v| v.as_str()).map(|s| s.to_string());

    match state.plugins.add_plugin(crate::plugins::PluginCreate { name, description, version }) {
        Ok(plugin) => Json(serde_json::json!({ "success": true, "plugin": plugin })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_manage_plugin(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let action = req.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let result = match action {
        "load" => state.plugins.load(&name),
        "unload" => state.plugins.unload(&name),
        "remove" => state.plugins.remove_plugin(&name).map(|_| state.plugins.get(&name).unwrap()),
        _ => Err(crate::error::AppError::InvalidInput(format!("Unknown action: {}", action))),
    };

    match result {
        Ok(plugin) => Json(serde_json::json!({
            "success": true,
            "plugin": plugin
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Tool toggle handler
async fn api_toggle_tool(Path(name): Path<String>, Json(req): Json<Value>) -> Json<Value> {
    let enabled = req.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    Json(serde_json::json!({
        "success": true,
        "message": format!("Tool {} {}", name, if enabled { "enabled" } else { "disabled" })
    }))
}

async fn api_execute_tool(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let arguments = req.get("arguments").and_then(|v| v.as_str()).unwrap_or("{}");

    match state.tool_registry.execute_tool(&name, arguments).await {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "result": result
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": format!("{}", e)
        })),
    }
}

// Platform handlers
async fn api_list_platforms(State(state): State<Arc<AppState>>) -> Json<Value> {
    let platforms = state.gateway.list_platforms();
    Json(serde_json::json!({ "platforms": platforms }))
}

async fn api_toggle_platform(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let action = req.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let result = match action {
        "connect" => state.gateway.connect_platform(&id),
        "disconnect" => state.gateway.disconnect_platform(&id),
        _ => Err(crate::error::AppError::InvalidInput(format!("Unknown action: {}", action))),
    };

    match result {
        Ok(platform) => Json(serde_json::json!({
            "success": true,
            "platform": platform
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_platform_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let messages = state.gateway.list_messages_by_platform(&id);
    Json(serde_json::json!({ "messages": messages }))
}

// Gateway handlers
async fn api_gateway_status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let platforms = state.gateway.list_platforms();
    let connected = state.gateway.list_connected();
    Json(serde_json::json!({
        "status": "running",
        "platforms": platforms,
        "connected_count": connected.len()
    }))
}

async fn api_gateway_messages(State(state): State<Arc<AppState>>) -> Json<Value> {
    let messages = state.gateway.list_messages();
    Json(serde_json::json!({ "messages": messages }))
}

async fn api_mark_message_read(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.gateway.mark_read(&id) {
        Ok(_) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Cron handlers
async fn api_list_cron(State(state): State<Arc<AppState>>) -> Json<Value> {
    let jobs = state.cron.list_jobs();
    Json(serde_json::json!({ "jobs": jobs }))
}

async fn api_add_cron(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let expression = req.get("expression").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let command = req.get("command").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match state.cron.add_job(crate::cron::CronJobCreate { name, expression, command }) {
        Ok(job) => Json(serde_json::json!({
            "success": true,
            "job": job
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_toggle_cron(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let action = req.get("action").and_then(|v| v.as_str()).unwrap_or("");
    let result = match action {
        "start" => state.cron.start_job(&id),
        "stop" => state.cron.stop_job(&id),
        _ => Err(crate::error::AppError::InvalidInput(format!("Unknown action: {}", action))),
    };

    match result {
        Ok(_) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_cron(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.cron.remove_job(&id) {
        Ok(_) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_run_cron_now(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.cron.run_now(&id) {
        Ok(_) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// WeChat QR handlers
async fn api_wechat_qr(State(state): State<Arc<AppState>>) -> Json<Value> {
    match state.wechat.generate_qr() {
        Ok(session) => Json(serde_json::json!({
            "success": true,
            "session_id": session.id,
            "qr_url": session.qr_url,
            "status": session.status.as_str(),
            "expires_at": session.expires_at,
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

async fn api_wechat_qr_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> impl axum::response::IntoResponse {
    match state.wechat.get_qr_image_path(&id) {
        Some(path) => {
            match std::fs::read(&path) {
                Ok(data) => (
                    axum::http::StatusCode::OK,
                    [(axum::http::header::CONTENT_TYPE, "image/png")],
                    data,
                ),
                Err(_) => (
                    axum::http::StatusCode::NOT_FOUND,
                    [(axum::http::header::CONTENT_TYPE, "text/plain")],
                    b"Image not found".to_vec(),
                ),
            }
        }
        None => (
            axum::http::StatusCode::NOT_FOUND,
            [(axum::http::header::CONTENT_TYPE, "text/plain")],
            b"QR session not found".to_vec(),
        ),
    }
}

async fn api_wechat_qr_status(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.wechat.check_status(&id) {
        Ok(status) => Json(serde_json::json!({
            "success": true,
            "status": status.as_str()
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

async fn api_wechat_qr_scan(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.wechat.simulate_scan(&id) {
        Ok(ok) => Json(serde_json::json!({
            "success": ok,
            "message": if ok { "QR code scanned" } else { "Cannot scan QR code" }
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

async fn api_wechat_qr_confirm(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.wechat.simulate_confirm(&id) {
        Ok(ok) => Json(serde_json::json!({
            "success": ok,
            "message": if ok { "QR code confirmed" } else { "Cannot confirm QR code" }
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

// Role configuration handlers
async fn api_get_role_config() -> Json<Value> {
    let soul_md = std::fs::read_to_string(".mahakala/soul.md")
        .unwrap_or_else(|_| "".to_string());
    let user_md = std::fs::read_to_string(".mahakala/user.md")
        .unwrap_or_else(|_| "".to_string());
    Json(serde_json::json!({
        "success": true,
        "soul_md": soul_md,
        "user_md": user_md
    }))
}

async fn api_save_role_config(Json(req): Json<Value>) -> Json<Value> {
    let _ = std::fs::create_dir_all(".mahakala");
    if let Some(soul) = req.get("soul_md").and_then(|v| v.as_str()) {
        let _ = std::fs::write(".mahakala/soul.md", soul);
    }
    if let Some(user) = req.get("user_md").and_then(|v| v.as_str()) {
        let _ = std::fs::write(".mahakala/user.md", user);
    }
    Json(serde_json::json!({ "success": true }))
}

// Auth handlers
async fn api_login(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let username = req.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let password = req.get("password").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match state.auth.login(crate::auth::LoginRequest { username, password }) {
        Ok(session) => Json(serde_json::json!({
            "success": true,
            "token": session.token,
            "username": session.username
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

async fn api_logout(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let token = req.get("token").and_then(|v| v.as_str()).unwrap_or("");
    match state.auth.logout(token) {
        Ok(_) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_users(State(state): State<Arc<AppState>>) -> Json<Value> {
    let users = state.auth.list_users();
    Json(serde_json::json!({ "users": users }))
}

// i18n handlers
async fn api_i18n_config(State(state): State<Arc<AppState>>) -> Json<Value> {
    let config = state.i18n.get_config();
    Json(serde_json::json!(config))
}

async fn api_set_locale(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let locale = req.get("locale").and_then(|v| v.as_str()).unwrap_or("zh");
    match state.i18n.set_locale(locale) {
        Ok(_) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Upload handlers
async fn api_upload_file(
    State(_state): State<Arc<AppState>>,
) -> Json<Value> {
    Json(serde_json::json!({
        "success": false,
        "error": "Multipart upload not implemented in this handler"
    }))
}

async fn api_get_upload(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.upload.get_file(&id) {
        Some(file) => Json(serde_json::json!({ "file": file })),
        None => Json(serde_json::json!({ "error": "File not found" })),
    }
}

async fn api_delete_upload(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.upload.delete_file(&id) {
        Ok(deleted) => Json(serde_json::json!({ "success": deleted })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// CLI handlers
async fn api_cli_execute(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let command = req.get("command").and_then(|v| v.as_str()).unwrap_or("");
    match state.cli.execute(command) {
        Ok(output) => Json(serde_json::json!({
            "success": true,
            "output": output
        })),
        Err(e) => Json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        })),
    }
}

async fn api_cli_commands(State(state): State<Arc<AppState>>) -> Json<Value> {
    let commands = state.cli.list_commands();
    Json(serde_json::json!({ "commands": commands }))
}

// WebSocket handler
async fn ws_handler(
    ws: WebSocketUpgrade,
    State(_state): State<Arc<AppState>>,
) -> Response {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: axum::extract::ws::WebSocket) {
    use axum::extract::ws::Message;
    
    let welcome = serde_json::json!({
        "type": "notification",
        "level": "info",
        "message": "WebSocket connected"
    });
    let _ = socket.send(Message::Text(welcome.to_string())).await;
    
    while let Some(msg) = socket.recv().await {
        if let Ok(msg) = msg {
            match msg {
                Message::Text(text) => {
                    let response = serde_json::json!({
                        "type": "status",
                        "message": format!("Received: {}", text)
                    });
                    let _ = socket.send(Message::Text(response.to_string())).await;
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    }
}
