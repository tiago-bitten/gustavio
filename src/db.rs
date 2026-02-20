use rusqlite::{Connection, params};
use std::path::PathBuf;

pub struct Database {
    conn: Connection,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct MessageRow {
    pub id: String,
    pub conversation_id: String,
    pub from_id: String,
    pub from_name: String,
    pub content: String,
    pub timestamp: String,
    pub is_group: bool,
    pub status: String,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GroupRow {
    pub group_id: String,
    pub name: String,
    pub creator_id: String,
}

impl Database {
    pub fn open() -> rusqlite::Result<Self> {
        let path = Self::db_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        let conn = Connection::open(&path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn db_path() -> PathBuf {
        #[cfg(target_os = "windows")]
        {
            let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
            PathBuf::from(base).join("gustavio").join("gustavio.db")
        }
        #[cfg(target_os = "macos")]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home)
                .join("Library")
                .join("Application Support")
                .join("gustavio")
                .join("gustavio.db")
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
            PathBuf::from(home)
                .join(".local")
                .join("share")
                .join("gustavio")
                .join("gustavio.db")
        }
    }

    fn init_schema(&self) -> rusqlite::Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS peers (
                peer_id   TEXT PRIMARY KEY,
                username  TEXT NOT NULL,
                last_ip   TEXT,
                last_seen TEXT
            );
            CREATE TABLE IF NOT EXISTS messages (
                id              TEXT PRIMARY KEY,
                conversation_id TEXT NOT NULL,
                from_id         TEXT NOT NULL,
                from_name       TEXT NOT NULL,
                content         TEXT NOT NULL,
                timestamp       TEXT NOT NULL,
                is_group        INTEGER NOT NULL DEFAULT 0,
                status          TEXT NOT NULL DEFAULT 'sent'
            );
            CREATE INDEX IF NOT EXISTS idx_messages_conv
                ON messages(conversation_id, timestamp);
            CREATE TABLE IF NOT EXISTS groups (
                group_id   TEXT PRIMARY KEY,
                name       TEXT NOT NULL,
                creator_id TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS group_members (
                group_id TEXT NOT NULL,
                peer_id  TEXT NOT NULL,
                PRIMARY KEY (group_id, peer_id)
            );
            ",
        )
    }

    // ── Config ───────────────────────────────────────────────

    pub fn get_config(&self, key: &str) -> Option<String> {
        self.conn
            .query_row(
                "SELECT value FROM config WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn set_config(&self, key: &str, value: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = ?2",
            params![key, value],
        )?;
        Ok(())
    }

    // ── Messages ─────────────────────────────────────────────

    pub fn insert_message(&self, msg: &MessageRow) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO messages
             (id, conversation_id, from_id, from_name, content, timestamp, is_group, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                msg.id,
                msg.conversation_id,
                msg.from_id,
                msg.from_name,
                msg.content,
                msg.timestamp,
                msg.is_group as i32,
                msg.status,
            ],
        )?;
        Ok(())
    }

    pub fn load_history(&self, conversation_id: &str, limit: i64) -> Vec<MessageRow> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, conversation_id, from_id, from_name, content, timestamp, is_group, status
                 FROM messages
                 WHERE conversation_id = ?1
                 ORDER BY timestamp ASC
                 LIMIT ?2",
            )
            .unwrap();
        stmt.query_map(params![conversation_id, limit], |row| {
            Ok(MessageRow {
                id: row.get(0)?,
                conversation_id: row.get(1)?,
                from_id: row.get(2)?,
                from_name: row.get(3)?,
                content: row.get(4)?,
                timestamp: row.get(5)?,
                is_group: row.get::<_, i32>(6)? != 0,
                status: row.get(7)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn update_message_status(&self, id: &str, status: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "UPDATE messages SET status = ?2 WHERE id = ?1",
            params![id, status],
        )?;
        Ok(())
    }

    // ── Peers ────────────────────────────────────────────────

    pub fn upsert_peer(&self, peer_id: &str, username: &str, ip: &str) -> rusqlite::Result<()> {
        let now = chrono::Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO peers (peer_id, username, last_ip, last_seen) VALUES (?1, ?2, ?3, ?4)
             ON CONFLICT(peer_id) DO UPDATE SET username=?2, last_ip=?3, last_seen=?4",
            params![peer_id, username, ip, now],
        )?;
        Ok(())
    }

    // ── Groups ───────────────────────────────────────────────

    pub fn create_group(&self, group_id: &str, name: &str, creator_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO groups (group_id, name, creator_id) VALUES (?1, ?2, ?3)",
            params![group_id, name, creator_id],
        )?;
        Ok(())
    }

    pub fn add_group_member(&self, group_id: &str, peer_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO group_members (group_id, peer_id) VALUES (?1, ?2)",
            params![group_id, peer_id],
        )?;
        Ok(())
    }

    pub fn remove_group_member(&self, group_id: &str, peer_id: &str) -> rusqlite::Result<()> {
        self.conn.execute(
            "DELETE FROM group_members WHERE group_id = ?1 AND peer_id = ?2",
            params![group_id, peer_id],
        )?;
        Ok(())
    }

    pub fn get_groups(&self) -> Vec<GroupRow> {
        let mut stmt = self
            .conn
            .prepare("SELECT group_id, name, creator_id FROM groups")
            .unwrap();
        stmt.query_map([], |row| {
            Ok(GroupRow {
                group_id: row.get(0)?,
                name: row.get(1)?,
                creator_id: row.get(2)?,
            })
        })
        .unwrap()
        .filter_map(|r| r.ok())
        .collect()
    }

    pub fn get_group_members(&self, group_id: &str) -> Vec<String> {
        let mut stmt = self
            .conn
            .prepare("SELECT peer_id FROM group_members WHERE group_id = ?1")
            .unwrap();
        stmt.query_map(params![group_id], |row| row.get(0))
            .unwrap()
            .filter_map(|r| r.ok())
            .collect()
    }
}
