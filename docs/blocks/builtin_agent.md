# builtin_agent

**Kind:** agent  
**Category:** Agents  
**Origin:** builtin  
**Version:** 1

## Description

Autonomous agent node that can invoke attached tools. Supports hybrid tool wiring — tools can be connected via canvas edges OR attached through the inline inspector drawer.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| prompt | string | Yes | The task prompt for the agent |
| tools | tool | Yes (multi) | One or more tool blocks attached to this agent |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| answer | string | The agent's final answer |
| trace | json | Execution trace of tool calls |
| status | string | Execution status (success, error, etc.) |

## Parameters

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| max_iterations | number | No | Maximum tool-calling iterations (default: 8) |
| model | string | No | Override the default model |

## Side Effects

- **per tools:** Depends on attached tools (network, file, etc.).

## Export Notes

- **LangGraph:** Emits an Agent node with tool bindings.
- **OpenAI Agents SDK:** Emits an Agent with tool functions.
- **Canonical JSON:** Preserved as `builtin:agent` node with attached tool references.
