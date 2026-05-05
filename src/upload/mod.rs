use crate::error::AppError;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use parking_lot::Mutex;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFile {
    pub id: String,
    pub original_name: String,
    pub stored_name: String,
    pub mime_type: String,
    pub size: u64,
    pub created_at: i64,
}

pub struct UploadManager {
    upload_dir: PathBuf,
    files: Arc<Mutex<HashMap<String, UploadFile>>>,
}

impl UploadManager {
    pub fn new(upload_dir: Option<PathBuf>) -> Result<Self, AppError> {
        let dir = upload_dir.unwrap_or_else(|| {
            let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            dir.push("uploads");
            dir
        });

        std::fs::create_dir_all(&dir).map_err(AppError::Io)?;

        Ok(Self {
            upload_dir: dir,
            files: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn save_file(&self, original_name: &str, data: &[u8], mime_type: &str) -> Result<UploadFile, AppError> {
        let id = Uuid::new_v4().to_string();
        let extension = Path::new(original_name)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("bin");
        let stored_name = format!("{}. {}", id, extension);
        let file_path = self.upload_dir.join(&stored_name);

        std::fs::write(&file_path, data).map_err(AppError::Io)?;

        let now = chrono::Utc::now().timestamp();
        let file = UploadFile {
            id: id.clone(),
            original_name: original_name.to_string(),
            stored_name,
            mime_type: mime_type.to_string(),
            size: data.len() as u64,
            created_at: now,
        };

        let mut files = self.files.lock();
        files.insert(id, file.clone());

        Ok(file)
    }

    pub fn get_file(&self, file_id: &str) -> Option<UploadFile> {
        let files = self.files.lock();
        files.get(file_id).cloned()
    }

    pub fn get_file_path(&self, file_id: &str) -> Option<PathBuf> {
        let files = self.files.lock();
        files.get(file_id).map(|f| self.upload_dir.join(&f.stored_name))
    }

    pub fn read_file(&self, file_id: &str) -> Result<Vec<u8>, AppError> {
        let path = self.get_file_path(file_id)
            .ok_or_else(|| AppError::NotFound(format!("File {} not found", file_id)))?;
        std::fs::read(&path).map_err(AppError::Io)
    }

    pub fn delete_file(&self, file_id: &str) -> Result<bool, AppError> {
        let mut files = self.files.lock();
        if let Some(file) = files.get(file_id) {
            let path = self.upload_dir.join(&file.stored_name);
            if path.exists() {
                std::fs::remove_file(&path).map_err(AppError::Io)?;
            }
            files.remove(file_id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn list_files(&self) -> Vec<UploadFile> {
        let files = self.files.lock();
        files.values().cloned().collect()
    }

    pub fn get_upload_dir(&self) -> &Path {
        &self.upload_dir
    }
}

#[derive(Clone)]
pub struct UploadHandle {
    inner: Arc<UploadManager>,
}

impl UploadHandle {
    pub fn new(upload_dir: Option<PathBuf>) -> Result<Self, AppError> {
        Ok(Self {
            inner: Arc::new(UploadManager::new(upload_dir)?),
        })
    }

    pub fn save_file(&self, original_name: &str, data: &[u8], mime_type: &str) -> Result<UploadFile, AppError> {
        self.inner.save_file(original_name, data, mime_type)
    }

    pub fn get_file(&self, file_id: &str) -> Option<UploadFile> {
        self.inner.get_file(file_id)
    }

    pub fn get_file_path(&self, file_id: &str) -> Option<PathBuf> {
        self.inner.get_file_path(file_id)
    }

    pub fn read_file(&self, file_id: &str) -> Result<Vec<u8>, AppError> {
        self.inner.read_file(file_id)
    }

    pub fn delete_file(&self, file_id: &str) -> Result<bool, AppError> {
        self.inner.delete_file(file_id)
    }

    pub fn list_files(&self) -> Vec<UploadFile> {
        self.inner.list_files()
    }

    pub fn get_upload_dir(&self) -> PathBuf {
        self.inner.get_upload_dir().to_path_buf()
    }
}
