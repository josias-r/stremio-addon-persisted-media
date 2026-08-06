use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use log::{info, error};

#[derive(Clone)]
pub struct DbClient {
    conn: Arc<Mutex<Connection>>,
}

impl DbClient {
    pub fn new(db_path: &str) -> Self {
        let conn = Connection::open(db_path).expect("Failed to open SQLite database");
        
        let client = Self {
            conn: Arc::new(Mutex::new(conn)),
        };

        client.init_db().expect("Failed to initialize database tables");
        client
    }

    fn init_db(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();

        // Create users table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL DEFAULT 'unknown',
                api_key TEXT UNIQUE NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )",
            [],
        )?;
        // Best-effort schema migration for existing early-dev dbs
        let _ = conn.execute("ALTER TABLE users ADD COLUMN username TEXT NOT NULL DEFAULT 'unknown'", []);

        // Create settings table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;

        // Create watch_history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS watch_history (
                api_key TEXT NOT NULL,
                item_id TEXT NOT NULL,
                item_type TEXT NOT NULL,
                last_watched_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                PRIMARY KEY (api_key, item_id)
            )",
            [],
        )?;

        Ok(())
    }

    pub fn get_admin_password(&self) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT value FROM settings WHERE key = 'admin_password'")?;
        
        // We have to collect because mapping borrows stmt
        let mut rows = stmt.query([])?;
        if let Some(row) = rows.next()? {
            Ok(row.get(0)?)
        } else {
            // Drop stmt before executing another query
            drop(rows);
            drop(stmt);
            
            // Generate a random default password if none exists
            let password = Uuid::new_v4().to_string().replace("-", "")[..12].to_string();
            conn.execute(
                "INSERT INTO settings (key, value) VALUES ('admin_password', ?)",
                params![&password],
            )?;
            info!("Generated default admin password: {}", password);
            Ok(password)
        }
    }

    pub fn create_user(&self, username: &str) -> Result<String> {
        let conn = self.conn.lock().unwrap();
        let api_key = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO users (username, api_key) VALUES (?, ?)",
            params![username, &api_key],
        )?;
        Ok(api_key)
    }

    pub fn delete_user(&self, api_key: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM users WHERE api_key = ?",
            params![api_key],
        )?;
        Ok(())
    }

    pub fn validate_api_key(&self, api_key: &str) -> bool {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare("SELECT 1 FROM users WHERE api_key = ?") {
            Ok(s) => s,
            Err(e) => {
                error!("Database error preparing validate query: {}", e);
                return false;
            }
        };
        
        stmt.exists(params![api_key]).unwrap_or(false)
    }

    pub fn update_watch_history(&self, api_key: &str, item_id: &str, item_type: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO watch_history (api_key, item_id, item_type, last_watched_at) 
             VALUES (?, ?, ?, CURRENT_TIMESTAMP)
             ON CONFLICT(api_key, item_id) 
             DO UPDATE SET last_watched_at = CURRENT_TIMESTAMP",
            params![api_key, item_id, item_type],
        )?;
        Ok(())
    }

    pub fn get_torrent_watch_times(&self) -> Result<std::collections::HashMap<String, String>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT item_id, MAX(last_watched_at) FROM watch_history GROUP BY item_id")?;
        let history = stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        let mut map = std::collections::HashMap::new();
        for item in history {
            let (id, time) = item?;
            map.insert(id.to_lowercase(), time);
        }
        Ok(map)
    }

    pub fn delete_watch_history(&self, item_id: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM watch_history WHERE item_id = ?", params![item_id])?;
        Ok(())
    }

    pub fn get_users(&self) -> Result<Vec<(String, String, String)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare("SELECT username, api_key, created_at FROM users ORDER BY created_at DESC")?;
        let users = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;

        let mut results = Vec::new();
        for u in users {
            results.push(u?);
        }
        Ok(results)
    }
}
