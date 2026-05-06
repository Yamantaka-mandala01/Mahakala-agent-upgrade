use crate::error::AppError;
use crate::memory::MemoryManager;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemory {
    pub id: String,
    pub content: String,
    pub embedding: Vec<f64>,
    pub metadata: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub content: String,
    pub similarity: f64,
    pub metadata: serde_json::Value,
}

pub struct EmbeddingGenerator;

impl EmbeddingGenerator {
    pub fn generate(text: &str) -> Vec<f64> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut embedding = vec![0.0; 128];
        
        for (i, word) in words.iter().enumerate() {
            let hash = Self::hash_word(word);
            for j in 0..8 {
                let idx = (i * 8 + j) % 128;
                embedding[idx] += ((hash >> (j * 8)) & 0xFF) as f64 / 255.0;
            }
        }
        
        let magnitude: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
        if magnitude > 0.0 {
            for val in embedding.iter_mut() {
                *val /= magnitude;
            }
        }
        
        embedding
    }
    
    fn hash_word(word: &str) -> u64 {
        let mut hash: u64 = 0;
        for byte in word.bytes() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
        }
        hash
    }
}

pub struct SemanticSearchEngine {
    memories: Arc<Mutex<Vec<SemanticMemory>>>,
    memory_manager: Arc<MemoryManager>,
}

impl SemanticSearchEngine {
    pub fn new(memory_manager: Arc<MemoryManager>) -> Self {
        let engine = Self {
            memories: Arc::new(Mutex::new(Vec::new())),
            memory_manager,
        };
        
        engine.load_existing_memories();
        
        engine
    }
    
    fn load_existing_memories(&self) {
        if let Ok(facts) = self.memory_manager.list_facts(None) {
            let mut memories = self.memories.lock();
            for fact in facts {
                let embedding = EmbeddingGenerator::generate(&fact.content);
                memories.push(SemanticMemory {
                    id: fact.id,
                    content: fact.content,
                    embedding,
                    metadata: serde_json::json!({ "category": fact.category }),
                    created_at: fact.created_at,
                });
            }
        }
    }
    
    pub fn add_memory(&self, content: &str, metadata: Option<serde_json::Value>) -> Result<String, AppError> {
        let embedding = EmbeddingGenerator::generate(content);
        let id = uuid::Uuid::new_v4().to_string();
        
        let memory = SemanticMemory {
            id: id.clone(),
            content: content.to_string(),
            embedding,
            metadata: metadata.unwrap_or(serde_json::Value::Null),
            created_at: chrono::Utc::now().timestamp(),
        };
        
        let mut memories = self.memories.lock();
        memories.push(memory);
        
        self.memory_manager.add_fact(content, None)?;
        
        Ok(id)
    }
    
    pub fn search(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>, AppError> {
        let query_embedding = EmbeddingGenerator::generate(query);
        let memories = self.memories.lock();
        
        let mut results: Vec<SearchResult> = memories
            .iter()
            .map(|memory| {
                let similarity = Self::cosine_similarity(&query_embedding, &memory.embedding);
                SearchResult {
                    id: memory.id.clone(),
                    content: memory.content.clone(),
                    similarity,
                    metadata: memory.metadata.clone(),
                }
            })
            .collect();
        
        results.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(top_k);
        
        Ok(results)
    }
    
    pub fn cosine_similarity(a: &[f64], b: &[f64]) -> f64 {
        if a.len() != b.len() {
            return 0.0;
        }
        
        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let magnitude_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let magnitude_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        if magnitude_a > 0.0 && magnitude_b > 0.0 {
            dot_product / (magnitude_a * magnitude_b)
        } else {
            0.0
        }
    }
    
    pub fn delete_memory(&self, id: &str) -> Result<bool, AppError> {
        let mut memories = self.memories.lock();
        let initial_len = memories.len();
        memories.retain(|m| m.id != id);
        
        if memories.len() < initial_len {
            self.memory_manager.delete_fact(id)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    
    pub fn get_memory_count(&self) -> usize {
        self.memories.lock().len()
    }
    
    pub fn find_similar_concepts(&self, query: &str, threshold: f64) -> Result<Vec<String>, AppError> {
        let results = self.search(query, 20)?;
        let concepts: Vec<String> = results
            .into_iter()
            .filter(|r| r.similarity >= threshold)
            .map(|r| r.content)
            .collect();
        
        Ok(concepts)
    }
    
    pub fn cluster_memories(&self, num_clusters: usize) -> Result<Vec<Vec<String>>, AppError> {
        let memories = self.memories.lock();
        if memories.is_empty() {
            return Ok(vec![]);
        }
        
        let mut clusters: Vec<Vec<String>> = vec![Vec::new(); num_clusters];
        
        for (i, memory) in memories.iter().enumerate() {
            let cluster_idx = i % num_clusters;
            clusters[cluster_idx].push(memory.content.clone());
        }
        
        Ok(clusters)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_embedding_generation() {
        let text = "Hello world test embedding";
        let embedding = EmbeddingGenerator::generate(text);
        
        assert_eq!(embedding.len(), 128);
        
        let magnitude: f64 = embedding.iter().map(|x| x * x).sum::<f64>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.0001, "Embedding should be normalized");
    }
    
    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];
        
        assert!((SemanticSearchEngine::cosine_similarity(&a, &b) - 1.0).abs() < 0.0001);
        assert!((SemanticSearchEngine::cosine_similarity(&a, &c) - 0.0).abs() < 0.0001);
    }
    
    #[test]
    fn test_semantic_search() {
        let memory_manager = Arc::new(MemoryManager::new(None).unwrap());
        let engine = SemanticSearchEngine::new(memory_manager);
        
        engine.add_memory("Rust programming language is great", None).unwrap();
        engine.add_memory("Python is good for data science", None).unwrap();
        engine.add_memory("JavaScript is used for web development", None).unwrap();
        
        let results = engine.search("programming languages", 2).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].similarity >= results[1].similarity);
    }
}
