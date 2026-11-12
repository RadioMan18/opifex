//! Run history: writing a finished run and reading it back.
//!
//! The engine keeps node runs in memory for the duration of a run; this module
//! is what makes them survive the run so the canvas can show what happened
//! later. Statuses are stored lowercase because that is what the UI speaks.

use opifex_core::store::{NodeRun, NodeRunStatus, Run, RunMode, RunStatus};
use opifex_core::AgentGraph;
use rusqlite::{params, Connection};
use serde::Serialize;
use serde_json::Value;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunSummary {
    pub id: String,
    pub agent_id: String,
    pub status: String,
    pub mode: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub node_count: i64,
    pub error: Option<Value>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunNodeRecord {
    pub node_id: String,
    pub status: String,
    pub output: Option<Value>,
    pub error: Option<Value>,
    pub tokens_in: Option<i64>,
    pub tokens_out: Option<i64>,
    pub started_at: Option<String>,
    pub ended_at: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RunDetail {
    pub summary: RunSummary,
    pub nodes: Vec<RunNodeRecord>,
}

fn run_status_key(status: &RunStatus) -> &'static str {
    match status {
        RunStatus::Running => "running",
        RunStatus::Succeeded => "succeeded",
        RunStatus::Failed => "failed",
        RunStatus::Cancelled => "cancelled",
    }
}

fn node_status_key(status: &NodeRunStatus) -> &'static str {
    match status {
        NodeRunStatus::Queued => "queued",
        NodeRunStatus::Running => "running",
        NodeRunStatus::Succeeded => "succeeded",
        NodeRunStatus::Failed => "failed",
        NodeRunStatus::Skipped => "skipped",
        NodeRunStatus::Cancelled => "cancelled",
    }
}

fn mode_key(mode: &RunMode) -> &'static str {
    match mode {
        RunMode::Live => "live",
        RunMode::Mock => "mock",
    }
}

fn parse_json(text: Option<String>) -> Option<Value> {
    text.and_then(|value| serde_json::from_str(&value).ok())
}

/// Stores the graph a run is about to execute.
///
/// runs.graph_id references agent_graphs, and this SQLite build enforces
/// foreign keys, so the graph row has to exist before its run does. Keeping the
/// row also means a historical run points at the exact graph it executed.
pub fn save_graph(conn: &Connection, agent_id: &str, graph: &AgentGraph) -> Result<(), String> {
    let graph_json = serde_json::to_string(graph).map_err(|error| error.to_string())?;

    conn.execute(
        // Upsert, not REPLACE: REPLACE deletes the row first, which fails once a
        // run references this graph.
        "INSERT INTO agent_graphs
            (id, agent_id, name, schema_version, graph_json, graph_hash, is_default)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 0)
         ON CONFLICT(id) DO UPDATE SET
            name = excluded.name,
            schema_version = excluded.schema_version,
            graph_json = excluded.graph_json,
            graph_hash = excluded.graph_hash",
        params![
            graph.id,
            agent_id,
            graph.name,
            graph.schema_version,
            graph_json,
            graph.compute_hash(),
        ],
    )
    .map_err(|error| error.to_string())?;

    Ok(())
}

