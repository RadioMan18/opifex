//! Database migrations for Opifex v2.0.0.
//! All migrations are additive — no destructive changes.
//! Applied via `run_migrations` in the Tauri `db.rs`.

use rusqlite::Connection;

/// Run all additive migrations for v2.0.0.
/// Safe to call repeatedly — each statement uses IF NOT EXISTS / ALTER ... ADD COLUMN IF NOT EXISTS pattern.
pub fn run_v2_migrations(conn: &Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- Block definitions (reusable block templates, incl. user-authored)
        CREATE TABLE IF NOT EXISTS block_definitions (
            id TEXT NOT NULL,
            version INTEGER NOT NULL,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT NOT NULL,
            kind TEXT NOT NULL CHECK (kind IN ('trigger','llm','agent','tool','control','output','utility','subgraph')),
            origin TEXT NOT NULL CHECK (origin IN ('user','imported')),
            spec_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (id, version)
        );

        -- Agent block graphs (canonical agent form, replaces/augments spec_json)
        CREATE TABLE IF NOT EXISTS agent_graphs (
            id TEXT PRIMARY KEY,
            agent_id TEXT NOT NULL REFERENCES agents(id),
            name TEXT NOT NULL,
            schema_version INTEGER NOT NULL,
            graph_json TEXT NOT NULL,
            graph_hash TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Graph snapshots (auto/manual/pre-migration/pre-ai-edit)
        CREATE TABLE IF NOT EXISTS graph_snapshots (
            id TEXT PRIMARY KEY,
            graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
            kind TEXT NOT NULL CHECK (kind IN ('auto','manual','pre-migration','pre-ai-edit')),
            label TEXT,
            graph_json TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Validation runs (audit trail of logic checks)
        CREATE TABLE IF NOT EXISTS validation_runs (
            id TEXT PRIMARY KEY,
            graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
            graph_hash TEXT NOT NULL,
            status TEXT NOT NULL,
            error_count INTEGER NOT NULL DEFAULT 0,
            warning_count INTEGER NOT NULL DEFAULT 0,
            errors_json TEXT NOT NULL,
            duration_ms INTEGER NOT NULL,
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        -- Runs (per-node execution history)
        CREATE TABLE IF NOT EXISTS runs (
            id TEXT PRIMARY KEY,
            graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
            graph_hash TEXT NOT NULL,
            mode TEXT NOT NULL CHECK (mode IN ('live','mock')),
            scope_json TEXT NOT NULL,
            trigger_json TEXT,
            status TEXT NOT NULL CHECK (status IN ('running','succeeded','failed','cancelled')),
            error_json TEXT,
            started_at TEXT NOT NULL,
            ended_at TEXT
        );

        -- Node runs (individual node execution within a run)
        CREATE TABLE IF NOT EXISTS node_runs (
            id TEXT PRIMARY KEY,
            run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
            parent_node_run_id TEXT,
            node_path TEXT NOT NULL,
            attempt INTEGER NOT NULL DEFAULT 1,
            status TEXT NOT NULL CHECK (status IN ('queued','running','succeeded','failed','skipped','cancelled')),
            input_json TEXT,
            output_json TEXT,
            error_json TEXT,
            truncated INTEGER NOT NULL DEFAULT 0,
            tokens_in INTEGER,
            tokens_out INTEGER,
            started_at TEXT,
            ended_at TEXT
        );

        -- Pinned data (user-pinned node outputs for iterative development)
        CREATE TABLE IF NOT EXISTS pinned_data (
            graph_id TEXT NOT NULL REFERENCES agent_graphs(id) ON DELETE CASCADE,
            node_id TEXT NOT NULL,
            port_id TEXT NOT NULL,
            value_json TEXT NOT NULL,
            source_node_run_id TEXT,
            created_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (graph_id, node_id, port_id)
        );

        -- Agent grants (capability grants, e.g. network access)
        CREATE TABLE IF NOT EXISTS agent_grants (
            agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
            kind TEXT NOT NULL CHECK (kind IN ('network')),
            value TEXT NOT NULL,
            granted_at TEXT NOT NULL DEFAULT (datetime('now')),
            PRIMARY KEY (agent_id, kind, value)
        );

        -- Model capabilities (probed tier per provider/model)
        CREATE TABLE IF NOT EXISTS model_capabilities (
            provider TEXT NOT NULL,
            model TEXT NOT NULL,
            probe_json TEXT NOT NULL,
            tier INTEGER NOT NULL,
            probed_at TEXT NOT NULL,
            PRIMARY KEY (provider, model)
        );

        -- Add default_graph_id to agents (additive, nullable)
        "#,
    )?;

    // Add default_graph_id only if not already present (SQLite lacks ADD COLUMN IF NOT EXISTS)
    {
        let has_column: bool = conn
            .prepare("SELECT 1 FROM pragma_table_info('agents') WHERE name='default_graph_id'")
            .and_then(|mut stmt| {
                stmt.query_row([], |row| row.get::<_, i64>(0).map(|c: i64| c > 0))
            })
            .unwrap_or(false);
        if !has_column {
            conn.execute("ALTER TABLE agents ADD COLUMN default_graph_id TEXT", [])?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_v2_migrations_are_additive() {
        let conn = Connection::open_in_memory().unwrap();
        // First create base v1 tables
        conn.execute_batch(
            r#"
            CREATE TABLE agents (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                spec_json TEXT NOT NULL,
                model TEXT NOT NULL DEFAULT 'ollama',
                is_starter INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            "#,
        )
        .unwrap();

        // Run v2 migrations
        run_v2_migrations(&conn).unwrap();

        // Verify all v2 tables exist
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        let expected = [
            "agent_graphs",
            "agent_grants",
            "block_definitions",
            "graph_snapshots",
            "model_capabilities",
            "node_runs",
            "pinned_data",
            "runs",
            "validation_runs",
        ];
        for expected_table in &expected {
            assert!(
                tables.contains(&expected_table.to_string()),
                "missing table: {}",
                expected_table
            );
        }

        // Verify agents has default_graph_id column
        let columns: Vec<String> = conn
            .prepare("PRAGMA table_info(agents)")
            .unwrap()
            .query_map([], |row| row.get(1))
            .unwrap()
            .map(|r| r.unwrap())
            .collect();
        assert!(
            columns.contains(&"default_graph_id".to_string()),
            "agents table missing default_graph_id column"
        );
    }

    #[test]
    fn test_v2_migrations_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE agents (id TEXT PRIMARY KEY, name TEXT NOT NULL, spec_json TEXT NOT NULL);",
        )
        .unwrap();

        // Run twice — should not error
        run_v2_migrations(&conn).unwrap();
        run_v2_migrations(&conn).unwrap();
    }
}
