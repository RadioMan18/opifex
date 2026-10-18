use rusqlite::Connection;
use std::path::Path;
use std::sync::Mutex;

pub struct DbState {
    pub conn: Mutex<Connection>,
}

pub fn init_db(db_path: &Path) -> Result<DbState, Box<dyn std::error::Error>> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let conn = Connection::open(db_path)?;
    run_migrations(&conn)?;
    Ok(DbState { conn: Mutex::new(conn) })
}

pub fn run_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS agents (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            spec_json TEXT NOT NULL,
            model TEXT NOT NULL DEFAULT 'ollama',
            is_starter INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            title TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        );

        CREATE TABLE IF NOT EXISTS messages (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            role TEXT NOT NULL,
            content TEXT NOT NULL,
            turn_plan_json TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (conversation_id) REFERENCES conversations(id)
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS plan_versions (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            label TEXT,
            spec_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        );
        "#,
    )?;
    Ok(())
}
