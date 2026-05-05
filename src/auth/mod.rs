use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub user_id: String,
    pub username: String,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

pub struct AuthManager {
    users: Arc<Mutex<HashMap<String, User>>>,
    sessions: Arc<Mutex<HashMap<String, Session>>>,
}

impl AuthManager {
    pub fn new() -> Self {
        let mut users = HashMap::new();
        let now = chrono::Utc::now().timestamp();

        users.insert("admin".to_string(), User {
            id: "admin".to_string(),
            username: "admin".to_string(),
            password_hash: hash_password("admin"),
            role: "admin".to_string(),
            created_at: now,
        });

        Self {
            users: Arc::new(Mutex::new(users)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn login(&self, req: LoginRequest) -> Result<Session, AppError> {
        let users = self.users.lock();
        let user = users.get(&req.username)
            .ok_or_else(|| AppError::Auth("Invalid username or password".to_string()))?;

        if !verify_password(&req.password, &user.password_hash) {
            return Err(AppError::Auth("Invalid username or password".to_string()));
        }

        let now = chrono::Utc::now().timestamp();
        let token = Uuid::new_v4().to_string();
        let session = Session {
            token: token.clone(),
            user_id: user.id.clone(),
            username: user.username.clone(),
            created_at: now,
            expires_at: now + 86400,
        };

        let mut sessions = self.sessions.lock();
        sessions.insert(token, session.clone());

        Ok(session)
    }

    pub fn logout(&self, token: &str) -> Result<bool, AppError> {
        let mut sessions = self.sessions.lock();
        Ok(sessions.remove(token).is_some())
    }

    pub fn validate_token(&self, token: &str) -> Result<Session, AppError> {
        let sessions = self.sessions.lock();
        let session = sessions.get(token)
            .ok_or_else(|| AppError::Auth("Invalid or expired token".to_string()))?;

        let now = chrono::Utc::now().timestamp();
        if session.expires_at < now {
            return Err(AppError::Auth("Token expired".to_string()));
        }

        Ok(session.clone())
    }

    pub fn create_user(&self, username: &str, password: &str, role: &str) -> Result<User, AppError> {
        let mut users = self.users.lock();
        if users.contains_key(username) {
            return Err(AppError::InvalidInput(format!("User {} already exists", username)));
        }

        let now = chrono::Utc::now().timestamp();
        let user = User {
            id: Uuid::new_v4().to_string(),
            username: username.to_string(),
            password_hash: hash_password(password),
            role: role.to_string(),
            created_at: now,
        };

        users.insert(username.to_string(), user.clone());
        Ok(user)
    }

    pub fn get_user(&self, username: &str) -> Option<User> {
        let users = self.users.lock();
        users.get(username).cloned()
    }

    pub fn list_users(&self) -> Vec<User> {
        let users = self.users.lock();
        users.values().cloned().collect()
    }

    pub fn delete_user(&self, username: &str) -> Result<bool, AppError> {
        if username == "admin" {
            return Err(AppError::InvalidInput("Cannot delete admin user".to_string()));
        }
        let mut users = self.users.lock();
        Ok(users.remove(username).is_some())
    }

    pub fn change_password(&self, username: &str, new_password: &str) -> Result<bool, AppError> {
        let mut users = self.users.lock();
        if let Some(user) = users.get_mut(username) {
            user.password_hash = hash_password(new_password);
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("User {} not found", username)))
        }
    }

    pub fn cleanup_expired_sessions(&self) {
        let now = chrono::Utc::now().timestamp();
        let mut sessions = self.sessions.lock();
        sessions.retain(|_, session| session.expires_at > now);
    }
}

#[derive(Clone)]
pub struct AuthHandle {
    inner: Arc<AuthManager>,
}

impl AuthHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(AuthManager::new()),
        }
    }

    pub fn login(&self, req: LoginRequest) -> Result<Session, AppError> {
        self.inner.login(req)
    }

    pub fn logout(&self, token: &str) -> Result<bool, AppError> {
        self.inner.logout(token)
    }

    pub fn validate_token(&self, token: &str) -> Result<Session, AppError> {
        self.inner.validate_token(token)
    }

    pub fn create_user(&self, username: &str, password: &str, role: &str) -> Result<User, AppError> {
        self.inner.create_user(username, password, role)
    }

    pub fn get_user(&self, username: &str) -> Option<User> {
        self.inner.get_user(username)
    }

    pub fn list_users(&self) -> Vec<User> {
        self.inner.list_users()
    }

    pub fn delete_user(&self, username: &str) -> Result<bool, AppError> {
        self.inner.delete_user(username)
    }

    pub fn change_password(&self, username: &str, new_password: &str) -> Result<bool, AppError> {
        self.inner.change_password(username, new_password)
    }

    pub fn cleanup_expired_sessions(&self) {
        self.inner.cleanup_expired_sessions();
    }
}

fn hash_password(password: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn verify_password(password: &str, hash: &str) -> bool {
    hash_password(password) == hash
}
