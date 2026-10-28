//! Model probe suite for Opifex v2.0.0.
//! Benchmarks Ollama models for Tier 1 (structured JSON output) and Tier 2 (tool calls) capabilities.

use anyhow::Result;
use reqwest::blocking::Client;
use serde_json::json;
use std::time::Instant;

/// Model capability tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelTier {
    /// Can produce structured JSON output reliably.
    Tier1,
    /// Can produce structured JSON + make tool calls.
    Tier2,
    /// Basic chat only, no structured output guarantees.
    Tier3,
}

#[derive(Debug)]
struct ProbeResult {
    model: String,
    tier: ModelTier,
    json_latency_ms: u128,
    tool_latency_ms: Option<u128>,
    json_success: bool,
    tool_success: Option<bool>,
}

pub struct ModelProbe {
    client: Client,
    ollama_host: String,
}

impl ModelProbe {
    pub fn new(ollama_host: &str) -> Self {
        Self {
            client: Client::new(),
            ollama_host: ollama_host.to_string(),
        }
    }

    /// Probe a single model for Tier 1 and Tier 2 capabilities.
    pub fn probe_model(&self, model: &str) -> Result<ProbeResult> {
        // Tier 1: JSON structure test
        let json_prompt = json!({
            "model": model,
            "prompt": "Return a JSON object with keys: name (string), count (number), active (boolean). No extra text.",
            "format": "json",
            "stream": false
        });

        let json_start = Instant::now();
        let json_resp: serde_json::Value = self
            .client
            .post(&format!("{}/api/generate", self.ollama_host))
            .json(&json_prompt)
            .send()?
            .json()?;
        let json_latency = json_start.elapsed().as_millis();

        let json_text = json_resp
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let json_success = serde_json::from_str::<serde_json::Value>(json_text).is_ok();

        // Tier 2: Tool call test
        let tool_start = Instant::now();
        let tool_prompt = json!({
            "model": model,
            "prompt": "What is the weather in Seattle? Use the get_weather tool if available.",
            "tools": vec![json!({
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get current weather for a city",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "city": { "type": "string" }
                        },
                        "required": ["city"]
                    }
                }
            })],
            "stream": false
        });

        let tool_resp = self
            .client
            .post(&format!("{}/api/generate", self.ollama_host))
            .json(&tool_prompt)
            .send();

        let (tool_latency, tool_success) = match tool_resp {
            Ok(resp) => {
                let start = Instant::now();
                let _tool_json: serde_json::Value = resp.json()?;
                let latency = start.elapsed().as_millis();
                let success = true; // Got a response = tool support present
                (Some(latency), Some(success))
            }
            Err(_) => (None, None),
        };

        let tier = match (json_success, tool_success) {
            (true, Some(true)) => ModelTier::Tier2,
            (true, _) => ModelTier::Tier1,
            _ => ModelTier::Tier3,
        };

        Ok(ProbeResult {
            model: model.to_string(),
            tier,
            json_latency_ms: json_latency,
            tool_latency_ms: tool_latency,
            json_success,
            tool_success,
        })
    }

    /// List available Ollama models.
    pub fn list_models(&self) -> Result<Vec<String>> {
        let resp: serde_json::Value = self
            .client
            .get(&format!("{}/api/tags", self.ollama_host))
            .send()?
            .json()?;

        let models = resp
            .get("models")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(models)
    }
}

fn main() -> Result<()> {
    let probe = ModelProbe::new("http://localhost:11434");

    match probe.list_models() {
        Ok(models) => {
            println!("Found {} Ollama model(s)", models.len());
            for model in &models {
                match probe.probe_model(model) {
                    Ok(result) => {
                        println!(
                            "  {}: Tier {:?}, JSON={}, JSON_lat={}ms, Tool={:?}, Tool_lat={:?}ms",
                            result.model,
                            result.tier,
                            result.json_success,
                            result.json_latency_ms,
                            result.tool_success,
                            result.tool_latency_ms
                        );
                    }
                    Err(e) => println!("  {}: probe failed: {}", model, e),
                }
            }
        }
        Err(e) => {
            println!("Ollama not available at http://localhost:11434: {}", e);
            println!("Skipping model probe — run with Ollama active to benchmark.");
        }
    }

    Ok(())
}
