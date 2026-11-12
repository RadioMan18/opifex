//! Provider wire formats.
//!
//! Providers differ only in where they put the text and the token counts, so
//! the parsing is kept here as pure functions: the host does the HTTP, and the
//! shapes stay testable without a network or an API key.

use crate::runtime::LlmResponse;
use serde_json::Value;

/// Ollama: `message.content`, with `prompt_eval_count`/`eval_count` tokens.
pub fn parse_ollama_completion(body: &Value, model: &str) -> Result<LlmResponse, String> {
    let text = body
        .get("message")
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or_default()
        .to_string();
    if text.trim().is_empty() {
        return Err(format!(
            "Ollama returned an empty completion for {}. Is the model pulled?",
            model
        ));
    }

    Ok(LlmResponse {
        text,
        tokens_in: body.get("prompt_eval_count").and_then(|value| value.as_u64()),
        tokens_out: body.get("eval_count").and_then(|value| value.as_u64()),
    })
}

/// OpenAI: `choices[0].message.content`, with `usage.*_tokens` counts.
pub fn parse_openai_completion(body: &Value, model: &str) -> Result<LlmResponse, String> {
    let text = body
        .get("choices")
        .and_then(|choices| choices.get(0))
        .and_then(|choice| choice.get("message"))
        .and_then(|message| message.get("content"))
        .and_then(|content| content.as_str())
        .unwrap_or_default()
        .to_string();
    if text.trim().is_empty() {
        return Err(format!("OpenAI returned an empty completion for {}", model));
    }

    Ok(LlmResponse {
        text,
        tokens_in: body
            .get("usage")
            .and_then(|usage| usage.get("prompt_tokens"))
            .and_then(|value| value.as_u64()),
        tokens_out: body
            .get("usage")
            .and_then(|usage| usage.get("completion_tokens"))
            .and_then(|value| value.as_u64()),
    })
}

/// Anthropic: `content[0].text`, with `usage.input_tokens`/`output_tokens`.
pub fn parse_anthropic_completion(body: &Value, model: &str) -> Result<LlmResponse, String> {
    let text = body
        .get("content")
        .and_then(|content| content.get(0))
        .and_then(|block| block.get("text"))
        .and_then(|text| text.as_str())
        .unwrap_or_default()
        .to_string();
    if text.trim().is_empty() {
        return Err(format!("Anthropic returned an empty completion for {}", model));
    }

    Ok(LlmResponse {
        text,
        tokens_in: body
            .get("usage")
            .and_then(|usage| usage.get("input_tokens"))
            .and_then(|value| value.as_u64()),
        tokens_out: body
            .get("usage")
            .and_then(|usage| usage.get("output_tokens"))
            .and_then(|value| value.as_u64()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Captured from a local `llama3.2` run of `/api/chat` with `stream: false`.
    #[test]
    fn reads_a_real_ollama_completion() {
        let body = json!({
            "model": "llama3.2",
            "created_at": "2026-11-12T01:43:10.2525131Z",
            "message": { "role": "assistant", "content": "ready" },
            "done": true,
            "done_reason": "stop",
            "total_duration": 15149345300u64,
            "load_duration": 14263930500u64,
            "prompt_eval_count": 32,
            "eval_count": 2
        });

        let response = parse_ollama_completion(&body, "llama3.2").expect("must parse");
        assert_eq!(response.text, "ready");
        assert_eq!(response.tokens_in, Some(32));
        assert_eq!(response.tokens_out, Some(2));
    }

    #[test]
    fn ollama_json_mode_content_survives_intact() {
        let body = json!({
            "message": { "role": "assistant", "content": "{\"ok\": true}" },
            "done": true
        });

        let response = parse_ollama_completion(&body, "llama3.2").expect("must parse");
        assert_eq!(response.text, "{\"ok\": true}");
    }

    #[test]
    fn empty_completions_are_errors_not_empty_answers() {
        let body = json!({ "message": { "role": "assistant", "content": "  " } });
        let error = parse_ollama_completion(&body, "llama3.2").expect_err("must fail");
        assert!(error.contains("empty completion"), "unexpected: {}", error);
        assert!(error.contains("llama3.2"), "model should be named: {}", error);
    }

    #[test]
    fn malformed_bodies_are_errors() {
        let body = json!({ "choices": [] });
        assert!(parse_openai_completion(&body, "gpt-4o-mini").is_err());
        let body = json!({ "content": [] });
        assert!(parse_anthropic_completion(&body, "claude").is_err());
    }

    #[test]
    fn reads_an_openai_completion() {
        let body = json!({
            "choices": [{
                "index": 0,
                "message": { "role": "assistant", "content": "hello there" },
                "finish_reason": "stop"
            }],
            "usage": { "prompt_tokens": 21, "completion_tokens": 5, "total_tokens": 26 }
        });

        let response = parse_openai_completion(&body, "gpt-4o-mini").expect("must parse");
        assert_eq!(response.text, "hello there");
        assert_eq!(response.tokens_in, Some(21));
        assert_eq!(response.tokens_out, Some(5));
    }

    #[test]
    fn reads_an_anthropic_completion() {
        let body = json!({
            "content": [{ "type": "text", "text": "answer from claude" }],
            "stop_reason": "end_turn",
            "usage": { "input_tokens": 14, "output_tokens": 9 }
        });

        let response = parse_anthropic_completion(&body, "claude-3-5-sonnet-latest").expect("must parse");
        assert_eq!(response.text, "answer from claude");
        assert_eq!(response.tokens_in, Some(14));
        assert_eq!(response.tokens_out, Some(9));
    }
}
