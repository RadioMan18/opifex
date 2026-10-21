use crate::db::DbState;
use rusqlite::Connection;
use rusqlite::params;
use rusqlite::types::Type;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::PathBuf;
use tauri::State;
use uuid::Uuid;

const MANIFEST_JSON: &str = include_str!("../../seeds/agents.manifest.json");
const CUSTOMER_SUPPORT_JSON: &str = include_str!("../../seeds/customer_support.json");
const RESEARCH_AGENT_JSON: &str = include_str!("../../seeds/research_agent.json");
const RAG_KNOWLEDGE_AGENT_JSON: &str = include_str!("../../seeds/rag_knowledge_agent.json");
const CODE_REVIEW_AGENT_JSON: &str = include_str!("../../seeds/code_review_agent.json");
const CONTENT_WRITER_AGENT_JSON: &str = include_str!("../../seeds/content_writer_agent.json");

fn seed_file_content(name: &str) -> Option<&'static str> {
    match name {
        "customer_support.json" => Some(CUSTOMER_SUPPORT_JSON),
        "research_agent.json" => Some(RESEARCH_AGENT_JSON),
        "rag_knowledge_agent.json" => Some(RAG_KNOWLEDGE_AGENT_JSON),
        "code_review_agent.json" => Some(CODE_REVIEW_AGENT_JSON),
        "content_writer_agent.json" => Some(CONTENT_WRITER_AGENT_JSON),
        _ => None,
    }
}

fn validate_agent_spec(spec: &Value) -> Result<(), String> {
    let goal = spec
        .get("goal")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Invalid agent: missing required field 'goal'".to_string())?;
    if goal.trim().is_empty() {
        return Err("Invalid agent: field 'goal' must not be empty".to_string());
    }
    if spec.get("steps").is_none() {
        return Err("Invalid agent: missing required field 'steps'".to_string());
    }
    Ok(())
}

