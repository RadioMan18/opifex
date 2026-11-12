//! Everything a node needs from the outside world.
//!
//! The engine owns scheduling and data flow; it deliberately knows nothing about
//! HTTP clients, API keys or the OS credential store. A [`EngineRuntime`] is
//! supplied by the host application (the Tauri layer) and is the only way a run
//! can reach a provider or the network.

use serde_json::Value;
use std::future::Future;
use std::pin::Pin;

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// One turn of a chat conversation.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl ChatMessage {
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".to_string(), content: content.into() }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".to_string(), content: content.into() }
    }
}

/// A completion request. `model` is the block's own choice; when it is absent the
/// host falls back to the app's configured default model.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub model: Option<String>,
    pub system: Option<String>,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f64>,
    /// Ask the provider for JSON output (structured blocks).
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_in: Option<u64>,
    pub tokens_out: Option<u64>,
}

/// A tool call. `kind` names the tool block that asked for it.
#[derive(Debug, Clone)]
pub struct ToolRequest {
    pub kind: String,
    pub url: Option<String>,
    pub method: Option<String>,
    pub headers: Option<Value>,
    pub body: Option<Value>,
    pub code: Option<String>,
    pub input: Option<Value>,
}

/// Host capabilities used by node executors.
pub trait EngineRuntime: Send + Sync {
    fn run_llm<'a>(&'a self, request: LlmRequest) -> BoxFuture<'a, Result<LlmResponse, String>>;

    fn run_tool<'a>(&'a self, request: ToolRequest) -> BoxFuture<'a, Result<Value, String>>;
}

/// Used when a host supplies no provider: every call fails with a message that
/// names the missing piece instead of inventing a result.
pub struct UnavailableRuntime;

impl EngineRuntime for UnavailableRuntime {
    fn run_llm<'a>(&'a self, _request: LlmRequest) -> BoxFuture<'a, Result<LlmResponse, String>> {
        Box::pin(async move {
            Err("No model provider is configured for runs".to_string())
        })
    }

    fn run_tool<'a>(&'a self, request: ToolRequest) -> BoxFuture<'a, Result<Value, String>> {
        Box::pin(async move {
            Err(format!("No tool runtime is configured for {}", request.kind))
        })
    }
}
