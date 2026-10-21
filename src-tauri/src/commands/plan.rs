use crate::db::DbState;
use rusqlite::{params, OptionalExtension, types::Type};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlanVersion {
    pub id: String,
    pub agent_id: String,
    pub label: Option<String>,
    pub spec_json: serde_json::Value,
    pub created_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavePlanVersionInput {
    pub agent_id: String,
    pub label: Option<String>,
    pub spec_json: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RestorePlanVersionInput {
    pub version_id: String,
}

fn row_to_plan_version(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlanVersion> {
    let spec_str: String = row.get(3)?;
    Ok(PlanVersion {
        id: row.get(0)?,
        agent_id: row.get(1)?,
        label: row.get(2)?,
        spec_json: serde_json::from_str(&spec_str)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(e)))?,
        created_at: row.get(4)?,
    })
}

#[tauri::command]
pub fn save_plan_version(
    state: State<'_, DbState>,
    input: SavePlanVersionInput,
) -> Result<PlanVersion, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let spec_json = serde_json::to_string(&input.spec_json).map_err(|e| e.to_string())?;

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO plan_versions (id, agent_id, label, spec_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, input.agent_id, input.label, spec_json, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(PlanVersion {
        id,
        agent_id: input.agent_id,
        label: input.label,
        spec_json: input.spec_json,
        created_at: now,
    })
}

#[tauri::command]
pub fn list_plan_versions(
    state: State<'_, DbState>,
    agent_id: String,
) -> Result<Vec<PlanVersion>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, label, spec_json, created_at FROM plan_versions WHERE agent_id = ?1 ORDER BY created_at DESC",
    ).map_err(|e| e.to_string())?;

    let versions: Vec<PlanVersion> = stmt
        .query_map(params![agent_id], row_to_plan_version)
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(versions)
}

#[tauri::command]
pub fn restore_plan_version(
    state: State<'_, DbState>,
    input: RestorePlanVersionInput,
) -> Result<serde_json::Value, String> {
    let conn = &*state.conn.lock().unwrap();

    let version: Option<PlanVersion> = conn
        .prepare("SELECT id, agent_id, label, spec_json, created_at FROM plan_versions WHERE id = ?1")
        .map_err(|e| e.to_string())?
        .query_row(params![input.version_id], row_to_plan_version)
        .optional()
        .map_err(|e| e.to_string())?;

    let plan = version.ok_or_else(|| format!("Plan version not found: {}", input.version_id))?;

    conn.execute(
        "UPDATE agents SET spec_json = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![serde_json::to_string(&plan.spec_json).map_err(|e| e.to_string())?, plan.agent_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(plan.spec_json)
}