fn user_agents_dir() -> Result<PathBuf, String> {
    if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(|p| PathBuf::from(p).join("Opifex").join("agents"))
            .map_err(|_| "APPDATA environment variable not set".to_string())
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|p| PathBuf::from(p).join("Library/Application Support/Opifex/agents"))
            .map_err(|_| "HOME environment variable not set".to_string())
    } else {
        std::env::var("HOME")
            .map(|p| PathBuf::from(p).join(".local/share/Opifex/agents"))
            .map_err(|_| "HOME environment variable not set".to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub spec_json: serde_json::Value,
    pub model: String,
    pub is_starter: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAgentInput {
    pub name: String,
    pub description: Option<String>,
    pub spec_json: serde_json::Value,
    pub model: Option<String>,
    pub is_starter: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAgentInput {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub spec_json: Option<serde_json::Value>,
    pub model: Option<String>,
    pub is_starter: Option<bool>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentListItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model: String,
    pub is_starter: bool,
    pub updated_at: String,
}

fn row_to_agent(row: &rusqlite::Row<'_>) -> rusqlite::Result<Agent> {
    let spec_str: String = row.get(3)?;
    Ok(Agent {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        spec_json: serde_json::from_str(&spec_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(e)))?,
        model: row.get(4)?,
        is_starter: row.get::<_, i32>(5)? != 0,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

#[tauri::command]
pub fn create_agent(state: State<'_, DbState>, input: CreateAgentInput) -> Result<Agent, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let model = input.model.unwrap_or_else(|| "ollama".to_string());
    let is_starter = input.is_starter.unwrap_or(false);
    let spec_json = serde_json::to_string(&input.spec_json).map_err(|e| e.to_string())?;

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.name,
            input.description,
            spec_json,
            model,
            is_starter as i32,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Agent {
        id,
        name: input.name,
        description: input.description,
        spec_json: input.spec_json,
        model,
        is_starter,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub fn get_agent(state: State<'_, DbState>, id: String) -> Result<Option<Agent>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, name, description, spec_json, model, is_starter, created_at, updated_at
         FROM agents WHERE id = ?1",
    ).map_err(|e| e.to_string())?;

    let agent: Option<Agent> = stmt
        .query_row(params![id], row_to_agent)
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(agent)
}

#[tauri::command]
pub fn list_agents(
    state: State<'_, DbState>,
    include_starters: Option<bool>,
) -> Result<Vec<AgentListItem>, String> {
    let conn = &*state.conn.lock().unwrap();
    let query = match include_starters {
        Some(false) => {
            "SELECT id, name, description, model, is_starter, updated_at FROM agents WHERE is_starter = 0 ORDER BY updated_at DESC"
        }
        _ => {
            "SELECT id, name, description, model, is_starter, updated_at FROM agents ORDER BY is_starter DESC, updated_at DESC"
        }
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;
    let agents: Vec<AgentListItem> = stmt
        .query_map([], |row| {
            Ok(AgentListItem {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                model: row.get(3)?,
                is_starter: row.get::<_, i32>(4)? != 0,
                updated_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(agents)
}

#[tauri::command]
pub fn update_agent(
    state: State<'_, DbState>,
    id: String,
    input: UpdateAgentInput,
) -> Result<Agent, String> {
    let id_for_lookup = id.clone();

    {
        let conn = &*state.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT name, description, spec_json, model, is_starter FROM agents WHERE id = ?1",
        ).map_err(|e| e.to_string())?;

        let existing: Option<(String, Option<String>, String, String, i32)> = stmt
            .query_row(params![&id_for_lookup], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })
            .optional()
            .map_err(|e| e.to_string())?;

        let (name, description, spec_json_str, model, is_starter) = match existing {
            Some(t) => t,
            None => return Err(format!("Agent not found: {}", id_for_lookup)),
        };

        let new_name = input.name.unwrap_or(name);
        let new_description = input.description.unwrap_or(description);
        let new_spec_json: serde_json::Value = match &input.spec_json {
            Some(v) => v.clone(),
            None => serde_json::from_str(&spec_json_str).map_err(|e| e.to_string())?,
        };
        let new_model = input.model.unwrap_or(model);
        let new_is_starter: i32 = input.is_starter.map(|b| b as i32).unwrap_or(is_starter);
        let new_spec_json_str = serde_json::to_string(&new_spec_json).map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE agents SET name = ?1, description = ?2, spec_json = ?3, model = ?4, is_starter = ?5, updated_at = datetime('now') WHERE id = ?6",
            params![new_name, new_description, new_spec_json_str, new_model, new_is_starter, id_for_lookup],
        )
        .map_err(|e| e.to_string())?;
    }

        get_agent(state, id.clone())?
            .ok_or_else(|| format!("Agent not found: {}", id))
}

#[tauri::command]
pub fn delete_agent(state: State<'_, DbState>, id: String) -> Result<bool, String> {
    let conn = &*state.conn.lock().unwrap();
    let changes = conn.execute("DELETE FROM agents WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(changes > 0)
}

#[tauri::command]
pub fn export_agent(state: State<'_, DbState>, id: String) -> Result<String, String> {
    let agent = get_agent(state, id.clone())?
        .ok_or_else(|| format!("Agent not found: {}", id))?;
    serde_json::to_string(&agent).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_agent(state: State<'_, DbState>, agent_json: String) -> Result<Agent, String> {
    let mut agent: Agent = serde_json::from_str(&agent_json).map_err(|e| e.to_string())?;

    // Validate required AgentPlan fields
    let spec_value = &agent.spec_json;
    let goal = spec_value.get("goal")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Invalid agent: missing required field 'goal'".to_string())?;
    if goal.trim().is_empty() {
        return Err("Invalid agent: field 'goal' must not be empty".to_string());
    }
    if spec_value.get("steps").is_none() {
        return Err("Invalid agent: missing required field 'steps'".to_string());
    }

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    agent.id = id;
    agent.created_at = now.clone();
    agent.updated_at = now.clone();
    agent.is_starter = false;

    let spec_json = serde_json::to_string(&agent.spec_json).map_err(|e| e.to_string())?;

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            agent.id,
            agent.name,
            agent.description,
            spec_json,
            agent.model,
            agent.is_starter as i32,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(agent)
}

#[tauri::command]
pub fn seed_starter_agents(state: State<'_, DbState>) -> Result<Vec<String>, String> {
    let conn = &*state.conn.lock().unwrap();

    let mut stmt = conn.prepare("SELECT id FROM agents WHERE is_starter = 1").map_err(|e| e.to_string())?;
    let existing: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    if !existing.is_empty() {
        return Ok(existing);
    }

    let now = chrono::Utc::now().to_rfc3339();
    let manifest: Value = serde_json::from_str(MANIFEST_JSON)
        .map_err(|e| format!("Failed to parse seed manifest: {}", e))?;

    let starter_files: Vec<String> = manifest["starters"]
        .as_array()
        .ok_or("Seed manifest missing 'starters' array")?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect();

    let mut created = Vec::new();
    for file_name in starter_files {
        let content = seed_file_content(&file_name)
            .ok_or_else(|| format!("Seed file not embedded: {}", file_name))?;

        let mut agent: Agent = serde_json::from_str(content)
            .map_err(|e| format!("Failed to parse {}: {}", file_name, e))?;

        validate_agent_spec(&agent.spec_json)?;

        let id = Uuid::new_v4().to_string();
        agent.id = id.clone();
        agent.created_at = now.clone();
        agent.updated_at = now.clone();
        agent.is_starter = true;

        let spec_json = serde_json::to_string(&agent.spec_json).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                agent.id,
                agent.name,
                agent.description,
                spec_json,
                agent.model,
                agent.is_starter as i32,
                now,
                now
            ],
        )
        .map_err(|e| e.to_string())?;
        created.push(id);
    }

    Ok(created)
}

#[tauri::command]
pub fn load_user_agents(state: State<'_, DbState>) -> Result<Vec<String>, String> {
    let user_agents_dir = user_agents_dir()?;

    if !user_agents_dir.exists() {
        return Ok(Vec::new());
    }

    let conn = &*state.conn.lock().unwrap();
    let now = chrono::Utc::now().to_rfc3339();
    let mut created = Vec::new();

    for entry in std::fs::read_dir(&user_agents_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) != Some("json") {
            continue;
        }

        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let mut agent: Agent = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

        validate_agent_spec(&agent.spec_json)?;

        let mut stmt = conn.prepare("SELECT id FROM agents WHERE name = ?1 AND is_starter = 0").map_err(|e| e.to_string())?;
        let existing: Option<String> = stmt
            .query_row(params![agent.name], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;

        if existing.is_some() {
            continue;
        }

        let id = Uuid::new_v4().to_string();
        agent.id = id.clone();
        agent.created_at = now.clone();
        agent.updated_at = now.clone();
        agent.is_starter = false;

        let spec_json = serde_json::to_string(&agent.spec_json).map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                agent.id,
                agent.name,
                agent.description,
                spec_json,
                agent.model,
                agent.is_starter as i32,
                now,
                now
            ],
        )
        .map_err(|e| e.to_string())?;
        created.push(id);
    }

    Ok(created)
}