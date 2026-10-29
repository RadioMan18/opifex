# builtin_http-request

**Kind:** tool  
**Category:** Tools  
**Origin:** builtin  
**Version:** 1

## Description

Generic HTTP request block. Can be used as a standalone tool or attached to an Agent node. Display name: **Web Courier**.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| url | string | Yes | The request URL |
| body | any | No | Request body (for POST/PUT/PATCH) |
| headers | object | No | Optional HTTP headers |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| response | json | The HTTP response (status, body, headers) |
| tool | tool | Tool output port for agent attachment |

## Parameters

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| method | select | No | GET, POST, PUT, PATCH, DELETE (default: GET) |
| timeout_ms | number | No | Request timeout in milliseconds |

## Side Effects

- **read/write:** Makes outbound HTTP requests. Requires `network` agent grant.

## Export Notes

- **LangGraph:** Emits a tool function with httpx/requests call.
- **OpenAI Agents SDK:** Emits a FunctionTool with HTTP logic.
- **Canonical JSON:** Preserved as `builtin:http-request` node.
