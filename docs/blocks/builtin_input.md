# builtin_input

**Kind:** trigger  
**Category:** Triggers  
**Origin:** builtin  
**Version:** 1

## Description

Entry point for an agent graph. Every graph requires exactly one Input block. Emits a `message` (string) or `payload` (json) output to start execution.

## Inputs

None.

## Outputs

| Port | Type | Description |
|------|------|-------------|
| message | string | The initial user message or trigger payload |
| payload | json | Structured trigger payload (webhook, schedule, etc.) |

## Parameters

None.

## Side Effects

None.

## Export Notes

- **LangGraph:** Maps to the graph entry node.
- **OpenAI Agents SDK:** Maps to the initial user input.
- **Canonical JSON:** Preserved as `builtin:input` node.
