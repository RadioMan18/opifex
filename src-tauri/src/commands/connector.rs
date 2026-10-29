use crate::db::DbState;
use opifex_core::{
    AccountSelection, ConnectorBlock, ConnectorGrant, ToolApprovalMode, ToolManagementConfig,
    ToolOverride,
};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateConnectorInput {
    pub name: String,
    pub category: String,
    pub account_selection: AccountSelection,
    pub tool_management: ToolManagementConfig,
}

#[derive(Deserialize)]
pub struct UpdateConnectorInput {
    pub id: String,
    pub name: Option<String>,
    pub category: Option<String>,
    pub account_selection: Option<AccountSelection>,
    pub tool_management: Option<ToolManagementConfig>,
}

#[derive(Deserialize)]
pub struct DeleteConnectorInput {
    pub id: String,
}

#[derive(Deserialize)]
pub struct GetConnectorInput {
    pub id: String,
}

#[derive(Deserialize)]
pub struct ListConnectorsInput {
    pub category: Option<String>,
}

#[derive(Deserialize)]
pub struct GrantConnectorInput {
    pub agent_id: String,
    pub connector_id: String,
    pub account_id: Option<String>,
}

#[derive(Deserialize)]
pub struct RevokeConnectorInput {
    pub agent_id: String,
    pub connector_id: String,
}

#[derive(Deserialize)]
pub struct ListGrantsInput {
    pub agent_id: String,
}

#[derive(Serialize)]
pub struct ConnectorBlockDto {
    pub id: String,
    pub name: String,
    pub category: String,
    pub account_selection: AccountSelection,
    pub tool_management: ToolManagementConfig,
    pub is_builtin: bool,
    pub is_custom: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize)]
pub struct ConnectorGrantDto {
    pub agent_id: String,
    pub connector_id: String,
    pub account_id: Option<String>,
    pub granted_at: String,
}

fn connector_to_dto(c: &ConnectorBlock) -> ConnectorBlockDto {
    ConnectorBlockDto {
        id: c.id.clone(),
        name: c.name.clone(),
        category: c.category.clone(),
        account_selection: c.account_selection.clone(),
        tool_management: c.tool_management.clone(),
        is_builtin: c.is_builtin,
        is_custom: c.is_custom,
        created_at: c.created_at.clone(),
        updated_at: c.updated_at.clone(),
    }
}

fn grant_to_dto(g: &ConnectorGrant) -> ConnectorGrantDto {
    ConnectorGrantDto {
        agent_id: g.agent_id.clone(),
        connector_id: g.connector_id.clone(),
        account_id: g.account_id.clone(),
        granted_at: g.granted_at.clone(),
    }
}

#[tauri::command]
pub async fn create_connector(
    db: State<'_, DbState>,
    input: CreateConnectorInput,
) -> Result<ConnectorBlockDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let now_clone = now.clone();

    conn.execute(
        "INSERT INTO block_definitions (id, version, name, description, category, kind, origin, spec_json, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            id,
            1,
            input.name,
            None::<String>,
            input.category,
            "tool",
            "user",
            serde_json::to_string(&serde_json::json!({
                "account_selection": input.account_selection,
                "tool_management": input.tool_management
            })).unwrap(),
            now,
            now_clone
        ],
    ).map_err(|e| e.to_string())?;

    Ok(ConnectorBlockDto {
        id,
        name: input.name,
        category: input.category,
        account_selection: input.account_selection,
        tool_management: input.tool_management,
        is_builtin: false,
        is_custom: true,
        created_at: now,
        updated_at: now_clone,
    })
}

