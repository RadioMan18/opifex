//! Host side of a run: the model providers and the tools.
//!
//! The engine asks for completions and tool calls through `EngineRuntime`; this
//! is where keys, HTTP and provider shapes live, so the engine stays testable
//! without a network.

use opifex_engine::providers::{
    parse_anthropic_completion, parse_ollama_completion, parse_openai_completion,
};
use opifex_engine::{BoxFuture, EngineRuntime, LlmRequest, LlmResponse, ToolRequest};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

use crate::db::DbState;

const OLLAMA_HOST: &str = "http://localhost:11434";
const OPENAI_API_BASE: &str = "https://api.openai.com/v1";
const ANTHROPIC_API_BASE: &str = "https://api.anthropic.com/v1";

const DEFAULT_OLLAMA_MODEL: &str = "llama3.2";
const DEFAULT_OPENAI_MODEL: &str = "gpt-4o-mini";
const DEFAULT_ANTHROPIC_MODEL: &str = "claude-3-5-sonnet-latest";

const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const TOOL_TIMEOUT: Duration = Duration::from_secs(30);
/// Tool output is fed back into prompts, so it is capped rather than trusted.
const MAX_TOOL_BYTES: usize = 64 * 1024;
const ANTHROPIC_MAX_TOKENS: u32 = 4096;

/// Everything a run needs from the host, resolved once when the run starts.
pub struct AppRuntime {
    client: Client,
    provider: String,
    api_key: Option<String>,
    default_model: Option<String>,
}

impl AppRuntime {
    /// Snapshots the provider, default model and key. The key is read from the
    /// OS credential store; the settings row is only a fallback for databases
    /// written before keys moved out of the table.
    pub fn from_settings(db: &DbState) -> Self {
        let (provider, default_model, legacy_key) = {
            let conn = &*db.conn.lock().unwrap_or_else(|e| e.into_inner());
            let provider = read_setting(conn, "provider").unwrap_or_else(|| "ollama".to_string());
            let default_model = read_setting(conn, "default_model");
            let key_name = api_key_name(&provider);
            let legacy_key = key_name
                .and_then(|name| read_setting(conn, name))
                .filter(|value| !value.is_empty());
            (provider, default_model, legacy_key)
        };

        let api_key = api_key_name(&provider)
            .and_then(crate::secrets::get_secret)
            .or(legacy_key);

        Self {
            client: Client::builder()
                .timeout(REQUEST_TIMEOUT)
                .build()
                .unwrap_or_else(|_| Client::new()),
            provider,
            api_key,
            default_model,
        }
    }

    fn model_or_default(&self, model: Option<String>) -> String {
        if let Some(model) = model.filter(|value| !value.trim().is_empty()) {
            return model;
        }
        if let Some(model) = self.default_model.clone().filter(|value| !value.is_empty()) {
            return model;
        }
        match self.provider.as_str() {
            "openai" => DEFAULT_OPENAI_MODEL.to_string(),
            "anthropic" => DEFAULT_ANTHROPIC_MODEL.to_string(),
            _ => DEFAULT_OLLAMA_MODEL.to_string(),
        }
    }

    fn require_key(&self) -> Result<String, String> {
        self.api_key
            .clone()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                format!(
                    "{} API key is not configured. Add it in Configure API Keys.",
                    provider_label(&self.provider)
                )
            })
    }
}

impl EngineRuntime for AppRuntime {
    fn run_llm<'a>(&'a self, request: LlmRequest) -> BoxFuture<'a, Result<LlmResponse, String>> {
        Box::pin(async move {
            let model = self.model_or_default(request.model.clone());
            match self.provider.as_str() {
                "openai" => self.openai_chat(&model, &request).await,
                "anthropic" => self.anthropic_chat(&model, &request).await,
                _ => self.ollama_chat(&model, &request).await,
            }
        })
    }

    fn run_tool<'a>(&'a self, request: ToolRequest) -> BoxFuture<'a, Result<Value, String>> {
        Box::pin(async move {
            match request.kind.as_str() {
                "http-request" => self.http_request(&request).await,
                "code-js" => Err(
                    "JavaScript blocks are not available: this build ships no sandbox runtime."
                        .to_string(),
                ),
                other => Err(format!("Unknown tool block: {}", other)),
            }
        })
    }
}

impl AppRuntime {
    async fn ollama_chat(&self, model: &str, request: &LlmRequest) -> Result<LlmResponse, String> {
        let mut messages = Vec::new();
        if let Some(system) = request.system.clone() {
            messages.push(json!({ "role": "system", "content": system }));
        }
        messages.extend(
            request
                .messages
                .iter()
                .map(|message| json!({ "role": message.role, "content": message.content })),
        );

        let mut payload = json!({
            "model": model,
            "messages": messages,
            "stream": false,
        });
        if let Some(temperature) = request.temperature {
            payload["options"] = json!({ "temperature": temperature });
        }
        if request.json {
            payload["format"] = json!("json");
        }

        let response = self
            .client
            .post(format!("{}/api/chat", OLLAMA_HOST))
            .json(&payload)
            .send()
            .await
            .map_err(|error| format!("Failed to reach Ollama: {}", error))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|error| format!("Failed to read Ollama response: {}", error))?;
        if !status.is_success() {
            return Err(format!("Ollama returned {}: {}", status, body));
        }

