use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchDocument {
    pub id: String,
    pub title: String,
    pub content: String,
    pub document_type: String,
    pub tags: Vec<String>,
    pub created_at: i64,
    pub updated_at: i64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: SearchDocument,
    pub score: f64,
    pub highlights: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub document_id: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub sentiment: String,
    pub topics: Vec<String>,
    pub generated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchIndex {
    pub documents: HashMap<String, SearchDocument>,
    pub fts_index: HashMap<String, Vec<String>>,
    pub tag_index: HashMap<String, Vec<String>>,
    pub type_index: HashMap<String, Vec<String>>,
}

pub struct FtsSearchEngine {
    index: Arc<Mutex<SearchIndex>>,
    summaries: Arc<Mutex<HashMap<String, DocumentSummary>>>,
    search_history: Arc<Mutex<Vec<SearchRecord>>>,
    config: FtsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRecord {
    pub id: String,
    pub query: String,
    pub results_count: usize,
    pub timestamp: i64,
    pub filters: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FtsConfig {
    pub max_results: usize,
    pub enable_summarization: bool,
    pub enable_highlights: bool,
    pub summary_max_length: usize,
    pub min_search_length: usize,
}

impl Default for FtsConfig {
    fn default() -> Self {
        Self {
            max_results: 50,
            enable_summarization: true,
            enable_highlights: true,
            summary_max_length: 200,
            min_search_length: 2,
        }
    }
}

impl FtsSearchEngine {
    pub fn new(config: FtsConfig) -> Self {
        Self {
            index: Arc::new(Mutex::new(SearchIndex {
                documents: HashMap::new(),
                fts_index: HashMap::new(),
                tag_index: HashMap::new(),
                type_index: HashMap::new(),
            })),
            summaries: Arc::new(Mutex::new(HashMap::new())),
            search_history: Arc::new(Mutex::new(Vec::new())),
            config,
        }
    }

    pub fn add_document(&self, document: SearchDocument) -> Result<String, AppError> {
        let mut index = self.index.lock().map_err(|e| AppError::Internal(format!("Failed to lock index: {}", e)))?;
        
        let id = document.id.clone();
        
        let words: Vec<String> = self.tokenize(&format!("{} {}", document.title, document.content));
        for word in &words {
            index.fts_index.entry(word.clone()).or_insert_with(Vec::new).push(id.clone());
        }
        
        for tag in &document.tags {
            index.tag_index.entry(tag.clone()).or_insert_with(Vec::new).push(id.clone());
        }
        
        index.type_index.entry(document.document_type.clone()).or_insert_with(Vec::new).push(id.clone());
        
        index.documents.insert(id.clone(), document);
        
        Ok(id)
    }

    pub fn update_document(&self, document: SearchDocument) -> Result<(), AppError> {
        let mut index = self.index.lock().map_err(|e| AppError::Internal(format!("Failed to lock index: {}", e)))?;
        
        let id = document.id.clone();
        if !index.documents.contains_key(&id) {
            return Err(AppError::NotFound(format!("Document {} not found", id)));
        }
        
        let old_doc = &index.documents[&id];
        let old_words: Vec<String> = self.tokenize(&format!("{} {}", old_doc.title, old_doc.content));
        for word in old_words {
            if let Some(docs) = index.fts_index.get_mut(&word) {
                docs.retain(|d| d != &id);
                if docs.is_empty() {
                    index.fts_index.remove(&word);
                }
            }
        }
        
        let words: Vec<String> = self.tokenize(&format!("{} {}", document.title, document.content));
        for word in &words {
            index.fts_index.entry(word.clone()).or_insert_with(Vec::new).push(id.clone());
        }
        
        index.documents.insert(id, document);
        
        Ok(())
    }

    pub fn delete_document(&self, document_id: &str) -> Result<(), AppError> {
        let mut index = self.index.lock().map_err(|e| AppError::Internal(format!("Failed to lock index: {}", e)))?;
        
        if let Some(doc) = index.documents.remove(document_id) {
            let words: Vec<String> = self.tokenize(&format!("{} {}", doc.title, doc.content));
            for word in words {
                if let Some(docs) = index.fts_index.get_mut(&word) {
                    docs.retain(|d| d != document_id);
                    if docs.is_empty() {
                        index.fts_index.remove(&word);
                    }
                }
            }
            
            for tag in &doc.tags {
                if let Some(docs) = index.tag_index.get_mut(tag) {
                    docs.retain(|d| d != document_id);
                }
            }
            
            if let Some(docs) = index.type_index.get_mut(&doc.document_type) {
                docs.retain(|d| d != document_id);
            }
        }
        
        Ok(())
    }

    pub fn search(&self, query: &str, filters: Option<HashMap<String, String>>) -> Vec<SearchResult> {
        if query.len() < self.config.min_search_length {
            return Vec::new();
        }

        let index = self.index.lock().unwrap();
        let query_words: Vec<String> = self.tokenize(query);
        
        let mut scores: HashMap<String, f64> = HashMap::new();
        
        for word in &query_words {
            if let Some(doc_ids) = index.fts_index.get(word) {
                for doc_id in doc_ids {
                    *scores.entry(doc_id.clone()).or_insert(0.0) += 1.0;
                }
            }
        }
        
        let mut results: Vec<SearchResult> = scores.into_iter()
            .filter_map(|(doc_id, score)| {
                index.documents.get(&doc_id).map(|doc| {
                    let mut highlights = Vec::new();
                    if self.config.enable_highlights {
                        highlights = self.extract_highlights(&doc.content, &query_words);
                    }
                    SearchResult {
                        document: doc.clone(),
                        score,
                        highlights,
                    }
                })
            })
            .collect();
        
        if let Some(f) = &filters {
            if let Some(doc_type) = f.get("document_type") {
                results.retain(|r| r.document.document_type == *doc_type);
            }
            if let Some(tag) = f.get("tag") {
                results.retain(|r| r.document.tags.contains(tag));
            }
            if let Some(date_from) = f.get("date_from") {
                if let Ok(timestamp) = date_from.parse::<i64>() {
                    results.retain(|r| r.document.created_at >= timestamp);
                }
            }
        }
        
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        
        let results_count = results.len();
        let final_results = results.into_iter().take(self.config.max_results).collect();
        
        drop(index);
        
        let record = SearchRecord {
            id: uuid::Uuid::new_v4().to_string(),
            query: query.to_string(),
            results_count,
            timestamp: chrono::Utc::now().timestamp(),
            filters: filters.unwrap_or_default(),
        };
        
        if let Ok(mut history) = self.search_history.lock() {
            history.push(record);
        }
        
        final_results
    }

    fn tokenize(&self, text: &str) -> Vec<String> {
        text.to_lowercase()
            .split(|c: char| c.is_whitespace() || c.is_ascii_punctuation())
            .filter(|s| !s.is_empty() && s.len() >= self.config.min_search_length)
            .map(String::from)
            .collect()
    }

    fn extract_highlights(&self, content: &str, query_words: &[String]) -> Vec<String> {
        let mut highlights = Vec::new();
        let sentences: Vec<&str> = content.split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        for sentence in sentences {
            let sentence_lower = sentence.to_lowercase();
            for word in query_words {
                if sentence_lower.contains(word) {
                    highlights.push(sentence.trim().to_string());
                    break;
                }
            }
            if highlights.len() >= 3 {
                break;
            }
        }
        
        highlights
    }

    pub fn generate_summary(&self, document_id: &str) -> Result<DocumentSummary, AppError> {
        let index = self.index.lock().map_err(|e| AppError::Internal(format!("Failed to lock index: {}", e)))?;
        let document = index.documents.get(document_id)
            .ok_or_else(|| AppError::NotFound(format!("Document {} not found", document_id)))?;
        
        let summary = self.extractive_summary(&document.content, self.config.summary_max_length);
        let key_points = self.extract_key_points(&document.content, 5);
        let sentiment = self.analyze_document_sentiment(&document.content);
        let topics = self.extract_topics(&document.content);
        
        let doc_summary = DocumentSummary {
            document_id: document_id.to_string(),
            summary,
            key_points,
            sentiment,
            topics,
            generated_at: chrono::Utc::now().timestamp(),
        };
        
        drop(index);
        
        let mut summaries = self.summaries.lock().map_err(|e| AppError::Internal(format!("Failed to lock summaries: {}", e)))?;
        summaries.insert(document_id.to_string(), doc_summary.clone());
        
        Ok(doc_summary)
    }

    fn extractive_summary(&self, content: &str, max_length: usize) -> String {
        let sentences: Vec<&str> = content.split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        let mut summary = String::new();
        for sentence in sentences {
            let trimmed = sentence.trim();
            if summary.len() + trimmed.len() + 1 > max_length {
                break;
            }
            if !summary.is_empty() {
                summary.push_str(". ");
            }
            summary.push_str(trimmed);
        }
        
        if summary.len() < content.len() && !summary.ends_with('.') {
            summary.push_str("...");
        }
        
        summary
    }

    fn extract_key_points(&self, content: &str, max_points: usize) -> Vec<String> {
        let sentences: Vec<&str> = content.split(['.', '!', '?'])
            .filter(|s| !s.trim().is_empty())
            .collect();
        
        let mut key_points = Vec::new();
        for sentence in sentences.iter().take(max_points) {
            let trimmed = sentence.trim().to_string();
            if trimmed.len() > 20 {
                key_points.push(trimmed);
            }
        }
        
        key_points
    }

    fn analyze_document_sentiment(&self, content: &str) -> String {
        let positive_words = ["good", "great", "excellent", "positive", "beneficial", "successful", "improved"];
        let negative_words = ["bad", "poor", "negative", "failed", "error", "problem", "issue"];
        
        let content_lower = content.to_lowercase();
        let mut positive_count = 0;
        let mut negative_count = 0;
        
        for word in &positive_words {
            if content_lower.contains(word) {
                positive_count += 1;
            }
        }
        
        for word in &negative_words {
            if content_lower.contains(word) {
                negative_count += 1;
            }
        }
        
        if positive_count > negative_count {
            "positive".to_string()
        } else if negative_count > positive_count {
            "negative".to_string()
        } else {
            "neutral".to_string()
        }
    }

    fn extract_topics(&self, content: &str) -> Vec<String> {
        let common_topics = [
            "technology", "science", "business", "health", "education",
            "programming", "ai", "machine learning", "data", "security",
        ];
        
        let content_lower = content.to_lowercase();
        let mut topics = Vec::new();
        
        for topic in &common_topics {
            if content_lower.contains(topic) {
                topics.push(topic.to_string());
            }
        }
        
        topics
    }

    pub fn get_summary(&self, document_id: &str) -> Result<DocumentSummary, AppError> {
        let summaries = self.summaries.lock().map_err(|e| AppError::Internal(format!("Failed to lock summaries: {}", e)))?;
        summaries.get(document_id).cloned().ok_or_else(|| AppError::NotFound(format!("Summary for document {} not found", document_id)))
    }

    pub fn get_document(&self, document_id: &str) -> Result<SearchDocument, AppError> {
        let index = self.index.lock().map_err(|e| AppError::Internal(format!("Failed to lock index: {}", e)))?;
        index.documents.get(document_id).cloned().ok_or_else(|| AppError::NotFound(format!("Document {} not found", document_id)))
    }

    pub fn list_documents(&self, filters: Option<HashMap<String, String>>) -> Vec<SearchDocument> {
        let index = self.index.lock().unwrap();
        let mut documents: Vec<SearchDocument> = index.documents.values().cloned().collect();
        
        if let Some(f) = filters {
            if let Some(doc_type) = f.get("document_type") {
                documents.retain(|d| d.document_type == *doc_type);
            }
            if let Some(tag) = f.get("tag") {
                documents.retain(|d| d.tags.contains(tag));
            }
        }
        
        documents.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        documents
    }

    pub fn get_search_history(&self, limit: usize) -> Vec<SearchRecord> {
        let history = self.search_history.lock().unwrap();
        let start = history.len().saturating_sub(limit);
        history[start..].to_vec()
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let index = self.index.lock().unwrap();
        let summaries = self.summaries.lock().unwrap();
        let history = self.search_history.lock().unwrap();
        
        let mut type_counts: HashMap<String, usize> = HashMap::new();
        for doc in index.documents.values() {
            *type_counts.entry(doc.document_type.clone()).or_insert(0) += 1;
        }
        
        serde_json::json!({
            "total_documents": index.documents.len(),
            "total_summaries": summaries.len(),
            "total_searches": history.len(),
            "index_size": index.fts_index.len(),
            "document_types": type_counts,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_engine() -> FtsSearchEngine {
        FtsSearchEngine::new(FtsConfig::default())
    }

    fn create_test_document(id: &str, title: &str, content: &str, doc_type: &str) -> SearchDocument {
        SearchDocument {
            id: id.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            document_type: doc_type.to_string(),
            tags: vec!["test".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        }
    }

    #[test]
    fn test_add_document() {
        let engine = create_test_engine();
        let doc = create_test_document("doc1", "Test Document", "This is a test document content", "article");
        let result = engine.add_document(doc);
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_documents() {
        let engine = create_test_engine();
        let doc1 = create_test_document("doc1", "Rust Programming", "Rust is a systems programming language", "article");
        let doc2 = create_test_document("doc2", "Python Guide", "Python is great for data science", "guide");
        
        let _ = engine.add_document(doc1);
        let _ = engine.add_document(doc2);

        let results = engine.search("rust programming", None);
        assert!(!results.is_empty());
        assert!(results[0].document.id == "doc1");
    }

    #[test]
    fn test_delete_document() {
        let engine = create_test_engine();
        let doc = create_test_document("doc1", "Test", "Test content", "article");
        let _ = engine.add_document(doc);

        let result = engine.delete_document("doc1");
        assert!(result.is_ok());

        let results = engine.search("test", None);
        assert!(results.is_empty());
    }

    #[test]
    fn test_generate_summary() {
        let engine = create_test_engine();
        let content = "This is the first sentence. This is the second sentence. This is the third sentence. This is the fourth sentence.";
        let doc = create_test_document("doc1", "Test Summary", content, "article");
        let _ = engine.add_document(doc);

        let summary = engine.generate_summary("doc1").unwrap();
        assert!(!summary.summary.is_empty());
        assert!(!summary.key_points.is_empty());
    }

    #[test]
    fn test_get_summary() {
        let engine = create_test_engine();
        let doc = create_test_document("doc1", "Test", "Test content for summary", "article");
        let _ = engine.add_document(doc);

        let _ = engine.generate_summary("doc1");
        let summary = engine.get_summary("doc1").unwrap();
        assert_eq!(summary.document_id, "doc1");
    }

    #[test]
    fn test_list_documents() {
        let engine = create_test_engine();
        let doc1 = create_test_document("doc1", "Test 1", "Content 1", "article");
        let doc2 = create_test_document("doc2", "Test 2", "Content 2", "guide");
        
        let _ = engine.add_document(doc1);
        let _ = engine.add_document(doc2);

        let documents = engine.list_documents(None);
        assert_eq!(documents.len(), 2);

        let mut filters = HashMap::new();
        filters.insert("document_type".to_string(), "article".to_string());
        let filtered = engine.list_documents(Some(filters));
        assert_eq!(filtered.len(), 1);
    }

    #[test]
    fn test_search_with_filters() {
        let engine = create_test_engine();
        let doc1 = create_test_document("doc1", "Rust Article", "Rust programming article", "article");
        let doc2 = create_test_document("doc2", "Rust Guide", "Rust programming guide", "guide");
        
        let _ = engine.add_document(doc1);
        let _ = engine.add_document(doc2);

        let mut filters = HashMap::new();
        filters.insert("document_type".to_string(), "article".to_string());
        let results = engine.search("rust", Some(filters));
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].document.id, "doc1");
    }

    #[test]
    fn test_get_stats() {
        let engine = create_test_engine();
        let doc = create_test_document("doc1", "Test", "Test content", "article");
        let _ = engine.add_document(doc);

        let stats = engine.get_stats();
        assert_eq!(stats["total_documents"].as_u64().unwrap(), 1);
    }

    #[test]
    fn test_update_document() {
        let engine = create_test_engine();
        let doc = create_test_document("doc1", "Original", "Original content", "article");
        let _ = engine.add_document(doc);

        let updated_doc = SearchDocument {
            id: "doc1".to_string(),
            title: "Updated".to_string(),
            content: "Updated content".to_string(),
            document_type: "article".to_string(),
            tags: vec!["test".to_string(), "updated".to_string()],
            created_at: chrono::Utc::now().timestamp(),
            updated_at: chrono::Utc::now().timestamp(),
            metadata: HashMap::new(),
        };

        let result = engine.update_document(updated_doc);
        assert!(result.is_ok());

        let results = engine.search("updated", None);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_search_history() {
        let engine = create_test_engine();
        let doc = create_test_document("doc1", "Test", "Test content for search", "article");
        let _ = engine.add_document(doc);

        let _ = engine.search("test", None);
        let _ = engine.search("content", None);

        let history = engine.get_search_history(10);
        assert_eq!(history.len(), 2);
    }
}