#[tauri::command]
pub async fn get_connector(
    db: State<'_, DbState>,
    input: GetConnectorInput,
) -> Result<Option<ConnectorBlockDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let row = conn.query_row(
        "SELECT id, name, category, spec_json, created_at, updated_at FROM block_definitions WHERE id = ?1 AND kind = 'tool'",
        params![input.id],
        |row| {
            let spec_str: String = row.get(3)?;
            let spec: serde_json::Value = serde_json::from_str(&spec_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let account_selection: AccountSelection = serde_json::from_value(
                spec.get("account_selection").cloned().unwrap_or(serde_json::Value::Null)
            ).unwrap_or(AccountSelection::PersonalDefault);
            let tool_management: ToolManagementConfig = serde_json::from_value(
                spec.get("tool_management").cloned().unwrap_or(serde_json::Value::Null)
            ).unwrap_or_default();

            Ok(ConnectorBlockDto {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                account_selection,
                tool_management,
                is_builtin: false,
                is_custom: true,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        },
    ).ok();

    let row = match row {
        Some(row) => row,
        None => return Ok(None),
    };

    Ok(Some(row))
}

#[tauri::command]
pub async fn list_connectors(
    db: State<'_, DbState>,
    input: ListConnectorsInput,
) -> Result<Vec<ConnectorBlockDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt: rusqlite::Statement<'_> = if let Some(ref cat) = input.category {
        conn.prepare(
            "SELECT id, name, category, spec_json, created_at, updated_at FROM block_definitions WHERE kind = 'tool' AND category = ?1"
        ).map_err(|e| e.to_string())?
    } else {
        conn.prepare(
            "SELECT id, name, category, spec_json, created_at, updated_at FROM block_definitions WHERE kind = 'tool'"
        ).map_err(|e| e.to_string())?
    };

    let connectors: Vec<ConnectorBlockDto> = stmt
        .query_map(params![input.category.unwrap_or_default()], |row| {
            let spec_str: String = row.get(3)?;
            let spec: serde_json::Value = serde_json::from_str(&spec_str)
                .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;
            let account_selection: AccountSelection = serde_json::from_value(
                spec.get("account_selection").cloned().unwrap_or(serde_json::Value::Null)
            ).unwrap_or(AccountSelection::PersonalDefault);
            let tool_management: ToolManagementConfig = serde_json::from_value(
                spec.get("tool_management").cloned().unwrap_or(serde_json::Value::Null)
            ).unwrap_or_default();

            Ok(ConnectorBlockDto {
                id: row.get(0)?,
                name: row.get(1)?,
                category: row.get(2)?,
                account_selection,
                tool_management,
                is_builtin: false,
                is_custom: true,
                created_at: row.get(4)?,
                updated_at: row.get(5)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    Ok(connectors)
}

#[tauri::command]
pub async fn update_connector(
    db: State<'_, DbState>,
    input: UpdateConnectorInput,
) -> Result<ConnectorBlockDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    // Get existing connector
    let existing: Option<(String, String)> = conn.query_row(
        "SELECT name, category FROM block_definitions WHERE id = ?1 AND kind = 'tool'",
        params![input.id],
        |row| Ok((row.get(0)?, row.get(1)?))
    ).ok();

    let (name, category) = existing.ok_or_else(|| format!("Connector not found: {}", input.id))?;

    let name = input.name.unwrap_or(name);
    let category = input.category.unwrap_or(category);

    // Get existing spec
    let spec_str: String = conn.query_row(
        "SELECT spec_json FROM block_definitions WHERE id = ?1",
        params![input.id],
        |row| row.get(0)
    ).map_err(|e| e.to_string())?;
    let spec: serde_json::Value = serde_json::from_str(&spec_str)
        .map_err(|e| e.to_string())?;
    let spec: serde_json::Value = serde_json::from_str(&spec_str)
        .map_err(|e| e.to_string())?;

    let mut new_spec = spec;
    if let Some(as_) = input.account_selection.as_ref() {
        new_spec["account_selection"] = serde_json::to_value(as_).unwrap();
    }
    if let Some(tm) = input.tool_management.as_ref() {
        new_spec["tool_management"] = serde_json::to_value(tm).unwrap();
    }

    conn.execute(
        "UPDATE block_definitions SET name = ?1, category = ?2, spec_json = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, category, serde_json::to_string(&new_spec).unwrap(), now, input.id]
    ).map_err(|e| e.to_string())?;

    Ok(ConnectorBlockDto {
        id: input.id,
        name,
        category,
        account_selection: input.account_selection.unwrap_or(AccountSelection::PersonalDefault),
        tool_management: input.tool_management.unwrap_or_default(),
        is_builtin: false,
        is_custom: true,
        created_at: now.clone(),
        updated_at: now,
    })
}

#[tauri::command]
pub async fn delete_connector(
    db: State<'_, DbState>,
    input: DeleteConnectorInput,
) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let rows = conn.execute(
        "DELETE FROM block_definitions WHERE id = ?1 AND kind = 'tool'",
        params![input.id]
    ).map_err(|e| e.to_string())?;

    Ok(rows > 0)
}

#[tauri::command]
pub async fn grant_connector(
    db: State<'_, DbState>,
    input: GrantConnectorInput,
) -> Result<ConnectorGrantDto, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let now = chrono::Utc::now().to_rfc3339();

    conn.execute(
        "INSERT OR REPLACE INTO agent_grants (agent_id, kind, value, granted_at) VALUES (?1, ?2, ?3, ?4)",
        params![input.agent_id, "connector", input.connector_id, now]
    ).map_err(|e| e.to_string())?;

    Ok(ConnectorGrantDto {
        agent_id: input.agent_id,
        connector_id: input.connector_id,
        account_id: input.account_id,
        granted_at: now,
    })
}

#[tauri::command]
pub async fn revoke_connector(
    db: State<'_, DbState>,
    input: RevokeConnectorInput,
) -> Result<bool, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let rows = conn.execute(
        "DELETE FROM agent_grants WHERE agent_id = ?1 AND kind = 'connector' AND value = ?2",
        params![input.agent_id, input.connector_id]
    ).map_err(|e| e.to_string())?;

    Ok(rows > 0)
}

#[tauri::command]
pub async fn list_connector_grants(
    db: State<'_, DbState>,
    input: ListGrantsInput,
) -> Result<Vec<ConnectorGrantDto>, String> {
    let conn = db.conn.lock().map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare(
        "SELECT agent_id, value, granted_at FROM agent_grants WHERE kind = 'connector' AND agent_id = ?1"
    ).map_err(|e| e.to_string())?;

    let grants: Vec<ConnectorGrantDto> = stmt
        .query_map(params![input.agent_id], |row| {
            Ok(ConnectorGrantDto {
                agent_id: input.agent_id.clone(),
                connector_id: row.get(1)?,
                account_id: None,
                granted_at: row.get(2)?,
            })
        }).map_err(|e| e.to_string())?
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;

    Ok(grants)
}

#[tauri::command]
pub async fn check_tool_approval(
    tool_id: String,
    connector_id: String,
    preset: String,
    overrides: Option<Vec<ToolOverride>>,
) -> Result<(bool, bool), String> {
    let preset_mode: ToolApprovalMode = serde_json::from_str(&format!("\"{}\"", preset))
        .map_err(|e| e.to_string())?;

    let config = ToolManagementConfig {
        connector_id,
        preset: preset_mode,
        overrides: overrides.unwrap_or_default(),
    };

    Ok(config.check_tool(&tool_id))
}