        parse_ollama_completion(&body, model)
    }

    async fn openai_chat(&self, model: &str, request: &LlmRequest) -> Result<LlmResponse, String> {
        let api_key = self.require_key()?;

        let mut messages = Vec::new();
        if let Some(system) = request.system.clone() {
            messages.push(json!({ "role": "system", "content": system }));
        }
        messages.extend(
            request
                .messages
                .iter()
                .map(|message| json!({ "role": message.role, "content": message.content })),
        );

        let mut payload = json!({
            "model": model,
            "messages": messages,
            "stream": false,
        });
        if let Some(temperature) = request.temperature {
            payload["temperature"] = json!(temperature);
        }
        if request.json {
            payload["response_format"] = json!({ "type": "json_object" });
        }

        let response = self
            .client
            .post(format!("{}/chat/completions", OPENAI_API_BASE))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&payload)
            .send()
            .await
            .map_err(|error| format!("Failed to reach OpenAI: {}", error))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|error| format!("Failed to read OpenAI response: {}", error))?;
        if !status.is_success() {
            return Err(format!("OpenAI returned {}: {}", status, body));
        }

        parse_openai_completion(&body, model)
    }

    async fn anthropic_chat(&self, model: &str, request: &LlmRequest) -> Result<LlmResponse, String> {
        let api_key = self.require_key()?;

        let messages: Vec<Value> = request
            .messages
            .iter()
            .filter(|message| message.role != "system")
            .map(|message| json!({ "role": message.role, "content": message.content }))
            .collect();
        if messages.is_empty() {
            return Err("Anthropic needs at least one user message".to_string());
        }

        let mut payload = json!({
            "model": model,
            "max_tokens": ANTHROPIC_MAX_TOKENS,
            "messages": messages,
        });
        if let Some(system) = request.system.clone() {
            payload["system"] = json!(system);
        }
        if let Some(temperature) = request.temperature {
            payload["temperature"] = json!(temperature);
        }

        let response = self
            .client
            .post(format!("{}/messages", ANTHROPIC_API_BASE))
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await
            .map_err(|error| format!("Failed to reach Anthropic: {}", error))?;

        let status = response.status();
        let body: Value = response
            .json()
            .await
            .map_err(|error| format!("Failed to read Anthropic response: {}", error))?;
        if !status.is_success() {
            return Err(format!("Anthropic returned {}: {}", status, body));
        }

        parse_anthropic_completion(&body, model)
    }

    /// Runs the Web Courier block: one request, capped response, no credentials
    /// from the URL's own host are sent beyond what the user wrote in Headers.
    async fn http_request(&self, request: &ToolRequest) -> Result<Value, String> {
        let url = request
            .url
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "Web Courier needs a URL".to_string())?;
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err(format!("Web Courier only speaks http(s): {}", url));
        }

        let method = request
            .method
            .clone()
            .unwrap_or_else(|| "GET".to_string())
            .to_uppercase();
        let method = reqwest::Method::from_bytes(method.as_bytes())
            .map_err(|_| format!("Unsupported HTTP method: {}", method))?;

        let mut builder = self
            .client
            .request(method.clone(), &url)
            .timeout(TOOL_TIMEOUT);

        if let Some(Value::Object(headers)) = &request.headers {
            for (name, value) in headers {
                if let Some(text) = value.as_str() {
                    builder = builder.header(name.as_str(), text);
                }
            }
        }

        if let Some(body) = &request.body {
            builder = match body {
                Value::String(text) => builder.body(text.clone()),
                other => builder.json(other),
            };
        }

        let response = builder
            .send()
            .await
            .map_err(|error| format!("Request to {} failed: {}", url, error))?;
        let status = response.status();
        let bytes = response
            .bytes()
            .await
            .map_err(|error| format!("Failed to read response from {}: {}", url, error))?;

        let truncated = bytes.len() > MAX_TOOL_BYTES;
        let slice = &bytes[..bytes.len().min(MAX_TOOL_BYTES)];
        let text = String::from_utf8_lossy(slice).to_string();
        let body = serde_json::from_str::<Value>(&text).unwrap_or(Value::String(text));

        Ok(json!({
            "status": status.as_u16(),
            "method": method.as_str(),
            "url": url,
            "truncated": truncated,
            "body": body,
        }))
    }
}

fn read_setting(conn: &rusqlite::Connection, key: &str) -> Option<String> {
    use rusqlite::OptionalExtension;

    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        rusqlite::params![key],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .ok()
    .flatten()
}

fn api_key_name(provider: &str) -> Option<&'static str> {
    match provider {
        "openai" => Some("openai_api_key"),
        "anthropic" => Some("anthropic_api_key"),
        _ => None,
    }
}

fn provider_label(provider: &str) -> &str {
    match provider {
        "openai" => "OpenAI",
        "anthropic" => "Anthropic",
        _ => "Ollama",
    }
}
