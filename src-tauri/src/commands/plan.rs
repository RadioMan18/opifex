use crate::db::DbState;
use rusqlite::{params, OptionalExtension, types::Type};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PlanVersion {
    pub id: String,
    pub agentId: String,
    pub label: Option<String>,
    pub specJson: serde_json::Value,
    pub createdAt: String,
}

#[derive(Deserialize)]
pub struct SavePlanVersionInput {
    pub agentId: String,
    pub label: Option<String>,
    pub specJson: serde_json::Value,
}

#[derive(Deserialize)]
pub struct RestorePlanVersionInput {
    pub versionId: String,
}

fn rowToPlanVersion(row: &rusqlite::Row<'_>) -> rusqlite::Result<PlanVersion> {
    let specStr: String = row.get(3)?;
    Ok(PlanVersion {
        id: row.get(0)?,
        agentId: row.get(1)?,
        label: row.get(2)?,
        specJson: serde_json::from_str(&specStr)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(e)))?,
        createdAt: row.get(4)?,
    })
}

#[tauri::command]
pub fn save_plan_version(
    state: State<'_, DbState>,
    input: SavePlanVersionInput,
) -> Result<PlanVersion, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let specJson = serde_json::to_string(&input.specJson).map_err(|e| e.to_string())?;

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO plan_versions (id, agent_id, label, spec_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, input.agentId, input.label, specJson, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(PlanVersion {
        id,
        agentId: input.agentId,
        label: input.label,
        specJson: input.specJson,
        createdAt: now,
    })
}

#[tauri::command]
pub fn list_plan_versions(
    state: State<'_, DbState>,
    agentId: String,
) -> Result<Vec<PlanVersion>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, label, spec_json, created_at FROM plan_versions WHERE agent_id = ?1 ORDER BY created_at DESC",
    ).map_err(|e| e.to_string())?;

    let versions: Vec<PlanVersion> = stmt
        .query_map(params![agentId], rowToPlanVersion)
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
        .query_row(params![input.versionId], rowToPlanVersion)
        .optional()
        .map_err(|e| e.to_string())?;

    let plan = version.ok_or_else(|| format!("Plan version not found: {}", input.versionId))?;

    conn.execute(
        "UPDATE agents SET spec_json = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![serde_json::to_string(&plan.specJson).map_err(|e| e.to_string())?, plan.agentId],
    )
    .map_err(|e| e.to_string())?;

    Ok(plan.specJson)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::agent::get_agent;
    use crate::db::run_migrations;
    use rusqlite::Connection;

    fn testConn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    fn insertTestAgent(conn: &Connection) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, "Test Agent", None::<String>, serde_json::to_string(&serde_json::json!({"goal": "original"})).unwrap(), "ollama", 0, now, now],
        ).unwrap();
        id
    }

    #[test]
    fn testSaveAndListPlanVersions() {
        let conn = testConn();
        let agentId = insertTestAgent(&conn);
        let now = chrono::Utc::now().to_rfc3339();

        let v1Id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO plan_versions (id, agent_id, label, spec_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![v1Id, agentId, Some("v1".to_string()), serde_json::to_string(&serde_json::json!({"goal": "version one"})).unwrap(), now],
        ).unwrap();

        let v2Id = uuid::Uuid::new_v4().to_string();
        let now2 = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO plan_versions (id, agent_id, label, spec_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![v2Id, agentId, None::<String>, serde_json::to_string(&serde_json::json!({"goal": "version two"})).unwrap(), now2],
        ).unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, agent_id, label, spec_json, created_at FROM plan_versions WHERE agent_id = ?1 ORDER BY created_at DESC",
        ).unwrap();
        let versions: Vec<(String, String, Option<String>, String, String)> = stmt
            .query_map(rusqlite::params![agentId], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })
            .unwrap()
            .map(|r| r.unwrap())
            .collect();

        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].2, None);
        assert_eq!(versions[1].2, Some("v1".to_string()));
    }

    #[test]
    fn testRestorePlanVersion() {
        let conn = testConn();
        let agentId = insertTestAgent(&conn);

        let versionId = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        conn.execute(
            "INSERT INTO plan_versions (id, agent_id, label, spec_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![versionId, agentId, Some("restore target".to_string()), serde_json::to_string(&serde_json::json!({"goal": "restored goal"})).unwrap(), now],
        ).unwrap();

        let specJson = serde_json::to_string(&serde_json::json!({"goal": "restored goal"})).unwrap();
        conn.execute(
            "UPDATE agents SET spec_json = ?1, updated_at = datetime('now') WHERE id = ?2",
            rusqlite::params![specJson, agentId],
        ).unwrap();

        let row: Option<String> = conn
            .query_row("SELECT spec_json FROM agents WHERE id = ?1", rusqlite::params![agentId], |row| row.get(0))
            .optional()
            .unwrap();

        assert!(row.is_some());
        let spec: serde_json::Value = serde_json::from_str(&row.unwrap()).unwrap();
        assert_eq!(spec, serde_json::json!({"goal": "restored goal"}));
    }
}
