use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedImage {
    pub id: String,
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub path: String,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAnalysis {
    pub image_path: String,
    pub description: String,
    pub objects: Vec<String>,
    pub colors: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageConfig {
    pub model: String,
    pub default_width: u32,
    pub default_height: u32,
    pub default_format: String,
}

pub struct ImageManager {
    generated_images: Arc<Mutex<HashMap<String, GeneratedImage>>>,
    analyses: Arc<Mutex<HashMap<String, ImageAnalysis>>>,
    config: ImageConfig,
}

impl ImageManager {
    pub fn new(config: ImageConfig) -> Self {
        Self {
            generated_images: Arc::new(Mutex::new(HashMap::new())),
            analyses: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn generate_image(&self, prompt: &str, width: Option<u32>, height: Option<u32>) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let width = width.unwrap_or(self.config.default_width);
        let height = height.unwrap_or(self.config.default_height);
        
        let image = GeneratedImage {
            id: id.clone(),
            prompt: prompt.to_string(),
            width,
            height,
            format: self.config.default_format.clone(),
            path: format!("/tmp/images/{}.{}", id, self.config.default_format),
            created_at: chrono::Utc::now().timestamp(),
        };

        let mut images = self.generated_images.lock();
        images.insert(id.clone(), image);
        Ok(id)
    }

    pub fn analyze_image(&self, image_path: &str, analysis_prompt: Option<&str>) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let prompt = analysis_prompt.unwrap_or("Describe this image in detail");

        let analysis = ImageAnalysis {
            image_path: image_path.to_string(),
            description: format!("Analysis of image: {}. Prompt: {}", image_path, prompt),
            objects: vec!["object1".to_string(), "object2".to_string()],
            colors: vec!["blue".to_string(), "green".to_string()],
            confidence: 0.92,
        };

        let mut analyses = self.analyses.lock();
        analyses.insert(id.clone(), analysis);
        Ok(id)
    }

    pub fn get_generated_image(&self, id: &str) -> Option<GeneratedImage> {
        let images = self.generated_images.lock();
        images.get(id).cloned()
    }

    pub fn get_analysis(&self, id: &str) -> Option<ImageAnalysis> {
        let analyses = self.analyses.lock();
        analyses.get(id).cloned()
    }

    pub fn list_generated_images(&self) -> Vec<GeneratedImage> {
        let images = self.generated_images.lock();
        images.values().cloned().collect()
    }

    pub fn list_analyses(&self) -> Vec<ImageAnalysis> {
        let analyses = self.analyses.lock();
        analyses.values().cloned().collect()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let images = self.generated_images.lock();
        let analyses = self.analyses.lock();

        serde_json::json!({
            "total_generated_images": images.len(),
            "total_analyses": analyses.len(),
            "model": self.config.model,
            "default_size": format!("{}x{}", self.config.default_width, self.config.default_height),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_image() {
        let config = ImageConfig {
            model: "dall-e-3".to_string(),
            default_width: 1024,
            default_height: 1024,
            default_format: "png".to_string(),
        };
        let manager = ImageManager::new(config);

        let id = manager.generate_image("A beautiful sunset", None, None);
        assert!(id.is_ok());
        assert_eq!(manager.list_generated_images().len(), 1);
    }

    #[test]
    fn test_analyze_image() {
        let config = ImageConfig {
            model: "dall-e-3".to_string(),
            default_width: 1024,
            default_height: 1024,
            default_format: "png".to_string(),
        };
        let manager = ImageManager::new(config);

        let id = manager.analyze_image("/tmp/test.png", None);
        assert!(id.is_ok());
        assert_eq!(manager.list_analyses().len(), 1);
    }

    #[test]
    fn test_get_generated_image() {
        let config = ImageConfig {
            model: "dall-e-3".to_string(),
            default_width: 1024,
            default_height: 1024,
            default_format: "png".to_string(),
        };
        let manager = ImageManager::new(config);

        let id = manager.generate_image("A cat", None, None).unwrap();
        let image = manager.get_generated_image(&id);
        assert!(image.is_some());
        assert_eq!(image.unwrap().prompt, "A cat");
    }
}
