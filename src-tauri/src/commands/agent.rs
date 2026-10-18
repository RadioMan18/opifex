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
    pub specJson: serde_json::Value,
    pub model: String,
    pub isStarter: bool,
    pub createdAt: String,
    pub updatedAt: String,
}

#[derive(Deserialize)]
pub struct CreateAgentInput {
    pub name: String,
    pub description: Option<String>,
    pub specJson: serde_json::Value,
    pub model: Option<String>,
    pub isStarter: Option<bool>,
}

#[derive(Deserialize)]
pub struct UpdateAgentInput {
    pub name: Option<String>,
    pub description: Option<Option<String>>,
    pub specJson: Option<serde_json::Value>,
    pub model: Option<String>,
    pub isStarter: Option<bool>,
}

#[derive(Serialize)]
pub struct AgentListItem {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub model: String,
    pub isStarter: bool,
    pub updatedAt: String,
}

fn rowToAgent(row: &rusqlite::Row<'_>) -> rusqlite::Result<Agent> {
    let specStr: String = row.get(3)?;
    Ok(Agent {
        id: row.get(0)?,
        name: row.get(1)?,
        description: row.get(2)?,
        specJson: serde_json::from_str(&specStr)
            .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(e)))?,
        model: row.get(4)?,
        isStarter: row.get::<_, i32>(5)? != 0,
        createdAt: row.get(6)?,
        updatedAt: row.get(7)?,
    })
}

#[tauri::command]
pub fn create_agent(state: State<'_, DbState>, input: CreateAgentInput) -> Result<Agent, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let model = input.model.unwrap_or_else(|| "ollama".to_string());
    let isStarter = input.isStarter.unwrap_or(false);
    let specJson = serde_json::to_string(&input.specJson).map_err(|e| e.to_string())?;

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            id,
            input.name,
            input.description,
            specJson,
            model,
            isStarter as i32,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Agent {
        id,
        name: input.name,
        description: input.description,
        specJson: input.specJson,
        model,
        isStarter,
        createdAt: now.clone(),
        updatedAt: now,
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
        .query_row(params![id], rowToAgent)
        .optional()
        .map_err(|e| e.to_string())?;
    Ok(agent)
}

#[tauri::command]
pub fn list_agents(
    state: State<'_, DbState>,
    includeStarters: Option<bool>,
) -> Result<Vec<AgentListItem>, String> {
    let conn = &*state.conn.lock().unwrap();
    let query = match includeStarters {
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
                isStarter: row.get::<_, i32>(4)? != 0,
                updatedAt: row.get(5)?,
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
    let idForLookup = id.clone();

    {
        let conn = &*state.conn.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT name, description, spec_json, model, is_starter FROM agents WHERE id = ?1",
        ).map_err(|e| e.to_string())?;

        let existing: Option<(String, Option<String>, String, String, i32)> = stmt
            .query_row(params![&idForLookup], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?))
            })
            .optional()
            .map_err(|e| e.to_string())?;

        let Some((name, description, specJsonStr, model, isStarter)) = existing else {
            return Err(format!("Agent not found: {}", idForLookup));
        };

        let newName = input.name.unwrap_or(name);
        let newDescription = input.description.unwrap_or(description);
        let newSpecJson: serde_json::Value = match &input.specJson {
            Some(v) => v.clone(),
            None => serde_json::from_str(&specJsonStr).map_err(|e| e.to_string())?,
        };
        let newModel = input.model.unwrap_or(model);
        let newIsStarter: i32 = input.isStarter.map(|b| b as i32).unwrap_or(isStarter);
        let newSpecJsonStr = serde_json::to_string(&newSpecJson).map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE agents SET name = ?1, description = ?2, spec_json = ?3, model = ?4, is_starter = ?5, updated_at = datetime('now') WHERE id = ?6",
            params![newName, newDescription, newSpecJsonStr, newModel, newIsStarter, idForLookup],
        )
        .map_err(|e| e.to_string())?;
    }

    let agent = get_agent(state, id.clone())?;
    agent.ok_or_else(|| format!("Agent not found: {}", id))
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
                "steps": ["Greet customer", "Understand the issue", "Search knowledge base for relevant answers", "Provide solution or escalate", "Follow up and confirm resolution"],
                "tools": ["knowledge_base", "ticket_system", "customer_database"],
                "knowledgeSources": ["faq.md", "product_docs.pdf", "support_runbooks.md"],
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
                "knowledgeSources": [],
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
                "knowledgeSources": ["company_handbook.pdf", "policy_documents.md", "technical_docs.md", "onboarding_guide.md"],
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
                "knowledgeSources": ["coding_standards.md", "security_guidelines.md", "review_checklist.md"],
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
                "knowledgeSources": ["brand_voice_guide.md", "style_guide.md", "previous_campaigns.md", "messaging_framework.md"],
                "triggers": ["content_request", "newsletter_request", "press_release_request", "announcement_request"],
                "outputs": ["content_draft", "final_copy", "headline_options", "call_to_action_suggestions"]
            }),
        ),
    ];

    let mut created = Vec::new();
    for (name, description, spec) in starters {
        let id = Uuid::new_v4().to_string();
        let specJson = serde_json::to_string(&spec).map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, name, description, specJson, "ollama", 1, now, now],
        )
        .map_err(|e| e.to_string())?;
        created.push(id);
    }

    Ok(created)
}

