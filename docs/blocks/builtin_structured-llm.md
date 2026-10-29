# builtin_structured-llm

**Kind:** llm  
**Category:** LLM  
**Origin:** builtin  
**Version:** 1

## Description

Sends a message to the LLM with a JSON schema constraint and returns a typed `result` object. Display name: **Schema Weaver**.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| message | string | Yes | The user message or upstream output to send to the LLM |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| result | json | Schema-typed JSON result from the LLM |

## Parameters

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| schema | json | Yes | JSON Schema defining the expected output shape |
| model | string | No | Override the default model |

## Side Effects

- **read:** Calls the LLM provider API with structured output mode.

## Export Notes

- **LangGraph:** Emits a structured LLM node with output parsing.
- **OpenAI Agents SDK:** Emits an Agent with output_type.
- **Canonical JSON:** Preserved as `builtin:structured-llm` node.
