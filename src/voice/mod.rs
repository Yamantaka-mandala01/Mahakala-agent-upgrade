use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioStream {
    pub id: String,
    pub session_id: String,
    pub format: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub is_active: bool,
    pub created_at: i64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionResult {
    pub text: String,
    pub language: String,
    pub confidence: f64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechConfig {
    pub model: String,
    pub language: String,
    pub sample_rate: u32,
    pub channels: u16,
    pub format: String,
}

pub struct VoiceManager {
    streams: Arc<Mutex<HashMap<String, AudioStream>>>,
    transcriptions: Arc<Mutex<HashMap<String, TranscriptionResult>>>,
    config: SpeechConfig,
}

impl VoiceManager {
    pub fn new(config: SpeechConfig) -> Self {
        Self {
            streams: Arc::new(Mutex::new(HashMap::new())),
            transcriptions: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }

    pub fn start_stream(&self, session_id: &str) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let stream = AudioStream {
            id: id.clone(),
            session_id: session_id.to_string(),
            format: self.config.format.clone(),
            sample_rate: self.config.sample_rate,
            channels: self.config.channels,
            is_active: true,
            created_at: chrono::Utc::now().timestamp(),
            duration_ms: 0,
        };

        let mut streams = self.streams.lock();
        streams.insert(id.clone(), stream);
        Ok(id)
    }

    pub fn stop_stream(&self, stream_id: &str) -> Result<(), AppError> {
        let mut streams = self.streams.lock();
        if let Some(stream) = streams.get_mut(stream_id) {
            stream.is_active = false;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Audio stream {} not found", stream_id)))
        }
    }

    pub fn process_audio_chunk(&self, stream_id: &str, _audio_data: &[u8]) -> Result<(), AppError> {
        let mut streams = self.streams.lock();
        if let Some(stream) = streams.get_mut(stream_id) {
            if !stream.is_active {
                return Err(AppError::Internal(format!("Audio stream {} is not active", stream_id)));
            }
            stream.duration_ms += 100;
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Audio stream {} not found", stream_id)))
        }
    }

    pub fn transcribe(&self, stream_id: &str) -> Result<TranscriptionResult, AppError> {
        let streams = self.streams.lock();
        if let Some(stream) = streams.get(stream_id) {
            let result = TranscriptionResult {
                text: "This is a simulated transcription of the audio stream".to_string(),
                language: self.config.language.clone(),
                confidence: 0.95,
                duration_ms: stream.duration_ms,
            };

            let mut transcriptions = self.transcriptions.lock();
            transcriptions.insert(stream_id.to_string(), result.clone());

            Ok(result)
        } else {
            Err(AppError::NotFound(format!("Audio stream {} not found", stream_id)))
        }
    }

    pub fn synthesize_speech(&self, text: &str) -> Result<Vec<u8>, AppError> {
        Ok(format!("Synthesized audio for: {}", text).into_bytes())
    }

    pub fn list_streams(&self) -> Vec<AudioStream> {
        let streams = self.streams.lock();
        streams.values().cloned().collect()
    }

    pub fn get_stream(&self, stream_id: &str) -> Option<AudioStream> {
        let streams = self.streams.lock();
        streams.get(stream_id).cloned()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let streams = self.streams.lock();
        let total = streams.len();
        let active = streams.values().filter(|s| s.is_active).count();

        serde_json::json!({
            "total_streams": total,
            "active_streams": active,
            "model": self.config.model,
            "language": self.config.language,
            "sample_rate": self.config.sample_rate,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_start_stream() {
        let config = SpeechConfig {
            model: "whisper".to_string(),
            language: "en".to_string(),
            sample_rate: 16000,
            channels: 1,
            format: "pcm".to_string(),
        };
        let manager = VoiceManager::new(config);

        let id = manager.start_stream("session_1");
        assert!(id.is_ok());
        assert_eq!(manager.list_streams().len(), 1);
    }

    #[test]
    fn test_process_and_transcribe() {
        let config = SpeechConfig {
            model: "whisper".to_string(),
            language: "en".to_string(),
            sample_rate: 16000,
            channels: 1,
            format: "pcm".to_string(),
        };
        let manager = VoiceManager::new(config);

        let stream_id = manager.start_stream("session_1").unwrap();
        
        let audio_data = vec![0u8; 1024];
        manager.process_audio_chunk(&stream_id, &audio_data).unwrap();
        manager.process_audio_chunk(&stream_id, &audio_data).unwrap();

        let result = manager.transcribe(&stream_id);
        assert!(result.is_ok());
        let transcription = result.unwrap();
        assert!(!transcription.text.is_empty());
        assert_eq!(transcription.language, "en");
    }

    #[test]
    fn test_stop_stream() {
        let config = SpeechConfig {
            model: "whisper".to_string(),
            language: "en".to_string(),
            sample_rate: 16000,
            channels: 1,
            format: "pcm".to_string(),
        };
        let manager = VoiceManager::new(config);

        let stream_id = manager.start_stream("session_1").unwrap();
        manager.stop_stream(&stream_id).unwrap();

        let stream = manager.get_stream(&stream_id).unwrap();
        assert!(!stream.is_active);
    }
}
