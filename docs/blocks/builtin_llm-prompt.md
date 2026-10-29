# builtin_llm-prompt

**Kind:** llm  
**Category:** LLM  
**Origin:** builtin  
**Version:** 1

## Description

Sends a message to the selected LLM provider and returns the completion as a string. Display name: **Prompt Weaver**.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| message | string | Yes | The user message or upstream output to send to the LLM |
| system | string | No | Optional system prompt/instruction |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| completion | string | The LLM's text completion |

## Parameters

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| model | string | No | Override the default model for this block |
| temperature | number | No | Sampling temperature (0.0–1.0) |
| max_tokens | number | No | Maximum tokens in the completion |

## Side Effects

- **read:** Calls the LLM provider API (Ollama/OpenAI/Anthropic).

## Export Notes

- **LangGraph:** Emits an LLM call node with the configured model.
- **OpenAI Agents SDK:** Emits an Agent with the system prompt and model.
- **Canonical JSON:** Preserved as `builtin:llm-prompt` node.