#[derive(Deserialize)]
pub struct RenameAgentInput {
    pub id: String,
    pub newName: String,
}

#[derive(Deserialize)]
pub struct DuplicateAgentInput {
    pub id: String,
    pub newName: Option<String>,
}

#[derive(Deserialize)]
pub struct ImportAgentInput {
    pub agentJson: String,
}

#[tauri::command]
pub fn export_agent(state: State<'_, DbState>, id: String) -> Result<String, String> {
    let conn = &*state.conn.lock().unwrap();
    let agent = getAgentInner(conn, &id)?
        .ok_or_else(|| format!("Agent not found: {}", id))?;
    serde_json::to_string(&agent).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_agent(state: State<'_, DbState>, input: ImportAgentInput) -> Result<Agent, String> {
    let conn = &*state.conn.lock().unwrap();
    let agent: Agent = serde_json::from_str(&input.agentJson).map_err(|e| e.to_string())?;

    let newId = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let specJson = serde_json::to_string(&agent.specJson).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            newId,
            agent.name,
            agent.description,
            specJson,
            agent.model,
            0,
            now,
            now
        ],
    )
    .map_err(|e| e.to_string())?;

    Ok(Agent {
        id: newId,
        name: agent.name,
        description: agent.description,
        specJson: agent.specJson,
        model: agent.model,
        isStarter: false,
        createdAt: now.clone(),
        updatedAt: now.clone(),
    })
}

#[tauri::command]
pub fn rename_agent(state: State<'_, DbState>, input: RenameAgentInput) -> Result<Agent, String> {
    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "UPDATE agents SET name = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![input.newName, input.id],
    )
    .map_err(|e| e.to_string())?;

    getAgentInner(conn, &input.id)?
        .ok_or_else(|| format!("Agent not found: {}", input.id))
}

#[tauri::command]
pub fn duplicate_agent(state: State<'_, DbState>, input: DuplicateAgentInput) -> Result<Agent, String> {
    let conn = &*state.conn.lock().unwrap();
    let agent = getAgentInner(conn, &input.id)?
        .ok_or_else(|| format!("Agent not found: {}", input.id))?;

    let newId = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let newName = input.newName.unwrap_or_else(|| format!("Copy of {}", agent.name));
    let specJson = serde_json::to_string(&agent.specJson).map_err(|e| e.to_string())?;

    conn.execute(
        "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![newId, newName, agent.description, specJson, agent.model, 0, now, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(Agent {
        id: newId,
        name: newName,
        description: agent.description,
        specJson: agent.specJson,
        model: agent.model,
        isStarter: false,
        createdAt: now.clone(),
        updatedAt: now.clone(),
    })
}

