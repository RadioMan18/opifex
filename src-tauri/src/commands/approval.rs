use crate::db::DbState;
use rusqlite::{params, OptionalExtension, types::Type};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tauri::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApprovalRequest {
    pub id: String,
    pub agent_id: String,
    pub task_id: String,
    pub command: String,
    pub context: String,
    pub status: String,
    pub response: Option<String>,
    pub requested_at: String,
    pub responded_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApprovalInput {
    pub agent_id: String,
    pub command: String,
    pub context: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalStatus {
    pub task_id: String,
    pub status: String,
    pub response: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PollApprovalInput {
    pub task_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelApprovalInput {
    pub task_id: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListPendingApprovalsInput {
    pub agent_id: Option<String>,
}

fn row_to_approval_request(row: &rusqlite::Row<'_>) -> rusqlite::Result<ApprovalRequest> {
    let response: Option<String> = row.get(6)?;
    let responded_at: Option<String> = row.get(8)?;
    Ok(ApprovalRequest {
        id: row.get(0)?,
        agent_id: row.get(1)?,
        task_id: row.get(2)?,
        command: row.get(3)?,
        context: row.get(4)?,
        status: row.get(5)?,
        response,
        requested_at: row.get(7)?,
        responded_at,
    })
}

fn notifier_base_url(conn: &rusqlite::Connection) -> Result<String, String> {
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = 'notifier_base_url'")
        .map_err(|e| e.to_string())?;
    let url: Option<String> = stmt.query_row([], |row| row.get(0)).optional().map_err(|e| e.to_string())?;
    Ok(url.unwrap_or_else(|| "http://localhost:3001".to_string()))
}

#[tauri::command]
pub fn request_approval(
    state: State<'_, DbState>,
    input: CreateApprovalInput,
) -> Result<HashMap<String, serde_json::Value>, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = &*state.conn.lock().unwrap();
    let base_url = notifier_base_url(conn)?;

    // Store pending record first
    conn.execute(
        "INSERT INTO approval_requests (id, agent_id, task_id, command, context, status, requested_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![id, input.agent_id, id, input.command, input.context, "pending", now],
    )
    .map_err(|e| e.to_string())?;

    // Post to Notifier
    let client = reqwest::blocking::Client::new();
    let payload = serde_json::json!({
        "command": input.command,
        "context": input.context,
        "requester": "opifex"
    });

    let resp = client
        .post(format!("{}/api/request-approval", base_url))
        .json(&payload)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| format!("Notifier request failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Notifier returned status {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().map_err(|e| format!("Notifier response parse error: {}", e))?;

    let task_id = body["taskId"]
        .as_str()
        .ok_or_else(|| "Notifier response missing taskId".to_string())?
        .to_string();

    // Update the local record with the notifier task_id (same as our id for simplicity)
    conn.execute(
        "UPDATE approval_requests SET task_id = ?1 WHERE id = ?2",
        params![task_id, id],
    )
    .map_err(|e| e.to_string())?;

    let mut result = HashMap::new();
    result.insert("taskId".to_string(), serde_json::Value::String(task_id.clone()));
    result.insert("approveUrl".to_string(), body["approveUrl"].clone());
    result.insert("denyUrl".to_string(), body["denyUrl"].clone());
    result.insert("cancelUrl".to_string(), body["cancelUrl"].clone());
    result.insert("status".to_string(), serde_json::Value::String("pending".to_string()));

    Ok(result)
}

#[tauri::command]
pub fn poll_approval_status(
    state: State<'_, DbState>,
    input: PollApprovalInput,
) -> Result<ApprovalStatus, String> {
    let conn = &*state.conn.lock().unwrap();
    let base_url = notifier_base_url(conn)?;

    // Poll Notifier
    let client = reqwest::blocking::Client::new();
    let resp = client
        .get(format!("{}/api/status/{}", base_url, input.task_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| format!("Notifier poll failed: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Notifier returned status {}", resp.status()));
    }

    let body: serde_json::Value = resp.json().map_err(|e| format!("Notifier response parse error: {}", e))?;

    let status = body["status"]
        .as_str()
        .unwrap_or("pending")
        .to_string();
    let response = body["response"].as_str().map(|s| s.to_string());

    // Update local record
    conn.execute(
        "UPDATE approval_requests SET status = ?1, response = ?2, responded_at = datetime('now') WHERE task_id = ?3",
        params![&status, response, input.task_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(ApprovalStatus {
        task_id: input.task_id,
        status,
        response,
    })
}

#[tauri::command]
pub fn cancel_approval(
    state: State<'_, DbState>,
    input: CancelApprovalInput,
) -> Result<bool, String> {
    let conn = &*state.conn.lock().unwrap();
    let base_url = notifier_base_url(conn)?;

    // Call Notifier cancel
    let client = reqwest::blocking::Client::new();
    let _resp = client
        .get(format!("{}/api/cancel/{}", base_url, input.task_id))
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .map_err(|e| format!("Notifier cancel failed: {}", e))?;

    // Update local record
    let changes = conn.execute(
        "UPDATE approval_requests SET status = 'cancelled', response = 'cancelled', responded_at = datetime('now') WHERE task_id = ?1",
        params![input.task_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(changes > 0)
}

#[tauri::command]
pub fn list_pending_approvals(
    state: State<'_, DbState>,
    input: Option<ListPendingApprovalsInput>,
) -> Result<Vec<ApprovalRequest>, String> {
    let conn = &*state.conn.lock().unwrap();

    let (query, params) = match &input {
        Some(inp) if inp.agent_id.is_some() => (
            "SELECT id, agent_id, task_id, command, context, status, response, requested_at, responded_at
             FROM approval_requests WHERE status = 'pending' AND agent_id = ?1 ORDER BY requested_at DESC",
            vec![inp.agent_id.as_ref().unwrap().as_str()],
        ),
        _ => (
            "SELECT id, agent_id, task_id, command, context, status, response, requested_at, responded_at
             FROM approval_requests WHERE status = 'pending' ORDER BY requested_at DESC",
            vec![],
        ),
    };

    let mut stmt = conn.prepare(query).map_err(|e| e.to_string())?;

    let param_refs: Vec<&dyn rusqlite::ToSql> = params.iter().map(|p| p as &dyn rusqlite::ToSql).collect();

    let approvals: Vec<ApprovalRequest> = stmt
        .query_map(&param_refs[..], row_to_approval_request)
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(approvals)
}
