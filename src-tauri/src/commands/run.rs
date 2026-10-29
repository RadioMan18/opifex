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

// ============================================================================
// Skill commands (Phase 4)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateSkillInput {
    pub agent_id: String,
    pub name: String,
    pub description: Option<String>,
    pub kind: String, // "create-with-ai" | "upload-files" | "write-instructions" | "add-existing"
    pub ai_editing_enabled: bool,
    pub instructions: String,
}

#[derive(Serialize)]
pub struct SkillDto {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub kind: String,
    pub ai_editing_enabled: bool,
    pub instructions: String,
    pub attached_to: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn create_skill(
    db: State<'_, DbState>,
    input: CreateSkillInput,
) -> Result<SkillDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO skills (id, agent_id, name, description, kind, ai_editing_enabled, instructions, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            input.agent_id,
            input.name,
            input.description,
            input.kind,
            input.ai_editing_enabled,
            input.instructions,
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(SkillDto {
        id,
        name: input.name,
        description: input.description,
        kind: input.kind,
        ai_editing_enabled: input.ai_editing_enabled,
        instructions: input.instructions,
        attached_to: vec![],
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListSkillsInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_skills(
    db: State<'_, DbState>,
    input: ListSkillsInput,
) -> Result<Vec<SkillDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, name, description, kind, ai_editing_enabled, instructions, created_at, updated_at FROM skills WHERE agent_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(SkillDto {
            id: row.get(0)?,
            name: row.get(1)?,
            description: row.get(2)?,
            kind: row.get(3)?,
            ai_editing_enabled: row.get(4)?,
            instructions: row.get(5)?,
            attached_to: vec![],
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Knowledge Source commands (Phase 4)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateKnowledgeSourceInput {
    pub agent_id: String,
    pub name: String,
    pub provider: String,
    pub source_ids: Vec<String>,
    pub citations_enabled: bool,
}

#[derive(Serialize)]
pub struct KnowledgeSourceDto {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub source_ids: Vec<String>,
    pub citations_enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn create_knowledge_source(
    db: State<'_, DbState>,
    input: CreateKnowledgeSourceInput,
) -> Result<KnowledgeSourceDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO knowledge_sources (id, agent_id, name, provider, source_ids, citations_enabled, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.agent_id,
            input.name,
            input.provider,
            serde_json::to_string(&input.source_ids).map_err(|e| e.to_string())?,
            input.citations_enabled,
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(KnowledgeSourceDto {
        id,
        name: input.name,
        provider: input.provider,
        source_ids: input.source_ids,
        citations_enabled: input.citations_enabled,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[derive(Deserialize)]
pub struct ListKnowledgeSourcesInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_knowledge_sources(
    db: State<'_, DbState>,
    input: ListKnowledgeSourcesInput,
) -> Result<Vec<KnowledgeSourceDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, name, provider, source_ids, citations_enabled, created_at, updated_at FROM knowledge_sources WHERE agent_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(KnowledgeSourceDto {
            id: row.get(0)?,
            name: row.get(1)?,
            provider: row.get(2)?,
            source_ids: serde_json::from_str(&row.get::<_, String>(3)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            citations_enabled: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Trigger commands (Phase 4)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateTriggerInput {
    pub agent_id: String,
    pub kind: String, // "app-trigger" | "webhook" | "scheduled" | "one-time" | "create-with-ai"
    pub config: Value,
    pub ai_managed: bool,
}

#[derive(Serialize)]
pub struct TriggerDto {
    pub id: String,
    pub kind: String,
    pub config: Value,
    pub ai_managed: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn create_trigger(
    db: State<'_, DbState>,
    input: CreateTriggerInput,
) -> Result<TriggerDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO triggers (id, agent_id, kind, config, ai_managed, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            input.agent_id,
            input.kind,
            serde_json::to_string(&input.config).map_err(|e| e.to_string())?,
            input.ai_managed,
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(TriggerDto {
        id,
        kind: input.kind,
        config: input.config,
        ai_managed: input.ai_managed,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[derive(Deserialize)]
pub struct ListTriggersInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_triggers(
    db: State<'_, DbState>,
    input: ListTriggersInput,
) -> Result<Vec<TriggerDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, kind, config, ai_managed, created_at, updated_at FROM triggers WHERE agent_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(TriggerDto {
            id: row.get(0)?,
            kind: row.get(1)?,
            config: serde_json::from_str(&row.get::<_, String>(2)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
        ai_managed: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Subagent commands (Phase 5)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateSubagentInput {
    pub agent_id: String,
    pub task: String,
    pub max_concurrent: u32,
    pub wait_for_results: bool,
    pub depth_limit: u32,
}

#[derive(Serialize)]
pub struct SubagentDto {
    pub id: String,
    pub agent_id: String,
    pub task: String,
    pub max_concurrent: u32,
    pub wait_for_results: bool,
    pub depth_limit: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn create_subagent(
    db: State<'_, DbState>,
    input: CreateSubagentInput,
) -> Result<SubagentDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO subagents (id, agent_id, task, max_concurrent, wait_for_results, depth_limit, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.agent_id,
            input.task,
            input.max_concurrent,
            input.wait_for_results,
            input.depth_limit,
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(SubagentDto {
        id,
        agent_id: input.agent_id,
        task: input.task,
        max_concurrent: input.max_concurrent,
        wait_for_results: input.wait_for_results,
        depth_limit: input.depth_limit,
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListSubagentsInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_subagents(
    db: State<'_, DbState>,
    input: ListSubagentsInput,
) -> Result<Vec<SubagentDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, task, max_concurrent, wait_for_results, depth_limit, created_at, updated_at FROM subagents WHERE agent_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(SubagentDto {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            task: row.get(2)?,
            max_concurrent: row.get(3)?,
            wait_for_results: row.get(4)?,
            depth_limit: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Ability commands (Phase 5)
// ============================================================================

#[derive(Deserialize)]
pub struct ListAbilitiesInput {
    pub agent_id: String,
}

#[derive(Serialize)]
pub struct AbilityDto {
    pub kind: String,
    pub enabled: bool,
}

#[tauri::command]
pub async fn list_abilities(
    db: State<'_, DbState>,
    input: ListAbilitiesInput,
) -> Result<Vec<AbilityDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let abilities = vec![
        ("web-search", true),
        ("web-fetch", true),
        ("image-generation", true),
        ("chat-widgets", true),
        ("search-past-conversations", true),
        ("ask-question", true),
        ("tool-discovery", true),
        ("rules-creation", true),
    ];

    Ok(abilities.into_iter().map(|(kind, enabled)| AbilityDto {
        kind: kind.to_string(),
        enabled,
    }).collect())
}

#[tauri::command]
pub async fn toggle_ability(
    agent_id: String,
    ability_id: String,
    enabled: bool,
) -> Result<(), String> {
    Ok(())
}

// ============================================================================
// Channel commands (Phase 5)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateChannelInput {
    pub agent_id: String,
    pub kind: String,
    pub config: serde_json::Value,
}

#[derive(Serialize)]
pub struct ChannelDto {
    pub id: String,
    pub kind: String,
    pub config: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn create_channel(
    db: State<'_, DbState>,
    input: CreateChannelInput,
) -> Result<ChannelDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO channels (id, agent_id, kind, config, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.agent_id,
            input.kind,
            serde_json::to_string(&input.config).map_err(|e| e.to_string())?,
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(ChannelDto {
        id,
        kind: input.kind,
        config: input.config,
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListChannelsInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_channels(
    db: State<'_, DbState>,
    input: ListChannelsInput,
) -> Result<Vec<ChannelDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, kind, config, created_at, updated_at FROM channels WHERE agent_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(ChannelDto {
            id: row.get(0)?,
            kind: row.get(1)?,
            config: serde_json::from_str(&row.get::<_, String>(2)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            created_at: row.get(3)?,
            updated_at: row.get(4)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}
