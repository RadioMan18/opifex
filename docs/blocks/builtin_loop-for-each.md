# builtin_loop-for-each

**Kind:** control  
**Category:** Control Flow  
**Origin:** builtin  
**Version:** 1

## Description

Iterates over an array of items, executing a sub-graph body for each item. Bounded — max iterations enforced to prevent runaway loops.

## Inputs

| Port | Type | Required | Description |
|------|------|----------|-------------|
| items | array<any> | Yes | The array to iterate over |
| body | sub-graph | Yes | The sub-graph to execute for each item |

## Outputs

| Port | Type | Description |
|------|------|-------------|
| results | array<any> | Collected results from each iteration |
| errors | json | Any errors encountered during iteration |

## Parameters

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| max_iterations | number | No | Maximum iterations (default: 100) |

## Side Effects

- **per body:** Depends on the sub-graph body contents.

## Export Notes

- **LangGraph:** Emits a for-loop node over the StateGraph.
- **OpenAI Agents SDK:** Emits a loop construct.
- **Canonical JSON:** Preserved as `builtin:loop-for-each` node with embedded sub-graph.
