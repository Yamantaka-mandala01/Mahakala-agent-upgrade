use axum::{
    routing::{get, post, delete, put},
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
use crate::learning::LearningLoop;
use crate::semantic_memory::SemanticSearchEngine;
use crate::delegation::DelegationSystem;
use crate::mcp::McpClient;
use crate::gateways::GatewayManager;
use crate::terminal::{DockerManager, DockerConfig, SshManager};
use crate::browser::{BrowserManager, BrowserConfig};
use crate::voice::{VoiceManager, SpeechConfig};
use crate::image::{ImageManager, ImageConfig};
use crate::multiagent::{MultiAgentFramework, CollaborationConfig};
use crate::rbac::{RbacSystem, RbacConfig};
use crate::audit::{AuditSystem, AuditConfig};
use crate::serverless::ServerlessManager;
use crate::community_skills::CommunitySkillCenter;
use crate::trajectory::TrajectoryGenerator;
use crate::rl_environment::RlEnvironment;
use crate::honcho::HonchoModeling;
use crate::fts_search::FtsSearchEngine;
use crate::memory_prompt::MemoryPromptingSystem;
use std::collections::HashMap;
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
    pub learning: Arc<LearningLoop>,
    pub semantic_search: Arc<SemanticSearchEngine>,
    pub delegation: Arc<DelegationSystem>,
    pub mcp: Arc<McpClient>,
    pub gateways: Arc<GatewayManager>,
    pub docker: Arc<DockerManager>,
    pub ssh: Arc<SshManager>,
    pub browser: Arc<BrowserManager>,
    pub voice: Arc<VoiceManager>,
    pub image: Arc<ImageManager>,
    pub multiagent: Arc<MultiAgentFramework>,
    pub rbac: Arc<RbacSystem>,
    pub audit: Arc<AuditSystem>,
    pub serverless: Arc<ServerlessManager>,
    pub community_skills: Arc<CommunitySkillCenter>,
    pub trajectory_generator: Arc<TrajectoryGenerator>,
    pub rl_environment: Arc<RlEnvironment>,
    pub honcho_modeling: Arc<HonchoModeling>,
    pub fts_search: Arc<FtsSearchEngine>,
    pub memory_prompting: Arc<MemoryPromptingSystem>,
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
        .route("/api/learning/experiences", get(api_get_experiences))
        .route("/api/learning/experiences", post(api_record_experience))
        .route("/api/learning/insights", get(api_get_insights))
        .route("/api/learning/skills", post(api_create_skill_from_experience))
        .route("/api/learning/improvements", get(api_get_improvements))
        .route("/api/semantic/search", post(api_semantic_search))
        .route("/api/semantic/add", post(api_semantic_add))
        .route("/api/semantic/delete/:id", delete(api_semantic_delete))
        .route("/api/semantic/similar", post(api_semantic_similar))
        .route("/api/semantic/clusters", get(api_semantic_clusters))
        .route("/api/semantic/stats", get(api_semantic_stats))
        .route("/api/delegation/agents", get(api_list_sub_agents))
        .route("/api/delegation/agents", post(api_create_sub_agent))
        .route("/api/delegation/agents/:id", delete(api_remove_sub_agent))
        .route("/api/delegation/tasks", get(api_list_tasks))
        .route("/api/delegation/tasks", post(api_create_task))
        .route("/api/delegation/tasks/:id/delegate", post(api_delegate_task))
        .route("/api/delegation/tasks/parallel", post(api_delegate_parallel))
        .route("/api/delegation/tasks/:id/cancel", post(api_cancel_task))
        .route("/api/delegation/stats", get(api_delegation_stats))
        .route("/api/mcp/servers", get(api_list_mcp_servers))
        .route("/api/mcp/servers", post(api_add_mcp_server))
        .route("/api/mcp/servers/:id", delete(api_remove_mcp_server))
        .route("/api/mcp/servers/:id/connect", post(api_connect_mcp_server))
        .route("/api/mcp/servers/:id/disconnect", post(api_disconnect_mcp_server))
        .route("/api/mcp/tools", get(api_list_mcp_tools))
        .route("/api/gateways", get(api_list_gateways))
        .route("/api/gateways", post(api_add_gateway))
        .route("/api/gateways/:id", delete(api_remove_gateway))
        .route("/api/gateways/:id/enable", post(api_enable_gateway))
        .route("/api/gateways/:id/disable", post(api_disable_gateway))
        .route("/api/gateways/status", get(api_gateway_statuses))
        .route("/api/gateways/:id/send", post(api_gateway_send_message))
        .route("/api/docker/containers", get(api_list_containers))
        .route("/api/docker/containers", post(api_create_container))
        .route("/api/docker/containers/:id/start", post(api_start_container))
        .route("/api/docker/containers/:id/stop", post(api_stop_container))
        .route("/api/docker/containers/:id", delete(api_remove_container))
        .route("/api/docker/containers/:id/exec", post(api_exec_in_container))
        .route("/api/docker/containers/:id/logs", get(api_container_logs))
        .route("/api/docker/stats", get(api_docker_stats))
        .route("/api/ssh/connections", get(api_list_ssh_connections))
        .route("/api/ssh/connections", post(api_add_ssh_connection))
        .route("/api/ssh/connections/:id", delete(api_remove_ssh_connection))
        .route("/api/ssh/connections/:id/connect", post(api_connect_ssh))
        .route("/api/ssh/connections/:id/disconnect", post(api_disconnect_ssh))
        .route("/api/ssh/connections/:id/exec", post(api_ssh_exec))
        .route("/api/ssh/sessions", get(api_list_ssh_sessions))
        .route("/api/browser/instances", get(api_list_browser_instances))
        .route("/api/browser/launch", post(api_launch_browser))
        .route("/api/browser/:id/close", post(api_close_browser))
        .route("/api/browser/:id/navigate", post(api_browser_navigate))
        .route("/api/browser/:id/click", post(api_browser_click))
        .route("/api/browser/:id/fill", post(api_browser_fill))
        .route("/api/browser/:id/content", get(api_browser_content))
        .route("/api/browser/:id/screenshot", post(api_browser_screenshot))
        .route("/api/browser/:id/evaluate", post(api_browser_evaluate))
        .route("/api/browser/stats", get(api_browser_stats))
        .route("/api/voice/streams", get(api_list_voice_streams))
        .route("/api/voice/stream", post(api_start_voice_stream))
        .route("/api/voice/stream/:id/stop", post(api_stop_voice_stream))
        .route("/api/voice/stream/:id/process", post(api_process_audio))
        .route("/api/voice/stream/:id/transcribe", post(api_transcribe_audio))
        .route("/api/voice/synthesize", post(api_synthesize_speech))
        .route("/api/voice/stats", get(api_voice_stats))
        .route("/api/image/generate", post(api_generate_image))
        .route("/api/image/analyze", post(api_analyze_image))
        .route("/api/image/generated", get(api_list_generated_images))
        .route("/api/image/generated/:id", get(api_get_generated_image))
        .route("/api/image/analyses", get(api_list_analyses))
        .route("/api/image/analyses/:id", get(api_get_analysis))
        .route("/api/image/stats", get(api_image_stats))
        .route("/api/multiagent/agents", get(api_ma_list_agents))
        .route("/api/multiagent/agents", post(api_ma_register_agent))
        .route("/api/multiagent/agents/:id", delete(api_ma_unregister_agent))
        .route("/api/multiagent/agents/:id", get(api_ma_get_agent))
        .route("/api/multiagent/agents/:id/messages", get(api_ma_get_agent_messages))
        .route("/api/multiagent/tasks", get(api_ma_list_tasks))
        .route("/api/multiagent/tasks", post(api_ma_create_task))
        .route("/api/multiagent/tasks/:id", get(api_ma_get_task))
        .route("/api/multiagent/tasks/:id/start", post(api_ma_start_task))
        .route("/api/multiagent/tasks/:id/progress", post(api_ma_update_progress))
        .route("/api/multiagent/tasks/:id/complete", post(api_ma_complete_task))
        .route("/api/multiagent/message", post(api_ma_send_message))
        .route("/api/multiagent/stats", get(api_ma_stats))
        .route("/api/rbac/roles", get(api_list_roles))
        .route("/api/rbac/roles", post(api_create_role))
        .route("/api/rbac/roles/:id", get(api_get_role))
        .route("/api/rbac/roles/:id", put(api_update_role))
        .route("/api/rbac/roles/:id", delete(api_delete_role))
        .route("/api/rbac/users", get(api_list_users_rbac))
        .route("/api/rbac/users", post(api_create_user_rbac))
        .route("/api/rbac/users/:id", get(api_get_user_rbac))
        .route("/api/rbac/users/:id/role", put(api_update_user_role))
        .route("/api/rbac/users/:id", delete(api_delete_user_rbac))
        .route("/api/rbac/users/:id/permissions", get(api_get_user_permissions))
        .route("/api/rbac/users/:id/check-permission", post(api_check_user_permission))
        .route("/api/rbac/permissions", get(api_list_permissions))
        .route("/api/rbac/permissions", post(api_create_permission))
        .route("/api/rbac/stats", get(api_rbac_stats))
        .route("/api/audit/logs", get(api_get_audit_logs))
        .route("/api/audit/logs/:id", get(api_get_audit_log))
        .route("/api/audit/logs/:id", delete(api_delete_audit_log))
        .route("/api/audit/logs/purge", post(api_purge_audit_logs))
        .route("/api/audit/logs/export", post(api_export_audit_logs))
        .route("/api/audit/reports", get(api_get_audit_reports))
        .route("/api/audit/reports/:id", get(api_get_audit_report))
        .route("/api/audit/reports", post(api_generate_audit_report))
        .route("/api/audit/stats", get(api_audit_stats))
        .route("/api/serverless/configs", get(api_list_serverless_configs))
        .route("/api/serverless/configs", post(api_create_serverless_config))
        .route("/api/serverless/configs/:id", get(api_get_serverless_config))
        .route("/api/serverless/configs/:id", put(api_update_serverless_config))
        .route("/api/serverless/configs/:id", delete(api_delete_serverless_config))
        .route("/api/serverless/packages", get(api_list_serverless_packages))
        .route("/api/serverless/packages", post(api_create_serverless_package))
        .route("/api/serverless/packages/:id/deploy", post(api_deploy_serverless_package))
        .route("/api/serverless/scaling-policies", get(api_list_scaling_policies))
        .route("/api/serverless/scaling-policies", post(api_create_scaling_policy))
        .route("/api/serverless/scaling-policies/:id", delete(api_delete_scaling_policy))
        .route("/api/serverless/providers", get(api_list_serverless_providers))
        .route("/api/serverless/providers/:name", get(api_get_serverless_provider))
        .route("/api/serverless/invocations/:id", post(api_record_invocation))
        .route("/api/serverless/stats", get(api_serverless_stats))
        .route("/api/serverless/template/:id", post(api_generate_serverless_template))
        .route("/api/community-skills", get(api_list_community_skills))
        .route("/api/community-skills", post(api_publish_community_skill))
        .route("/api/community-skills/:id", get(api_get_community_skill))
        .route("/api/community-skills/:id", put(api_update_community_skill))
        .route("/api/community-skills/:id", delete(api_delete_community_skill))
        .route("/api/community-skills/:id/download", post(api_download_community_skill))
        .route("/api/community-skills/:id/reviews", get(api_get_skill_reviews))
        .route("/api/community-skills/:id/reviews", post(api_add_skill_review))
        .route("/api/community-skills/user/:user_id", get(api_get_user_skills))
        .route("/api/community-skills/categories", get(api_get_skill_categories))
        .route("/api/community-skills/stats", get(api_community_skills_stats))
        .route("/api/trajectory/configs", post(api_create_trajectory_config))
        .route("/api/trajectory/configs/:id", get(api_get_trajectory_config))
        .route("/api/trajectory/batches", get(api_list_trajectory_batches))
        .route("/api/trajectory/batches", post(api_generate_trajectory_batch))
        .route("/api/trajectory/batches/:id", get(api_get_trajectory_batch))
        .route("/api/trajectory/batches/:id", delete(api_delete_trajectory_batch))
        .route("/api/trajectory/batches/:id/export", post(api_export_trajectory_batch))
        .route("/api/trajectory/trajectories/:id", get(api_get_trajectory))
        .route("/api/trajectory/stats", get(api_trajectory_stats))
        .route("/api/rl-environment/configs", post(api_create_rl_config))
        .route("/api/rl-environment/configs/:id", get(api_get_rl_config))
        .route("/api/rl-environment/environments", get(api_list_rl_environments))
        .route("/api/rl-environment/environments", post(api_create_rl_environment))
        .route("/api/rl-environment/environments/:id", get(api_get_rl_environment))
        .route("/api/rl-environment/environments/:id/step", post(api_rl_environment_step))
        .route("/api/rl-environment/environments/:id/reset", post(api_rl_environment_reset))
        .route("/api/rl-environment/environments/:id/close", post(api_rl_environment_close))
        .route("/api/rl-environment/environments/:id/history", get(api_rl_environment_history))
        .route("/api/rl-environment/stats", get(api_rl_environment_stats))
        .route("/api/honcho/users", post(api_honcho_create_user))
        .route("/api/honcho/users", get(api_honcho_list_users))
        .route("/api/honcho/users/:id", get(api_honcho_get_user))
        .route("/api/honcho/users/:id", delete(api_honcho_delete_user))
        .route("/api/honcho/users/:id/interact", post(api_honcho_record_interaction))
        .route("/api/honcho/users/:id/profile", get(api_honcho_get_profile))
        .route("/api/honcho/users/:id/dialectical", get(api_honcho_get_dialectical_state))
        .route("/api/honcho/users/:id/history", get(api_honcho_get_history))
        .route("/api/honcho/users/:id/preference", post(api_honcho_update_preference))
        .route("/api/honcho/users/:id/goal", post(api_honcho_add_goal))
        .route("/api/honcho/users/:id/analytics", get(api_honcho_get_analytics))
        .route("/api/honcho/stats", get(api_honcho_stats))
        .route("/api/fts/documents", post(api_fts_add_document))
        .route("/api/fts/documents/:id", get(api_fts_get_document))
        .route("/api/fts/documents/:id", put(api_fts_update_document))
        .route("/api/fts/documents/:id", delete(api_fts_delete_document))
        .route("/api/fts/documents", get(api_fts_list_documents))
        .route("/api/fts/search", post(api_fts_search))
        .route("/api/fts/summaries/:id", post(api_fts_generate_summary))
        .route("/api/fts/summaries/:id", get(api_fts_get_summary))
        .route("/api/fts/history", get(api_fts_search_history))
        .route("/api/fts/stats", get(api_fts_stats))
        .route("/api/memory-prompt/schedules", post(api_mp_create_schedule))
        .route("/api/memory-prompt/schedules/:id", get(api_mp_get_schedule))
        .route("/api/memory-prompt/schedules", get(api_mp_list_schedules))
        .route("/api/memory-prompt/prompts", post(api_mp_create_prompt))
        .route("/api/memory-prompt/prompts/:id", get(api_mp_get_prompt))
        .route("/api/memory-prompt/prompts", get(api_mp_list_prompts))
        .route("/api/memory-prompt/prompts/:id/trigger", post(api_mp_trigger_prompt))
        .route("/api/memory-prompt/prompts/:id/deactivate", post(api_mp_deactivate_prompt))
        .route("/api/memory-prompt/prompts/:id/activate", post(api_mp_activate_prompt))
        .route("/api/memory-prompt/responses", post(api_mp_submit_response))
        .route("/api/memory-prompt/responses/:prompt_id", get(api_mp_get_responses))
        .route("/api/memory-prompt/config", post(api_mp_set_user_config))
        .route("/api/memory-prompt/config/:user_id", get(api_mp_get_user_config))
        .route("/api/memory-prompt/due", get(api_mp_get_due_prompts))
        .route("/api/memory-prompt/stats/:user_id", get(api_mp_get_user_stats))
        .route("/api/memory-prompt/stats", get(api_mp_get_stats))
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
    State(state): State<Arc<AppState>>,
    mut multipart: axum::extract::Multipart,
) -> Json<Value> {
    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let name = field.name().unwrap_or("").to_string();
        if name == "file" {
            let filename = field.file_name().unwrap_or("unknown").to_string();
            let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
            let data = match field.bytes().await {
                Ok(d) => d.to_vec(),
                Err(e) => {
                    return Json(serde_json::json!({ "error": format!("Failed to read file: {}", e) }));
                }
            };
            match state.upload.save_file(&filename, &data, &content_type) {
                Ok(file) => {
                    return Json(serde_json::json!({
                        "success": true,
                        "id": file.id,
                        "name": file.original_name,
                        "size": file.size,
                        "mime_type": file.mime_type
                    }));
                }
                Err(e) => {
                    return Json(serde_json::json!({ "error": format!("Failed to save file: {}", e) }));
                }
            }
        }
    }
    Json(serde_json::json!({ "error": "No file field found in multipart data" }))
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

