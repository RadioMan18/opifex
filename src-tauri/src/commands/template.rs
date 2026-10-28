use crate::db::DbState;
use rusqlite::OptionalExtension;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use tauri::State;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub added_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddTemplateSourceInput {
    pub name: String,
    pub url: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct InstalledTemplate {
    pub id: String,
    pub template_id: String,
    pub version: String,
    pub installed_at: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallTemplateInput {
    pub template_id: String,
    pub version: String,
    pub source_id: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallTemplateResult {
    pub agent_id: String,
    pub name: String,
    pub template_id: String,
    pub version: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CatalogTemplateEntry {
    id: String,
    name: String,
    description: Option<String>,
    version: String,
    author: Option<String>,
    download_url: Option<String>,
    checksum: Option<String>,
    spec_json: Option<serde_json::Value>,
    model: Option<String>,
    is_starter: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TemplateEntry {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub version: String,
    pub author: Option<String>,
    pub download_url: Option<String>,
    pub checksum: Option<String>,
    pub spec_json: Option<serde_json::Value>,
    pub model: Option<String>,
    pub is_starter: bool,
}

fn row_to_template_source(row: &rusqlite::Row<'_>) -> rusqlite::Result<TemplateSource> {
    Ok(TemplateSource {
        id: row.get(0)?,
        name: row.get(1)?,
        url: row.get(2)?,
        enabled: row.get::<_, i32>(3)? != 0,
        added_at: row.get(4)?,
    })
}

fn row_to_installed_template(row: &rusqlite::Row<'_>) -> rusqlite::Result<InstalledTemplate> {
    Ok(InstalledTemplate {
        id: row.get(0)?,
        template_id: row.get(1)?,
        version: row.get(2)?,
        installed_at: row.get(3)?,
    })
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}

#[tauri::command]
pub fn add_template_source(
    state: State<'_, DbState>,
    input: AddTemplateSourceInput,
) -> Result<TemplateSource, String> {
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();

    let conn = &*state.conn.lock().unwrap_or_else(|e| e.into_inner());
    conn.execute(
        "INSERT INTO template_sources (id, name, url, enabled, added_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, input.name, input.url, 1, now],
    )
    .map_err(|e| e.to_string())?;

    Ok(TemplateSource {
        id,
        name: input.name,
        url: input.url,
        enabled: true,
        added_at: now,
    })
}

#[tauri::command]
pub fn list_template_sources(state: State<'_, DbState>) -> Result<Vec<TemplateSource>, String> {
    let conn = &*state.conn.lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = conn.prepare(
        "SELECT id, name, url, enabled, added_at FROM template_sources ORDER BY added_at DESC",
    ).map_err(|e| e.to_string())?;

    let sources: Vec<TemplateSource> = stmt
        .query_map([], row_to_template_source)
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(sources)
}

#[tauri::command]
pub async fn fetch_templates(state: State<'_, DbState>, source_id: String) -> Result<Vec<TemplateEntry>, String> {
    // First, get source info from DB (synchronous, lock released before await)
    let source: Option<TemplateSource> = {
        let conn = &*state.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(
            "SELECT id, name, url, enabled, added_at FROM template_sources WHERE id = ?1",
        ).map_err(|e| e.to_string())?;

        stmt.query_row(params![source_id], row_to_template_source)
            .optional()
            .map_err(|e| e.to_string())?
    };

    let source = source.ok_or_else(|| format!("Template source not found: {}", source_id))?;

    if !source.enabled {
        return Err(format!("Template source is disabled: {}", source.name));
    }

    // Now make async HTTP request (no DB lock held)
    let client = reqwest::Client::new();
    let resp = client
        .get(&source.url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch catalog: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!("Catalog returned status {}", resp.status()));
    }

    let entries: Vec<CatalogTemplateEntry> = resp.json().await.map_err(|e| {
        format!("Failed to parse catalog JSON: {}", e)
    })?;

    Ok(entries
        .into_iter()
        .map(|e| TemplateEntry {
            id: e.id,
            name: e.name,
            description: e.description,
            version: e.version,
            author: e.author,
            download_url: e.download_url,
            checksum: e.checksum,
            spec_json: e.spec_json,
            model: e.model,
            is_starter: e.is_starter.unwrap_or(false),
        })
        .collect())
}

#[tauri::command]
pub async fn install_template(
    state: State<'_, DbState>,
    input: InstallTemplateInput,
) -> Result<InstallTemplateResult, String> {
    // Phase 1: Read from DB synchronously (lock released before await)
    let source: TemplateSource = {
        let conn = &*state.conn.lock().unwrap_or_else(|e| e.into_inner());

        let mut stmt = conn.prepare(
            "SELECT id, name, url, enabled, added_at FROM template_sources WHERE id = ?1",
        ).map_err(|e| e.to_string())?;

        let source: Option<TemplateSource> = stmt
            .query_row(params![input.source_id], row_to_template_source)
            .optional()
            .map_err(|e| e.to_string())?;

        let source = source.ok_or_else(|| format!("Template source not found: {}", input.source_id))?;

        if !source.enabled {
            return Err(format!("Template source is disabled: {}", source.name));
        }

        source
    };

    // Check if already installed (separate lock scope)
    {
        let conn = &*state.conn.lock().unwrap_or_else(|e| e.into_inner());
        let mut stmt = conn.prepare(
            "SELECT id FROM installed_templates WHERE template_id = ?1 AND version = ?2",
        ).map_err(|e| e.to_string())?;
        let existing: Option<String> = stmt
            .query_row(params![&input.template_id, &input.version], |row| row.get(0))
            .optional()
            .map_err(|e| e.to_string())?;
        if existing.is_some() {
            return Err(format!("Template {} version {} already installed", input.template_id, input.version));
        }
    }

    // Phase 2: Fetch catalog via async HTTP (no DB lock held)
    let client = reqwest::Client::new();
    let resp = client
        .get(&source.url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch catalog: {}", e))?;

    let entries: Vec<CatalogTemplateEntry> = resp.json().await.map_err(|e| {
        format!("Failed to parse catalog JSON: {}", e)
    })?;

    let entry = entries
        .into_iter()
        .find(|e| e.id == input.template_id && e.version == input.version)
        .ok_or_else(|| {
            format!(
                "Template {} version {} not found in source {}",
                input.template_id, input.version, source.name
            )
        })?;

    // Download and verify checksum if download_url is provided (async)
    let spec_json = if let Some(spec) = entry.spec_json {
        spec
    } else if let Some(download_url) = &entry.download_url {
        let resp = client
            .get(download_url)
            .timeout(std::time::Duration::from_secs(60))
            .send()
            .await
            .map_err(|e| format!("Failed to download template: {}", e))?;

        let bytes = resp.bytes().await.map_err(|e| {
            format!("Failed to read template download: {}", e)
        })?;

        // Verify checksum
        if let Some(expected_checksum) = &entry.checksum {
            let actual = sha256_hex(&bytes);
            let expected = expected_checksum.strip_prefix("sha256:").unwrap_or(expected_checksum);
            if actual != expected {
                return Err(format!(
                    "Checksum mismatch for template {}: expected {}, got {}",
                    input.template_id, expected, actual
                ));
            }
        }

        // Try to parse as JSON (agent spec or zip manifest)
        let text = String::from_utf8_lossy(&bytes);
        serde_json::from_str::<serde_json::Value>(&text).map_err(|e| {
            format!("Failed to parse template JSON: {}", e)
        })?
    } else {
        return Err(format!("Template {} has no spec_json or download_url", input.template_id));
    };

    // Validate required AgentPlan fields
    let goal = spec_json.get("goal")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Invalid template: missing required field 'goal'".to_string())?;
    if goal.trim().is_empty() {
        return Err("Invalid template: field 'goal' must not be empty".to_string());
    }
    if spec_json.get("steps").is_none() {
        return Err("Invalid template: missing required field 'steps'".to_string());
    }

    // Phase 3: Write to DB synchronously (lock acquired and released within this block)
    let id = Uuid::new_v4().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let model = entry.model.clone().unwrap_or_else(|| "ollama".to_string());
    let is_starter = entry.is_starter.unwrap_or(false);

    let spec_json_str = serde_json::to_string(&spec_json).map_err(|e| e.to_string())?;

    {
        let conn = &*state.conn.lock().unwrap_or_else(|e| e.into_inner());
        conn.execute(
            "INSERT INTO agents (id, name, description, spec_json, model, is_starter, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                entry.name,
                entry.description,
                spec_json_str,
                model,
                is_starter as i32,
                now,
                now
            ],
        )
        .map_err(|e| e.to_string())?;

        // Record installed template
        let template_install_id = Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO installed_templates (id, template_id, version, installed_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![template_install_id, input.template_id, input.version, now],
        )
        .map_err(|e| e.to_string())?;
    }

    Ok(InstallTemplateResult {
        agent_id: id,
        name: entry.name,
        template_id: input.template_id,
        version: input.version,
    })
}

#[tauri::command]
pub fn list_installed_templates(state: State<'_, DbState>) -> Result<Vec<InstalledTemplate>, String> {
    let conn = &*state.conn.lock().unwrap_or_else(|e| e.into_inner());
    let mut stmt = conn.prepare(
        "SELECT id, template_id, version, installed_at FROM installed_templates ORDER BY installed_at DESC",
    ).map_err(|e| e.to_string())?;

    let templates: Vec<InstalledTemplate> = stmt
        .query_map([], row_to_installed_template)
        .map_err(|e| e.to_string())?
        .map(|r| r.map_err(|e| e.to_string()))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(templates)
}