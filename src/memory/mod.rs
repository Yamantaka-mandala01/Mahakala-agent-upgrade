use crate::error::AppError;
use parking_lot::Mutex;
use rusqlite::{Connection, OptionalExtension, Row};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryFact {
    pub id: String,
    pub content: String,
    pub category: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntity {
    pub id: String,
    pub name: String,
    pub entity_type: String,
    pub attributes: serde_json::Value,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MemoryStats {
    pub facts: usize,
    pub entities: usize,
    pub dimensions: usize,
}

pub struct MemoryStore {
    conn: Arc<Mutex<Connection>>,
}

fn map_fact_row(row: &Row) -> Result<MemoryFact, rusqlite::Error> {
    Ok(MemoryFact {
        id: row.get(0)?,
        content: row.get(1)?,
        category: row.get(2)?,
        created_at: row.get(3)?,
        updated_at: row.get(4)?,
    })
}

fn map_entity_row(row: &Row) -> Result<MemoryEntity, rusqlite::Error> {
    let attrs_str: String = row.get(3)?;
    Ok(MemoryEntity {
        id: row.get(0)?,
        name: row.get(1)?,
        entity_type: row.get(2)?,
        attributes: serde_json::from_str(&attrs_str).unwrap_or(serde_json::Value::Null),
        created_at: row.get(4)?,
    })
}

impl MemoryStore {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self, AppError> {
        let path = db_path.unwrap_or_else(|| {
            let mut dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            dir.push("data");
            std::fs::create_dir_all(&dir).ok();
            dir.push("memory.db");
            dir
        });

        let conn = Connection::open(&path).map_err(|e| {
            AppError::Internal(format!("Failed to open memory database: {}", e))
        })?;

        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.init_tables()?;
        Ok(store)
    }

    fn init_tables(&self) -> Result<(), AppError> {
        let conn = self.conn.lock();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS facts (
                id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                category TEXT DEFAULT 'general',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_facts_category ON facts(category);
            CREATE INDEX IF NOT EXISTS idx_facts_content ON facts(content);

            CREATE TABLE IF NOT EXISTS entities (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                entity_type TEXT NOT NULL,
                attributes TEXT,
                created_at INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
            CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);

            CREATE TABLE IF NOT EXISTS memory_log (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_memory_log_session ON memory_log(session_id);
            "
        ).map_err(|e| AppError::Internal(format!("Failed to init memory tables: {}", e)))?;
        Ok(())
    }

    pub fn add_fact(&self, content: &str, category: Option<&str>) -> Result<MemoryFact, AppError> {
        let now = chrono::Utc::now().timestamp();
        let fact = MemoryFact {
            id: Uuid::new_v4().to_string(),
            content: content.to_string(),
            category: category.unwrap_or("general").to_string(),
            created_at: now,
            updated_at: now,
        };

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO facts (id, content, category, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&fact.id, &fact.content, &fact.category, fact.created_at, fact.updated_at),
        ).map_err(|e| AppError::Internal(format!("Failed to add fact: {}", e)))?;

        Ok(fact)
    }

    pub fn get_fact(&self, id: &str) -> Result<Option<MemoryFact>, AppError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT id, content, category, created_at, updated_at FROM facts WHERE id = ?1"
        ).map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

        let result = stmt.query_row([id], map_fact_row)
            .optional()
            .map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?;

        Ok(result)
    }

    pub fn list_facts(&self, category: Option<&str>) -> Result<Vec<MemoryFact>, AppError> {
        let conn = self.conn.lock();
        let sql = if category.is_some() {
            "SELECT id, content, category, created_at, updated_at FROM facts WHERE category = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, content, category, created_at, updated_at FROM facts ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(sql)
            .map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

        let rows = if let Some(cat) = category {
            stmt.query_map([cat], map_fact_row)
        } else {
            stmt.query_map([], map_fact_row)
        };

        let facts: Result<Vec<_>, _> = rows
            .map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?
            .collect();

        facts.map_err(|e| AppError::Internal(format!("Row mapping failed: {}", e)))
    }

    pub fn search_facts(&self, query: &str) -> Result<Vec<MemoryFact>, AppError> {
        let conn = self.conn.lock();
        let pattern = format!("%{}%", query);
        let mut stmt = conn.prepare(
            "SELECT id, content, category, created_at, updated_at FROM facts WHERE content LIKE ?1 ORDER BY created_at DESC"
        ).map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

        let facts: Result<Vec<_>, _> = stmt.query_map([&pattern], map_fact_row)
            .map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?
            .collect();

        facts.map_err(|e| AppError::Internal(format!("Row mapping failed: {}", e)))
    }

    pub fn delete_fact(&self, id: &str) -> Result<bool, AppError> {
        let conn = self.conn.lock();
        let rows = conn.execute("DELETE FROM facts WHERE id = ?1", [id])
            .map_err(|e| AppError::Internal(format!("Delete failed: {}", e)))?;
        Ok(rows > 0)
    }

    pub fn update_fact(&self, id: &str, content: &str) -> Result<bool, AppError> {
        let now = chrono::Utc::now().timestamp();
        let conn = self.conn.lock();
        let rows = conn.execute(
            "UPDATE facts SET content = ?1, updated_at = ?2 WHERE id = ?3",
            (content, now, id)
        ).map_err(|e| AppError::Internal(format!("Update failed: {}", e)))?;
        Ok(rows > 0)
    }

    pub fn add_entity(&self, name: &str, entity_type: &str, attributes: serde_json::Value) -> Result<MemoryEntity, AppError> {
        let now = chrono::Utc::now().timestamp();
        let entity = MemoryEntity {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            entity_type: entity_type.to_string(),
            attributes,
            created_at: now,
        };

        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO entities (id, name, entity_type, attributes, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&entity.id, &entity.name, &entity.entity_type, entity.attributes.to_string(), entity.created_at),
        ).map_err(|e| AppError::Internal(format!("Failed to add entity: {}", e)))?;

        Ok(entity)
    }

    pub fn list_entities(&self, entity_type: Option<&str>) -> Result<Vec<MemoryEntity>, AppError> {
        let conn = self.conn.lock();
        let sql = if entity_type.is_some() {
            "SELECT id, name, entity_type, attributes, created_at FROM entities WHERE entity_type = ?1 ORDER BY created_at DESC"
        } else {
            "SELECT id, name, entity_type, attributes, created_at FROM entities ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(sql)
            .map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

        let rows = if let Some(et) = entity_type {
            stmt.query_map([et], map_entity_row)
        } else {
            stmt.query_map([], map_entity_row)
        };

        let entities: Result<Vec<_>, _> = rows
            .map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?
            .collect();

        entities.map_err(|e| AppError::Internal(format!("Row mapping failed: {}", e)))
    }

    pub fn delete_entity(&self, id: &str) -> Result<bool, AppError> {
        let conn = self.conn.lock();
        let rows = conn.execute("DELETE FROM entities WHERE id = ?1", [id])
            .map_err(|e| AppError::Internal(format!("Delete failed: {}", e)))?;
        Ok(rows > 0)
    }

    pub fn log_interaction(&self, session_id: Option<&str>, role: &str, content: &str) -> Result<(), AppError> {
        let now = chrono::Utc::now().timestamp();
        let id = Uuid::new_v4().to_string();
        let conn = self.conn.lock();
        conn.execute(
            "INSERT INTO memory_log (id, session_id, role, content, timestamp) VALUES (?1, ?2, ?3, ?4, ?5)",
            (&id, session_id, role, content, now),
        ).map_err(|e| AppError::Internal(format!("Failed to log interaction: {}", e)))?;
        Ok(())
    }

    pub fn get_session_history(&self, session_id: &str, limit: usize) -> Result<Vec<(String, String, i64)>, AppError> {
        let conn = self.conn.lock();
        let mut stmt = conn.prepare(
            "SELECT role, content, timestamp FROM memory_log WHERE session_id = ?1 ORDER BY timestamp DESC LIMIT ?2"
        ).map_err(|e| AppError::Internal(format!("Prepare failed: {}", e)))?;

        let rows: Result<Vec<_>, _> = stmt.query_map([session_id, &limit.to_string()], |row: &Row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, i64>(2)?))
        }).map_err(|e| AppError::Internal(format!("Query failed: {}", e)))?
            .collect();

        rows.map_err(|e| AppError::Internal(format!("Row mapping failed: {}", e)))
    }

    pub fn get_stats(&self) -> Result<MemoryStats, AppError> {
        let conn = self.conn.lock();
        let facts: i64 = conn.query_row("SELECT COUNT(*) FROM facts", [], |row| row.get(0))
            .map_err(|e| AppError::Internal(format!("Count facts failed: {}", e)))?;
        let entities: i64 = conn.query_row("SELECT COUNT(*) FROM entities", [], |row| row.get(0))
            .map_err(|e| AppError::Internal(format!("Count entities failed: {}", e)))?;

        Ok(MemoryStats {
            facts: facts as usize,
            entities: entities as usize,
            dimensions: 384,
        })
    }

    pub fn clear_all(&self) -> Result<(), AppError> {
        let conn = self.conn.lock();
        conn.execute("DELETE FROM facts", [])
            .map_err(|e| AppError::Internal(format!("Clear facts failed: {}", e)))?;
        conn.execute("DELETE FROM entities", [])
            .map_err(|e| AppError::Internal(format!("Clear entities failed: {}", e)))?;
        conn.execute("DELETE FROM memory_log", [])
            .map_err(|e| AppError::Internal(format!("Clear memory_log failed: {}", e)))?;
        Ok(())
    }
}

