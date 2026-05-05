use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub active: bool,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: i64,
}

pub struct WorkspaceManager {
    workspaces: Arc<Mutex<HashMap<String, Workspace>>>,
    active_workspace: Arc<Mutex<Option<String>>>,
}

impl WorkspaceManager {
    pub fn new() -> Self {
        Self {
            workspaces: Arc::new(Mutex::new(HashMap::new())),
            active_workspace: Arc::new(Mutex::new(None)),
        }
    }

    pub fn create_workspace(&self, name: &str, root_path: &str) -> Result<Workspace, AppError> {
        let path = PathBuf::from(root_path);
        if !path.exists() {
            std::fs::create_dir_all(&path).map_err(|e| {
                AppError::Io(e)
            })?;
        }

        let now = chrono::Utc::now().timestamp();
        let id = format!("ws_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        let workspace = Workspace {
            id: id.clone(),
            name: name.to_string(),
            root_path: root_path.to_string(),
            active: false,
            created_at: now,
        };

        let mut workspaces = self.workspaces.lock();
        workspaces.insert(id.clone(), workspace.clone());
        Ok(workspace)
    }

    pub fn set_active(&self, workspace_id: &str) -> Result<Workspace, AppError> {
        let mut workspaces = self.workspaces.lock();
        
        for ws in workspaces.values_mut() {
            ws.active = false;
        }

        if let Some(ws) = workspaces.get_mut(workspace_id) {
            ws.active = true;
            let mut active = self.active_workspace.lock();
            *active = Some(workspace_id.to_string());
            Ok(ws.clone())
        } else {
            Err(AppError::NotFound(format!("Workspace {} not found", workspace_id)))
        }
    }

    pub fn get_active(&self) -> Option<Workspace> {
        let active = self.active_workspace.lock();
        if let Some(ref id) = *active {
            let workspaces = self.workspaces.lock();
            workspaces.get(id).cloned()
        } else {
            None
        }
    }

    pub fn get_workspace(&self, workspace_id: &str) -> Option<Workspace> {
        let workspaces = self.workspaces.lock();
        workspaces.get(workspace_id).cloned()
    }

    pub fn list_workspaces(&self) -> Vec<Workspace> {
        let workspaces = self.workspaces.lock();
        workspaces.values().cloned().collect()
    }

    pub fn delete_workspace(&self, workspace_id: &str) -> Result<bool, AppError> {
        let mut workspaces = self.workspaces.lock();
        
        if let Some(ws) = workspaces.get(workspace_id) {
            if ws.active {
                let mut active = self.active_workspace.lock();
                *active = None;
            }
        }
        
        Ok(workspaces.remove(workspace_id).is_some())
    }

    pub fn list_files(&self, workspace_id: &str, sub_path: Option<&str>) -> Result<Vec<FileEntry>, AppError> {
        let workspace = self.get_workspace(workspace_id)
            .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", workspace_id)))?;

        let base = PathBuf::from(&workspace.root_path);
        let target = if let Some(sub) = sub_path {
            base.join(sub)
        } else {
            base
        };

        if !target.exists() || !target.is_dir() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        let read_dir = std::fs::read_dir(&target).map_err(AppError::Io)?;

        for entry in read_dir {
            let entry = entry.map_err(AppError::Io)?;
            let metadata = entry.metadata().map_err(AppError::Io)?;
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path().to_string_lossy().to_string();
            let modified = metadata.modified()
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|d| d.as_secs() as i64)
                .unwrap_or(0);

            entries.push(FileEntry {
                name,
                path,
                is_dir: metadata.is_dir(),
                size: metadata.len(),
                modified,
            });
        }

        entries.sort_by(|a, b| b.is_dir.cmp(&a.is_dir).then_with(|| a.name.cmp(&b.name)));
        Ok(entries)
    }

    pub fn read_file(&self, workspace_id: &str, file_path: &str) -> Result<String, AppError> {
        let workspace = self.get_workspace(workspace_id)
            .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", workspace_id)))?;

        let base = PathBuf::from(&workspace.root_path);
        let target = base.join(file_path);

        if !target.exists() || !target.is_file() {
            return Err(AppError::NotFound(format!("File {} not found", file_path)));
        }

        std::fs::read_to_string(&target).map_err(AppError::Io)
    }

    pub fn write_file(&self, workspace_id: &str, file_path: &str, content: &str) -> Result<(), AppError> {
        let workspace = self.get_workspace(workspace_id)
            .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", workspace_id)))?;

        let base = PathBuf::from(&workspace.root_path);
        let target = base.join(file_path);

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(AppError::Io)?;
        }

        std::fs::write(&target, content).map_err(AppError::Io)
    }

    pub fn delete_file(&self, workspace_id: &str, file_path: &str) -> Result<(), AppError> {
        let workspace = self.get_workspace(workspace_id)
            .ok_or_else(|| AppError::NotFound(format!("Workspace {} not found", workspace_id)))?;

        let base = PathBuf::from(&workspace.root_path);
        let target = base.join(file_path);

        if target.is_dir() {
            std::fs::remove_dir_all(&target).map_err(AppError::Io)
        } else {
            std::fs::remove_file(&target).map_err(AppError::Io)
        }
    }
}

#[derive(Clone)]
pub struct WorkspaceHandle {
    inner: Arc<WorkspaceManager>,
}

impl WorkspaceHandle {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(WorkspaceManager::new()),
        }
    }

    pub fn create_workspace(&self, name: &str, root_path: &str) -> Result<Workspace, AppError> {
        self.inner.create_workspace(name, root_path)
    }

    pub fn set_active(&self, workspace_id: &str) -> Result<Workspace, AppError> {
        self.inner.set_active(workspace_id)
    }

    pub fn get_active(&self) -> Option<Workspace> {
        self.inner.get_active()
    }

    pub fn get_workspace(&self, workspace_id: &str) -> Option<Workspace> {
        self.inner.get_workspace(workspace_id)
    }

    pub fn list_workspaces(&self) -> Vec<Workspace> {
        self.inner.list_workspaces()
    }

    pub fn delete_workspace(&self, workspace_id: &str) -> Result<bool, AppError> {
        self.inner.delete_workspace(workspace_id)
    }

    pub fn list_files(&self, workspace_id: &str, sub_path: Option<&str>) -> Result<Vec<FileEntry>, AppError> {
        self.inner.list_files(workspace_id, sub_path)
    }

    pub fn read_file(&self, workspace_id: &str, file_path: &str) -> Result<String, AppError> {
        self.inner.read_file(workspace_id, file_path)
    }

    pub fn write_file(&self, workspace_id: &str, file_path: &str, content: &str) -> Result<(), AppError> {
        self.inner.write_file(workspace_id, file_path, content)
    }

    pub fn delete_file(&self, workspace_id: &str, file_path: &str) -> Result<(), AppError> {
        self.inner.delete_file(workspace_id, file_path)
    }
}
