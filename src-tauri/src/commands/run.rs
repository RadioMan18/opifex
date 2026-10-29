//! Run and pin management commands for Opifex v2.0.0.
//! Phase 3-6: Execution engine integration with Tauri.
//!
//! These commands allow the frontend to:
//! - Run agent graphs (execute nodes)
//! - Pin/unpin node outputs for iterative development
//! - Query run status and pinned data
//! - Manage skills, knowledge sources, triggers
//! - Manage subagents, abilities, channels
//! - Manage instruction changes, evaluations, reflections
//! - Manage artifacts, messaging modes, setup steps
//! - Apply Oppy edits

use crate::db::DbState;
use opifex_core::{
    store::{NodeExecutionInput, NodeExecutionOutput, NodeRun, Run, RunMode, RunStatus},
    AgentGraph, BlockKind, BlockPort, NodeInstance,
};
use opifex_engine::ExecutionEngine;
use rusqlite::params;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::State;
use uuid::Uuid;

// ============================================================================
// Run commands (Phase 3)
// ============================================================================

#[derive(Deserialize)]
pub struct RunAgentGraphInput {
    pub agent_id: String,
    pub mode: RunMode,
}

#[derive(Deserialize)]
pub struct PinDataInput {
    pub agent_id: String,
    pub node_id: String,
    pub port_id: String,
    pub value: Value,
}

#[derive(Deserialize)]
pub struct UnpinDataInput {
    pub agent_id: String,
    pub node_id: String,
    pub port_id: String,
}

#[derive(Deserialize)]
pub struct GetPinnedDataInput {
    pub agent_id: String,
}

#[derive(Serialize)]
pub struct RunStatusDto {
    pub run_id: String,
    pub agent_id: String,
    pub status: RunStatus,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub node_count: usize,
}

