use tracing_subscriber::EnvFilter;

pub fn init_logging() {
    let log_dir = crate::constants::get_logs_dir();
    
    match std::fs::create_dir_all(&log_dir) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Warning: Failed to create log directory {}: {}", log_dir.display(), e);
        }
    }

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));

    let result = tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .try_init();

    match result {
        Ok(_) => {
            tracing::info!("Logging initialized (stdout only)");
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize logging: {}", e);
        }
    }
}
