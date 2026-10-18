use crate::db::DbState;
use rusqlite::{params, OptionalExtension, types::Type};
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Conversation {
    pub id: String,
    pub agentId: String,
    pub title: String,
    pub createdAt: String,
}

#[derive(Deserialize)]
pub struct CreateConversationInput {
    pub agentId: String,
    pub title: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub id: String,
    pub conversationId: String,
    pub role: String,
    pub content: String,
    pub turnPlanJson: Option<serde_json::Value>,
    pub createdAt: String,
}

#[derive(Deserialize)]
pub struct SaveMessageInput {
    pub conversationId: String,
    pub role: String,
    pub content: String,
    pub turnPlanJson: Option<serde_json::Value>,
}

fn row_to_conversation(row: &rusqlite::Row<'_>) -> rusqlite::Result<Conversation> {
    Ok(Conversation {
        id: row.get(0)?,
        agentId: row.get(1)?,
        title: row.get(2)?,
        createdAt: row.get(3)?,
    })
}

fn row_to_message(row: &rusqlite::Row<'_>) -> rusqlite::Result<Message> {
    let planStr: Option<String> = row.get(4)?;
    Ok(Message {
        id: row.get(0)?,
        conversationId: row.get(1)?,
        role: row.get(2)?,
        content: row.get(3)?,
        turnPlanJson: planStr
            .map(|s| serde_json::from_str(&s))
            .transpose()
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(4, Type::Text, Box::new(e)))?,
        createdAt: row.get(5)?,
    })
}

#[tauri::command]
pub fn create_conversation(
    state: State<'_, DbState>,
    input: CreateConversationInput,
) -> Result<Conversation, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO conversations (id, agent_id, title, created_at)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, input.agentId, input.title, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Conversation {
        id,
        agentId: input.agentId,
        title: input.title,
        createdAt: now,
    })
}

#[tauri::command]
pub fn get_conversation(state: State<'_, DbState>, id: String) -> Result<Option<Conversation>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, title, created_at FROM conversations WHERE id = ?1",
    ).map_err(|e| e.to_string())?;

    let conversation: Option<Conversation> = stmt
        .query_row(params![id], row_to_conversation)
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(conversation)
}

#[tauri::command]
pub fn list_conversations(
    state: State<'_, DbState>,
    agentId: String,
) -> Result<Vec<Conversation>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, agent_id, title, created_at FROM conversations WHERE agent_id = ?1 ORDER BY created_at DESC",
    ).map_err(|e| e.to_string())?;

    let conversations: Vec<Conversation> = stmt
        .query_map(params![agentId], row_to_conversation)
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(conversations)
}

#[tauri::command]
pub fn save_message(
    state: State<'_, DbState>,
    input: SaveMessageInput,
) -> Result<Message, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let planJson = match &input.turnPlanJson {
        Some(p) => Some(serde_json::to_string(p).map_err(|e| e.to_string())?),
        None => None,
    };

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO messages (id, conversation_id, role, content, turn_plan_json, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, input.conversationId, input.role, input.content, planJson, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Message {
        id,
        conversationId: input.conversationId,
        role: input.role,
        content: input.content,
        turnPlanJson: input.turnPlanJson,
        createdAt: now,
    })
}

#[tauri::command]
pub fn load_messages(
    state: State<'_, DbState>,
    conversationId: String,
) -> Result<Vec<Message>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn.prepare(
        "SELECT id, conversation_id, role, content, turn_plan_json, created_at FROM messages WHERE conversation_id = ?1 ORDER BY created_at ASC",
    ).map_err(|e| e.to_string())?;

    let messages: Vec<Message> = stmt
        .query_map(params![conversationId], row_to_message)
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(messages)
}
