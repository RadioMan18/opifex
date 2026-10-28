# builtin_condition

**Kind:** control  
**Category:** Control Flow  
**Origin:** builtin  
**Version:** 1

## Description

Branches execution based on a boolean condition. Outputs on either `true` or `false` port.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| value | any | Yes | The value to evaluate |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| true | any | Emitted when condition is truthy |
| false | any | Emitted when condition is falsy |

## Parameters

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| condition | string | No | Expression to evaluate (e.g. `{{ input.count > 5 }}`) |

## Side Effects

None.

## Export Notes

- **LangGraph:** Emits a conditional edge in the StateGraph.
- **OpenAI Agents SDK:** Emits an if/else branch.
- **Canonical JSON:** Preserved as `builtin:condition` node.