/// Records a finished run and the state of each node in it.
pub fn save_run(
    conn: &Connection,
    agent_id: &str,
    run: &Run,
    node_runs: &[NodeRun],
) -> Result<(), String> {
    let transaction = conn
        .unchecked_transaction()
        .map_err(|error| error.to_string())?;

    transaction
        .execute(
            "INSERT INTO runs
                (id, agent_id, graph_id, graph_hash, mode, scope_json, trigger_json, status, error_json, started_at, ended_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                error_json = excluded.error_json,
                ended_at = excluded.ended_at",
            params![
                run.id,
                agent_id,
                run.graph_id,
                run.graph_hash,
                mode_key(&run.mode),
                run.scope_json.to_string(),
                run.trigger_json.as_ref().map(|value| value.to_string()),
                run_status_key(&run.status),
                run.error_json.as_ref().map(|value| value.to_string()),
                run.started_at,
                run.ended_at,
            ],
        )
        .map_err(|error| error.to_string())?;

    // Node rows are rewritten rather than upserted by id: their ids are fresh
    // per engine run, so a re-save would otherwise duplicate every node.
    transaction
        .execute("DELETE FROM node_runs WHERE run_id = ?1", params![run.id])
        .map_err(|error| error.to_string())?;

    for node_run in node_runs {
        transaction
            .execute(
                "INSERT INTO node_runs
                    (id, run_id, parent_node_run_id, node_path, attempt, status, input_json, output_json, error_json, truncated, tokens_in, tokens_out, started_at, ended_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
                 ON CONFLICT(id) DO UPDATE SET
                    status = excluded.status,
                    output_json = excluded.output_json,
                    error_json = excluded.error_json,
                    tokens_in = excluded.tokens_in,
                    tokens_out = excluded.tokens_out,
                    started_at = excluded.started_at,
                    ended_at = excluded.ended_at",
                params![
                    node_run.id,
                    node_run.run_id,
                    node_run.parent_node_run_id,
                    node_run.node_path,
                    node_run.attempt,
                    node_status_key(&node_run.status),
                    node_run.input_json.as_ref().map(|value| value.to_string()),
                    node_run.output_json.as_ref().map(|value| value.to_string()),
                    node_run.error_json.as_ref().map(|value| value.to_string()),
                    node_run.truncated as i32,
                    node_run.tokens_in.map(|value| value as i64),
                    node_run.tokens_out.map(|value| value as i64),
                    node_run.started_at,
                    node_run.ended_at,
                ],
            )
            .map_err(|error| error.to_string())?;
    }

    transaction.commit().map_err(|error| error.to_string())
}

/// The most recent runs for an agent, newest first.
pub fn list_runs(conn: &Connection, agent_id: &str, limit: i64) -> Result<Vec<RunSummary>, String> {
    let mut statement = conn
        .prepare(
            "SELECT r.id, r.agent_id, r.status, r.mode, r.started_at, r.ended_at, r.error_json,
                    (SELECT COUNT(*) FROM node_runs n WHERE n.run_id = r.id) AS node_count
             FROM runs r
             WHERE r.agent_id = ?1
             ORDER BY r.started_at DESC, r.rowid DESC
             LIMIT ?2",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map(params![agent_id, limit], |row| {
            Ok(RunSummary {
                id: row.get(0)?,
                agent_id: row.get(1)?,
                status: row.get(2)?,
                mode: row.get(3)?,
                started_at: row.get(4)?,
                ended_at: row.get(5)?,
                error: parse_json(row.get(6)?),
                node_count: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;

    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())
}

/// One run with every node it touched, in execution order.
pub fn load_run(conn: &Connection, run_id: &str) -> Result<Option<RunDetail>, String> {
    let summary = conn
        .query_row(
            "SELECT r.id, r.agent_id, r.status, r.mode, r.started_at, r.ended_at, r.error_json,
                    (SELECT COUNT(*) FROM node_runs n WHERE n.run_id = r.id) AS node_count
             FROM runs r WHERE r.id = ?1",
            params![run_id],
            |row| {
                Ok(RunSummary {
                    id: row.get(0)?,
                    agent_id: row.get(1)?,
                    status: row.get(2)?,
                    mode: row.get(3)?,
                    started_at: row.get(4)?,
                    ended_at: row.get(5)?,
                    error: parse_json(row.get(6)?),
                    node_count: row.get(7)?,
                })
            },
        )
        .ok();

    let Some(summary) = summary else {
        return Ok(None);
    };

    let mut statement = conn
        .prepare(
            "SELECT node_path, status, output_json, error_json, tokens_in, tokens_out, started_at, ended_at
             FROM node_runs WHERE run_id = ?1 ORDER BY rowid",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map(params![run_id], |row| {
            Ok(RunNodeRecord {
                node_id: row.get(0)?,
                status: row.get(1)?,
                output: parse_json(row.get(2)?),
                error: parse_json(row.get(3)?),
                tokens_in: row.get(4)?,
                tokens_out: row.get(5)?,
                started_at: row.get(6)?,
                ended_at: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let nodes = rows
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| error.to_string())?;

    Ok(Some(RunDetail { summary, nodes }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;
    use opifex_core::store::{NodeRun, Run, RunMode, RunStatus};

    fn memory_db() -> Connection {
        let conn = Connection::open_in_memory().expect("in-memory db");
        run_migrations(&conn).expect("schema");
        seed_agent(&conn);
        conn
    }

    /// runs.agent_id references agents, so the tests need a real agent row.
    fn seed_agent(conn: &Connection) {
        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter)
             VALUES ('agent-1', 'Test Agent', NULL, '{}', 'ollama', 0)",
            [],
        )
        .expect("agent row");
    }

    fn sample_graph() -> AgentGraph {
        AgentGraph {
            id: "graph-1".to_string(),
            name: "Graph".to_string(),
            description: None,
            schema_version: 1,
            nodes: vec![],
            edges: vec![],
            viewport: None,
            metadata: None,
        }
    }

    fn sample_run() -> Run {
        Run {
            id: "run-1".to_string(),
            graph_id: "graph-1".to_string(),
            graph_hash: "hash".to_string(),
            mode: RunMode::Live,
            scope_json: serde_json::json!({}),
            trigger_json: None,
            status: RunStatus::Failed,
            error_json: Some(serde_json::json!("Execution stopped at node n2: boom")),
            started_at: "2026-01-01T00:00:00Z".to_string(),
            ended_at: Some("2026-01-01T00:00:03Z".to_string()),
        }
    }

    fn sample_node_runs() -> Vec<NodeRun> {
        let mut first = NodeRun::new("run-1".to_string(), "n1".to_string());
        first.status = NodeRunStatus::Succeeded;
        first.output_json = Some(serde_json::json!({ "message": "hello" }));
        first.started_at = Some("2026-01-01T00:00:00Z".to_string());
        first.ended_at = Some("2026-01-01T00:00:01Z".to_string());

        let mut second = NodeRun::new("run-1".to_string(), "n2".to_string());
        second.status = NodeRunStatus::Failed;
        second.error_json = Some(serde_json::json!("boom"));
        second.tokens_in = Some(12);
        second.tokens_out = Some(3);

        vec![first, second]
    }

    #[test]
    fn saves_a_failed_run_with_its_node_outputs() {
        let conn = memory_db();
        save_graph(&conn, "agent-1", &sample_graph()).expect("graph");
        save_run(&conn, "agent-1", &sample_run(), &sample_node_runs()).expect("save");

        let runs = list_runs(&conn, "agent-1", 10).expect("list");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].status, "failed");
        assert_eq!(runs[0].mode, "live");
        assert_eq!(runs[0].node_count, 2);
        assert_eq!(
            runs[0].error.as_ref().and_then(|value| value.as_str()),
            Some("Execution stopped at node n2: boom")
        );

        let detail = load_run(&conn, "run-1").expect("load").expect("present");
        assert_eq!(detail.nodes.len(), 2);
        assert_eq!(detail.nodes[0].node_id, "n1");
        assert_eq!(detail.nodes[0].status, "succeeded");
        assert_eq!(
            detail.nodes[0]
                .output
                .as_ref()
                .and_then(|value| value.get("message"))
                .and_then(|value| value.as_str()),
            Some("hello")
        );
        assert_eq!(detail.nodes[1].status, "failed");
        assert_eq!(detail.nodes[1].tokens_in, Some(12));
        assert_eq!(
            detail.nodes[1].error.as_ref().and_then(|value| value.as_str()),
            Some("boom")
        );
    }

    #[test]
    fn lists_only_this_agents_runs_newest_first() {
        let conn = memory_db();

        save_graph(&conn, "agent-1", &sample_graph()).expect("graph");

        let mut older = sample_run();
        older.id = "run-0".to_string();
        older.started_at = "2025-12-31T00:00:00Z".to_string();
        save_run(&conn, "agent-1", &older, &[]).expect("save older");
        save_run(&conn, "agent-1", &sample_run(), &sample_node_runs()).expect("save newer");
        save_run(&conn, "agent-2", &sample_run(), &[]).expect("save other agent");

        let runs = list_runs(&conn, "agent-1", 10).expect("list");
        assert_eq!(runs.iter().map(|run| run.id.as_str()).collect::<Vec<_>>(), vec!["run-1", "run-0"]);

        let limited = list_runs(&conn, "agent-1", 1).expect("list limited");
        assert_eq!(limited.len(), 1);
        assert_eq!(limited[0].id, "run-1");
    }

    #[test]
    fn a_run_needs_its_graph_row_because_foreign_keys_are_enforced() {
        let conn = memory_db();
        // Without save_graph the FK from runs.graph_id to agent_graphs rejects
        // the write, which is why run_agent_graph stores the graph first.
        assert!(save_run(&conn, "agent-1", &sample_run(), &[]).is_err());

        save_graph(&conn, "agent-1", &sample_graph()).expect("graph");
        save_run(&conn, "agent-1", &sample_run(), &[]).expect("save");
    }

    #[test]
    fn a_second_run_on_the_same_graph_still_records() {
        let conn = memory_db();
        save_graph(&conn, "agent-1", &sample_graph()).expect("graph");
        save_run(&conn, "agent-1", &sample_run(), &sample_node_runs()).expect("first");

        // Saving the same graph again must not trip the foreign key from the
        // existing run, and recording the same run twice must stay an update.
        save_graph(&conn, "agent-1", &sample_graph()).expect("graph again");
        save_run(&conn, "agent-1", &sample_run(), &sample_node_runs()).expect("second");

        let runs = list_runs(&conn, "agent-1", 10).expect("list");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].node_count, 2);
    }

    #[test]
    fn unknown_runs_and_agents_read_as_empty() {
        let conn = memory_db();
        assert!(load_run(&conn, "nope").expect("load").is_none());
        assert!(list_runs(&conn, "nobody", 10).expect("list").is_empty());
    }
}
