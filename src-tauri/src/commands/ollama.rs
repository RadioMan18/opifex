use crate::db::DbState;
use rusqlite::{params, OptionalExtension};
use reqwest::blocking::Client as BlockingClient;
use reqwest::Client as AsyncClient;
use serde::{Deserialize, Serialize};
use std::process::Command;
use tauri::{State, AppHandle, Emitter};
use tokio_stream::StreamExt;

const OLLAMA_HOST: &str = "http://localhost:11434";
const OPENAI_API_BASE: &str = "https://api.openai.com/v1";
const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OllamaModel {
    pub name: String,
    pub size: Option<i64>,
    pub modified_at: Option<String>,
    pub digest: Option<String>,
}

#[derive(Deserialize)]
struct OllamaListResponse {
    models: Vec<OllamaModelRaw>,
}

#[derive(Deserialize)]
struct OllamaModelRaw {
    name: String,
    size: Option<i64>,
    modified_at: Option<String>,
    digest: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct OllamaMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaChatInput {
    pub model: String,
    pub messages: Vec<OllamaMessage>,
}

#[derive(Serialize, Deserialize)]
pub struct OllamaChatOutput {
    pub content: String,
}

#[derive(Deserialize)]
struct OllamaStreamChunk {
    message: OllamaStreamMessage,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaStreamMessage {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum LlmProvider {
    Ollama,
    OpenAI,
    Anthropic,
}

impl std::fmt::Display for LlmProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LlmProvider::Ollama => write!(f, "ollama"),
            LlmProvider::OpenAI => write!(f, "openai"),
            LlmProvider::Anthropic => write!(f, "anthropic"),
        }
    }
}

impl std::str::FromStr for LlmProvider {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ollama" => Ok(LlmProvider::Ollama),
            "openai" => Ok(LlmProvider::OpenAI),
            "anthropic" => Ok(LlmProvider::Anthropic),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct SetProviderInput {
    pub provider: String,
}

#[derive(Serialize, Deserialize)]
pub struct SetApiKeyInput {
    pub provider: String,
    pub apiKey: String,
}

// OpenAI request/response
#[derive(Serialize)]
struct OpenAIChatRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct OpenAIMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIMessage,
}

// Anthropic request/response
#[derive(Serialize)]
struct AnthropicChatRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    system: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicChatResponse {
    content: Vec<AnthropicContentBlock>,
}

#[derive(Deserialize)]
struct AnthropicContentBlock {
    r#type: String,
    text: Option<String>,
}

#[tauri::command]
pub fn ollama_list_models() -> Result<Vec<OllamaModel>, String> {
    let client = BlockingClient::new();
    let resp: OllamaListResponse = client
        .get(format!("{}/api/tags", OLLAMA_HOST))
        .send()
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    Ok(resp
        .models
        .into_iter()
        .map(|m| OllamaModel {
            name: m.name,
            size: m.size,
            modified_at: m.modified_at,
            digest: m.digest,
        })
        .collect())
}

#[tauri::command]
pub fn ollama_pull_model(model: String) -> Result<String, String> {
    let output = Command::new("ollama")
        .args(["pull", &model])
        .output()
        .map_err(|e| format!("Failed to execute ollama CLI: {}", e))?;

    if output.status.success() {
        Ok(format!("Model '{}' pulled successfully", model))
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("Failed to pull model '{}': {}", model, stderr))
    }
}

#[tauri::command]
pub async fn ollama_chat_stream(
    app: AppHandle,
    provider: String,
    model: String,
    apiKey: Option<String>,
    messages: Vec<OllamaMessage>,
) -> Result<(), String> {
    let provider: LlmProvider = provider.parse().map_err(|e: String| e.to_string())?;

    match provider {
        LlmProvider::Ollama => {
            ollama_chat_stream_impl(app, model, messages).await
        }
        LlmProvider::OpenAI => {
            let key = apiKey.ok_or_else(|| "OpenAI API key not configured".to_string())?;
            openai_chat_stream_impl(app, model, key, messages).await
        }
        LlmProvider::Anthropic => {
            let key = apiKey.ok_or_else(|| "Anthropic API key not configured".to_string())?;
            anthropic_chat_stream_impl(app, model, key, messages).await
        }
    }
}

