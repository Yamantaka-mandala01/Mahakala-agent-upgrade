use rusqlite::{Connection, Result};
use std::path::Path;
use std::sync::Mutex;

pub struct SessionDB {
    conn: Mutex<Connection>,
}

#[derive(Debug)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub messages: String,
}

impl SessionDB {
    pub fn new(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .unwrap_or_else(|e| panic!("Failed to create database directory {}: {}", parent.display(), e));
        }
        
        let conn = Connection::open(db_path)
            .map_err(|e| {
                eprintln!("Failed to open database at {}: {}", db_path.display(), e);
                e
            })?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                messages TEXT NOT NULL DEFAULT '[]'
            )",
            [],
        )?;
        
        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn create_session(&self, id: &str, title: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO sessions (id, title, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            [id, title, &now, &now],
        )?;
        Ok(())
    }

    pub fn get_session(&self, id: &str) -> Result<Option<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, created_at, updated_at, messages FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query([id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                messages: row.get(4)?,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_sessions(&self) -> Result<Vec<Session>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT id, title, created_at, updated_at, messages FROM sessions ORDER BY updated_at DESC")?;
        let rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                title: row.get(1)?,
                created_at: row.get(2)?,
                updated_at: row.get(3)?,
                messages: row.get(4)?,
            })
        })?;
        
        rows.collect()
    }

    pub fn delete_session(&self, id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM sessions WHERE id = ?1", [id])?;
        Ok(())
    }

    pub fn update_session_title(&self, id: &str, title: &str) -> Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE sessions SET title = ?1, updated_at = ?2 WHERE id = ?3",
            [title, &now, id],
        )?;
        Ok(())
    }

    pub fn add_message(&self, session_id: &str, role: &str, content: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT messages FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query([session_id])?;
        
        if let Some(row) = rows.next()? {
            let messages_str: String = row.get(0)?;
            let mut messages: Vec<serde_json::Value> = serde_json::from_str(&messages_str).unwrap_or_default();
            messages.push(serde_json::json!({
                "role": role,
                "content": content,
                "timestamp": chrono::Utc::now().to_rfc3339()
            }));
            
            let messages_json = serde_json::to_string(&messages).map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let now = chrono::Utc::now().to_rfc3339();
            
            conn.execute(
                "UPDATE sessions SET messages = ?1, updated_at = ?2 WHERE id = ?3",
                [&messages_json, &now, session_id],
            )?;
        }
        Ok(())
    }

    pub fn get_messages(&self, session_id: &str) -> Result<Vec<serde_json::Value>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT messages FROM sessions WHERE id = ?1")?;
        let mut rows = stmt.query([session_id])?;
        
        if let Some(row) = rows.next()? {
            let messages_str: String = row.get(0)?;
            let messages: Vec<serde_json::Value> = serde_json::from_str(&messages_str).unwrap_or_default();
            Ok(messages)
        } else {
            Ok(Vec::new())
        }
    }
}