#[derive(Clone)]
pub struct MemoryManager {
    store: Arc<MemoryStore>,
}

impl MemoryManager {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self, AppError> {
        let store = Arc::new(MemoryStore::new(db_path)?);
        Ok(Self { store })
    }

    pub fn store(&self) -> Arc<MemoryStore> {
        self.store.clone()
    }

    pub fn add_fact(&self, content: &str, category: Option<&str>) -> Result<MemoryFact, AppError> {
        self.store.add_fact(content, category)
    }

    pub fn search_facts(&self, query: &str) -> Result<Vec<MemoryFact>, AppError> {
        self.store.search_facts(query)
    }

    pub fn list_facts(&self, category: Option<&str>) -> Result<Vec<MemoryFact>, AppError> {
        self.store.list_facts(category)
    }

    pub fn delete_fact(&self, id: &str) -> Result<bool, AppError> {
        self.store.delete_fact(id)
    }

    pub fn get_stats(&self) -> Result<MemoryStats, AppError> {
        self.store.get_stats()
    }

    pub fn log_interaction(&self, session_id: Option<&str>, role: &str, content: &str) -> Result<(), AppError> {
        self.store.log_interaction(session_id, role, content)
    }

    pub fn get_session_history(&self, session_id: &str, limit: usize) -> Result<Vec<(String, String, i64)>, AppError> {
        self.store.get_session_history(session_id, limit)
    }
}
