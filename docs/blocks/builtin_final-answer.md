# builtin_final-answer

**Kind:** output  
**Category:** Outputs  
**Origin:** builtin  
**Version:** 1

## Description

Terminal output block. Returns the final result to the user. Every graph should end at a Final Answer block.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| message | string/json | Yes | The final result to return |

## Outputs

None.

## Parameters

None.

## Side Effects

None.

## Export Notes

- **LangGraph:** Emits the final return node.
- **OpenAI Agents SDK:** Emits the final output.
- **Canonical JSON:** Preserved as `builtin:final-answer` node.