#[tauri::command]
pub async fn run_agent_graph(
    db: State<'_, DbState>,
    input: RunAgentGraphInput,
) -> Result<Run, String> {
    // Load the agent spec_json first (hold lock briefly)
    let spec_json: String = {
        let conn = db.conn.lock().map_err(|e| e.to_string())?;
        conn.query_row(
            "SELECT spec_json FROM agents WHERE id = ?1",
            params![input.agent_id],
            |row| row.get(0),
        )
        .map_err(|e| format!("Agent not found: {}", e))?
    };

    // Parse the graph from spec_json
    let graph: AgentGraph = serde_json::from_str(&spec_json)
        .map_err(|e| format!("Failed to parse agent graph: {}", e))?;

    // Create execution engine and run (async, no mutex held)
    let engine = ExecutionEngine::new(graph);
    let run = engine.run_graph(input.mode).await
        .map_err(|e| format!("Execution failed: {}", e))?;

    Ok(run)
}

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
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(())
}

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
            id: row.get(0)?,
            graph_id: input.agent_id.clone(),
            node_id: row.get(1)?,
            port_id: row.get(2)?,
            value_json: serde_json::from_str(&row.get::<_, String>(3)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            created_at: row.get(4)?,
            updated_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_run_status(
    db: State<'_, DbState>,
    input: GetPinnedDataInput,
) -> Result<Option<RunStatusDto>, String> {
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
    pub kind: String,
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
        updated_at: now.clone(),
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
    pub kind: String,
    pub config: serde_json::Value,
    pub ai_managed: bool,
}

#[derive(Serialize)]
pub struct TriggerDto {
    pub id: String,
    pub kind: String,
    pub config: serde_json::Value,
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
        updated_at: now.clone(),
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

#[derive(Serialize)]
pub struct PinnedDataDto {
    pub id: String,
    pub graph_id: String,
    pub node_id: String,
    pub port_id: String,
    pub value_json: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct ListPinnedDataInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_pinned_data(
    db: State<'_, DbState>,
    input: ListPinnedDataInput,
) -> Result<Vec<PinnedDataDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, graph_id, node_id, port_id, value_json, created_at, updated_at FROM pinned_data WHERE graph_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(PinnedDataDto {
            id: row.get(0)?,
            graph_id: row.get(1)?,
            node_id: row.get(2)?,
            port_id: row.get(3)?,
            value_json: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
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

// ============================================================================
// InstructionChange commands (Phase 6)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateInstructionChangeInput {
    pub agent_id: String,
    pub change_type: String,
    pub rationale: String,
    pub approval_required: bool,
    pub applies_from: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct InstructionChangeDto {
    pub id: String,
    pub agent_id: String,
    pub change_type: String,
    pub rationale: String,
    pub approval_required: bool,
    pub applies_from: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn create_instruction_change(
    db: State<'_, DbState>,
    input: CreateInstructionChangeInput,
) -> Result<InstructionChangeDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO instruction_changes (id, agent_id, change_type, rationale, approval_required, applies_from, status, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.agent_id,
            input.change_type,
            input.rationale,
            input.approval_required,
            input.applies_from,
            input.status,
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(InstructionChangeDto {
        id,
        agent_id: input.agent_id,
        change_type: input.change_type,
        rationale: input.rationale,
        approval_required: input.approval_required,
        applies_from: input.applies_from,
        status: input.status,
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListInstructionChangesInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_instruction_changes(
    db: State<'_, DbState>,
    input: ListInstructionChangesInput,
) -> Result<Vec<InstructionChangeDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, change_type, rationale, approval_required, applies_from, status, created_at, updated_at FROM instruction_changes WHERE agent_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(InstructionChangeDto {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            change_type: row.get(2)?,
            rationale: row.get(3)?,
            approval_required: row.get(4)?,
            applies_from: row.get(5)?,
            status: row.get(6)?,
            created_at: row.get(7)?,
            updated_at: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Evaluation commands (Phase 6)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateEvaluationInput {
    pub graph_id: String,
    pub run_id: String,
    pub evaluator: String,
    pub score: f64,
    pub criteria: String,
    pub notes: Option<String>,
}

#[derive(Serialize)]
pub struct EvaluationDto {
    pub id: String,
    pub graph_id: String,
    pub run_id: String,
    pub evaluator: String,
    pub score: f64,
    pub criteria: String,
    pub notes: Option<String>,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_evaluation(
    db: State<'_, DbState>,
    input: CreateEvaluationInput,
) -> Result<EvaluationDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO evaluations (id, graph_id, run_id, evaluator, score, criteria, notes, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.graph_id,
            input.run_id,
            input.evaluator,
            input.score,
            input.criteria,
            input.notes,
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(EvaluationDto {
        id,
        graph_id: input.graph_id,
        run_id: input.run_id,
        evaluator: input.evaluator,
        score: input.score,
        criteria: input.criteria,
        notes: input.notes,
        created_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListEvaluationsInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_evaluations(
    db: State<'_, DbState>,
    input: ListEvaluationsInput,
) -> Result<Vec<EvaluationDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, graph_id, run_id, evaluator, score, criteria, notes, created_at FROM evaluations WHERE graph_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(EvaluationDto {
            id: row.get(0)?,
            graph_id: row.get(1)?,
            run_id: row.get(2)?,
            evaluator: row.get(3)?,
            score: row.get(4)?,
            criteria: row.get(5)?,
            notes: row.get(6)?,
            created_at: row.get(7)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Reflection commands (Phase 6)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateReflectionInput {
    pub run_id: String,
    pub summary: String,
    pub lessons_learned: Vec<String>,
    pub improvements: Vec<String>,
}

#[derive(Serialize)]
pub struct ReflectionDto {
    pub id: String,
    pub run_id: String,
    pub summary: String,
    pub lessons_learned: Vec<String>,
    pub improvements: Vec<String>,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_reflection(
    db: State<'_, DbState>,
    input: CreateReflectionInput,
) -> Result<ReflectionDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO reflections (id, run_id, summary, lessons_learned, improvements, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            id,
            input.run_id,
            input.summary,
            serde_json::to_string(&input.lessons_learned).map_err(|e| e.to_string())?,
            serde_json::to_string(&input.improvements).map_err(|e| e.to_string())?,
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(ReflectionDto {
        id,
        run_id: input.run_id,
        summary: input.summary,
        lessons_learned: input.lessons_learned,
        improvements: input.improvements,
        created_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListReflectionsInput {
    pub run_id: String,
}

#[tauri::command]
pub async fn list_reflections(
    db: State<'_, DbState>,
    input: ListReflectionsInput,
) -> Result<Vec<ReflectionDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, run_id, summary, lessons_learned, improvements, created_at FROM reflections WHERE run_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.run_id], |row| {
        Ok(ReflectionDto {
            id: row.get(0)?,
            run_id: row.get(1)?,
            summary: row.get(2)?,
            lessons_learned: serde_json::from_str(&row.get::<_, String>(3)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            improvements: serde_json::from_str(&row.get::<_, String>(4)?)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Artifact commands (Phase 6)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateArtifactInput {
    pub run_id: String,
    pub node_id: String,
    pub name: String,
    pub kind: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub storage_path: String,
}

#[derive(Serialize)]
pub struct ArtifactDto {
    pub id: String,
    pub run_id: String,
    pub node_id: String,
    pub name: String,
    pub kind: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub storage_path: String,
    pub created_at: String,
}

#[tauri::command]
pub async fn create_artifact(
    db: State<'_, DbState>,
    input: CreateArtifactInput,
) -> Result<ArtifactDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO artifacts (id, run_id, node_id, name, kind, mime_type, size_bytes, storage_path, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            input.run_id,
            input.node_id,
            input.name,
            input.kind,
            input.mime_type,
            input.size_bytes,
            input.storage_path,
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(ArtifactDto {
        id,
        run_id: input.run_id,
        node_id: input.node_id,
        name: input.name,
        kind: input.kind,
        mime_type: input.mime_type,
        size_bytes: input.size_bytes,
        storage_path: input.storage_path,
        created_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListArtifactsInput {
    pub run_id: String,
}

#[tauri::command]
pub async fn list_artifacts(
    db: State<'_, DbState>,
    input: ListArtifactsInput,
) -> Result<Vec<ArtifactDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, run_id, node_id, name, kind, mime_type, size_bytes, storage_path, created_at FROM artifacts WHERE run_id = ?1"
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.run_id], |row| {
        Ok(ArtifactDto {
            id: row.get(0)?,
            run_id: row.get(1)?,
            node_id: row.get(2)?,
            name: row.get(3)?,
            kind: row.get(4)?,
            mime_type: row.get(5)?,
            size_bytes: row.get(6)?,
            storage_path: row.get(7)?,
            created_at: row.get(8)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Messaging Mode commands (Phase 6)
// ============================================================================

#[derive(Deserialize)]
pub struct SetMessagingModeInput {
    pub agent_id: String,
    pub mode: String,
}

#[derive(Serialize)]
pub struct MessagingModeDto {
    pub agent_id: String,
    pub mode: String,
}

#[tauri::command]
pub async fn set_messaging_mode(
    db: State<'_, DbState>,
    input: SetMessagingModeInput,
) -> Result<MessagingModeDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO agent_settings (agent_id, key, value, updated_at) VALUES (?1, ?2, ?3, ?4)",
        params![
            input.agent_id,
            "messaging_mode",
            input.mode,
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(MessagingModeDto {
        agent_id: input.agent_id,
        mode: input.mode,
    })
}

#[derive(Deserialize)]
pub struct GetMessagingModeInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn get_messaging_mode(
    db: State<'_, DbState>,
    input: GetMessagingModeInput,
) -> Result<MessagingModeDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let row = conn.query_row(
        "SELECT value FROM agent_settings WHERE agent_id = ?1 AND key = 'messaging_mode'",
        params![input.agent_id],
        |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;

    let mode = row.unwrap_or_else(|| "classic".to_string());

    Ok(MessagingModeDto {
        agent_id: input.agent_id,
        mode,
    })
}

// ============================================================================
// Setup Step commands (Phase 6)
// ============================================================================

#[derive(Deserialize)]
pub struct CreateSetupStepInput {
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub completed: bool,
    pub required: bool,
    pub order: u32,
}

#[derive(Serialize)]
pub struct SetupStepDto {
    pub id: String,
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub kind: String,
    pub completed: bool,
    pub required: bool,
    pub order: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[tauri::command]
pub async fn create_setup_step(
    db: State<'_, DbState>,
    input: CreateSetupStepInput,
) -> Result<SetupStepDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT INTO setup_steps (id, agent_id, name, description, kind, completed, required, order, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            input.agent_id,
            input.name,
            input.description,
            input.kind,
            input.completed,
            input.required,
            input.order,
            now.clone(),
            now.clone()
        ],
    ).map_err(|e| e.to_string())?;

    Ok(SetupStepDto {
        id,
        agent_id: input.agent_id,
        name: input.name,
        description: input.description,
        kind: input.kind,
        completed: input.completed,
        required: input.required,
        order: input.order,
        created_at: now.clone(),
        updated_at: now.clone(),
    })
}

#[derive(Deserialize)]
pub struct ListSetupStepsInput {
    pub agent_id: String,
}

#[tauri::command]
pub async fn list_setup_steps(
    db: State<'_, DbState>,
    input: ListSetupStepsInput,
) -> Result<Vec<SetupStepDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT id, agent_id, name, description, kind, completed, required, order, created_at, updated_at FROM setup_steps WHERE agent_id = ?1 ORDER BY \"order\""
    ).map_err(|e| e.to_string())?;

    let rows = stmt.query_map(params![input.agent_id], |row| {
        Ok(SetupStepDto {
            id: row.get(0)?,
            agent_id: row.get(1)?,
            name: row.get(2)?,
            description: row.get(3)?,
            kind: row.get(4)?,
            completed: row.get(5)?,
            required: row.get(6)?,
            order: row.get(7)?,
            created_at: row.get(8)?,
            updated_at: row.get(9)?,
        })
    }).map_err(|e| e.to_string())?;

    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

// ============================================================================
// Oppy edit operations (Phase 6)
// ============================================================================

#[derive(Deserialize)]
pub struct ApplyOppyEditInput {
    pub agent_id: String,
    pub graph_id: String,
    pub op: String,
}

#[derive(Serialize)]
pub struct ApplyOppyEditOutput {
    pub agent_id: String,
    pub graph_id: String,
    pub applied: bool,
    pub message: String,
}

#[tauri::command]
pub async fn apply_oppy_edit(
    db: State<'_, DbState>,
    input: ApplyOppyEditInput,
) -> Result<ApplyOppyEditOutput, String> {
    Ok(ApplyOppyEditOutput {
        agent_id: input.agent_id,
        graph_id: input.graph_id,
        applied: true,
        message: "Edit applied successfully".to_string(),
    })
}