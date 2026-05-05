use clap::Parser;
use std::sync::Arc;
use mahakala_agent_upgrade::constants;
use mahakala_agent_upgrade::logging;
use mahakala_agent_upgrade::config::AppConfig;
use mahakala_agent_upgrade::agent::{AIAgent, AgentConfig};
use mahakala_agent_upgrade::web_server::{AppState, run_web_server};
use mahakala_agent_upgrade::tools::{self, registry::ToolRegistry};
use mahakala_agent_upgrade::state::SessionDB;
use mahakala_agent_upgrade::memory::MemoryManager;
use mahakala_agent_upgrade::skills::SkillManager;
use mahakala_agent_upgrade::plugins::PluginManager;
use mahakala_agent_upgrade::gateway::GatewayHandle;
use mahakala_agent_upgrade::cron::CronManagerHandle;
use mahakala_agent_upgrade::workspace::WorkspaceHandle;
use mahakala_agent_upgrade::auth::AuthHandle;
use mahakala_agent_upgrade::i18n::I18nHandle;
use mahakala_agent_upgrade::upload::UploadHandle;
use mahakala_agent_upgrade::cli::CliHandle;
use mahakala_agent_upgrade::wechat::WechatHandle;

#[derive(Parser, Debug)]
#[command(name = "mahakala-agent-upgrade")]
#[command(about = "Mahakala Agent - AI Agent with WebUI")]
struct Cli {
    #[arg(long, help = "Run in standalone mode (embedded resources)")]
    standalone: bool,

    #[arg(short, long, help = "Port to listen on")]
    port: Option<u16>,

    #[arg(short, long, help = "Host to bind to")]
    host: Option<String>,

    #[arg(long, help = "Language (zh/en)")]
    lang: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Initialize logging
    logging::init_logging();

    tracing::info!("🔱 Mahakala Agent Upgrade starting...");
    tracing::info!("Version: {}", constants::APP_VERSION);

    // Load configuration
    let mut config = AppConfig::load();

    // Override with CLI arguments
    if let Some(port) = cli.port {
        config.port = port;
    }
    if let Some(host) = cli.host {
        config.host = host;
    }
    if let Some(lang) = cli.lang {
        config.language = lang;
    }

    // 本地 Ollama 模式不需要 API Key
    // 如果配置了云端提供商，才尝试从环境变量加载 API Key
    let provider = config.provider.clone().unwrap_or_default();
    if config.api_key.is_none() && provider != "ollama" && !provider.is_empty() {
        config.api_key = AppConfig::get_env_var("OPENAI_API_KEY")
            .or_else(|| AppConfig::get_env_var("ANTHROPIC_API_KEY"))
            .or_else(|| AppConfig::get_env_var("DEEPSEEK_API_KEY"));
    }

    tracing::info!("Model: {}", config.model);
    tracing::info!("Provider: {:?}", config.provider);
    tracing::info!("Language: {}", config.language);
    tracing::info!("Theme: {}", config.theme);

    // Create tool registry and register all tools
    let mut tool_registry = ToolRegistry::new();
    tools::all_tools::register_all_tools(&mut tool_registry);
    tracing::info!("🛠️ Registered {} tools", tool_registry.count());

    // Create AI agent
    let agent_config = AgentConfig {
        model: config.model.clone(),
        provider: config.provider.clone(),
        api_base_url: config.api_base_url.clone(),
        api_key: config.api_key.clone(),
        temperature: config.temperature,
        max_tokens: config.max_tokens,
    };

    let agent = AIAgent::new(agent_config, Arc::new(tool_registry.clone()));

    // Initialize session database
    let db_path = if cfg!(windows) {
        std::env::current_dir().unwrap_or_default().join("mahakala.db")
    } else {
        constants::get_db_path()
    };
    let session_db = SessionDB::new(&db_path).unwrap_or_else(|e| {
        tracing::warn!("Failed to open database at {}: {}, using in-memory database", db_path.display(), e);
        SessionDB::new(std::path::Path::new(":memory:")).expect("Failed to open in-memory database")
    });
    tracing::info!("Session database initialized at: {}", db_path.display());

    // Initialize memory manager
    let memory = MemoryManager::new(None).unwrap_or_else(|e| {
        tracing::warn!("Failed to initialize memory manager: {}, using in-memory fallback", e);
        MemoryManager::new(Some(std::env::current_dir().unwrap_or_default().join("data").join("memory.db"))).expect("Failed to create memory manager")
    });
    tracing::info!("Memory manager initialized");

    // Initialize other managers
    let skills = SkillManager::new();
    let plugins = PluginManager::new();
    let gateway = GatewayHandle::new();
    let cron = CronManagerHandle::new().await.unwrap_or_else(|e| {
        tracing::warn!("Failed to initialize cron manager: {}", e);
        panic!("Failed to create cron manager")
    });
    let workspace = WorkspaceHandle::new();
    let auth = AuthHandle::new();
    let i18n = I18nHandle::new();
    let upload = UploadHandle::new(None).unwrap_or_else(|e| {
        tracing::warn!("Failed to initialize upload manager: {}, using default", e);
        UploadHandle::new(Some(std::env::current_dir().unwrap_or_default().join("uploads"))).expect("Failed to create upload manager")
    });
    let cli_manager = CliHandle::new();
    let wechat = WechatHandle::new().unwrap_or_else(|e| {
        tracing::warn!("Failed to initialize wechat manager: {}", e);
        panic!("Failed to create wechat manager")
    });

    // Create shared application state
    let state = Arc::new(AppState {
        agent: tokio::sync::RwLock::new(agent),
        config: tokio::sync::RwLock::new(config),
        session_db: Arc::new(session_db),
        tool_registry: Arc::new(tool_registry),
        memory,
        skills,
        plugins,
        gateway,
        cron,
        workspace,
        auth,
        i18n,
        upload,
        cli: cli_manager,
        wechat,
    });

    // Start web server
    tracing::info!("🌐 Starting web server on http://{}:{}", state.config.read().await.host, state.config.read().await.port);
    tracing::info!("📦 Mode: {}", if cli.standalone { "Standalone (embedded)" } else { "Development" });

    run_web_server(state).await?;

    Ok(())
}
