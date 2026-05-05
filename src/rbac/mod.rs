use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub id: String,
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub role_id: String,
    pub is_active: bool,
    pub created_at: i64,
    pub last_login: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub id: String,
    pub name: String,
    pub description: String,
    pub resource: String,
    pub action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RbacConfig {
    pub max_roles: usize,
    pub max_users_per_role: usize,
    pub enable_default_roles: bool,
}

impl Default for RbacConfig {
    fn default() -> Self {
        Self {
            max_roles: 50,
            max_users_per_role: 1000,
            enable_default_roles: true,
        }
    }
}

pub struct RbacSystem {
    roles: Arc<Mutex<HashMap<String, Role>>>,
    users: Arc<Mutex<HashMap<String, User>>>,
    permissions: Arc<Mutex<HashMap<String, Permission>>>,
    user_roles: Arc<Mutex<HashMap<String, Vec<String>>>>,
    config: RbacConfig,
}

impl RbacSystem {
    pub fn new(config: RbacConfig) -> Self {
        let enable_defaults = config.enable_default_roles;
        let system = Self {
            roles: Arc::new(Mutex::new(HashMap::new())),
            users: Arc::new(Mutex::new(HashMap::new())),
            permissions: Arc::new(Mutex::new(HashMap::new())),
            user_roles: Arc::new(Mutex::new(HashMap::new())),
            config,
        };

        if enable_defaults {
            system.initialize_default_roles();
        }

        system
    }

    fn initialize_default_roles(&self) {
        let admin_permissions = vec![
            "user:create".to_string(),
            "user:read".to_string(),
            "user:update".to_string(),
            "user:delete".to_string(),
            "role:create".to_string(),
            "role:read".to_string(),
            "role:update".to_string(),
            "role:delete".to_string(),
            "permission:create".to_string(),
            "permission:read".to_string(),
            "permission:update".to_string(),
            "permission:delete".to_string(),
            "system:configure".to_string(),
            "system:restart".to_string(),
            "system:backup".to_string(),
            "tool:execute".to_string(),
            "skill:execute".to_string(),
            "plugin:execute".to_string(),
            "agent:chat".to_string(),
        ];

        let editor_permissions = vec![
            "user:read".to_string(),
            "role:read".to_string(),
            "permission:read".to_string(),
            "tool:execute".to_string(),
            "skill:execute".to_string(),
            "plugin:execute".to_string(),
            "agent:chat".to_string(),
        ];

        let viewer_permissions = vec![
            "user:read".to_string(),
            "role:read".to_string(),
            "permission:read".to_string(),
            "agent:chat".to_string(),
        ];

        let _ = self.create_role("admin".to_string(), "System Administrator".to_string(), admin_permissions);
        let _ = self.create_role("editor".to_string(), "Content Editor".to_string(), editor_permissions);
        let _ = self.create_role("viewer".to_string(), "Read-only Viewer".to_string(), viewer_permissions);
    }

    pub fn create_role(&self, name: String, description: String, permissions: Vec<String>) -> Result<String, AppError> {
        let roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        if roles.len() >= self.config.max_roles {
            return Err(AppError::Internal(format!("Maximum role limit ({}) reached", self.config.max_roles)));
        }
        drop(roles);

        let id = uuid::Uuid::new_v4().to_string();
        let role = Role {
            id: id.clone(),
            name,
            description,
            permissions,
            created_at: chrono::Utc::now().timestamp(),
        };

        let mut roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        roles.insert(id.clone(), role);
        Ok(id)
    }

    pub fn get_role(&self, role_id: &str) -> Result<Role, AppError> {
        let roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        roles.get(role_id).cloned().ok_or_else(|| AppError::NotFound(format!("Role {} not found", role_id)))
    }

    pub fn list_roles(&self) -> Vec<Role> {
        let roles = self.roles.lock().unwrap();
        roles.values().cloned().collect()
    }

    pub fn update_role(&self, role_id: &str, name: Option<String>, description: Option<String>, permissions: Option<Vec<String>>) -> Result<(), AppError> {
        let mut roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        let role = roles.get_mut(role_id).ok_or_else(|| AppError::NotFound(format!("Role {} not found", role_id)))?;
        
        if let Some(n) = name {
            role.name = n;
        }
        if let Some(d) = description {
            role.description = d;
        }
        if let Some(p) = permissions {
            role.permissions = p;
        }
        
        Ok(())
    }