fn getAgentInner(conn: &Connection, id: &str) -> Result<Option<Agent>, String> {
    let mut stmt = conn.prepare(
        "SELECT id, name, description, spec_json, model, is_starter, created_at, updated_at
         FROM agents WHERE id = ?1",
    ).map_err(|e| e.to_string())?;

    let agent: Option<Agent> = stmt
        .query_row(params![id], |row| {
            let specStr: String = row.get(3)?;
            Ok(Agent {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                specJson: serde_json::from_str(&specStr)
                    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(3, Type::Text, Box::new(e)))?,
                model: row.get(4)?,
                isStarter: row.get::<_, i32>(5)? != 0,
                createdAt: row.get(6)?,
                updatedAt: row.get(7)?,
            })
        })
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(agent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::run_migrations;
    use rusqlite::Connection;

    fn testConn() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        run_migrations(&conn).unwrap();
        conn
    }

    #[test]
    fn testSchemaCreatesTables() {
        let conn = testConn();
        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agents'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);

        let count: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='plan_versions'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn testStarterAgentsSeeded() {
        let conn = testConn();
        let spec = serde_json::json!({"goal": "test"});

        for i in 0..6 {
            conn.execute(
                "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                rusqlite::params![
                    uuid::Uuid::new_v4().to_string(),
                    format!("Starter {}", i),
                    None::<String>,
                    serde_json::to_string(&spec).unwrap(),
                    "ollama",
                    1,
                    chrono::Utc::now().to_rfc3339(),
                    chrono::Utc::now().to_rfc3339(),
                ],
            ).unwrap();
        }

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM agents WHERE is_starter = 1", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 6);
    }

    #[test]
    fn testAgentCrudDirectSql() {
        let conn = testConn();
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        let spec = serde_json::json!({"goal": "test goal"});

        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, "Test Agent", "A test", serde_json::to_string(&spec).unwrap(), "ollama", 0, now, now],
        )
        .unwrap();

        let row: Option<(String, String)> = conn
            .query_row("SELECT name, spec_json FROM agents WHERE id = ?1", rusqlite::params![id], |row| {
                Ok((row.get(0)?, row.get(1)?))
            })
            .optional()
            .unwrap();

        assert!(row.is_some());
        let (name, specJson) = row.unwrap();
        assert_eq!(name, "Test Agent");
        let spec: serde_json::Value = serde_json::from_str(&specJson).unwrap();
        assert_eq!(spec, serde_json::json!({"goal": "test goal"}));
    }

    #[test]
    fn testExportImportRoundtrip() {
        let conn = testConn();
        let now = chrono::Utc::now().to_rfc3339();
        let id = uuid::Uuid::new_v4().to_string();
        let spec = serde_json::json!({"goal": "export test", "steps": ["step1"]});

        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![id, "Export Agent", "desc", serde_json::to_string(&spec).unwrap(), "ollama", 0, now, now],
        )
        .unwrap();

        let agentRow: Option<(String, String, Option<String>, String, String, i32, String, String)> = conn
            .query_row(
                "SELECT id, name, description, spec_json, model, is_starter, created_at, updated_at FROM agents WHERE id = ?1",
                rusqlite::params![id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?, row.get(5)?, row.get(6)?, row.get(7)?)),
            )
            .optional()
            .unwrap();

        assert!(agentRow.is_some());
        let (aid, name, desc, specJson, model, isStarter, _createdAt, _updatedAt) = agentRow.unwrap();
        assert_eq!(aid, id);
        assert_eq!(name, "Export Agent");
        assert_eq!(model, "ollama");
        assert_eq!(isStarter, 0);

        let parsedSpec: serde_json::Value = serde_json::from_str(&specJson).unwrap();
        assert_eq!(parsedSpec, spec);

        let newId = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            rusqlite::params![newId, "Imported Copy", desc, specJson, model, 0, chrono::Utc::now().to_rfc3339(), chrono::Utc::now().to_rfc3339()],
        )
        .unwrap();

        let count: i32 = conn.query_row("SELECT COUNT(*) FROM agents WHERE name = 'Imported Copy'", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 1);
    }
}
