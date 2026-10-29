# builtin_code-js

**Kind:** utility  
**Category:** Utilities  
**Origin:** user  
**Version:** 1

## Description

Sandboxed JavaScript execution block. Runs in a QuickJS sidecar under a Windows Job Object (128 MB RAM, 5s CPU cap, no file system or network access by default). Display name: **Script Vault**.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| input | any | Yes | Input data passed to the JavaScript code as the `input` global |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| output | any | The result of the JavaScript expression |
| tool | tool | Tool output port for agent attachment |

## Parameters

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| code | code | Yes | JavaScript code to execute |
| timeout_ms | number | No | Execution timeout in milliseconds (default: 5000) |

## Side Effects

None (sandboxed — no file system, no network, no child processes).

## Security

- Runs in `opifex-sandbox` sidecar under Windows Job Object.
- Requires explicit "Developer mode" to be enabled.
- Flagged as `contains_custom_code: true` on agent graphs.
- Not portable to Python export targets — emits `NotImplementedError` stub.

## Export Notes

- **LangGraph:** Emits `NotImplementedError` stub with JS source in docstring.
- **OpenAI Agents SDK:** Emits `NotImplementedError` stub.
- **Canonical JSON:** Preserved as `builtin:code-js` node.
