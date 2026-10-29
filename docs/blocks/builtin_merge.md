# builtin_merge

**Kind:** control  
**Category:** Control Flow  
**Origin:** builtin  
**Version:** 1

## Description

Merges multiple inputs into a single output. Skip-tolerant — missing inputs do not block the merge. Display name: **Stream Merger**.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| (multiple) | any | No | One or more input ports (dynamically added) |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| merged | any | Combined output from all connected inputs |

## Parameters

None.

## Side Effects

None.

## Export Notes

- **LangGraph:** Emits a merge/join node.
- **OpenAI Agents SDK:** Emits a list concatenation.
- **Canonical JSON:** Preserved as `builtin:merge` node.