    pub fn delete_role(&self, role_id: &str) -> Result<(), AppError> {
        let mut roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        roles.remove(role_id).ok_or_else(|| AppError::NotFound(format!("Role {} not found", role_id)))?;
        Ok(())
    }

    pub fn create_user(&self, username: String, email: String, role_id: String) -> Result<String, AppError> {
        let roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        if !roles.contains_key(&role_id) {
            return Err(AppError::NotFound(format!("Role {} not found", role_id)));
        }
        drop(roles);

        let id = uuid::Uuid::new_v4().to_string();
        let user = User {
            id: id.clone(),
            username,
            email,
            role_id,
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
            last_login: None,
        };

        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        users.insert(id.clone(), user);
        Ok(id)
    }

    pub fn get_user(&self, user_id: &str) -> Result<User, AppError> {
        let users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        users.get(user_id).cloned().ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))
    }

    pub fn list_users(&self) -> Vec<User> {
        let users = self.users.lock().unwrap();
        users.values().cloned().collect()
    }

    pub fn update_user_role(&self, user_id: &str, role_id: &str) -> Result<(), AppError> {
        let roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        if !roles.contains_key(role_id) {
            return Err(AppError::NotFound(format!("Role {} not found", role_id)));
        }
        drop(roles);

        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        let user = users.get_mut(user_id).ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        user.role_id = role_id.to_string();
        Ok(())
    }

    pub fn delete_user(&self, user_id: &str) -> Result<(), AppError> {
        let mut users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        users.remove(user_id).ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        Ok(())
    }

    pub fn check_permission(&self, user_id: &str, permission: &str) -> Result<bool, AppError> {
        let users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        let user = users.get(user_id).ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        let role_id = user.role_id.clone();
        drop(users);

        let roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        let role = roles.get(&role_id).ok_or_else(|| AppError::NotFound(format!("Role {} not found", role_id)))?;
        
        Ok(role.permissions.contains(&permission.to_string()) || role.permissions.contains(&"*".to_string()))
    }

    pub fn get_user_permissions(&self, user_id: &str) -> Result<Vec<String>, AppError> {
        let users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        let user = users.get(user_id).ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        let role_id = user.role_id.clone();
        drop(users);

        let roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        let role = roles.get(&role_id).ok_or_else(|| AppError::NotFound(format!("Role {} not found", role_id)))?;
        
        Ok(role.permissions.clone())
    }

    pub fn create_permission(&self, name: String, description: String, resource: String, action: String) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let permission = Permission {
            id: id.clone(),
            name,
            description,
            resource,
            action,
        };

        let mut permissions = self.permissions.lock().map_err(|e| AppError::Internal(format!("Failed to lock permissions: {}", e)))?;
        permissions.insert(id.clone(), permission);
        Ok(id)
    }

    pub fn list_permissions(&self) -> Vec<Permission> {
        let permissions = self.permissions.lock().unwrap();
        permissions.values().cloned().collect()
    }

    pub fn get_user_role_name(&self, user_id: &str) -> Result<String, AppError> {
        let users = self.users.lock().map_err(|e| AppError::Internal(format!("Failed to lock users: {}", e)))?;
        let user = users.get(user_id).ok_or_else(|| AppError::NotFound(format!("User {} not found", user_id)))?;
        let role_id = user.role_id.clone();
        drop(users);

        let roles = self.roles.lock().map_err(|e| AppError::Internal(format!("Failed to lock roles: {}", e)))?;
        let role = roles.get(&role_id).ok_or_else(|| AppError::NotFound(format!("Role {} not found", role_id)))?;
        
        Ok(role.name.clone())
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let roles = self.roles.lock().unwrap();
        let users = self.users.lock().unwrap();
        let permissions = self.permissions.lock().unwrap();

        serde_json::json!({
            "total_roles": roles.len(),
            "total_users": users.len(),
            "total_permissions": permissions.len(),
            "roles": roles.values().map(|r| {
                let user_count = users.values().filter(|u| u.role_id == r.id).count();
                serde_json::json!({
                    "id": r.id,
                    "name": r.name,
                    "user_count": user_count
                })
            }).collect::<Vec<_>>()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_system() -> RbacSystem {
        RbacSystem::new(RbacConfig {
            max_roles: 10,
            max_users_per_role: 100,
            enable_default_roles: false,
        })
    }

    #[test]
    fn test_create_role() {
        let system = create_test_system();
        let permissions = vec!["user:read".to_string(), "user:write".to_string()];
        let result = system.create_role("test_role".to_string(), "Test Role".to_string(), permissions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_list_roles() {
        let system = create_test_system();
        let _ = system.create_role("role1".to_string(), "Role 1".to_string(), vec![]);
        let _ = system.create_role("role2".to_string(), "Role 2".to_string(), vec![]);
        let roles = system.list_roles();
        assert_eq!(roles.len(), 2);
    }

    #[test]
    fn test_create_user() {
        let system = create_test_system();
        let role_id = system.create_role("admin".to_string(), "Admin".to_string(), vec!["*".to_string()]).unwrap();
        let result = system.create_user("testuser".to_string(), "test@example.com".to_string(), role_id);
        assert!(result.is_ok());
    }

    #[test]
    fn test_check_permission() {
        let system = create_test_system();
        let permissions = vec!["user:read".to_string(), "user:write".to_string()];
        let role_id = system.create_role("editor".to_string(), "Editor".to_string(), permissions).unwrap();
        let user_id = system.create_user("editor1".to_string(), "editor@example.com".to_string(), role_id).unwrap();
        
        let has_read = system.check_permission(&user_id, "user:read").unwrap();
        assert!(has_read);
        
        let has_delete = system.check_permission(&user_id, "user:delete").unwrap();
        assert!(!has_delete);
    }

    #[test]
    fn test_update_user_role() {
        let system = create_test_system();
        let role1_id = system.create_role("viewer".to_string(), "Viewer".to_string(), vec!["user:read".to_string()]).unwrap();
        let role2_id = system.create_role("admin".to_string(), "Admin".to_string(), vec!["*".to_string()]).unwrap();
        let user_id = system.create_user("testuser".to_string(), "test@example.com".to_string(), role1_id).unwrap();
        
        let result = system.update_user_role(&user_id, &role2_id);
        assert!(result.is_ok());
        
        let has_all = system.check_permission(&user_id, "user:delete").unwrap();
        assert!(has_all);
    }

    #[test]
    fn test_get_user_permissions() {
        let system = create_test_system();
        let permissions = vec!["tool:execute".to_string(), "skill:execute".to_string()];
        let role_id = system.create_role("operator".to_string(), "Operator".to_string(), permissions.clone()).unwrap();
        let user_id = system.create_user("operator1".to_string(), "op@example.com".to_string(), role_id).unwrap();
        
        let user_perms = system.get_user_permissions(&user_id).unwrap();
        assert_eq!(user_perms.len(), 2);
        assert!(user_perms.contains(&"tool:execute".to_string()));
        assert!(user_perms.contains(&"skill:execute".to_string()));
    }

    #[test]
    fn test_delete_role() {
        let system = create_test_system();
        let role_id = system.create_role("temp".to_string(), "Temporary".to_string(), vec![]).unwrap();
        let result = system.delete_role(&role_id);
        assert!(result.is_ok());
        
        let roles = system.list_roles();
        assert!(roles.is_empty());
    }

    #[test]
    fn test_delete_user() {
        let system = create_test_system();
        let role_id = system.create_role("admin".to_string(), "Admin".to_string(), vec![]).unwrap();
        let user_id = system.create_user("tempuser".to_string(), "temp@example.com".to_string(), role_id).unwrap();
        
        let result = system.delete_user(&user_id);
        assert!(result.is_ok());
        
        let users = system.list_users();
        assert!(users.is_empty());
    }

    #[test]
    fn test_get_stats() {
        let system = create_test_system();
        let role_id = system.create_role("admin".to_string(), "Admin".to_string(), vec![]).unwrap();
        let _ = system.create_user("user1".to_string(), "user1@example.com".to_string(), role_id.clone()).unwrap();
        let _ = system.create_user("user2".to_string(), "user2@example.com".to_string(), role_id).unwrap();
        
        let stats = system.get_stats();
        assert_eq!(stats["total_roles"].as_u64().unwrap(), 1);
        assert_eq!(stats["total_users"].as_u64().unwrap(), 2);
    }

    #[test]
    fn test_default_roles_initialization() {
        let system = RbacSystem::new(RbacConfig::default());
        let roles = system.list_roles();
        assert!(roles.len() >= 3);
        
        let has_admin = roles.iter().any(|r| r.name == "admin");
        let has_editor = roles.iter().any(|r| r.name == "editor");
        let has_viewer = roles.iter().any(|r| r.name == "viewer");
        
        assert!(has_admin);
        assert!(has_editor);
        assert!(has_viewer);
    }
}
