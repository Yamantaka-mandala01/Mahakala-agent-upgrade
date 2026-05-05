use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Tool error: {0}")]
    Tool(String),

    #[error("Auth error: {0}")]
    Auth(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            AppError::NotFound(_) => (axum::http::StatusCode::NOT_FOUND, self.to_string()),
            AppError::InvalidInput(_) => (axum::http::StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Auth(_) => (axum::http::StatusCode::UNAUTHORIZED, self.to_string()),
            _ => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string()),
        };
        (status, axum::Json(serde_json::json!({"error": message}))).into_response()
    }
}