async fn api_get_experiences(State(state): State<Arc<AppState>>) -> Json<Value> {
    let experiences = state.learning.get_experiences();
    let exp_list: Vec<Value> = experiences.iter().map(|e| {
        serde_json::json!({
            "id": e.id,
            "task_description": e.task_description,
            "tools_used": e.tools_used,
            "skills_used": e.skills_used,
            "success": e.success,
            "feedback": e.feedback,
            "timestamp": e.timestamp,
            "lessons_learned": e.lessons_learned,
        })
    }).collect();
    Json(serde_json::json!({ "experiences": exp_list }))
}

async fn api_record_experience(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    use crate::learning::LearningExperience;
    
    let id = uuid::Uuid::new_v4().to_string();
    let task_description = req.get("task_description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let tools_used: Vec<String> = req.get("tools_used")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let skills_used: Vec<String> = req.get("skills_used")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let success = req.get("success").and_then(|v| v.as_bool()).unwrap_or(false);
    let feedback = req.get("feedback").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let lessons_learned = req.get("lessons_learned").and_then(|v| v.as_str()).unwrap_or("").to_string();

    let experience = LearningExperience {
        id,
        task_description,
        tools_used,
        skills_used,
        success,
        feedback,
        timestamp: chrono::Utc::now().timestamp(),
        lessons_learned,
    };

    match state.learning.record_experience(experience.clone()) {
        Ok(_) => {
            let _ = state.learning.analyze_experience(&experience);
            Json(serde_json::json!({
                "success": true,
                "experience": experience
            }))
        }
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_insights(State(state): State<Arc<AppState>>) -> Json<Value> {
    match state.learning.get_learning_insights() {
        Ok(insights) => Json(serde_json::json!({
            "success": true,
            "insights": insights
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_create_skill_from_experience(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let experience_id = req.get("experience_id").and_then(|v| v.as_str()).unwrap_or("");
    
    let experiences = state.learning.get_experiences();
    let experience = experiences.iter().find(|e| e.id == experience_id).cloned();
    
    if let Some(exp) = experience {
        match state.learning.create_skill_from_experience(&exp) {
            Ok(skill) => Json(serde_json::json!({
                "success": true,
                "skill": skill
            })),
            Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
        }
    } else {
        Json(serde_json::json!({ "error": "Experience not found" }))
    }
}

async fn api_get_improvements(State(state): State<Arc<AppState>>) -> Json<Value> {
    match state.learning.suggest_skill_improvements() {
        Ok(improvements) => Json(serde_json::json!({
            "success": true,
            "improvements": improvements
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_semantic_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let query = req.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let top_k = req.get("top_k").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
    
    match state.semantic_search.search(query, top_k) {
        Ok(results) => Json(serde_json::json!({
            "success": true,
            "results": results,
            "query": query
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_semantic_add(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let metadata = req.get("metadata").cloned();
    
    match state.semantic_search.add_memory(content, metadata) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_semantic_delete(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.semantic_search.delete_memory(&id) {
        Ok(deleted) => Json(serde_json::json!({
            "success": true,
            "deleted": deleted
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_semantic_similar(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let query = req.get("query").and_then(|v| v.as_str()).unwrap_or("");
    let threshold = req.get("threshold").and_then(|v| v.as_f64()).unwrap_or(0.5);
    
    match state.semantic_search.find_similar_concepts(query, threshold) {
        Ok(concepts) => Json(serde_json::json!({
            "success": true,
            "concepts": concepts
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_semantic_clusters(State(state): State<Arc<AppState>>) -> Json<Value> {
    let num_clusters = 5;
    match state.semantic_search.cluster_memories(num_clusters) {
        Ok(clusters) => Json(serde_json::json!({
            "success": true,
            "clusters": clusters
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_semantic_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let count = state.semantic_search.get_memory_count();
    Json(serde_json::json!({
        "success": true,
        "memory_count": count
    }))
}

async fn api_list_sub_agents(State(state): State<Arc<AppState>>) -> Json<Value> {
    let agents = state.delegation.list_sub_agents();
    Json(serde_json::json!({
        "success": true,
        "agents": agents
    }))
}

async fn api_create_sub_agent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    use crate::delegation::SubAgentConfig;
    
    let id = uuid::Uuid::new_v4().to_string();
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("Sub Agent").to_string();
    let role = req.get("role").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let model = req.get("model").and_then(|v| v.as_str()).unwrap_or("llama3").to_string();
    let provider = req.get("provider").and_then(|v| v.as_str()).map(String::from);
    let api_base_url = req.get("api_base_url").and_then(|v| v.as_str()).map(String::from);
    let api_key = req.get("api_key").and_then(|v| v.as_str()).map(String::from);
    let max_tokens = req.get("max_tokens").and_then(|v| v.as_u64()).map(|v| v as u32);
    let temperature = req.get("temperature").and_then(|v| v.as_f64());
    let tools: Vec<String> = req.get("tools")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let config = SubAgentConfig {
        id,
        name,
        role,
        model,
        provider,
        api_base_url,
        api_key,
        max_tokens,
        temperature,
        tools,
    };

    match state.delegation.create_sub_agent(config.clone()) {
        Ok(agent_id) => Json(serde_json::json!({
            "success": true,
            "agent_id": agent_id,
            "config": config
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_remove_sub_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.delegation.remove_sub_agent(&id) {
        Ok(removed) => Json(serde_json::json!({
            "success": true,
            "removed": removed
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_tasks(State(state): State<Arc<AppState>>) -> Json<Value> {
    let tasks = state.delegation.list_tasks();
    Json(serde_json::json!({
        "success": true,
        "tasks": tasks
    }))
}

async fn api_create_task(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let description = req.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let priority = req.get("priority").and_then(|v| v.as_u64()).unwrap_or(1) as u8;
    
    let task_id = state.delegation.create_task(&description, priority);
    
    Json(serde_json::json!({
        "success": true,
        "task_id": task_id
    }))
}

async fn api_delegate_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let sub_agent_id = req.get("sub_agent_id").and_then(|v| v.as_str()).unwrap_or("");
    
    match state.delegation.delegate_task(&id, sub_agent_id).await {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "result": result
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delegate_parallel(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let task_ids: Vec<String> = req.get("task_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    
    let sub_agent_ids: Vec<String> = req.get("sub_agent_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    
    if task_ids.is_empty() || sub_agent_ids.is_empty() {
        return Json(serde_json::json!({ "error": "task_ids and sub_agent_ids are required" }));
    }
    
    match state.delegation.delegate_parallel(&task_ids, &sub_agent_ids).await {
        Ok(results) => Json(serde_json::json!({
            "success": true,
            "results": results
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_cancel_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.delegation.cancel_task(&id) {
        Ok(cancelled) => Json(serde_json::json!({
            "success": true,
            "cancelled": cancelled
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delegation_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.delegation.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

async fn api_list_mcp_servers(State(state): State<Arc<AppState>>) -> Json<Value> {
    let servers = state.mcp.list_servers();
    Json(serde_json::json!({
        "success": true,
        "servers": servers
    }))
}

async fn api_add_mcp_server(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let id = uuid::Uuid::new_v4().to_string();
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("MCP Server").to_string();
    let url = req.get("url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = req.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match state.mcp.add_server(id.clone(), name, url, description) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "server_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_remove_mcp_server(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.mcp.remove_server(&id) {
        Ok(removed) => Json(serde_json::json!({
            "success": true,
            "removed": removed
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_connect_mcp_server(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.mcp.connect_server(&id) {
        Ok(tools) => Json(serde_json::json!({
            "success": true,
            "tools": tools
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_disconnect_mcp_server(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.mcp.disconnect_server(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_mcp_tools(State(state): State<Arc<AppState>>) -> Json<Value> {
    let tools = state.mcp.get_all_tools();
    Json(serde_json::json!({
        "success": true,
        "tools": tools
    }))
}

async fn api_list_gateways(State(state): State<Arc<AppState>>) -> Json<Value> {
    let gateways = state.gateways.list_gateways();
    Json(serde_json::json!({
        "success": true,
        "gateways": gateways
    }))
}

async fn api_add_gateway(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    use crate::gateways::GatewayConfig;
    
    let id = uuid::Uuid::new_v4().to_string();
    let platform = req.get("platform").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("Gateway").to_string();
    let token = req.get("token").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let webhook_url = req.get("webhook_url").and_then(|v| v.as_str()).map(String::from);
    let enabled = req.get("enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let settings = req.get("settings").cloned().unwrap_or(serde_json::json!({}));

    let config = GatewayConfig {
        id,
        platform,
        name,
        enabled,
        token,
        webhook_url,
        settings,
    };

    match state.gateways.add_gateway(config.clone()) {
        Ok(_) => Json(serde_json::json!({
            "success": true,
            "gateway_id": config.id,
            "config": config
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_remove_gateway(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.gateways.remove_gateway(&id) {
        Ok(removed) => Json(serde_json::json!({
            "success": true,
            "removed": removed
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_enable_gateway(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.gateways.enable_gateway(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_disable_gateway(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.gateways.disable_gateway(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_gateway_statuses(State(state): State<Arc<AppState>>) -> Json<Value> {
    let statuses = state.gateways.list_statuses();
    Json(serde_json::json!({
        "success": true,
        "statuses": statuses
    }))
}

async fn api_gateway_send_message(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let chat_id = req.get("chat_id").and_then(|v| v.as_str()).unwrap_or("");
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("");

    match state.gateways.send_message(&id, chat_id, content) {
        Ok(message_id) => Json(serde_json::json!({
            "success": true,
            "message_id": message_id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_containers(State(state): State<Arc<AppState>>) -> Json<Value> {
    let containers = state.docker.list_containers();
    Json(serde_json::json!({
        "success": true,
        "containers": containers
    }))
}

async fn api_create_container(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("container").to_string();
    let image = req.get("image").and_then(|v| v.as_str()).unwrap_or("nginx:latest").to_string();

    match state.docker.create_container(name, image) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "container_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_start_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.docker.start_container(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_stop_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.docker.stop_container(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_remove_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.docker.remove_container(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_exec_in_container(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let command = req.get("command").and_then(|v| v.as_str()).unwrap_or("");

    match state.docker.exec_in_container(&id, command) {
        Ok(output) => Json(serde_json::json!({
            "success": true,
            "output": output
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_container_logs(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.docker.get_container_logs(&id) {
        Ok(logs) => Json(serde_json::json!({
            "success": true,
            "logs": logs
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_docker_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.docker.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

async fn api_list_ssh_connections(State(state): State<Arc<AppState>>) -> Json<Value> {
    let connections = state.ssh.list_connections();
    Json(serde_json::json!({
        "success": true,
        "connections": connections
    }))
}

async fn api_add_ssh_connection(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let host = req.get("host").and_then(|v| v.as_str()).unwrap_or("localhost").to_string();
    let port = req.get("port").and_then(|v| v.as_u64()).unwrap_or(22) as u16;
    let username = req.get("username").and_then(|v| v.as_str()).unwrap_or("root").to_string();

    match state.ssh.add_connection(host, port, username) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "connection_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_remove_ssh_connection(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.ssh.remove_connection(&id) {
        Ok(removed) => Json(serde_json::json!({
            "success": true,
            "removed": removed
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_connect_ssh(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.ssh.connect(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_disconnect_ssh(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.ssh.disconnect(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ssh_exec(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let command = req.get("command").and_then(|v| v.as_str()).unwrap_or("");

    match state.ssh.execute_command(&id, command) {
        Ok(output) => Json(serde_json::json!({
            "success": true,
            "output": output
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_ssh_sessions(State(state): State<Arc<AppState>>) -> Json<Value> {
    let sessions = state.ssh.list_sessions();
    Json(serde_json::json!({
        "success": true,
        "sessions": sessions
    }))
}

async fn api_list_browser_instances(State(state): State<Arc<AppState>>) -> Json<Value> {
    let instances = state.browser.list_instances();
    Json(serde_json::json!({
        "success": true,
        "instances": instances
    }))
}

async fn api_launch_browser(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let browser_type = req.get("browser_type").and_then(|v| v.as_str()).map(String::from);

    match state.browser.launch_browser(browser_type) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "instance_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_close_browser(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.browser.close_browser(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_browser_navigate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let url = req.get("url").and_then(|v| v.as_str()).unwrap_or("");

    match state.browser.navigate(&id, url) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_browser_click(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let selector = req.get("selector").and_then(|v| v.as_str()).unwrap_or("");

    match state.browser.click(&id, selector) {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "result": result
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_browser_fill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let selector = req.get("selector").and_then(|v| v.as_str()).unwrap_or("");
    let value = req.get("value").and_then(|v| v.as_str()).unwrap_or("");

    match state.browser.fill(&id, selector, value) {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "result": result
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_browser_content(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.browser.get_content(&id) {
        Ok(content) => Json(serde_json::json!({
            "success": true,
            "content": content
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_browser_screenshot(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let path = req.get("path").and_then(|v| v.as_str()).unwrap_or("screenshot.png");
    let format = req.get("format").and_then(|v| v.as_str()).unwrap_or("png");

    match state.browser.screenshot(&id, path, format) {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "screenshot": result
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_browser_evaluate(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let script = req.get("script").and_then(|v| v.as_str()).unwrap_or("");

    match state.browser.evaluate(&id, script) {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "result": result
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_browser_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.browser.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

async fn api_list_voice_streams(State(state): State<Arc<AppState>>) -> Json<Value> {
    let streams = state.voice.list_streams();
    Json(serde_json::json!({
        "success": true,
        "streams": streams
    }))
}

async fn api_start_voice_stream(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let session_id = req.get("session_id").and_then(|v| v.as_str()).unwrap_or("default");

    match state.voice.start_stream(session_id) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "stream_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_stop_voice_stream(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.voice.stop_stream(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_process_audio(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let audio_data = req.get("audio_data").and_then(|v| v.as_str()).unwrap_or("");

    match state.voice.process_audio_chunk(&id, audio_data.as_bytes()) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_transcribe_audio(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.voice.transcribe(&id) {
        Ok(result) => Json(serde_json::json!({
            "success": true,
            "transcription": result
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_synthesize_speech(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let text = req.get("text").and_then(|v| v.as_str()).unwrap_or("");

    match state.voice.synthesize_speech(text) {
        Ok(audio_data) => Json(serde_json::json!({
            "success": true,
            "audio_length": audio_data.len()
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_voice_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.voice.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

async fn api_generate_image(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let prompt = req.get("prompt").and_then(|v| v.as_str()).unwrap_or("");
    let width = req.get("width").and_then(|v| v.as_u64()).map(|v| v as u32);
    let height = req.get("height").and_then(|v| v.as_u64()).map(|v| v as u32);

    match state.image.generate_image(prompt, width, height) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "image_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_analyze_image(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let image_path = req.get("image_path").and_then(|v| v.as_str()).unwrap_or("");
    let analysis_prompt = req.get("analysis_prompt").and_then(|v| v.as_str());

    match state.image.analyze_image(image_path, analysis_prompt) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "analysis_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_generated_images(State(state): State<Arc<AppState>>) -> Json<Value> {
    let images = state.image.list_generated_images();
    Json(serde_json::json!({
        "success": true,
        "images": images
    }))
}

async fn api_get_generated_image(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.image.get_generated_image(&id) {
        Some(image) => Json(serde_json::json!({
            "success": true,
            "image": image
        })),
        None => Json(serde_json::json!({ "error": "Image not found" })),
    }
}

async fn api_list_analyses(State(state): State<Arc<AppState>>) -> Json<Value> {
    let analyses = state.image.list_analyses();
    Json(serde_json::json!({
        "success": true,
        "analyses": analyses
    }))
}

async fn api_get_analysis(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.image.get_analysis(&id) {
        Some(analysis) => Json(serde_json::json!({
            "success": true,
            "analysis": analysis
        })),
        None => Json(serde_json::json!({ "error": "Analysis not found" })),
    }
}

async fn api_image_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.image.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

async fn api_ma_list_agents(State(state): State<Arc<AppState>>) -> Json<Value> {
    let agents = state.multiagent.list_agents();
    Json(serde_json::json!({
        "success": true,
        "agents": agents
    }))
}

async fn api_ma_register_agent(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("Agent").to_string();
    let role = req.get("role").and_then(|v| v.as_str()).unwrap_or("assistant").to_string();
    let capabilities: Vec<String> = req.get("capabilities")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    match state.multiagent.register_agent(name, role, capabilities) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "agent_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ma_unregister_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.multiagent.unregister_agent(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ma_get_agent(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.multiagent.get_agent(&id) {
        Some(agent) => Json(serde_json::json!({
            "success": true,
            "agent": agent
        })),
        None => Json(serde_json::json!({ "error": "Agent not found" })),
    }
}

async fn api_ma_get_agent_messages(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    let messages = state.multiagent.get_agent_messages(&id);
    Json(serde_json::json!({
        "success": true,
        "messages": messages
    }))
}

async fn api_ma_list_tasks(State(state): State<Arc<AppState>>) -> Json<Value> {
    let tasks = state.multiagent.list_tasks();
    Json(serde_json::json!({
        "success": true,
        "tasks": tasks
    }))
}

async fn api_ma_create_task(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let title = req.get("title").and_then(|v| v.as_str()).unwrap_or("Task").to_string();
    let description = req.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let agent_ids: Vec<String> = req.get("agent_ids")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    match state.multiagent.create_collaboration_task(title, description, agent_ids) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "task_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ma_get_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.multiagent.get_task(&id) {
        Some(task) => Json(serde_json::json!({
            "success": true,
            "task": task
        })),
        None => Json(serde_json::json!({ "error": "Task not found" })),
    }
}

async fn api_ma_start_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.multiagent.start_task(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ma_update_progress(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let progress = req.get("progress").and_then(|v| v.as_f64()).unwrap_or(0.0);

    match state.multiagent.update_task_progress(&id, progress) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ma_complete_task(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Json<Value> {
    match state.multiagent.complete_task(&id) {
        Ok(_) => Json(serde_json::json!({
            "success": true
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ma_send_message(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let from_agent = req.get("from_agent").and_then(|v| v.as_str()).unwrap_or("");
    let to_agent = req.get("to_agent").and_then(|v| v.as_str()).unwrap_or("");
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("");
    let message_type = req.get("message_type").and_then(|v| v.as_str()).unwrap_or("text");

    match state.multiagent.send_message(from_agent, to_agent, content, message_type) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "message_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_ma_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.multiagent.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// RBAC API handlers
async fn api_list_roles(State(state): State<Arc<AppState>>) -> Json<Value> {
    let roles = state.rbac.list_roles();
    Json(serde_json::json!({
        "success": true,
        "roles": roles
    }))
}

async fn api_create_role(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = req.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let permissions: Vec<String> = req.get("permissions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    match state.rbac.create_role(name, description, permissions) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "role_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_role(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rbac.get_role(&id) {
        Ok(role) => Json(serde_json::json!({
            "success": true,
            "role": role
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_update_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).map(String::from);
    let description = req.get("description").and_then(|v| v.as_str()).map(String::from);
    let permissions: Option<Vec<String>> = req.get("permissions")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());

    match state.rbac.update_role(&id, name, description, permissions) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_role(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rbac.delete_role(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_users_rbac(State(state): State<Arc<AppState>>) -> Json<Value> {
    let users = state.rbac.list_users();
    Json(serde_json::json!({
        "success": true,
        "users": users
    }))
}

async fn api_create_user_rbac(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let username = req.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let email = req.get("email").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let role_id = req.get("role_id").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match state.rbac.create_user(username, email, role_id) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "user_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_user_rbac(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rbac.get_user(&id) {
        Ok(user) => Json(serde_json::json!({
            "success": true,
            "user": user
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_update_user_role(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let role_id = req.get("role_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match state.rbac.update_user_role(&id, &role_id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_user_rbac(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rbac.delete_user(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_user_permissions(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rbac.get_user_permissions(&id) {
        Ok(permissions) => Json(serde_json::json!({
            "success": true,
            "permissions": permissions
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_check_user_permission(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let permission = req.get("permission").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match state.rbac.check_permission(&id, &permission) {
        Ok(has_permission) => Json(serde_json::json!({
            "success": true,
            "has_permission": has_permission
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_permissions(State(state): State<Arc<AppState>>) -> Json<Value> {
    let permissions = state.rbac.list_permissions();
    Json(serde_json::json!({
        "success": true,
        "permissions": permissions
    }))
}

async fn api_create_permission(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = req.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let resource = req.get("resource").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let action = req.get("action").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match state.rbac.create_permission(name, description, resource, action) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "permission_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_rbac_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.rbac.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// Audit API handlers
async fn api_get_audit_logs(State(state): State<Arc<AppState>>) -> Json<Value> {
    let logs = state.audit.get_logs(None);
    Json(serde_json::json!({
        "success": true,
        "logs": logs,
        "total": logs.len()
    }))
}

async fn api_get_audit_log(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.audit.get_log(&id) {
        Ok(log) => Json(serde_json::json!({
            "success": true,
            "log": log
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_audit_log(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.audit.delete_log(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_purge_audit_logs(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let days = req.get("days").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
    match state.audit.purge_old_logs(days) {
        Ok(count) => Json(serde_json::json!({
            "success": true,
            "purged_count": count
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_export_audit_logs(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let format = req.get("format").and_then(|v| v.as_str()).unwrap_or("json").to_string();
    match state.audit.export_logs(&format) {
        Ok(data) => Json(serde_json::json!({
            "success": true,
            "data": data,
            "format": format
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_audit_reports(State(state): State<Arc<AppState>>) -> Json<Value> {
    let reports = state.audit.get_reports();
    Json(serde_json::json!({
        "success": true,
        "reports": reports,
        "total": reports.len()
    }))
}

async fn api_get_audit_report(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.audit.get_report(&id) {
        Ok(report) => Json(serde_json::json!({
            "success": true,
            "report": report
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_generate_audit_report(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let report_type = req.get("report_type").and_then(|v| v.as_str()).unwrap_or("security_audit").to_string();
    let period_start = req.get("period_start").and_then(|v| v.as_i64()).unwrap_or(0);
    let period_end = req.get("period_end").and_then(|v| v.as_i64()).unwrap_or(chrono::Utc::now().timestamp());

    match state.audit.generate_compliance_report(report_type, period_start, period_end) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "report_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_audit_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.audit.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// Serverless API handlers
async fn api_list_serverless_configs(State(state): State<Arc<AppState>>) -> Json<Value> {
    let configs = state.serverless.list_configs();
    Json(serde_json::json!({
        "success": true,
        "configs": configs,
        "total": configs.len()
    }))
}

async fn api_create_serverless_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let provider = req.get("provider").and_then(|v| v.as_str()).unwrap_or("aws").to_string();
    let region = req.get("region").and_then(|v| v.as_str()).unwrap_or("us-east-1").to_string();
    let runtime = req.get("runtime").and_then(|v| v.as_str()).unwrap_or("python3.10").to_string();
    let memory_mb = req.get("memory_mb").and_then(|v| v.as_u64()).unwrap_or(512) as u32;
    let timeout_seconds = req.get("timeout_seconds").and_then(|v| v.as_u64()).unwrap_or(30) as u32;
    
    let env_vars: HashMap<String, String> = req.get("environment_variables")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|vs| (k.clone(), vs.to_string()))).collect())
        .unwrap_or_default();

    match state.serverless.create_deployment_config(name, provider, region, runtime, memory_mb, timeout_seconds, env_vars) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "config_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_serverless_config(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.serverless.get_config(&id) {
        Ok(config) => Json(serde_json::json!({
            "success": true,
            "config": config
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_update_serverless_config(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).map(String::from);
    let memory_mb = req.get("memory_mb").and_then(|v| v.as_u64()).map(|v| v as u32);
    let timeout_seconds = req.get("timeout_seconds").and_then(|v| v.as_u64()).map(|v| v as u32);
    let min_instances = req.get("min_instances").and_then(|v| v.as_u64()).map(|v| v as u32);
    let max_instances = req.get("max_instances").and_then(|v| v.as_u64()).map(|v| v as u32);
    
    let env_vars: Option<HashMap<String, String>> = req.get("environment_variables")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|vs| (k.clone(), vs.to_string()))).collect());

    match state.serverless.update_config(&id, name, memory_mb, timeout_seconds, min_instances, max_instances, env_vars) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_serverless_config(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.serverless.delete_config(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_serverless_packages(State(state): State<Arc<AppState>>) -> Json<Value> {
    let packages = state.serverless.list_packages(None);
    Json(serde_json::json!({
        "success": true,
        "packages": packages,
        "total": packages.len()
    }))
}

async fn api_create_serverless_package(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let config_id = req.get("config_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let version = req.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string();
    let package_data = req.get("package_data").and_then(|v| v.as_str()).unwrap_or("").as_bytes().to_vec();

    match state.serverless.create_deployment_package(&config_id, version, &package_data) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "package_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_deploy_serverless_package(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.serverless.deploy_package(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_scaling_policies(State(state): State<Arc<AppState>>) -> Json<Value> {
    let policies = state.serverless.list_scaling_policies(None);
    Json(serde_json::json!({
        "success": true,
        "policies": policies,
        "total": policies.len()
    }))
}

async fn api_create_scaling_policy(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let config_id = req.get("config_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let trigger_type = req.get("trigger_type").and_then(|v| v.as_str()).unwrap_or("cpu").to_string();
    let threshold = req.get("threshold").and_then(|v| v.as_f64()).unwrap_or(70.0);
    let scale_up_factor = req.get("scale_up_factor").and_then(|v| v.as_f64()).unwrap_or(2.0);
    let scale_down_factor = req.get("scale_down_factor").and_then(|v| v.as_f64()).unwrap_or(0.5);
    let cooldown_seconds = req.get("cooldown_seconds").and_then(|v| v.as_u64()).unwrap_or(300) as u32;

    match state.serverless.create_scaling_policy(&config_id, trigger_type, threshold, scale_up_factor, scale_down_factor, cooldown_seconds) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "policy_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_scaling_policy(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.serverless.delete_scaling_policy(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_serverless_providers(State(state): State<Arc<AppState>>) -> Json<Value> {
    let providers = state.serverless.get_providers();
    Json(serde_json::json!({
        "success": true,
        "providers": providers,
        "total": providers.len()
    }))
}

async fn api_get_serverless_provider(State(state): State<Arc<AppState>>, Path(name): Path<String>) -> Json<Value> {
    match state.serverless.get_provider(&name) {
        Ok(provider) => Json(serde_json::json!({
            "success": true,
            "provider": provider
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_record_invocation(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.serverless.record_invocation(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_serverless_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.serverless.get_deployment_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

async fn api_generate_serverless_template(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let format = req.get("format").and_then(|v| v.as_str()).unwrap_or("serverless.yml").to_string();
    match state.serverless.generate_deployment_template(&id, &format) {
        Ok(template) => Json(serde_json::json!({
            "success": true,
            "template": template,
            "format": format
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

// Community Skills API handlers
async fn api_list_community_skills(State(state): State<Arc<AppState>>) -> Json<Value> {
    let filters: Option<HashMap<String, String>> = None;
    let skills = state.community_skills.list_skills(filters);
    Json(serde_json::json!({
        "success": true,
        "skills": skills,
        "total": skills.len()
    }))
}

async fn api_publish_community_skill(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("anonymous").to_string();
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = req.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let version = req.get("version").and_then(|v| v.as_str()).unwrap_or("1.0.0").to_string();
    let category = req.get("category").and_then(|v| v.as_str()).unwrap_or("utilities").to_string();
    let tags: Vec<String> = req.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let code = req.get("code").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let dependencies: Vec<String> = req.get("dependencies")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let readme = req.get("readme").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let license = req.get("license").and_then(|v| v.as_str()).unwrap_or("MIT").to_string();

    match state.community_skills.publish_skill(user_id, name, description, version, category, tags, code, dependencies, readme, license) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "skill_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_community_skill(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.community_skills.get_skill(&id) {
        Ok(skill) => Json(serde_json::json!({
            "success": true,
            "skill": skill
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_update_community_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = req.get("name").and_then(|v| v.as_str()).map(String::from);
    let description = req.get("description").and_then(|v| v.as_str()).map(String::from);
    let version = req.get("version").and_then(|v| v.as_str()).map(String::from);
    let code = req.get("code").and_then(|v| v.as_str()).map(String::from);
    let dependencies: Option<Vec<String>> = req.get("dependencies")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());
    let readme = req.get("readme").and_then(|v| v.as_str()).map(String::from);

    match state.community_skills.update_skill(&id, &user_id, name, description, version, code, dependencies, readme) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_community_skill(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match state.community_skills.delete_skill(&id, &user_id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_download_community_skill(State(state): State<Arc<AppState>>, Path(id): Path<String>, Json(req): Json<Value>) -> Json<Value> {
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("anonymous").to_string();
    match state.community_skills.download_skill(&id, &user_id) {
        Ok(download_id) => Json(serde_json::json!({
            "success": true,
            "download_id": download_id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_skill_reviews(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let reviews = state.community_skills.get_reviews(&id);
    Json(serde_json::json!({
        "success": true,
        "reviews": reviews,
        "total": reviews.len()
    }))
}

async fn api_add_skill_review(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("anonymous").to_string();
    let rating = req.get("rating").and_then(|v| v.as_u64()).unwrap_or(5) as u8;
    let comment = req.get("comment").and_then(|v| v.as_str()).unwrap_or("").to_string();

    match state.community_skills.add_review(&id, &user_id, rating, comment) {
        Ok(review_id) => Json(serde_json::json!({
            "success": true,
            "review_id": review_id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_user_skills(State(state): State<Arc<AppState>>, Path(user_id): Path<String>) -> Json<Value> {
    let skills = state.community_skills.get_user_skills(&user_id);
    Json(serde_json::json!({
        "success": true,
        "skills": skills,
        "total": skills.len()
    }))
}

async fn api_get_skill_categories(State(state): State<Arc<AppState>>) -> Json<Value> {
    let categories = state.community_skills.get_categories();
    Json(serde_json::json!({
        "success": true,
        "categories": categories,
        "total": categories.len()
    }))
}

async fn api_community_skills_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.community_skills.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// Trajectory API handlers
use crate::trajectory::RlTrainingConfig;

async fn api_create_trajectory_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let environment = req.get("environment").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let agent_id = req.get("agent_id").and_then(|v| v.as_str()).unwrap_or("default-agent").to_string();
    let batch_size = req.get("batch_size").and_then(|v| v.as_u64()).unwrap_or(32) as usize;
    let max_steps_per_episode = req.get("max_steps_per_episode").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let exploration_rate = req.get("exploration_rate").and_then(|v| v.as_f64()).unwrap_or(0.1);
    let learning_rate = req.get("learning_rate").and_then(|v| v.as_f64()).unwrap_or(0.001);
    let discount_factor = req.get("discount_factor").and_then(|v| v.as_f64()).unwrap_or(0.99);
    let reward_scale = req.get("reward_scale").and_then(|v| v.as_f64()).unwrap_or(1.0);
    let save_trajectories = req.get("save_trajectories").and_then(|v| v.as_bool()).unwrap_or(true);
    let tags: Vec<String> = req.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let config = RlTrainingConfig {
        environment,
        agent_id,
        batch_size,
        max_steps_per_episode,
        exploration_rate,
        learning_rate,
        discount_factor,
        reward_scale,
        save_trajectories,
        tags,
    };

    match state.trajectory_generator.create_config(config) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "config_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_trajectory_config(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.trajectory_generator.get_config(&id) {
        Ok(config) => Json(serde_json::json!({
            "success": true,
            "config": config
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_trajectory_batches(State(state): State<Arc<AppState>>) -> Json<Value> {
    let batches = state.trajectory_generator.list_batches(None);
    Json(serde_json::json!({
        "success": true,
        "batches": batches,
        "total": batches.len()
    }))
}

async fn api_generate_trajectory_batch(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let config_id = req.get("config_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let batch_name = req.get("batch_name").and_then(|v| v.as_str()).unwrap_or("unnamed-batch").to_string();

    match state.trajectory_generator.generate_batch(&config_id, batch_name) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "batch_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_trajectory_batch(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.trajectory_generator.get_batch(&id) {
        Ok(batch) => Json(serde_json::json!({
            "success": true,
            "batch": batch
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_delete_trajectory_batch(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.trajectory_generator.delete_batch(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_export_trajectory_batch(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let format = req.get("format").and_then(|v| v.as_str()).unwrap_or("json").to_string();
    match state.trajectory_generator.export_batch(&id, &format) {
        Ok(export) => Json(serde_json::json!({
            "success": true,
            "export": export,
            "format": format
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_trajectory(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.trajectory_generator.get_trajectory(&id) {
        Ok(trajectory) => Json(serde_json::json!({
            "success": true,
            "trajectory": trajectory
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_trajectory_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.trajectory_generator.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// RL Environment API handlers
use crate::rl_environment::{EnvironmentConfig, EnvironmentAction};

async fn api_create_rl_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("default-env").to_string();
    let environment_type = req.get("environment_type").and_then(|v| v.as_str()).unwrap_or("cartpole").to_string();
    let max_steps = req.get("max_steps").and_then(|v| v.as_u64()).unwrap_or(100) as usize;
    let observation_space = req.get("observation_space").cloned().unwrap_or(serde_json::json!({}));
    let action_space = req.get("action_space").cloned().unwrap_or(serde_json::json!({}));
    let reward_min = req.get("reward_range").and_then(|v| v.get(0)).and_then(|v| v.as_f64()).unwrap_or(-1.0);
    let reward_max = req.get("reward_range").and_then(|v| v.get(1)).and_then(|v| v.as_f64()).unwrap_or(1.0);
    let metadata: HashMap<String, String> = req.get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|vs| (k.clone(), vs.to_string()))).collect())
        .unwrap_or_default();

    let config = EnvironmentConfig {
        name,
        environment_type,
        max_steps,
        observation_space,
        action_space,
        reward_range: (reward_min, reward_max),
        metadata,
    };

    match state.rl_environment.register_config(config) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "config_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_rl_config(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rl_environment.get_config(&id) {
        Ok(config) => Json(serde_json::json!({
            "success": true,
            "config": config
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_list_rl_environments(State(state): State<Arc<AppState>>) -> Json<Value> {
    let environments = state.rl_environment.list_environments(None);
    Json(serde_json::json!({
        "success": true,
        "environments": environments,
        "total": environments.len()
    }))
}

async fn api_create_rl_environment(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let config_id = req.get("config_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    match state.rl_environment.create_environment(&config_id) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "environment_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_get_rl_environment(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rl_environment.get_environment(&id) {
        Ok(env) => Json(serde_json::json!({
            "success": true,
            "environment": env
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_rl_environment_step(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let action_id = req.get("action_id").and_then(|v| v.as_str()).unwrap_or("action-0").to_string();
    let action_type = req.get("action_type").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let parameters = req.get("parameters").cloned().unwrap_or(serde_json::json!({}));

    let action = EnvironmentAction {
        action_id,
        action_type,
        parameters,
    };

    match state.rl_environment.step(&id, action) {
        Ok(state) => Json(serde_json::json!({
            "success": true,
            "state": state
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_rl_environment_reset(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rl_environment.reset(&id) {
        Ok(state) => Json(serde_json::json!({
            "success": true,
            "state": state
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_rl_environment_close(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.rl_environment.close_environment(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_rl_environment_history(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let limit = 100;
    let history = state.rl_environment.get_episode_history(Some(&id), limit);
    Json(serde_json::json!({
        "success": true,
        "history": history,
        "total": history.len()
    }))
}

async fn api_rl_environment_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.rl_environment.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// Honcho dialectical user modeling API handlers
use crate::honcho::HonchoConfig;

async fn api_honcho_create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("Anonymous").to_string();
    
    match state.honcho_modeling.create_user(user_id, name) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "user_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_list_users(State(state): State<Arc<AppState>>) -> Json<Value> {
    let users = state.honcho_modeling.list_users();
    Json(serde_json::json!({
        "success": true,
        "users": users,
        "total": users.len()
    }))
}

async fn api_honcho_get_user(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.honcho_modeling.get_user(&id) {
        Ok(user) => Json(serde_json::json!({
            "success": true,
            "user": user
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_delete_user(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.honcho_modeling.delete_user(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_record_interaction(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let input = req.get("input").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let response = req.get("response").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let topics: Vec<String> = req.get("topics")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    match state.honcho_modeling.record_interaction(&id, input, response, topics) {
        Ok(interaction_id) => Json(serde_json::json!({
            "success": true,
            "interaction_id": interaction_id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_get_profile(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.honcho_modeling.get_user_profile(&id) {
        Ok(profile) => Json(serde_json::json!({
            "success": true,
            "profile": profile
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_get_dialectical_state(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.honcho_modeling.get_dialectical_state(&id) {
        Ok(state) => Json(serde_json::json!({
            "success": true,
            "dialectical_state": state
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_get_history(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    let limit = 50;
    let history = state.honcho_modeling.get_interaction_history(&id, limit);
    Json(serde_json::json!({
        "success": true,
        "history": history,
        "total": history.len()
    }))
}

async fn api_honcho_update_preference(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let key = req.get("key").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let value = req.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    match state.honcho_modeling.update_user_preference(&id, key, value) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_add_goal(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let goal = req.get("goal").and_then(|v| v.as_str()).unwrap_or("").to_string();
    
    match state.honcho_modeling.add_user_goal(&id, goal) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_get_analytics(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.honcho_modeling.get_user_analytics(&id) {
        Ok(analytics) => Json(serde_json::json!({
            "success": true,
            "analytics": analytics
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_honcho_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.honcho_modeling.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// FTS Search API handlers
use crate::fts_search::{FtsConfig, SearchDocument};

async fn api_fts_add_document(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let id = req.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let title = req.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let document_type = req.get("document_type").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let tags: Vec<String> = req.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let metadata: HashMap<String, String> = req.get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|vs| (k.clone(), vs.to_string()))).collect())
        .unwrap_or_default();

    let document = SearchDocument {
        id,
        title,
        content,
        document_type,
        tags,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
        metadata,
    };

    match state.fts_search.add_document(document) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "document_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_fts_get_document(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.fts_search.get_document(&id) {
        Ok(doc) => Json(serde_json::json!({
            "success": true,
            "document": doc
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_fts_update_document(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let title = req.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let document_type = req.get("document_type").and_then(|v| v.as_str()).unwrap_or("default").to_string();
    let tags: Vec<String> = req.get("tags")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();

    let document = SearchDocument {
        id: id.clone(),
        title,
        content,
        document_type,
        tags,
        created_at: chrono::Utc::now().timestamp(),
        updated_at: chrono::Utc::now().timestamp(),
        metadata: HashMap::new(),
    };

    match state.fts_search.update_document(document) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_fts_delete_document(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.fts_search.delete_document(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_fts_list_documents(State(state): State<Arc<AppState>>) -> Json<Value> {
    let documents = state.fts_search.list_documents(None);
    Json(serde_json::json!({
        "success": true,
        "documents": documents,
        "total": documents.len()
    }))
}

async fn api_fts_search(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let query = req.get("query").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let filters: Option<HashMap<String, String>> = req.get("filters")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|vs| (k.clone(), vs.to_string()))).collect());

    let results = state.fts_search.search(&query, filters);
    Json(serde_json::json!({
        "success": true,
        "results": results,
        "total": results.len()
    }))
}

async fn api_fts_generate_summary(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.fts_search.generate_summary(&id) {
        Ok(summary) => Json(serde_json::json!({
            "success": true,
            "summary": summary
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_fts_get_summary(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.fts_search.get_summary(&id) {
        Ok(summary) => Json(serde_json::json!({
            "success": true,
            "summary": summary
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_fts_search_history(State(state): State<Arc<AppState>>) -> Json<Value> {
    let history = state.fts_search.get_search_history(50);
    Json(serde_json::json!({
        "success": true,
        "history": history,
        "total": history.len()
    }))
}

async fn api_fts_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.fts_search.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

// Memory Prompt API handlers
use crate::memory_prompt::{MemoryPromptConfig, MemoryPrompt, PromptSchedule, PromptResponse};

async fn api_mp_create_schedule(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let id = req.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let name = req.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = req.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let cron_expression = req.get("cron_expression").and_then(|v| v.as_str()).unwrap_or("0 9 * * *").to_string();
    let prompt_type = req.get("prompt_type").and_then(|v| v.as_str()).unwrap_or("check_in").to_string();
    let target_users: Vec<String> = req.get("target_users")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let is_active = req.get("is_active").and_then(|v| v.as_bool()).unwrap_or(true);

    let schedule = PromptSchedule {
        id,
        name,
        description,
        cron_expression,
        prompt_type,
        target_users,
        is_active,
        last_triggered: None,
        trigger_count: 0,
        created_at: chrono::Utc::now().timestamp(),
    };

    match state.memory_prompting.create_schedule(schedule) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "schedule_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_get_schedule(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.memory_prompting.get_schedule(&id) {
        Ok(schedule) => Json(serde_json::json!({
            "success": true,
            "schedule": schedule
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_list_schedules(State(state): State<Arc<AppState>>) -> Json<Value> {
    let active_only = false;
    let schedules = state.memory_prompting.list_schedules(active_only);
    Json(serde_json::json!({
        "success": true,
        "schedules": schedules,
        "total": schedules.len()
    }))
}

async fn api_mp_create_prompt(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let id = req.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let prompt_type = req.get("prompt_type").and_then(|v| v.as_str()).unwrap_or("check_in").to_string();
    let content = req.get("content").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let priority = req.get("priority").and_then(|v| v.as_u64()).unwrap_or(5) as u8;
    let scheduled_at = req.get("scheduled_at").and_then(|v| v.as_i64()).unwrap_or_else(|| chrono::Utc::now().timestamp());
    let context: HashMap<String, String> = req.get("context")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|vs| (k.clone(), vs.to_string()))).collect())
        .unwrap_or_default();
    let metadata: HashMap<String, String> = req.get("metadata")
        .and_then(|v| v.as_object())
        .map(|obj| obj.iter().filter_map(|(k, v)| v.as_str().map(|vs| (k.clone(), vs.to_string()))).collect())
        .unwrap_or_default();

    let prompt = MemoryPrompt {
        id,
        user_id,
        prompt_type,
        content,
        context,
        priority,
        created_at: chrono::Utc::now().timestamp(),
        scheduled_at,
        triggered_at: None,
        is_active: true,
        metadata,
    };

    match state.memory_prompting.create_prompt(prompt) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "prompt_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_get_prompt(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.memory_prompting.get_prompt(&id) {
        Ok(prompt) => Json(serde_json::json!({
            "success": true,
            "prompt": prompt
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_list_prompts(State(state): State<Arc<AppState>>) -> Json<Value> {
    let active_only = false;
    let prompts = state.memory_prompting.list_prompts(None, active_only);
    Json(serde_json::json!({
        "success": true,
        "prompts": prompts,
        "total": prompts.len()
    }))
}

async fn api_mp_trigger_prompt(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.memory_prompting.trigger_prompt(&id) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "prompt_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_deactivate_prompt(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.memory_prompting.deactivate_prompt(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_activate_prompt(State(state): State<Arc<AppState>>, Path(id): Path<String>) -> Json<Value> {
    match state.memory_prompting.activate_prompt(&id) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_submit_response(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let id = req.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let prompt_id = req.get("prompt_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let response = req.get("response").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let sentiment = req.get("sentiment").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let follow_up_required = req.get("follow_up_required").and_then(|v| v.as_bool()).unwrap_or(false);

    let prompt_response = PromptResponse {
        id,
        prompt_id,
        user_id,
        response,
        sentiment,
        timestamp: chrono::Utc::now().timestamp(),
        follow_up_required,
    };

    match state.memory_prompting.submit_response(prompt_response) {
        Ok(id) => Json(serde_json::json!({
            "success": true,
            "response_id": id
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_get_responses(State(state): State<Arc<AppState>>, Path(prompt_id): Path<String>) -> Json<Value> {
    let responses = state.memory_prompting.get_responses(&prompt_id);
    Json(serde_json::json!({
        "success": true,
        "responses": responses,
        "total": responses.len()
    }))
}

async fn api_mp_set_user_config(
    State(state): State<Arc<AppState>>,
    Json(req): Json<Value>,
) -> Json<Value> {
    let user_id = req.get("user_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let prompt_frequency_hours = req.get("prompt_frequency_hours").and_then(|v| v.as_u64()).unwrap_or(24);
    let preferred_prompt_types: Vec<String> = req.get("preferred_prompt_types")
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default();
    let max_prompts_per_day = req.get("max_prompts_per_day").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
    let quiet_hours_start = req.get("quiet_hours_start").and_then(|v| v.as_u64()).unwrap_or(22) as u8;
    let quiet_hours_end = req.get("quiet_hours_end").and_then(|v| v.as_u64()).unwrap_or(7) as u8;
    let enable_adaptive_scheduling = req.get("enable_adaptive_scheduling").and_then(|v| v.as_bool()).unwrap_or(true);

    let config = MemoryPromptConfig {
        user_id,
        prompt_frequency_hours,
        preferred_prompt_types,
        max_prompts_per_day,
        quiet_hours_start,
        quiet_hours_end,
        enable_adaptive_scheduling,
    };

    match state.memory_prompting.set_user_config(config) {
        Ok(()) => Json(serde_json::json!({ "success": true })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_get_user_config(State(state): State<Arc<AppState>>, Path(user_id): Path<String>) -> Json<Value> {
    match state.memory_prompting.get_user_config(&user_id) {
        Ok(config) => Json(serde_json::json!({
            "success": true,
            "config": config
        })),
        Err(e) => Json(serde_json::json!({ "error": e.to_string() })),
    }
}

async fn api_mp_get_due_prompts(State(state): State<Arc<AppState>>) -> Json<Value> {
    let due = state.memory_prompting.get_due_prompts();
    Json(serde_json::json!({
        "success": true,
        "due_prompts": due,
        "total": due.len()
    }))
}

async fn api_mp_get_user_stats(State(state): State<Arc<AppState>>, Path(user_id): Path<String>) -> Json<Value> {
    let stats = state.memory_prompting.get_user_prompt_stats(&user_id);
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}

async fn api_mp_get_stats(State(state): State<Arc<AppState>>) -> Json<Value> {
    let stats = state.memory_prompting.get_stats();
    Json(serde_json::json!({
        "success": true,
        "stats": stats
    }))
}
