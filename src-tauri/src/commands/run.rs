//! Run and pin management commands for Opifex v2.0.0.
//! Phase 3: Execution engine integration with Tauri.
//!
//! These commands allow the frontend to:
//! - Run agent graphs (execute nodes)
//! - Pin/unpin node outputs for iterative development
//! - Query run status and pinned data

use crate::db::DbState;
use opifex_core::{
    store::{NodeExecutionInput, NodeExecutionOutput, NodeRun, Run, RunMode, RunStatus},
    AgentGraph, BlockKind, BlockPort, NodeInstance,
};
use opifex_engine::ExecutionEngine;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

/// Request to run an agent graph.
#[derive(Deserialize)]
pub struct RunAgentGraphInput {
    pub agent_id: String,
    pub mode: RunMode,
}

/// Request to pin node output data.
#[derive(Deserialize)]
pub struct PinDataInput {
    pub agent_id: String,
    pub node_id: String,
    pub port_id: String,
    pub value: Value,
}

/// Request to unpin node output data.
#[derive(Deserialize)]
pub struct UnpinDataInput {
    pub agent_id: String,
    pub node_id: String,
    pub port_id: String,
}

/// Request to get pinned data for an agent/graph.
#[derive(Deserialize)]
pub struct GetPinnedDataInput {
    pub agent_id: String,
}

/// Response for pinned data.
#[derive(Serialize)]
pub struct PinnedDataDto {
    pub node_id: String,
    pub port_id: String,
    pub value: Value,
    pub created_at: String,
}

/// Run status response.
#[derive(Serialize)]
pub struct RunStatusDto {
    pub run_id: String,
    pub agent_id: String,
    pub status: RunStatus,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub node_count: usize,
}

/// Run a full agent graph.
#[tauri::command]
pub async fn run_agent_graph(
    db: State<'_, DbState>,
    input: RunAgentGraphInput,
) -> Result<Run, String> {
    // Load the agent from SQLite (hold lock only briefly)
    let spec_json = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT spec_json FROM agents WHERE id = ?1",
            params![input.agent_id],
            |row| row.get::<_, String>(2),
        )
        .map_err(|e| format!("Agent not found: {}", e))?
    };

    // Parse the graph from spec_json
    let graph: AgentGraph = serde_json::from_str(&spec_json)
        .map_err(|e| format!("Failed to parse agent graph: {}", e))?;

    // Create execution engine and run
    let engine = ExecutionEngine::new(graph);
    let run = engine.run_graph(input.mode).await
        .map_err(|e| format!("Execution failed: {}", e))?;

    Ok(run)
}

/// Pin a node output value for iterative development.
#[tauri::command]
pub async fn pin_agent_data(
    db: State<'_, DbState>,
    input: PinDataInput,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO pinned_data (graph_id, node_id, port_id, value_json, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            input.agent_id,
            input.node_id,
            input.port_id,
            serde_json::to_string(&input.value).map_err(|e| e.to_string())?,
            now
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Unpin a node output value.
#[tauri::command]
pub async fn unpin_agent_data(
    db: State<'_, DbState>,
    input: UnpinDataInput,
) -> Result<(), String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    conn.execute(
        "DELETE FROM pinned_data WHERE graph_id = ?1 AND node_id = ?2 AND port_id = ?3",
        params![input.agent_id, input.node_id, input.port_id],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

/// Get all pinned data for an agent/graph.
#[tauri::command]
pub async fn get_pinned_data(
    db: State<'_, DbState>,
    input: GetPinnedDataInput,
) -> Result<Vec<PinnedDataDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT node_id, port_id, value_json, created_at FROM pinned_data WHERE graph_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(PinnedDataDto {
            node_id: row.get(0)?,
            port_id: row.get(1)?,
            value: serde_json::from_str(&row.get::<_, String>(2)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            created_at: row.get(3)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

/// Get the current run status for an agent.
#[tauri::command]
pub async fn get_run_status(
    db: State<'_, DbState>,
    input: GetPinnedDataInput, // reuse the agent_id-only input
) -> Result<Option<RunStatusDto>, String> {
    // For now, we don't persist runs to SQLite - they're in-memory in the engine.
    // In a future phase, we'll persist runs to the `runs` table.
    // For now, return None since there's no active run tracking across Tauri invocations.
    // The frontend can track the current run state itself.
    Ok(None)
}
