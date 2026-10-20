use crate::db::DbState;
use rusqlite::Connection;
use rusqlite::params;
use rusqlite::types::Type;
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
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
pub struct CreateAgentInput {
    pub name: String,
    pub description: Option<String>,
    pub spec_json: serde_json::Value,
    pub model: Option<String>,
    pub is_starter: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateAgentInput {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub spec_json: Option<serde_json::Value>,
    pub model: Option<String>,
    pub is_starter: Option<bool>,
}

#[derive(Serialize)]
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
    let starters: Vec<(&str, &str, serde_json::Value)> = vec![
        (
            "Customer Support Agent",
            "Answers customer inquiries and handles support tickets, grounded in company FAQ and product documentation.",
            serde_json::json!({
                "goal": "Provide helpful, accurate customer support by resolving inquiries and managing support tickets",
                "steps": ["Greet customer", "Understand issue", "Search knowledge base for relevant answers", "Provide solution or escalate", "Follow up and confirm resolution"],
                "tools": ["knowledge_base", "ticket_system", "customer_database"],
                "knowledge_sources": ["faq.md", "product_docs.pdf", "support_runbooks.md"],
                "triggers": ["customer_message", "new_support_ticket", "escalation_request"],
                "outputs": ["customer_response", "support_ticket", "escalation_notice"]
            }),
        ),
        (
            "Research Agent",
            "Conducts web research and produces structured reports with citations on any business topic.",
            serde_json::json!({
                "goal": "Research a topic thoroughly and produce a structured, cited report",
                "steps": ["Clarify research scope and questions", "Perform web searches across multiple sources", "Extract and verify key facts", "Synthesize findings into a structured report", "Add citations and source links"],
                "tools": ["web_search", "document_summarizer", "citation_formatter"],
                "knowledge_sources": [],
                "triggers": ["research_request", "market_analysis_request", "competitor_research_request"],
                "outputs": ["research_report", "source_citations", "key_findings_summary"]
            }),
        ),
        (
            "RAG Knowledge Agent",
            "Answers questions by searching and reasoning over the company's internal knowledge base and documents.",
            serde_json::json!({
                "goal": "Answer employee questions accurately using internal company documents and knowledge base",
                "steps": ["Receive user question", "Search internal knowledge base for relevant documents", "Retrieve and rank most relevant passages", "Synthesize answer grounded in retrieved sources", "Provide answer with source references"],
                "tools": ["document_search", "semantic_search", "qa_over_docs"],
                "knowledge_sources": ["company_handbook.pdf", "policy_documents.md", "technical_docs.md", "onboarding_guide.md"],
                "triggers": ["employee_question", "policy_lookup", "procedure_inquiry"],
                "outputs": ["answered_question", "source_references", "confidence_score"]
            }),
        ),
        (
            "Code Review Agent",
            "Reviews pull requests, identifies issues, suggests fixes, and recommends approval or changes.",
            serde_json::json!({
                "goal": "Improve code quality by reviewing pull requests and providing actionable feedback",
                "steps": ["Review pull request code changes", "Identify bugs, security issues, and style violations", "Suggest specific code fixes and improvements", "Check test coverage and documentation", "Recommend approve, request changes, or comment"],
                "tools": ["linter", "static_analyzer", "test_runner", "github_api"],
                "knowledge_sources": ["coding_standards.md", "security_guidelines.md", "review_checklist.md"],
                "triggers": ["pull_request_opened", "pull_request_updated", "code_review_request"],
                "outputs": ["review_comments", "approval_status", "suggested_changes", "severity_flags"]
            }),
        ),
        (
            "Content Writer Agent",
            "Drafts on-brand internal and external communications including announcements, newsletters, and press releases.",
            serde_json::json!({
                "goal": "Write engaging, on-brand content for internal and external audiences",
                "steps": ["Understand content objective and target audience", "Research key messages and brand guidelines", "Draft outline and structure", "Write and refine content", "Review against brand voice and style guide"],
                "tools": ["grammar_checker", "brand_voice_checker", "word_counter", "readability_analyzer"],
                "knowledge_sources": ["brand_voice_guide.md", "style_guide.md", "previous_campaigns.md", "messaging_framework.md"],
                "triggers": ["content_request", "newsletter_request", "press_release_request", "announcement_request"],
                "outputs": ["content_draft", "final_copy", "headline_options", "call_to_action_suggestions"]
            }),
        ),
    ];

    let mut created = Vec::new();
    for (name, description, spec) in starters {
        let id = Uuid::new_v4().to_string();
        let spec_json = serde_json::to_string(&spec).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, name, description, spec_json, "ollama", 1, now, now],
        )
        .map_err(|e| e.to_string())?;
        created.push(id);
    }

    Ok(created)
}