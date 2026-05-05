use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WechatQRSession {
    pub id: String,
    pub qr_url: String,
    pub qr_data: Vec<u8>,
    pub status: WechatQRStatus,
    pub created_at: i64,
    pub expires_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WechatQRStatus {
    Wait,
    Scanned,
    Confirmed,
    Expired,
    Error(String),
}

impl WechatQRStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WechatQRStatus::Wait => "wait",
            WechatQRStatus::Scanned => "scaned",
            WechatQRStatus::Confirmed => "confirmed",
            WechatQRStatus::Expired => "expired",
            WechatQRStatus::Error(_) => "error",
        }
    }
}

pub struct WechatManager {
    sessions: Arc<Mutex<HashMap<String, WechatQRSession>>>,
    png_dir: std::path::PathBuf,
}

impl WechatManager {
    pub fn new() -> Result<Self, AppError> {
        let png_dir = crate::constants::get_mahakala_home().join("wechat_qr");
        std::fs::create_dir_all(&png_dir).map_err(|e| {
            AppError::Internal(format!("Failed to create wechat qr directory: {}", e))
        })?;

        Ok(Self {
            sessions: Arc::new(Mutex::new(HashMap::new())),
            png_dir,
        })
    }

    pub fn generate_qr(&self) -> Result<WechatQRSession, AppError> {
        let id = Uuid::new_v4().to_string();
        let now = chrono::Utc::now().timestamp();
        let expires = now + 300;

        let qr_data = generate_qr_png(&id)?;
        let file_name = format!("{}.png", id);
        let file_path = self.png_dir.join(&file_name);
        std::fs::write(&file_path, &qr_data).map_err(|e| {
            AppError::Internal(format!("Failed to write QR PNG: {}", e))
        })?;

        let session = WechatQRSession {
            id: id.clone(),
            qr_url: format!("/api/wechat/qr/image/{}", id),
            qr_data,
            status: WechatQRStatus::Wait,
            created_at: now,
            expires_at: expires,
        };

        let mut sessions = self.sessions.lock();
        sessions.insert(id, session.clone());

        Ok(session)
    }

    pub fn get_qr_image_path(&self, session_id: &str) -> Option<std::path::PathBuf> {
        let file_name = format!("{}.png", session_id);
        let path = self.png_dir.join(&file_name);
        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn get_session(&self, session_id: &str) -> Option<WechatQRSession> {
        let sessions = self.sessions.lock();
        sessions.get(session_id).cloned()
    }

    pub fn update_status(&self, session_id: &str, status: WechatQRStatus) -> Result<bool, AppError> {
        let mut sessions = self.sessions.lock();
        if let Some(session) = sessions.get_mut(session_id) {
            session.status = status;
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("QR session {} not found", session_id)))
        }
    }

    pub fn check_status(&self, session_id: &str) -> Result<WechatQRStatus, AppError> {
        let mut sessions = self.sessions.lock();
        if let Some(session) = sessions.get_mut(session_id) {
            let now = chrono::Utc::now().timestamp();
            if session.status == WechatQRStatus::Wait && now > session.expires_at {
                session.status = WechatQRStatus::Expired;
            }
            Ok(session.status.clone())
        } else {
            Err(AppError::NotFound(format!("QR session {} not found", session_id)))
        }
    }

    pub fn cleanup_expired(&self) {
        let now = chrono::Utc::now().timestamp();
        let mut sessions = self.sessions.lock();
        sessions.retain(|_, session| {
            if now > session.expires_at {
                let file_name = format!("{}.png", session.id);
                let _ = std::fs::remove_file(self.png_dir.join(&file_name));
                false
            } else {
                true
            }
        });
    }

    pub fn simulate_scan(&self, session_id: &str) -> Result<bool, AppError> {
        let mut sessions = self.sessions.lock();
        if let Some(session) = sessions.get_mut(session_id) {
            if session.status == WechatQRStatus::Wait {
                session.status = WechatQRStatus::Scanned;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(AppError::NotFound(format!("QR session {} not found", session_id)))
        }
    }

    pub fn simulate_confirm(&self, session_id: &str) -> Result<bool, AppError> {
        let mut sessions = self.sessions.lock();
        if let Some(session) = sessions.get_mut(session_id) {
            if session.status == WechatQRStatus::Scanned {
                session.status = WechatQRStatus::Confirmed;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(AppError::NotFound(format!("QR session {} not found", session_id)))
        }
    }
}

#[derive(Clone)]
pub struct WechatHandle {
    inner: Arc<WechatManager>,
}

impl WechatHandle {
    pub fn new() -> Result<Self, AppError> {
        Ok(Self {
            inner: Arc::new(WechatManager::new()?),
        })
    }

    pub fn generate_qr(&self) -> Result<WechatQRSession, AppError> {
        self.inner.generate_qr()
    }

    pub fn get_qr_image_path(&self, session_id: &str) -> Option<std::path::PathBuf> {
        self.inner.get_qr_image_path(session_id)
    }

    pub fn get_session(&self, session_id: &str) -> Option<WechatQRSession> {
        self.inner.get_session(session_id)
    }

    pub fn update_status(&self, session_id: &str, status: WechatQRStatus) -> Result<bool, AppError> {
        self.inner.update_status(session_id, status)
    }

    pub fn check_status(&self, session_id: &str) -> Result<WechatQRStatus, AppError> {
        self.inner.check_status(session_id)
    }

    pub fn cleanup_expired(&self) {
        self.inner.cleanup_expired();
    }

    pub fn simulate_scan(&self, session_id: &str) -> Result<bool, AppError> {
        self.inner.simulate_scan(session_id)
    }

    pub fn simulate_confirm(&self, session_id: &str) -> Result<bool, AppError> {
        self.inner.simulate_confirm(session_id)
    }
}

fn generate_qr_png(content: &str) -> Result<Vec<u8>, AppError> {
    use qrcode::QrCode;
    use qrcode::render::svg;

    let code = QrCode::new(content.as_bytes())
        .map_err(|e| AppError::Internal(format!("Failed to generate QR code: {}", e)))?;

    let svg_string = code.render::<svg::Color>()
        .min_dimensions(256, 256)
        .build();

    let png_data = svg_to_png(&svg_string)
        .map_err(|e| AppError::Internal(format!("Failed to convert SVG to PNG: {}", e)))?;

    Ok(png_data)
}

fn svg_to_png(svg_data: &str) -> Result<Vec<u8>, AppError> {
    let tree = resvg::usvg::Tree::from_str(svg_data, &resvg::usvg::Options::default())
        .map_err(|e| AppError::Internal(format!("Failed to parse SVG: {}", e)))?;

    let pixmap_size = tree.size().to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or_else(|| AppError::Internal("Failed to create pixmap".to_string()))?;

    resvg::render(&tree, resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let img = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
        .ok_or_else(|| AppError::Internal("Failed to create image buffer".to_string()))?;

    let mut png_data = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| AppError::Internal(format!("Failed to encode PNG: {}", e)))?;

    Ok(png_data)
}
