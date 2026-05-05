use std::path::PathBuf;

pub const APP_NAME: &str = "Mahakala Agent";
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const DEFAULT_PORT: u16 = 3000;
pub const DEFAULT_HOST: &str = "127.0.0.1";

pub fn get_mahakala_home() -> PathBuf {
    if cfg!(windows) {
        if let Ok(profile) = std::env::var("USERPROFILE") {
            return PathBuf::from(profile).join(".mahakala");
        }
    } else {
        if let Ok(home) = std::env::var("HOME") {
            return PathBuf::from(home).join(".mahakala");
        }
    }
    
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")).join(".mahakala")
}

pub fn get_config_path() -> PathBuf {
    get_mahakala_home().join("config.yaml")
}

pub fn get_db_path() -> PathBuf {
    get_mahakala_home().join("mahakala.db")
}

pub fn get_logs_dir() -> PathBuf {
    get_mahakala_home().join("logs")
}

pub fn get_sessions_dir() -> PathBuf {
    get_mahakala_home().join("sessions")
}

pub fn get_skills_dir() -> PathBuf {
    get_mahakala_home().join("skills")
}

pub fn get_plugins_dir() -> PathBuf {
    get_mahakala_home().join("plugins")
}

pub fn get_workspace_dir() -> PathBuf {
    get_mahakala_home().join("workspaces")
}

pub fn get_uploads_dir() -> PathBuf {
    get_mahakala_home().join("uploads")
}
