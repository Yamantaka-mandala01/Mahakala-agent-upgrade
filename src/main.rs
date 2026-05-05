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
use mahakala_agent_upgrade::learning::LearningLoop;
use mahakala_agent_upgrade::semantic_memory::SemanticSearchEngine;
use mahakala_agent_upgrade::delegation::DelegationSystem;
use mahakala_agent_upgrade::mcp::McpClient;
use mahakala_agent_upgrade::gateways::GatewayManager;
use mahakala_agent_upgrade::terminal::{DockerManager, DockerConfig, SshManager};
use mahakala_agent_upgrade::browser::{BrowserManager, BrowserConfig};
use mahakala_agent_upgrade::voice::{VoiceManager, SpeechConfig};
use mahakala_agent_upgrade::image::{ImageManager, ImageConfig};
use mahakala_agent_upgrade::multiagent::{MultiAgentFramework, CollaborationConfig};
use mahakala_agent_upgrade::rbac::{RbacSystem, RbacConfig};
use mahakala_agent_upgrade::audit::{AuditSystem, AuditConfig};
use mahakala_agent_upgrade::serverless::ServerlessManager;
use mahakala_agent_upgrade::community_skills::CommunitySkillCenter;
use mahakala_agent_upgrade::trajectory::TrajectoryGenerator;
use mahakala_agent_upgrade::rl_environment::RlEnvironment;
use mahakala_agent_upgrade::honcho::HonchoModeling;
use mahakala_agent_upgrade::fts_search::FtsSearchEngine;
use mahakala_agent_upgrade::memory_prompt::MemoryPromptingSystem;

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

    // Initialize learning loop system
    let learning = Arc::new(LearningLoop::new(
        Arc::new(memory.clone()),
        Arc::new(skills.clone()),
    ));
    tracing::info!("Learning loop system initialized");

    // Initialize semantic search engine
    let semantic_search = Arc::new(SemanticSearchEngine::new(Arc::new(memory.clone())));
    tracing::info!("Semantic search engine initialized with {} memories", semantic_search.get_memory_count());

    // Initialize delegation system
    let delegation = Arc::new(DelegationSystem::new(Arc::new(tool_registry.clone())));
    tracing::info!("Delegation system initialized");

    // Initialize MCP client
    let mcp = Arc::new(McpClient::new());
    tracing::info!("MCP client initialized");

    // Initialize gateway manager
    let gateways = Arc::new(GatewayManager::new());
    tracing::info!("Gateway manager initialized");

    // Initialize Docker manager
    let docker_config = DockerConfig {
        host: "localhost".to_string(),
        port: 2375,
        api_version: "1.41".to_string(),
    };
    let docker = Arc::new(DockerManager::new(docker_config));
    tracing::info!("Docker manager initialized");

    // Initialize SSH manager
    let ssh = Arc::new(SshManager::new());
    tracing::info!("SSH manager initialized");

    // Initialize browser manager
    let browser_config = BrowserConfig {
        browser_type: "chromium".to_string(),
        headless: true,
        viewport_width: 1920,
        viewport_height: 1080,
        user_agent: None,
    };
    let browser = Arc::new(BrowserManager::new(browser_config));
    tracing::info!("Browser manager initialized");

    // Initialize voice manager
    let speech_config = SpeechConfig {
        model: "whisper".to_string(),
        language: "zh".to_string(),
        sample_rate: 16000,
        channels: 1,
        format: "pcm".to_string(),
    };
    let voice = Arc::new(VoiceManager::new(speech_config));
    tracing::info!("Voice manager initialized");

    // Initialize image manager
    let image_config = ImageConfig {
        model: "dall-e-3".to_string(),
        default_width: 1024,
        default_height: 1024,
        default_format: "png".to_string(),
    };
    let image = Arc::new(ImageManager::new(image_config));
    tracing::info!("Image manager initialized");

    // Initialize multi-agent framework
    let collaboration_config = CollaborationConfig {
        max_agents: 10,
        default_role: "assistant".to_string(),
        enable_auto_assignment: true,
    };
    let multiagent = Arc::new(MultiAgentFramework::new(collaboration_config));
    tracing::info!("Multi-agent framework initialized");

    // Initialize RBAC system
    let rbac_config = RbacConfig::default();
    let rbac = Arc::new(RbacSystem::new(rbac_config));
    tracing::info!("Role-based access control system initialized with default roles");

    // Initialize audit system
    let audit_config = AuditConfig::default();
    let audit = Arc::new(AuditSystem::new(audit_config));
    tracing::info!("Audit logging and compliance system initialized");

    // Initialize serverless deployment manager
    let serverless = Arc::new(ServerlessManager::new());
    tracing::info!("Serverless deployment manager initialized with AWS, Azure, and GCP support");

    // Initialize community skill center
    let community_skills = Arc::new(CommunitySkillCenter::new(Default::default()));
    tracing::info!("Community skill center initialized");

    // Initialize trajectory generator
    let trajectory_generator = Arc::new(TrajectoryGenerator::new());
    tracing::info!("Trajectory generator for RL training initialized");

    // Initialize RL environment
    let rl_environment = Arc::new(RlEnvironment::new());
    tracing::info!("Atropos-compatible RL environment initialized");

    // Initialize Honcho dialectical user modeling
    let honcho_modeling = Arc::new(HonchoModeling::new(Default::default()));
    tracing::info!("Honcho dialectical user modeling initialized");

    // Initialize FTS search engine
    let fts_search = Arc::new(FtsSearchEngine::new(Default::default()));
    tracing::info!("FTS5 full-text search engine initialized");

    // Initialize memory prompting system
    let memory_prompting = Arc::new(MemoryPromptingSystem::new());
    tracing::info!("Periodic memory prompting system initialized");

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
        learning,
        semantic_search,
        delegation,
        mcp,
        gateways,
        docker,
        ssh,
        browser,
        voice,
        image,
        multiagent,
        rbac,
        audit,
        serverless,
        community_skills,
        trajectory_generator,
        rl_environment,
        honcho_modeling,
        fts_search,
        memory_prompting,
    });

    // Start web server
    tracing::info!("🌐 Starting web server on http://{}:{}", state.config.read().await.host, state.config.read().await.port);
    tracing::info!("📦 Mode: {}", if cli.standalone { "Standalone (embedded)" } else { "Development" });

    run_web_server(state).await?;

    Ok(())
}
