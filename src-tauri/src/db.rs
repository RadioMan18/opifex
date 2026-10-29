use opifex_core::run_v2_migrations;
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

        CREATE TABLE IF NOT EXISTS template_sources (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1,
            added_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS installed_templates (
            id TEXT PRIMARY KEY,
            template_id TEXT NOT NULL,
            version TEXT NOT NULL,
            installed_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS subagents (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            task TEXT NOT NULL,
            max_concurrent INTEGER NOT NULL DEFAULT 10,
            wait_for_results INTEGER NOT NULL DEFAULT 1,
            depth_limit INTEGER NOT NULL DEFAULT 1,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        );

        CREATE TABLE IF NOT EXISTS channels (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            kind TEXT NOT NULL,
            config TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        );

        CREATE TABLE IF NOT EXISTS instruction_changes (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            change_type TEXT NOT NULL,
            rationale TEXT NOT NULL,
            approval_required INTEGER NOT NULL DEFAULT 1,
            applies_from TEXT NOT NULL,
            status TEXT NOT NULL DEFAULT 'pending',
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        );

        CREATE TABLE IF NOT EXISTS evaluations (
            id TEXT PRIMARY KEY,
            graph_id TEXT NOT NULL,
            run_id TEXT NOT NULL,
            evaluator TEXT NOT NULL,
            score REAL NOT NULL,
            criteria TEXT NOT NULL,
            notes TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (graph_id) REFERENCES agents(id),
            FOREIGN KEY (run_id) REFERENCES agent_graphs(id)
        );

        CREATE TABLE IF NOT EXISTS reflections (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            lessons_learned TEXT NOT NULL,
            improvements TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (run_id) REFERENCES runs(id)
        );

        CREATE TABLE IF NOT EXISTS artifacts (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            node_id TEXT,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            storage_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (run_id) REFERENCES agent_graphs(id),
            FOREIGN KEY (node_id) REFERENCES agent_graphs(id)
        );

        CREATE TABLE IF NOT EXISTS agent_settings (
            agent_id TEXT NOT NULL,
            key TEXT NOT NULL,
            value TEXT NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (agent_id, key),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        );

        CREATE TABLE IF NOT EXISTS setup_steps (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            name TEXT NOT NULL,
            description TEXT NOT NULL,
            kind TEXT NOT NULL,
            completed INTEGER NOT NULL DEFAULT 0,
            required INTEGER NOT NULL DEFAULT 0,
            order INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (agent_id) REFERENCES agents(id)
        );

        CREATE TABLE IF NOT EXISTS runs (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL,
            graph_id TEXT NOT NULL,
            graph_hash TEXT NOT NULL,
            mode TEXT NOT NULL DEFAULT 'mock',
            scope_json TEXT NOT NULL,
            trigger_json TEXT,
            status TEXT NOT NULL DEFAULT 'running',
            error_json TEXT,
            started_at TEXT NOT NULL DEFAULT (datetime('now')),
            ended_at TEXT,
            FOREIGN KEY (agent_id) REFERENCES agents(id),
            FOREIGN KEY (graph_id) REFERENCES agent_graphs(id)
        );

        CREATE TABLE IF NOT EXISTS node_runs (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            parent_node_run_id TEXT,
            node_path TEXT NOT NULL,
            attempt INTEGER NOT NULL DEFAULT 1,
            status TEXT NOT NULL DEFAULT 'queued',
            input_json TEXT,
            output_json TEXT,
            error_json TEXT,
            truncated INTEGER NOT NULL DEFAULT 0,
            tokens_in INTEGER,
            tokens_out INTEGER,
            started_at TEXT,
            ended_at TEXT,
            FOREIGN KEY (run_id) REFERENCES runs(id),
            FOREIGN KEY (parent_node_run_id) REFERENCES node_runs(id)
        );

        CREATE TABLE IF NOT EXISTS reflections (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            summary TEXT NOT NULL,
            lessons_learned TEXT NOT NULL,
            improvements TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (run_id) REFERENCES runs(id)
        );

        CREATE TABLE IF NOT EXISTS artifacts (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL,
            node_id TEXT,
            name TEXT NOT NULL,
            kind TEXT NOT NULL,
            mime_type TEXT NOT NULL,
            size_bytes INTEGER NOT NULL,
            storage_path TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            FOREIGN KEY (run_id) REFERENCES runs(id),
            FOREIGN KEY (node_id) REFERENCES agent_graphs(id)
        );
        "#,
    )?;

    run_v2_migrations(&conn)?;

    Ok(())
}
