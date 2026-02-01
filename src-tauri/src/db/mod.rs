use rusqlite::{params, Connection};
use std::path::Path;
use std::sync::Mutex;

use crate::types::*;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(db_path: &Path) -> Result<Self, rusqlite::Error> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    pub fn new_in_memory() -> Result<Self, rusqlite::Error> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let db = Self {
            conn: Mutex::new(conn),
        };
        db.initialize()?;
        Ok(db)
    }

    fn initialize(&self) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS sessions (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                status TEXT DEFAULT 'active',
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
            );
            CREATE TABLE IF NOT EXISTS messages (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                metadata TEXT,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS documents (
                id TEXT PRIMARY KEY,
                session_id TEXT NOT NULL,
                filename TEXT NOT NULL,
                content TEXT NOT NULL,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS preferences (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY
            );
            INSERT OR IGNORE INTO schema_migrations (version) VALUES (1);
            CREATE INDEX IF NOT EXISTS idx_messages_session ON messages(session_id);
            CREATE INDEX IF NOT EXISTS idx_documents_session ON documents(session_id);
            CREATE INDEX IF NOT EXISTS idx_sessions_updated ON sessions(updated_at DESC);
            ",
        )?;
        Ok(())
    }

    pub fn is_ok(&self) -> bool {
        let conn = self.conn();
        conn.execute_batch("SELECT 1").is_ok()
    }

    // ---- Sessions ----

    pub fn create_session(&self, name: Option<&str>) -> Result<Session, rusqlite::Error> {
        let conn = self.conn();
        let id = uuid::Uuid::new_v4().to_string();
        let session_name = name.unwrap_or("New Project");

        conn.execute(
            "INSERT INTO sessions (id, name) VALUES (?1, ?2)",
            params![id, session_name],
        )?;

        Self::read_session_row(&conn, &id)
    }

    pub fn get_sessions(&self) -> Result<Vec<Session>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, name, description, status, created_at, updated_at FROM sessions ORDER BY updated_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Session {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                status: row.get(3)?,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        })?;

        rows.collect()
    }

    pub fn get_session(&self, session_id: &str) -> Result<Session, rusqlite::Error> {
        let conn = self.conn();
        Self::read_session_row(&conn, session_id)
    }

    pub fn update_session(
        &self,
        session_id: &str,
        name: Option<&str>,
        status: Option<&str>,
    ) -> Result<Session, rusqlite::Error> {
        let conn = self.conn();

        if let Some(n) = name {
            conn.execute(
                "UPDATE sessions SET name = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![n, session_id],
            )?;
        }
        if let Some(s) = status {
            conn.execute(
                "UPDATE sessions SET status = ?1, updated_at = CURRENT_TIMESTAMP WHERE id = ?2",
                params![s, session_id],
            )?;
        }

        Self::read_session_row(&conn, session_id)
    }

    pub fn delete_session(&self, session_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute("DELETE FROM sessions WHERE id = ?1", params![session_id])?;
        Ok(())
    }

    fn read_session_row(conn: &Connection, id: &str) -> Result<Session, rusqlite::Error> {
        conn.query_row(
            "SELECT id, name, description, status, created_at, updated_at FROM sessions WHERE id = ?1",
            params![id],
            |row| {
                Ok(Session {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    status: row.get(3)?,
                    created_at: row.get(4)?,
                    updated_at: row.get(5)?,
                })
            },
        )
    }

    // ---- Messages ----

    pub fn save_message(
        &self,
        session_id: &str,
        role: &str,
        content: &str,
        metadata: Option<&str>,
    ) -> Result<Message, rusqlite::Error> {
        let mut conn = self.conn();
        let id = uuid::Uuid::new_v4().to_string();
        let tx = conn.transaction()?;
        tx.execute(
            "INSERT INTO messages (id, session_id, role, content, metadata) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, session_id, role, content, metadata],
        )?;
        tx.execute(
            "UPDATE sessions SET updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
            params![session_id],
        )?;
        let msg = tx.query_row(
            "SELECT id, session_id, role, content, metadata, created_at FROM messages WHERE id = ?1",
            params![id],
            |row| {
                Ok(Message {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    metadata: row.get(4)?,
                    created_at: row.get(5)?,
                })
            },
        )?;
        tx.commit()?;
        Ok(msg)
    }

    pub fn get_messages(&self, session_id: &str) -> Result<Vec<Message>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, role, content, metadata, created_at FROM messages WHERE session_id = ?1 ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            Ok(Message {
                id: row.get(0)?,
                session_id: row.get(1)?,
                role: row.get(2)?,
                content: row.get(3)?,
                metadata: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?;

        rows.collect()
    }

    pub fn message_count(&self, session_id: &str) -> Result<i64, rusqlite::Error> {
        let conn = self.conn();
        conn.query_row(
            "SELECT COUNT(*) FROM messages WHERE session_id = ?1 AND role = 'user'",
            params![session_id],
            |row| row.get(0),
        )
    }

    // ---- Documents ----

    #[allow(dead_code)]
    pub fn save_document(
        &self,
        session_id: &str,
        filename: &str,
        content: &str,
    ) -> Result<GeneratedDocument, rusqlite::Error> {
        let conn = self.conn();
        let id = uuid::Uuid::new_v4().to_string();

        conn.execute(
            "INSERT INTO documents (id, session_id, filename, content) VALUES (?1, ?2, ?3, ?4)",
            params![id, session_id, filename, content],
        )?;

        conn.query_row(
            "SELECT id, session_id, filename, content, created_at FROM documents WHERE id = ?1",
            params![id],
            |row| {
                Ok(GeneratedDocument {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    filename: row.get(2)?,
                    content: row.get(3)?,
                    created_at: row.get(4)?,
                })
            },
        )
    }

    pub fn get_documents(
        &self,
        session_id: &str,
    ) -> Result<Vec<GeneratedDocument>, rusqlite::Error> {
        let conn = self.conn();
        let mut stmt = conn.prepare(
            "SELECT id, session_id, filename, content, created_at FROM documents WHERE session_id = ?1 ORDER BY created_at ASC",
        )?;

        let rows = stmt.query_map(params![session_id], |row| {
            Ok(GeneratedDocument {
                id: row.get(0)?,
                session_id: row.get(1)?,
                filename: row.get(2)?,
                content: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?;

        rows.collect()
    }

    #[allow(dead_code)]
    pub fn delete_documents(&self, session_id: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute(
            "DELETE FROM documents WHERE session_id = ?1",
            params![session_id],
        )?;
        Ok(())
    }

    pub fn replace_documents(
        &self,
        session_id: &str,
        docs: &[(String, String)],
    ) -> Result<Vec<GeneratedDocument>, rusqlite::Error> {
        let mut conn = self.conn();
        let tx = conn.transaction()?;
        tx.execute(
            "DELETE FROM documents WHERE session_id = ?1",
            params![session_id],
        )?;

        let mut inserted = Vec::with_capacity(docs.len());
        for (filename, content) in docs {
            let id = uuid::Uuid::new_v4().to_string();
            tx.execute(
                "INSERT INTO documents (id, session_id, filename, content) VALUES (?1, ?2, ?3, ?4)",
                params![id, session_id, filename, content],
            )?;

            let doc = tx.query_row(
                "SELECT id, session_id, filename, content, created_at FROM documents WHERE id = ?1",
                params![id],
                |row| {
                    Ok(GeneratedDocument {
                        id: row.get(0)?,
                        session_id: row.get(1)?,
                        filename: row.get(2)?,
                        content: row.get(3)?,
                        created_at: row.get(4)?,
                    })
                },
            )?;
            inserted.push(doc);
        }
        tx.commit()?;
        Ok(inserted)
    }

    pub fn latest_document_time(
        &self,
        session_id: &str,
    ) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.conn();
        conn.query_row(
            "SELECT MAX(created_at) FROM documents WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
    }

    pub fn latest_message_time(&self, session_id: &str) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.conn();
        conn.query_row(
            "SELECT MAX(created_at) FROM messages WHERE session_id = ?1",
            params![session_id],
            |row| row.get(0),
        )
    }

    // ---- Preferences ----

    pub fn get_preference(&self, key: &str) -> Result<Option<String>, rusqlite::Error> {
        let conn = self.conn();
        match conn.query_row(
            "SELECT value FROM preferences WHERE key = ?1",
            params![key],
            |row| row.get::<_, String>(0),
        ) {
            Ok(val) => Ok(Some(val)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn set_preference(&self, key: &str, value: &str) -> Result<(), rusqlite::Error> {
        let conn = self.conn();
        conn.execute(
            "INSERT OR REPLACE INTO preferences (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }

    fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Database {
        let dir = tempfile::tempdir().unwrap();
        Database::new(&dir.path().join("test.db")).unwrap()
    }

    // ---- Session Tests ----

    #[test]
    fn create_session_default_name() {
        let db = test_db();
        let session = db.create_session(None).unwrap();
        assert_eq!(session.name, "New Project");
        assert_eq!(session.status, "active");
        assert!(!session.id.is_empty());
    }

    #[test]
    fn create_session_custom_name() {
        let db = test_db();
        let session = db.create_session(Some("My App")).unwrap();
        assert_eq!(session.name, "My App");
    }

    #[test]
    fn get_sessions_returns_all() {
        let db = test_db();
        db.create_session(Some("First")).unwrap();
        db.create_session(Some("Second")).unwrap();

        let sessions = db.get_sessions().unwrap();
        assert_eq!(sessions.len(), 2);
        let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"First"));
        assert!(names.contains(&"Second"));
    }

    #[test]
    fn updated_session_moves_to_top() {
        let db = test_db();
        let s1 = db.create_session(Some("First")).unwrap();
        let _s2 = db.create_session(Some("Second")).unwrap();

        // Update s1 to bump its updated_at
        std::thread::sleep(std::time::Duration::from_millis(1100));
        db.update_session(&s1.id, Some("First Updated"), None)
            .unwrap();

        let sessions = db.get_sessions().unwrap();
        assert_eq!(sessions[0].id, s1.id);
    }

    #[test]
    fn update_session_name() {
        let db = test_db();
        let session = db.create_session(None).unwrap();
        let updated = db
            .update_session(&session.id, Some("Renamed"), None)
            .unwrap();
        assert_eq!(updated.name, "Renamed");
    }

    #[test]
    fn update_session_status() {
        let db = test_db();
        let session = db.create_session(None).unwrap();
        let updated = db
            .update_session(&session.id, None, Some("completed"))
            .unwrap();
        assert_eq!(updated.status, "completed");
    }

    #[test]
    fn delete_session() {
        let db = test_db();
        let session = db.create_session(None).unwrap();
        db.delete_session(&session.id).unwrap();
        assert!(db.get_session(&session.id).is_err());
    }

    #[test]
    fn delete_session_cascades_messages() {
        let db = test_db();
        let session = db.create_session(None).unwrap();
        db.save_message(&session.id, "user", "hello", None).unwrap();
        db.delete_session(&session.id).unwrap();
        let messages = db.get_messages(&session.id).unwrap();
        assert!(messages.is_empty());
    }

    // ---- Message Tests ----

    #[test]
    fn save_and_get_messages() {
        let db = test_db();
        let session = db.create_session(None).unwrap();

        db.save_message(&session.id, "user", "Hello", None).unwrap();
        db.save_message(&session.id, "assistant", "Hi there!", None)
            .unwrap();

        let messages = db.get_messages(&session.id).unwrap();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "Hello");
        assert_eq!(messages[1].role, "assistant");
    }

    #[test]
    fn save_message_with_metadata() {
        let db = test_db();
        let session = db.create_session(None).unwrap();
        let meta = r#"{"search_query":"react vs vue"}"#;
        let msg = db
            .save_message(&session.id, "assistant", "content", Some(meta))
            .unwrap();
        assert_eq!(msg.metadata.as_deref(), Some(meta));
    }

    #[test]
    fn message_count_only_user() {
        let db = test_db();
        let session = db.create_session(None).unwrap();

        db.save_message(&session.id, "user", "q1", None).unwrap();
        db.save_message(&session.id, "assistant", "a1", None)
            .unwrap();
        db.save_message(&session.id, "user", "q2", None).unwrap();

        let count = db.message_count(&session.id).unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn messages_isolated_per_session() {
        let db = test_db();
        let s1 = db.create_session(Some("S1")).unwrap();
        let s2 = db.create_session(Some("S2")).unwrap();

        db.save_message(&s1.id, "user", "msg for s1", None).unwrap();
        db.save_message(&s2.id, "user", "msg for s2", None).unwrap();

        assert_eq!(db.get_messages(&s1.id).unwrap().len(), 1);
        assert_eq!(db.get_messages(&s2.id).unwrap().len(), 1);
    }

    // ---- Document Tests ----

    #[test]
    fn save_and_get_documents() {
        let db = test_db();
        let session = db.create_session(None).unwrap();

        db.save_document(&session.id, "README.md", "# Hello")
            .unwrap();
        db.save_document(&session.id, "SPEC.md", "## Spec").unwrap();

        let docs = db.get_documents(&session.id).unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[0].filename, "README.md");
        assert_eq!(docs[0].content, "# Hello");
    }

    #[test]
    fn delete_documents() {
        let db = test_db();
        let session = db.create_session(None).unwrap();
        db.save_document(&session.id, "README.md", "content")
            .unwrap();
        db.delete_documents(&session.id).unwrap();
        assert!(db.get_documents(&session.id).unwrap().is_empty());
    }

    #[test]
    fn latest_times_for_staleness() {
        let db = test_db();
        let session = db.create_session(None).unwrap();

        // No messages or docs yet
        assert!(db.latest_message_time(&session.id).unwrap().is_none());
        assert!(db.latest_document_time(&session.id).unwrap().is_none());

        db.save_message(&session.id, "user", "hello", None).unwrap();
        assert!(db.latest_message_time(&session.id).unwrap().is_some());

        db.save_document(&session.id, "README.md", "content")
            .unwrap();
        assert!(db.latest_document_time(&session.id).unwrap().is_some());
    }

    #[test]
    fn database_is_ok() {
        let db = test_db();
        assert!(db.is_ok());
    }

    // ---- Preference Tests ----

    #[test]
    fn set_and_get_preference() {
        let db = test_db();
        db.set_preference("theme", "dark").unwrap();
        assert_eq!(db.get_preference("theme").unwrap(), Some("dark".to_string()));
    }

    #[test]
    fn get_missing_preference_returns_none() {
        let db = test_db();
        assert_eq!(db.get_preference("nonexistent").unwrap(), None);
    }

    #[test]
    fn overwrite_preference() {
        let db = test_db();
        db.set_preference("wizard_completed", "false").unwrap();
        db.set_preference("wizard_completed", "true").unwrap();
        assert_eq!(
            db.get_preference("wizard_completed").unwrap(),
            Some("true".to_string())
        );
    }
}