async fn ollama_chat_stream_impl(
    app: AppHandle,
    model: String,
    messages: Vec<OllamaMessage>,
) -> Result<(), String> {
    let client = AsyncClient::new();
    let resp = client
        .post(format!("{}/api/chat", OLLAMA_HOST))
        .json(&serde_json::json!({
            "model": model,
            "messages": messages,
            "stream": true
        }))
        .send()
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to connect to Ollama: {}", e);
            let _ = app.emit("ollama-stream-error", serde_json::json!({ "error": err_msg }));
            err_msg
        })?;

    let mut stream = resp.bytes_stream();
    let mut has_content = false;
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some((line_str, rest_str)) = buffer.split_once('\n') {
            let line = line_str.trim().to_string();
            let new_buffer = rest_str.to_string();
            buffer = new_buffer;

            if line.is_empty() {
                continue;
            }
            match serde_json::from_str::<OllamaStreamChunk>(&line) {
                Ok(parsed) => {
                    if !parsed.message.content.is_empty() {
                        has_content = true;
                        app.emit(
                            "ollama-stream-chunk",
                            serde_json::json!({ "content": parsed.message.content }),
                        )
                        .map_err(|e| e.to_string())?;
                    }
                    if parsed.done {
                        if !has_content {
                            let _ = app.emit(
                                "ollama-stream-error",
                                serde_json::json!({ "error": "Oppy returned an empty response" }),
                            );
                        }
                        app.emit("ollama-stream-end", serde_json::json!({}))
                            .map_err(|e| e.to_string())?;
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
        }
    }

    if !buffer.trim().is_empty() {
        match serde_json::from_str::<OllamaStreamChunk>(buffer.trim()) {
            Ok(parsed) => {
                if !parsed.message.content.is_empty() {
                    has_content = true;
                    let _ = app.emit(
                        "ollama-stream-chunk",
                        serde_json::json!({ "content": parsed.message.content }),
                    );
                }
                if parsed.done && !has_content {
                    let _ = app.emit(
                        "ollama-stream-error",
                        serde_json::json!({ "error": "Oppy returned an empty response" }),
                    );
                }
            }
            Err(_) => {}
        }
    }

    if !has_content {
        let _ = app.emit(
            "ollama-stream-error",
            serde_json::json!({ "error": "Oppy returned an empty response" }),
        );
    }
    app.emit("ollama-stream-end", serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn openai_chat_stream_impl(
    app: AppHandle,
    model: String,
    api_key: String,
    messages: Vec<OllamaMessage>,
) -> Result<(), String> {
    let client = AsyncClient::new();
    let openai_messages: Vec<OpenAIMessage> = messages
        .into_iter()
        .map(|m| OpenAIMessage { role: m.role, content: m.content })
        .collect();

    let resp = client
        .post(format!("{}/chat/completions", OPENAI_API_BASE))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": openai_messages,
            "stream": true
        }))
        .send()
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to connect to OpenAI: {}", e);
            let _ = app.emit("ollama-stream-error", serde_json::json!({ "error": err_msg }));
            err_msg
        })?;

    let mut stream = resp.bytes_stream();
    let mut has_content = false;
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some((line_str, rest_str)) = buffer.split_once('\n') {
            let line = line_str.trim().to_string();
            let new_buffer = rest_str.to_string();
            buffer = new_buffer;

            if line.is_empty() || !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];
            if data == "[DONE]" {
                if !has_content {
                    let _ = app.emit(
                        "ollama-stream-error",
                        serde_json::json!({ "error": "Oppy returned an empty response" }),
                    );
                }
                app.emit("ollama-stream-end", serde_json::json!({}))
                    .map_err(|e| e.to_string())?;
                return Ok(());
            }
            match serde_json::from_str::<serde_json::Value>(data) {
                Ok(parsed) => {
                    if let Some(delta) = parsed.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("delta")) {
                        if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                            if !content.is_empty() {
                                has_content = true;
                                app.emit(
                                    "ollama-stream-chunk",
                                    serde_json::json!({ "content": content }),
                                )
                                .map_err(|e| e.to_string())?;
                            }
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    if !buffer.trim().is_empty() {
        let line = buffer.trim();
        if line.starts_with("data: ") {
            let data = &line[6..];
            if data != "[DONE]" {
                match serde_json::from_str::<serde_json::Value>(data) {
                    Ok(parsed) => {
                        if let Some(delta) = parsed.get("choices").and_then(|c| c.get(0)).and_then(|c| c.get("delta")) {
                            if let Some(content) = delta.get("content").and_then(|c| c.as_str()) {
                                if !content.is_empty() {
                                    has_content = true;
                                    let _ = app.emit(
                                        "ollama-stream-chunk",
                                        serde_json::json!({ "content": content }),
                                    );
                                }
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
        }
    }

    if !has_content {
        let _ = app.emit(
            "ollama-stream-error",
            serde_json::json!({ "error": "Oppy returned an empty response" }),
        );
    }
    app.emit("ollama-stream-end", serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

async fn anthropic_chat_stream_impl(
    app: AppHandle,
    model: String,
    api_key: String,
    messages: Vec<OllamaMessage>,
) -> Result<(), String> {
    let client = AsyncClient::new();
    let (system, anthropic_messages): (Option<String>, Vec<AnthropicMessage>) = messages
        .into_iter()
        .fold((None, Vec::new()), |(mut sys, mut msgs), m| {
            if m.role == "system" {
                sys = Some(m.content);
            } else {
                msgs.push(AnthropicMessage { role: m.role, content: m.content });
            }
            (sys, msgs)
        });

    let resp = client
        .post(format!("{}/messages", ANTHROPIC_API_BASE))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "system": system,
            "messages": anthropic_messages,
            "stream": true
        }))
        .send()
        .await
        .map_err(|e| {
            let err_msg = format!("Failed to connect to Anthropic: {}", e);
            let _ = app.emit("ollama-stream-error", serde_json::json!({ "error": err_msg }));
            err_msg
        })?;

    let mut stream = resp.bytes_stream();
    let mut has_content = false;
    let mut buffer = String::new();

    while let Some(chunk_result) = stream.next().await {
        let chunk = chunk_result.map_err(|e| format!("Stream error: {}", e))?;
        buffer.push_str(&String::from_utf8_lossy(&chunk));

        while let Some((line_str, rest_str)) = buffer.split_once('\n') {
            let line = line_str.trim().to_string();
            let new_buffer = rest_str.to_string();
            buffer = new_buffer;

            if line.is_empty() || !line.starts_with("data: ") {
                continue;
            }
            let data = &line[6..];
            match serde_json::from_str::<serde_json::Value>(data) {
                Ok(parsed) => {
                    let event_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if event_type == "content_block_delta" {
                        if let Some(delta) = parsed.get("delta").and_then(|d| d.get("text")) {
                            if let Some(content) = delta.as_str() {
                                if !content.is_empty() {
                                    has_content = true;
                                    app.emit(
                                        "ollama-stream-chunk",
                                        serde_json::json!({ "content": content }),
                                    )
                                    .map_err(|e| e.to_string())?;
                                }
                            }
                        }
                    } else if event_type == "message_stop" {
                        if !has_content {
                            let _ = app.emit(
                                "ollama-stream-error",
                                serde_json::json!({ "error": "Oppy returned an empty response" }),
                            );
                        }
                        app.emit("ollama-stream-end", serde_json::json!({}))
                            .map_err(|e| e.to_string())?;
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
        }
    }

    if !buffer.trim().is_empty() {
        let line = buffer.trim();
        if line.starts_with("data: ") {
            let data = &line[6..];
            match serde_json::from_str::<serde_json::Value>(data) {
                Ok(parsed) => {
                    let event_type = parsed.get("type").and_then(|t| t.as_str()).unwrap_or("");
                    if event_type == "content_block_delta" {
                        if let Some(delta) = parsed.get("delta").and_then(|d| d.get("text")) {
                            if let Some(content) = delta.as_str() {
                                if !content.is_empty() {
                                    has_content = true;
                                    let _ = app.emit(
                                        "ollama-stream-chunk",
                                        serde_json::json!({ "content": content }),
                                    );
                                }
                            }
                        }
                    } else if event_type == "message_stop" {
                        if !has_content {
                            let _ = app.emit(
                                "ollama-stream-error",
                                serde_json::json!({ "error": "Oppy returned an empty response" }),
                            );
                        }
                    }
                }
                Err(_) => {}
            }
        }
    }

    if !has_content {
        let _ = app.emit(
            "ollama-stream-error",
            serde_json::json!({ "error": "Oppy returned an empty response" }),
        );
    }
    app.emit("ollama-stream-end", serde_json::json!({}))
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn ollama_chat(
    state: State<'_, DbState>,
    provider: String,
    model: String,
    messages: Vec<OllamaMessage>,
) -> Result<OllamaChatOutput, String> {
    let provider: LlmProvider = provider.parse().map_err(|e: String| e.to_string())?;

    match provider {
        LlmProvider::Ollama => ollama_chat_impl(model, messages),
        LlmProvider::OpenAI => {
            let key = get_api_key(state, "openai".to_string())?
                .ok_or_else(|| "OpenAI API key not configured".to_string())?;
            openai_chat_impl(model, messages, key)
        }
        LlmProvider::Anthropic => {
            let key = get_api_key(state, "anthropic".to_string())?
                .ok_or_else(|| "Anthropic API key not configured".to_string())?;
            anthropic_chat_impl(model, messages, key)
        }
    }
}

fn ollama_chat_impl(model: String, messages: Vec<OllamaMessage>) -> Result<OllamaChatOutput, String> {
    let client = BlockingClient::new();
    let req = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": false
    });

    let resp: serde_json::Value = client
        .post(format!("{}/api/chat", OLLAMA_HOST))
        .json(&req)
        .send()
        .map_err(|e| format!("Failed to connect to Ollama: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    let content = resp
        .get("message")
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(OllamaChatOutput { content })
}

fn openai_chat_impl(model: String, messages: Vec<OllamaMessage>, api_key: String) -> Result<OllamaChatOutput, String> {
    let client = BlockingClient::new();
    let openai_messages: Vec<OpenAIMessage> = messages
        .into_iter()
        .map(|m| OpenAIMessage { role: m.role, content: m.content })
        .collect();

    let resp: serde_json::Value = client
        .post(format!("{}/chat/completions", OPENAI_API_BASE))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "messages": openai_messages,
            "stream": false
        }))
        .send()
        .map_err(|e| format!("Failed to connect to OpenAI: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse OpenAI response: {}", e))?;

    let content = resp
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(OllamaChatOutput { content })
}

fn anthropic_chat_impl(model: String, messages: Vec<OllamaMessage>, api_key: String) -> Result<OllamaChatOutput, String> {
    let client = BlockingClient::new();
    let (system, anthropic_messages): (Option<String>, Vec<AnthropicMessage>) = messages
        .into_iter()
        .fold((None, Vec::new()), |(mut sys, mut msgs), m| {
            if m.role == "system" {
                sys = Some(m.content);
            } else {
                msgs.push(AnthropicMessage { role: m.role, content: m.content });
            }
            (sys, msgs)
        });

    let resp: serde_json::Value = client
        .post(format!("{}/messages", ANTHROPIC_API_BASE))
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": model,
            "max_tokens": 4096,
            "system": system,
            "messages": anthropic_messages,
        }))
        .send()
        .map_err(|e| format!("Failed to connect to Anthropic: {}", e))?
        .json()
        .map_err(|e| format!("Failed to parse Anthropic response: {}", e))?;

    let content = resp
        .get("content")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("text"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(OllamaChatOutput { content })
}

fn get_api_key(state: State<'_, DbState>, provider: String) -> Result<Option<String>, String> {
    let key_name = match provider.as_str() {
        "openai" => "openai_api_key",
        "anthropic" => "anthropic_api_key",
        _ => return Err(format!("Unknown provider: {}", provider)),
    };

    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;

    let api_key: Option<String> = stmt
        .query_row(params![key_name], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(api_key)
}

#[tauri::command]
pub fn ollama_get_default_model(state: State<'_, DbState>) -> Result<Option<String>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;

    let model: Option<String> = stmt
        .query_row(params!["default_model"], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(model)
}

#[tauri::command]
pub fn ollama_set_default_model(state: State<'_, DbState>, model: String) -> Result<(), String> {
    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params!["default_model", model],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn ollama_get_provider(state: State<'_, DbState>) -> Result<Option<String>, String> {
    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;

    let provider: Option<String> = stmt
        .query_row(params!["llm_provider"], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(provider)
}

#[tauri::command]
pub fn ollama_set_provider(state: State<'_, DbState>, provider: String) -> Result<(), String> {
    let _: LlmProvider = provider.parse().map_err(|e: String| e.to_string())?;
    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params!["llm_provider", provider],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn ollama_get_api_key(state: State<'_, DbState>, provider: String) -> Result<Option<String>, String> {
    let key_name = match provider.as_str() {
        "openai" => "openai_api_key",
        "anthropic" => "anthropic_api_key",
        _ => return Err(format!("Unknown provider: {}", provider)),
    };

    let conn = &*state.conn.lock().unwrap();
    let mut stmt = conn
        .prepare("SELECT value FROM settings WHERE key = ?1")
        .map_err(|e| e.to_string())?;

    let api_key: Option<String> = stmt
        .query_row(params![key_name], |row| row.get(0))
        .optional()
        .map_err(|e| e.to_string())?;

    Ok(api_key)
}

#[tauri::command]
pub fn ollama_set_api_key(
    state: State<'_, DbState>,
    input: SetApiKeyInput,
) -> Result<(), String> {
    let _: LlmProvider = input.provider.parse().map_err(|e: String| e.to_string())?;
    let key_name = match input.provider.as_str() {
        "openai" => "openai_api_key",
        "anthropic" => "anthropic_api_key",
        _ => return Err(format!("Unknown provider: {}", input.provider)),
    };

    let conn = &*state.conn.lock().unwrap();
    conn.execute(
        "INSERT INTO settings (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = ?2",
        params![key_name, input.apiKey],
    )
    .map_err(|e| e.to_string())?;

    Ok(())
}
