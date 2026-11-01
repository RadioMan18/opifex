# Opifex v2.0.0 Plan — "Block Studio" (Rev 2)

**Version:** 2.0.0
**Codename:** Block Studio
**Status:** Draft, rev 2 (supersedes rev 1)
**Parent release:** Opifex v1.0.0 (MIT, open source)
**Owner:** Dave (solo, agent-assisted development)
**Date:** 2026-10-28
**Platform:** Windows only (Tauri v2 on WebView2)

---

## 0. Changes from Rev 1

| Area | Rev 1 | Rev 2 |
|------|-------|-------|
| Audience | "Technical users" (implicit) | Technical community for MVP; non-technical is a later goal |
| Runtime | None (export-first) | **Per-node execution engine**: run node, run to here, run all, pin data, iterate |
| Custom code | Opt-in WebWorker iframe | Out-of-process QuickJS worker under a Windows Job Object; capability grants |
| Cycles | Banned | Allowed **only** through bounded constructs (Agent, Loop) |
| Block catalog | 15 blocks | **Top 10**, each held to a "bulletproof" definition of done |
| Oppy | Block-authoring assistant | On-demand "Clippy-style" assistant: analyze, explain, propose edits |
| Export | One JSON format | Canonical JSON + generated code for **LangGraph** and **OpenAI Agents SDK** |
| Validation | Duplicated in TS and Rust | **One authority: Rust core**; TS only has a conformance-tested `canConnect` |
| Canvas lib | `@xyflow/vue` (not an official package) | **Vue Flow** (`@vue-flow/core`, MIT, community-maintained), behind an adapter |
| Schema | `type UNIQUE`, no versions | Versioned block definitions, `(id, version)` key, graph schema versioning |
| Inspector | Bottom panel | Right panel; bottom panel for Problems and Run log |
| Sub-graph editing | Modal | **Drill-in with breadcrumbs** (flagged for your confirmation, see §24) |
| Data mapping | Static parameters only | Restricted, statically checkable expression language |

---

## 1. Vision and Positioning

Opifex v2 is a **local-first visual agent studio** for technical users. You compose typed blocks on a canvas, **run them node by node against real models and tools**, pin and edit data to iterate quickly, validate the whole graph statically, and **export portable artifacts**: a canonical JSON definition and runnable LangGraph / OpenAI Agents SDK projects.

Differentiators versus n8n and similar tools:

1. **Local-first and portable.** Everything works offline against Ollama; the canonical artifact is yours.
2. **Static guarantees.** Expressions, ports, loops and bounds are verified before anything runs.
3. **Compiles to real frameworks**, not just to a proprietary runtime.
4. **Oppy**, an on-demand assistant that sees your design surface, explains it, and proposes structured edits you can preview and undo.

### Design principles

- **P1 Static where possible.** If it can be checked before running, it is.
- **P2 Run it for real.** Fast feedback (per-node run, pinning) beats theory.
- **P3 Portable by construction.** Every block declares what it exports to which target.
- **P4 Bulletproof over broad.** Ten blocks that never surprise beat twenty that sometimes do.
- **P5 Accessible by default.** Keyboard and screen-reader paths are first-class, not a final audit.
- **P6 AI on demand.** Oppy never interrupts; it appears when summoned.
- **P7 Safe with untrusted input.** Imported graphs, model output and user code are all untrusted.

---

## 2. Scope and Cut Line

| Capability | v2.0.0 (MVP) | v2.1 | v3 |
|-----------|:-:|:-:|:-:|
| Canvas, palette, inspector, undo/redo, autosave | ✔ | | |
| Top 10 blocks (§8) | ✔ | | |
| Per-node run, pin, stale tracking, run history | ✔ | | |
| Bounded Agent and Loop | ✔ | | |
| Sandboxed Code block | ✔ | | |
| Logic check (Rust authority) + export gate | ✔ | | |
| Export: canonical JSON, LangGraph, OpenAI Agents SDK | ✔ | | |
| Oppy: analyze, explain, propose edits | ✔ | | |
| Custom blocks from sub-graphs (Path A) | ✔ | | |
| v1 agent migration and starter templates | ✔ | | |
| Model capability probing and tiers | ✔ | | |
| Webhook / Schedule triggers (needs hosting) | | ✔ | |
| Retrieval / Embed + knowledge base module | | ✔ | |
| MCP tool block | | ✔ | |
| n8n-format export | | ✔ | |
| Config-form blocks (Path B) | | ✔ | |
| Parallel for-each, Delay, Set Variable, Log | | ✔ | |
| Human-in-the-loop pause/approve | | ✔ | |
| Multi-tenancy, auth, cloud sync, marketplace | | | ✔ |
| Non-technical mode | | | ✔ |

**Non-goals for v2.0.0:** server hosting of triggers, multi-tenancy, cross-platform builds, arbitrary server-side code execution, template marketplace.

---

## 3. Architecture

Business logic lives in Rust crates that are testable without the UI and usable headless.

```
opifex/
  crates/
    opifex-core/      graph schema, types, validator, expression language, migrations
    opifex-engine/    scheduler, node runtime, providers, run store, cancellation
    opifex-sandbox/   child-process binary: QuickJS worker (Code block)
    opifex-export/    Exporter trait + json, langgraph, openai exporters
    opifex-cli/       headless: validate, run, export  (also used in CI)
  src-tauri/          thin Tauri commands over the crates (no business logic)
  src/                Vue 3 + TS frontend
```

**Rules**

- `opifex-core` is the **single authority** for validation. The frontend does not re-implement checks.
- The one exception is `canConnect(sourcePort, targetPort)` in TS, used for drag-time feedback. It is verified against **shared conformance vectors** (JSON test cases generated from Rust) in both test suites.
- The frontend never calls providers directly; all LLM/HTTP traffic goes through `opifex-engine`.
- `opifex-cli` enables `opifex validate|run|export graph.json`, which doubles as the CI harness.

**Canvas library:** Vue Flow (`@vue-flow/core`), MIT, community-maintained and based on React Flow. Risk: single-maintainer dependency. Mitigation: all canvas access goes through a thin `CanvasAdapter` interface; Spike S1 verifies Vue 3 compatibility, performance and keyboard behavior before commitment. Fallback is a custom SVG/Canvas renderer behind the same adapter.

**Storage:** SQLite (WAL mode), `PRAGMA user_version` migrations, automatic DB file backup before any migration.

---

## 4. Data Model

### 4.1 SQLite (additive migrations; v1 tables untouched)

```sql
-- User/imported block definitions. Built-in blocks live in code, not the DB.
CREATE TABLE IF NOT EXISTS block_definitions (
    id TEXT NOT NULL,
    version INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,             -- free text, palette grouping
    kind TEXT NOT NULL,                 -- 'trigger'|'llm'|'agent'|'tool'|'control'|'output'|'utility'|'subgraph'
    origin TEXT NOT NULL CHECK (origin IN ('user','imported')),
    spec_json TEXT NOT NULL,            -- serialized BlockDefinition
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (id, version)
);

ALTER TABLE agents ADD COLUMN default_graph_id TEXT;   -- snake_case, consistent with schema

CREATE TABLE IF NOT EXISTS agent_graphs (
    id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL REFERENCES agents(id),
    name TEXT NOT NULL,
    schema_version INTEGER NOT NULL,
    graph_json TEXT NOT NULL,
    graph_hash TEXT NOT NULL,
    is_default INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Version history (replaces v1 plan_versions for graphs). Retention-capped.
CREATE TABLE IF NOT EXISTS graph_snapshots (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
    kind TEXT NOT NULL CHECK (kind IN ('auto','manual','pre-migration','pre-ai-edit')),
    label TEXT,
    graph_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS validation_runs (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
    graph_hash TEXT NOT NULL,
    status TEXT NOT NULL,               -- 'valid'|'invalid'
    error_count INTEGER NOT NULL DEFAULT 0,
    warning_count INTEGER NOT NULL DEFAULT 0,
    errors_json TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
    graph_hash TEXT NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('live','mock')),
    scope_json TEXT NOT NULL,           -- {kind:'all'|'toNode'|'node', nodeId?}
    trigger_json TEXT,
    status TEXT NOT NULL,               -- 'running'|'succeeded'|'failed'|'cancelled'
    error_json TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT
);

CREATE TABLE IF NOT EXISTS node_runs (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    parent_node_run_id TEXT,            -- tool calls under an Agent, iterations under a Loop
    node_path TEXT NOT NULL,            -- 'n3' or 'n7/iter2/n1' for nested sub-graphs
    attempt INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL,               -- 'queued'|'running'|'succeeded'|'failed'|'skipped'|'cancelled'
    input_json TEXT,
    output_json TEXT,
    error_json TEXT,
    truncated INTEGER NOT NULL DEFAULT 0,
    tokens_in INTEGER, tokens_out INTEGER,
    started_at TEXT, ended_at TEXT
);

CREATE TABLE IF NOT EXISTS pinned_data (
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL,
    port_id TEXT NOT NULL,
    value_json TEXT NOT NULL,
    source_node_run_id TEXT,            -- null if hand-edited
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (graph_id, node_id, port_id)
);

CREATE TABLE IF NOT EXISTS agent_grants (   -- runtime capability grants per agent
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('network')),
    value TEXT NOT NULL,                -- e.g. host name
    granted_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (agent_id, kind, value)
);

CREATE TABLE IF NOT EXISTS model_capabilities (
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    probe_json TEXT NOT NULL,
    tier INTEGER NOT NULL,
    probed_at TEXT NOT NULL,
    PRIMARY KEY (provider, model)
);
```

Retention: keep the last 50 validation runs and 25 auto-snapshots per graph; keep the last N runs per graph (default 50), with large payloads truncated and flagged `truncated`.

### 4.2 TypeScript types (frontend mirror of `opifex-core`)

```ts
export type PortType =
  | 'string' | 'number' | 'boolean' | 'json' | 'any'
  | 'message' | 'messages'
  | { array: PortType }
  | 'tool' | 'error';

export interface BlockPort {
  id: string;
  name: string;
  type: PortType;
  direction: 'input' | 'output';
  required: boolean;
  multi?: boolean;            // allows multiple incoming edges (e.g. Agent tools); inputs only
  schema?: unknown;           // optional JSON Schema for 'json'
  description?: string;
}

export type ParamValue =
  | { mode: 'literal'; value: unknown }
  | { mode: 'expression'; expr: string };     // see §6.2

export interface BlockParameter {
  id: string;
  label: string;
  type: 'string' | 'number' | 'boolean' | 'select' | 'multi-select'
      | 'textarea' | 'json' | 'code' | 'credential' | 'model';
  required: boolean;
  allowExpression: boolean;
  defaultValue?: unknown;
  options?: string[];
  min?: number; max?: number;   // enforced by validator, not just the form
  description?: string;
}

export interface BlockDefinition {
  id: string;
  version: number;
  name: string;
  description?: string;
  category: string;
  kind: 'trigger'|'llm'|'agent'|'tool'|'control'|'output'|'utility'|'subgraph';
  icon?: string;
  inputs: BlockPort[];
  outputs: BlockPort[];
  parameters: BlockParameter[];
  sideEffects: 'none' | 'read' | 'write';         // drives mock/live prompts
  portability: Record<ExportTarget, 'native' | 'emulated' | 'unsupported'>;
  origin: 'builtin' | 'user' | 'imported';
  subGraph?: AgentGraph;                           // only for origin != builtin, kind 'subgraph'
}

export type ExportTarget = 'json' | 'langgraph' | 'openai-agents';

export interface NodeInstance {
  id: string;
  blockId: string;
  blockVersion: number;            // pinned; upgrades are explicit
  position: { x: number; y: number };
  parameters: Record<string, ParamValue>;
  credentialSlots?: Record<string, string>;        // param id -> slot name (never the secret)
  errorPolicy?: { mode: 'stop' | 'route'; retries?: number; backoffMs?: number };
  body?: AgentGraph;               // inline body for Loop only
  note?: string;
}

export interface EdgeInstance {
  id: string;
  source: string; sourceHandle: string;
  target: string; targetHandle: string;
}

export interface AgentGraph {
  schema: 'opifex.graph/1';
  id: string;
  name: string;
  description?: string;
  nodes: NodeInstance[];
  edges: EdgeInstance[];
  annotations?: Annotation[];      // sticky notes/frames; ignored by validation and export logic
  viewport?: { x: number; y: number; zoom: number };
  metadata?: Record<string, unknown>;
}

export interface ValidationError {
  severity: 'error' | 'warning';
  code: string;
  message: string;
  suggestion?: string;
  nodePath?: string[];             // ['n7','n1'] for nested; absent for graph-level
  portId?: string;
  edgeId?: string;
}

export interface ValidationResult {
  valid: boolean;                  // no errors (warnings allowed)
  errors: ValidationError[];
  nodeCount: number;
  edgeCount: number;
  graphHash: string;
  durationMs: number;
}
```

Notes:

- Custom block **instances reference** a definition by `(blockId, blockVersion)`; they do not copy it. Only the Loop's body is inline. This removes the rev 1 duplication/divergence risk.
- Editing a custom block creates a new version; existing graphs stay pinned until the user upgrades, with a diff and re-validation.
- Secrets never appear in graph JSON, only slot names.

---

## 5. Type System

- Types: `string`, `number`, `boolean`, `json` (optionally with JSON Schema), `any`, `message`, `messages`, `array<T>`, `tool`, `error`.
- **Compatibility:** exact match; `any` is a wildcard in either direction (validator emits a *warning* when `any` hides an unknowable mismatch); `array<T>` is compatible if element types are; `json` ↔ `json` is compatible, and with schemas on both sides a structural subset check is a *warning* in v2.0.0 (hard error later).
- **No implicit coercion** (number to string etc.). Explicit conversion is done through expressions.
- Drag-time: handles of incompatible type are dimmed while dragging, and the drop is rejected with a text explanation. The logic check is the safety net, not the only gate.

---

## 6. Expressions

### 6.1 Why
Without data mapping, wired blocks can only pass whole values. Mapping is the heart of n8n-style usability.

### 6.2 Design
- Syntax: `{{ ... }}` inside string/textarea/json parameters that set `allowExpression`.
- **References** only: `{{ input.message }}`, `{{ nodes.n3.completion }}`, `{{ item }}`, `{{ index }}` (inside Loop bodies), `{{ args.city }}` (inside tool blocks). Dotted/indexed paths only (`a.b[0].c`).
- **A small whitelist of pure functions** (e.g. `json()`, `join()`, `length()`, `default()`, `upper()`, `lower()`, `trim()`, `slice()`), no loops, no arbitrary code. Arbitrary logic goes in the Code block.
- Evaluated in Rust (`opifex-core`); parsed at validation time.
- **Static checks:** syntax valid; referenced node exists and is an *ancestor* (or in scope); referenced port exists; result type compatible with the parameter type where inferable. Codes: `INVALID_EXPRESSION`, `UNRESOLVED_REFERENCE`, `REFERENCE_NOT_UPSTREAM`.
- Deliberately restricted so expressions **translate cleanly to Python** in the code exporters.
- Implementation: hand-written parser (pest/nom), fuzzed with cargo-fuzz, with golden tests. Spike S4 confirms the function set.

---

## 7. Execution Model (normative)

This section is the contract that the engine, the validator and every exporter must honor.

1. **Run.** One run handles one trigger event. v2.0.0 graphs have **exactly one Input block** (`NO_TRIGGER`, `MULTIPLE_TRIGGERS` are errors).
2. **Scheduling.** A node becomes *ready* when all required inputs have a value or a skip token. Ready nodes run concurrently up to a concurrency limit (default 4). The event log order is deterministic; results do not depend on completion order.
3. **Values** are JSON-serializable and typed. Hard cap per value (default 10 MB) → `VALUE_TOO_LARGE`.
4. **Branching and skipping.** Condition emits to exactly one of `true`/`false`. The untaken output delivers a **skip token**. A node whose required inputs are all skip tokens is skipped, and skipping propagates. A node with a mix of values and skips is a validation error unless its block declares skip-tolerant inputs (Merge only).
5. **Merge semantics** (Merge block): `first` (first non-skipped value, error if none), `all` (wait for all non-skipped, emit array in input-port order).
6. **Errors.** Each node has an error policy: `stop` (default; run fails, in-flight siblings are cancelled gracefully) or `route` (node gets an `error` output port emitting an `ErrorValue {code, message, nodeId, retryable}`). Optional bounded retries with backoff (`retries` ≤ 5) for blocks that declare themselves retryable (LLM, HTTP).
7. **Timeouts and cancellation.** Per-node timeout and a run timeout; user cancel propagates to providers, HTTP and the sandbox. Cancelled nodes are recorded as `cancelled`.
8. **Bounded loops.** Cycles are permitted **only** inside Agent (internal tool loop) and Loop (iteration). Both have *required* bounds, validated statically and enforced at runtime (§8). Any other cycle is `CYCLE_DETECTED`.
9. **Loop scope is closed.** A Loop body sees only its boundary inputs (`item`, `index`, plus declared pass-through inputs). It cannot reference parent nodes. This keeps validation local and exports simple.
10. **Sub-graph boundaries** use internal-only `Subgraph Input` / `Subgraph Output` blocks (not shown in the main palette).
11. **Side effects.** Blocks declare `sideEffects`. In **live** test runs, `write` blocks (e.g. HTTP non-GET) require a per-run confirmation; in **mock** mode they use pinned data or return a clearly marked mock value.
12. **Staleness.** Each node has a *signature* = hash(its config + signatures of its upstream nodes + trigger fingerprint). If the signature differs from the one recorded when its data was produced, that data is **stale**. Pinned data is never silently discarded; it is flagged stale and kept.

---

## 8. Block Catalog (v2.0.0: top 10)

| # | Block | Kind | Inputs | Outputs | Side effects |
|---|-------|------|--------|---------|--------------|
| 1 | **Input** | trigger | none | `message` (string) or `payload` (json), per configured input schema | none |
| 2 | **LLM Prompt** | llm | `message` (string), `system` (string, opt) | `completion` (string) | read |
| 3 | **Structured LLM** | llm | `message` (string) | `result` (json, schema-typed) | read |
| 4 | **Agent** | agent | `prompt` (string), `tools` (tool, multi) | `answer` (string), `trace` (json), `status` (string) | per tools |
| 5 | **HTTP Request** | tool | `url`, `body`, `headers` (via params or ports) | `response` (json: status, headers, body); also a `tool` output | read/write by method |
| 6 | **Condition** | control | `value` (any) | `true`, `false` | none |
| 7 | **Loop (For Each)** | control | `items` (array<any>), pass-through inputs | `results` (array<any>), `errors` (json) | per body |
| 8 | **Merge** | control | multiple (skip-tolerant) | `merged` (any) | none |
| 9 | **Code (sandboxed JS)** | utility | `input` (any) | `output` (any); also a `tool` output | none (network only by grant) |
| 10 | **Final Answer** | output | `message` (string/json) | none | none |

### Block contracts (high points; full specs live in `docs/blocks/<block>.md`)

- **Input:** configurable schema; default is a chat message. In test runs the trigger payload is entered or pinned. Maps to a chat/HTTP-style entry in exports.
- **LLM Prompt:** provider/model/credential via parameters; temperature and max tokens are range-validated; streams tokens to the run inspector; records token counts; retryable.
- **Structured LLM:** output must conform to a JSON Schema parameter. Uses provider-native structured output where available, otherwise prompt + parse. **Bounded repair loop** (default 2 retries) on parse/schema failure; error code `SCHEMA_VIOLATION` with the raw output attached. Requires model tier ≥ 1 (§17).
- **Agent:** `maxIterations` **required** (default 8, hard cap 25), plus optional token budget and timeout. Each iteration = one model call plus tool calls. Stops on final answer or bound; on bound reached, behavior is configurable (`error` | `return partial`, default `error`). Tool calls are validated against the tool's declared input schema before execution. Tool errors are returned to the model as tool messages (bounded retries). Only tools **wired to its `tools` port** are callable. `trace` records every step. Requires tier ≥ 2.
- **HTTP Request:** method, URL, headers, body, timeout, response size cap, redirect cap; credentials injected from slots only; non-2xx is a defined, typed outcome (configurable: error or pass through). In **tool mode** the block declares name, description and an input schema; args are referenced as `{{ args.x }}`. Hosts require a network grant (§13).
- **Condition:** boolean expression over its input (restricted expression language), with typed comparison functions; non-boolean results are an error, not truthiness.
- **Loop:** body is an inline sub-graph (drill-in editing). Sequential in v2.0.0; `maxItems` required (default 1000, hard cap 10,000); per-item error policy `stop | skip | collect`; result order matches input order.
- **Merge:** modes per §7.5; the only block allowed skip/value mixes.
- **Code:** JS in QuickJS (§13). Input and output are JSON; memory, CPU time and output size are limited; `console.log` is captured into the run log. Marked **non-portable** to Python exports (§15).
- **Final Answer:** exactly one is recommended; multiple are allowed only on mutually exclusive branches (validator checks reachability exclusivity conservatively and warns otherwise).

### Block Definition of Done (applies to all ten)

A block is not "done" until it has:

1. A written spec (inputs, outputs, params with ranges, semantics, error taxonomy, side effects, portability).
2. Parameter validation enforced in Rust (not only in the form).
3. Unit tests: happy path, boundary values, every error code, cancellation, timeout, skip-token handling.
4. Property or fuzz tests where input is structured (expressions, JSON parsing, schema repair).
5. Redaction test: secrets never appear in logs, node_runs, pins or exports.
6. Golden export tests for each target (or an explicit `unsupported` declaration with a test).
7. Inspector form that is keyboard-operable and passes automated a11y checks.
8. Deterministic execution test using the fake provider (§19).

---

## 9. Run, Pin, Iterate

### 9.1 Run controls
- **Run node:** executes one node using its inputs from pinned data or the latest non-stale upstream outputs. If an input has neither, the UI says exactly which upstream node needs to run (and offers "Run to here").
- **Run to here:** runs the minimal ancestor set, skipping any ancestor whose output is pinned.
- **Run all:** full run from the Input block.
- **Cancel** is always visible while a run is active.
- **Mode toggle:** `Live` or `Mock`. `write` side effects in Live require confirmation per run.

### 9.2 Node states (each shown with color + icon + text)
idle, queued, running, succeeded, failed, skipped, stale, pinned, cancelled.

### 9.3 Run data inspector (right panel tab)
- Input / Output tabs per port: tree view, table view for arrays, raw view; copy and download.
- **Pin** a value (from a run, or hand-edited JSON validated against the port type). Pinned data is clearly badged on the node and in the inspector.
- Streaming output for LLM blocks; token and timing stats; error detail with the suggestion text.
- Large values are truncated in the UI and in storage (flagged), never silently.

### 9.4 Data hygiene
- Pins and run data may contain sensitive content (model output, API responses). They are stored locally, **excluded from export by default**, and a secret scrubber (exact-value match on all resolved credential values) runs before persistence and before anything is sent to Oppy.
- Run history list per graph with status, mode, duration, tokens; "re-run with same trigger."

---

## 10. Canvas UX

### 10.1 Layout
```
+--------------------------------------------------------------------------+
| Top bar: agent name | Run all | Cancel | Live/Mock | Validate | Export |   |
|          Oppy button | Settings                                          |
+--------------+------------------------------------------+----------------+
| Palette      |                                          | Inspector      |
| (collapsible)|              CANVAS                      | [Parameters]   |
| search,      |   nodes, edges, minimap, breadcrumb      | [Run data]     |
| categories   |   (sub-graph drill-in)                   | [Settings]     |
+--------------+------------------------------------------+----------------+
| Bottom panel (collapsible): [Problems] [Run log] [Run history] [Outline]  |
+--------------------------------------------------------------------------+
```
Oppy opens as a dockable/floating panel (resizable), not a permanent pane.

### 10.2 Interaction set (MVP)
- Add blocks: drag from palette, click-to-add at viewport center, **quick-add** (`Tab` or `/` opens search at the cursor/selection), "+" on edges to insert a block, **drop onto an edge to insert**, auto-connect new node to the selected node when types match.
- Edit: multi-select, copy/paste (clipboard JSON, validated on paste), duplicate, delete, **undo/redo** (command stack; session-persistent), align/distribute, snap to grid, **auto-layout** (dagre).
- Navigate: pan/zoom, minimap, fit view, **Ctrl+K command palette**, breadcrumb for sub-graph drill-in.
- Annotate: sticky notes and frames (annotations; ignored by validation and execution; kept in JSON metadata).
- Autosave with debounce; crash-safe via WAL; "unsaved/saved" indicator; snapshots on demand and automatically before AI edits and migrations.
- Node card: category-colored header, title, state icon + label, error/warning badge with count, pin badge, per-node toolbar (Run, Pin, more menu).
- Context menu: Duplicate, Delete, Group into sub-graph block, Collapse/Expand, Run node, Run to here, Ask Oppy about this, Error details, Open block docs.

### 10.3 Edges and ports
- Output → input only. Fan-out allowed. **Fan-in rejected** on single inputs; allowed on `multi` ports (Agent `tools`) and Merge.
- Edge color keyed to source port type **plus** a pattern/label (not color alone). Dashed = warning (e.g. `any` hiding a type).
- While dragging a connection, compatible handles highlight and incompatible ones dim with a reason tooltip.

### 10.4 Custom blocks (Path A only in v2.0.0)
Select nodes → "Group into block" → define boundary: unconnected inputs/outputs of the selection become ports, with names and types editable. Saved as `block_definitions` (origin `user`, version 1) with drill-in editing and explicit versioning (§4.2). Exports bundle referenced custom definitions.

---

## 11. Logic Check

### 11.1 When it runs
- **On demand:** Validate button (primary).
- **Continuous, lightweight:** debounced, non-blocking, runs the cheap checks only (dangling references, unconnected required inputs, bound missing); cycle analysis runs on demand and pre-export.
- **Pre-export gate:** see §15.4.
- **Pre-run:** a run refuses to start on `error`-severity problems and links to them.

### 11.2 Checks

| Code | Scope | Severity |
|------|-------|----------|
| `CYCLE_DETECTED` (outside Agent/Loop) | Graph | error |
| `BOUND_MISSING` / `BOUND_EXCEEDS_CAP` (Agent, Loop) | Node | error |
| `UNCONNECTED_INPUT` | Node/Port | error |
| `MULTIPLE_EDGES_INTO_SINGLE_INPUT` | Edge | error |
| `TYPE_MISMATCH` | Edge | error |
| `ANY_TYPE_UNCHECKED` | Edge | warning |
| `MISSING_REQUIRED_PARAM`, `PARAM_OUT_OF_RANGE` | Node | error |
| `INVALID_EXPRESSION`, `UNRESOLVED_REFERENCE`, `REFERENCE_NOT_UPSTREAM` | Node | error |
| `UNKNOWN_BLOCK`, `BLOCK_VERSION_MISSING`, `BLOCK_UPDATE_AVAILABLE` | Node | error / error / info |
| `DANGLING_EDGE`, `DUPLICATE_NODE_ID`, `DUPLICATE_PORT_ID` | Graph/Edge | error |
| `NO_TRIGGER`, `MULTIPLE_TRIGGERS` | Graph | error |
| `NO_OUTPUT` | Graph | warning |
| `UNREACHABLE_NODE` | Node | warning |
| `SKIP_VALUE_MIX` (non-Merge) | Node | error |
| `CREDENTIAL_SLOT_UNSET` | Node | error (live run), warning (export) |
| `MODEL_CAPABILITY_INSUFFICIENT` | Node | warning |
| `SUBGRAPH_BOUNDARY_MISSING` | Node | error |
| `EMPTY_GRAPH` | Graph | warning |
| `NON_PORTABLE_BLOCK` (for the chosen export target) | Node | error at export |

Validation recurses into Loop bodies and custom block definitions; nested errors carry `nodePath`.

### 11.3 Algorithm
1. Resolve block definitions (builtin + pinned custom); report unknowns.
2. Build the adjacency structure; verify IDs unique and edges well-formed.
3. Strongly-connected-component pass (Tarjan) to find cycles; allow only cycles that are internal to Agent/Loop (which have no edge cycles in the graph itself) and flag the rest.
4. Topological order; reachability from the Input block.
5. Per-node checks: required inputs, parameter ranges, expression parse and reference resolution (ancestor check uses the topological order), port-type compatibility per edge, skip/value mixing.
6. Bounds and capability warnings.
7. Return `ValidationResult` with a stable ordering (graph-level, then by topological position).

Property tests assert: never panics on arbitrary graph JSON, cycle detection is exact, and validation is deterministic.

---

## 12. Accessibility (WCAG 2.2 AA)

Accessibility covers the whole canvas, not only errors.

### 12.1 Errors and states (retained from rev 1, extended)
1. Color + **SVG icon with accessible name** + **text** for every error/warning/state (no emoji icons).
2. Problems panel is semantic (`<ul>`/`<li>`), each item: severity, node name, port, code, message, suggestion, and a **Go to** button that focuses and pans to the node.
3. `aria-live="polite"` region announces "Logic check complete: N errors, M warnings" plus the first error.
4. `aria-describedby` links a node to its problems.
5. `prefers-reduced-motion` replaces flashing/pulsing with a static highlight.
6. 4.5:1 contrast for text/icons, 3:1 for graphical objects and focus rings; both themes verified.

### 12.2 Operating the canvas without a mouse and without dragging
- **WCAG 2.5.7 (Dragging Movements):** every drag has a non-drag alternative: quick-add, "Connect to…" (choose source port, then pick a compatible target from a list), arrow-key move, auto-layout.
- **2.1.1 Keyboard:** `Tab`/`Shift+Tab` move through nodes in topological order; `Enter` opens the inspector; arrows move the selected node (Shift = large step); `Delete`, `Ctrl+C/V/D`, `Ctrl+Z/Y`, `Ctrl+K`, `Ctrl+Enter` (run node), documented in a shortcuts dialog.
- **Outline view** (bottom-panel tab): a hierarchical, screen-reader-friendly list of nodes, their ports and connections, with the same actions as the canvas. This is the accessible equivalent of the graph.
- Targets at least 24×24 CSS px (2.5.8); visible focus not obscured by panels (2.4.11).
- Verification: axe-core in automated tests, plus a manual **NVDA** and keyboard-only pass before release.

---

## 13. Sandbox and Security

### 13.1 Threat model
Untrusted: imported graphs (code, hosts, prompts), model output (prompt injection triggering tool calls), user-authored Code, and any text that flows into generated source code.

### 13.2 Code block sandbox
- **Out-of-process worker** (`opifex-sandbox.exe`) embedding QuickJS; the app talks to it over stdin/stdout JSON.
- Worker runs in a **Windows Job Object**: memory cap, CPU time limit, process-count 1, kill-on-close, no inherited handles.
- **No host capabilities by default:** no filesystem, no environment, no process, no network. QuickJS exposes only the JS language, `console`, and the block's `input`.
- **Optional `fetch` host function** that is backed by the engine, which enforces the agent's network grants, size caps and timeouts. Code never opens sockets itself.
- Hard limits: execution time, memory, output size, recursion depth; the engine kills and restarts the worker on violation and reports a typed error (`SANDBOX_TIMEOUT`, `SANDBOX_MEMORY`, `SANDBOX_OUTPUT_TOO_LARGE`).
- Hardening stretch (post-MVP): run the worker in an AppContainer.
- Spike S3 proves the Job Object setup and the escape/resource test suite (§19) before the Code block is built.

### 13.3 Network grants
- HTTP and sandbox `fetch` may contact only hosts the user has granted **for that agent**.
- On first run (and after import), the UI shows: "This agent will contact: api.example.com, …" with Allow / Deny. Grants are stored in `agent_grants`; imports never carry grants.
- Localhost/private ranges are allowed only via explicit grant.

### 13.4 Agent safety
- An Agent can call **only** tools wired to it. Tool arguments are validated against the declared schema before execution. Tool output returned to the model is size-capped.
- `write` tools in Live test runs require confirmation (§7.11).

### 13.5 Generated-code safety
Block names, parameter values and prompts flow into Python source in exports. The exporters must: emit strings only via a safe literal encoder, sanitize identifiers, never interpolate unescaped text into code, and sanitize output filenames (no traversal). Hostile-name test cases are mandatory (§19).

### 13.6 Import
Imported graphs are validated first, then opened **without running anything**. Code blocks are flagged with a shield icon and require explicit confirmation before their first execution. Imports never include credentials or grants.

---

## 14. Credentials

- Secrets are stored in **Windows Credential Manager** (via the `keyring` crate) behind a `CredentialStore` trait so other backends can be added later.
- Graphs reference **slot names** only (e.g. `openai_main`). A settings screen manages slots (create, update, delete, test); values are never displayed after entry.
- Secrets are injected into node execution at the last moment, never written to `node_runs`, pins, logs or exports, and the scrubber (§9.4) removes accidental echoes.
- Export lists required slots in a `requirements` section, and generated projects include a `.env.example` using those names.

---

## 15. Export and Import

### 15.1 Architecture
`opifex-export` defines:

```rust
trait Exporter {
    fn id(&self) -> &'static str;
    fn check(&self, g: &ResolvedGraph) -> CapabilityReport;   // per-node: native | emulated | unsupported
    fn export(&self, g: &ResolvedGraph, opts: &ExportOptions) -> Result<Vec<Artifact>>;
}
```

Every export first produces a **CapabilityReport**. Unsupported blocks produce `NON_PORTABLE_BLOCK` and block that export unless the user chooses "stub" (generates a clearly marked `NotImplementedError` placeholder with the original logic in comments).

### 15.2 Targets (v2.0.0)

1. **Canonical JSON** (`opifex.agent/1`): graph + resolved custom block definitions + `requirements` (credential slots, models and required capability tier, network hosts) + `schema` and `minOpifexVersion` + embedded validation status and graph hash. Runnable by `opifex-cli run`.
2. **LangGraph** (Python project): `graph.py`, `nodes/`, `requirements.txt` (pinned), `.env.example`, `README.md`. Targets a specific pinned LangGraph version recorded in the exporter.
3. **OpenAI Agents SDK** (Python project): same layout; Agent → `Agent` + `Runner`, Structured LLM → typed output, Condition/Loop/Merge → plain Python control flow. (Confirm target in §24.)

Provider handling: LangGraph uses provider chat-model classes per provider; the OpenAI target uses OpenAI models directly and OpenAI-compatible endpoints (e.g. Ollama) for others. Exporters declare which providers they support and warn otherwise. **Verify library APIs at implementation time**; both frameworks evolve quickly, and the exporters pin versions to stay reproducible.

### 15.3 Portability matrix (initial; confirmed during spike S5)

| Block | JSON (Opifex runner) | LangGraph | OpenAI Agents SDK |
|-------|:-:|:-:|:-:|
| Input | native | native | native |
| LLM Prompt | native | native | native |
| Structured LLM | native | native | native (typed output) |
| Agent | native | native | native |
| HTTP Request | native | native | native |
| Condition | native | native (conditional edge) | native (`if`) |
| Loop | native | emulated (loop in node or map) | emulated (`for`) |
| Merge | native | native | emulated (variables) |
| Code (JS) | native | **unsupported** (stub) | **unsupported** (stub) |
| Final Answer | native | native | native |

### 15.4 Export gate
- Default: **export requires a clean validation run** (no `error` severity), as decided in rev 1.
- Proposed addition (needs your confirmation, §24): an explicit **"Export draft"** action that embeds `validation.status: "invalid"` plus the error list in the file and requires a confirmation dialog. Reason: developers will want to share or debug broken graphs.
- Exports never include credentials, grants, pinned data or run history unless the user ticks explicit, separately-warned options (pins only).
- Custom code blocks are included in canonical JSON only with an explicit checkbox.

### 15.5 Import
Canonical JSON only in v2.0.0. Import validates schema version (migrating older ones), runs the logic check, flags code blocks and required slots/grants, and opens the graph without executing anything.

### 15.6 Export testing
- **Golden-file tests** for generated projects (deterministic output).
- **Execution tests in CI:** run generated Python projects under pinned dependency versions against a fake model, and compare outputs to the Opifex engine on the same graph and trigger (**cross-runner equivalence**).
- Hostile-input tests for codegen injection (§13.5).

---

## 16. Oppy (Assistant)

### 16.1 Role
An on-demand analyst and editor for the current design surface. It never opens by itself and never interrupts. Entry points: top-bar button, `Ctrl+/`, node context menu "Ask Oppy about this", and **Explain** buttons on Problems and failed node runs.

### 16.2 Capabilities
- **Analyze / explain:** "What does this agent do?", "Why did this node fail?", "Why is this edge invalid?", "Review my agent" (checks for missing error handling, unbounded behavior, unclear prompts, risky side effects, model-tier mismatch, possible cost hot spots).
- **Propose edits:** Oppy returns **structured edit operations**, never free-form mutation:
  `add_node`, `remove_node`, `set_param`, `connect`, `disconnect`, `move_node`, `rename`, `group_into_block`.
- **Safety flow:** operations are applied to a scratch copy → validated → shown as a **diff preview** (ghosted nodes/edges on the canvas plus a text list and the validation delta) → user clicks **Apply** → applied atomically as one undo step, with an automatic `pre-ai-edit` snapshot.
- "Describe an agent and I'll draft the graph" is the same mechanism with more operations.
- v1's Wizard (chat → plan) remains available as **Wizard (legacy)** for v1 agents.

### 16.3 Context
Oppy gets a compact serialization (graph summary, selected node, latest validation result, last-run status, optionally run data) plus **tools**: `get_graph_summary`, `get_node`, `get_validation`, `get_last_run`, `list_blocks`, `get_block_spec`. It pulls details on demand rather than receiving the whole graph, which matters for small local models.

- Run data is excluded unless the user toggles "Include run data"; the scrubber always runs.
- Credentials are never included.
- If the selected provider is cloud-hosted, a persistent banner states that graph content leaves the machine.

### 16.4 Model gating
Propose-edits mode requires a model of tier ≥ 2 (§17); analyze mode works on tier ≥ 1 with a "compact context" fallback. Below that, Oppy says so and recommends a model.

### 16.5 Persistence
Reuses `conversations`/`messages`, keyed by agent, with an Oppy thread per graph.

---

## 17. Model Capability Tiers

Opifex does not guess from model names; it **probes**.

| Tier | Meaning | Required probe |
|------|---------|----------------|
| 0 | Plain text generation only | none |
| 1 | **Structured** — reliably returns schema-conformant JSON | k of n schema-conformance prompts pass |
| 2 | **Agent** — reliably performs tool calls with valid arguments | k of n tool-calling prompts pass (incl. multi-step and "no tool needed") |

- A **Model check** action runs a small probe suite against the selected model and caches results in `model_capabilities`. Probe suite versions are recorded so results can be re-run.
- Minimum context window for tier 2 is a parameter (initial proposal: ≥ 8k tokens; Oppy's analyze mode prefers ≥ 16k). **Thresholds (k/n, context size) are set after Spike S2 benchmarks** on the models you actually use.
- Blocks declare their minimum tier. A mismatch yields `MODEL_CAPABILITY_INSUFFICIENT` (warning) with a plain suggestion, not a hard block, because probes can be wrong and users may know better.
- Cloud models are marked tier 2 by default but can still be probed.

---

## 18. v1 Migration and Starter Templates

- v1 `agents.spec_json` is preserved. Agents without a graph open in **Wizard (legacy)** with a "Convert to block graph" action.
- **Conversion** (user-triggered, previewed, with a snapshot): goal → Input → LLM Prompt (system prompt from goal/steps) → Final Answer; tools → HTTP Request/Code placeholders where sensible; knowledge sources → a visible **placeholder note** because Retrieval is deferred to v2.1.
- **Starter templates:** the five v1 starters are re-authored as v2 graphs. Customer Support, Code Review and Content Writer map well. **Research** uses an Agent + HTTP tool. **RAG Knowledge** ships as a documented placeholder until v2.1. Templates are read-only and cloneable, and double as the empty-state gallery and as e2e fixtures.
- **Fixture-based migration tests** run against a real v1.0.0 database file, plus automatic DB backup before migration.

---

## 19. Quality Bar and Testing

| Layer | Approach |
|-------|----------|
| `opifex-core` | Unit + **property tests** (`proptest`): validator never panics, cycle detection exact, deterministic ordering; **cargo-fuzz** on the expression parser and graph JSON loader; schema migration tests with fixtures per schema version |
| `opifex-engine` | Deterministic **fake provider** (scripted responses, tool calls, failures, latency); tests for scheduling, skipping, merge, retries, timeouts, cancellation, Agent bounds, Loop caps |
| Sandbox | Escape/resource suite: infinite loop, memory bomb, deep recursion, giant strings, prototype pollution, attempts to reach fs/net/process/env, worker kill and restart, concurrent runs |
| Exporters | Golden files; cross-runner equivalence in CI; hostile identifier/string tests; pinned dependency versions |
| Frontend | Vitest for store/adapter/`canConnect` (conformance vectors); component tests; **axe-core** checks; Playwright e2e on key flows (build, validate, run, pin, export); manual **NVDA** and keyboard-only pass |
| Windows | Installer smoke test; clean-machine first-run; DB migration from v1 |
| CI | `cargo fmt`, `cargo clippy -D warnings`, `cargo test`, `vue-tsc --noEmit`, Vitest, conformance vectors, golden exports, generated-project execution |

### Agentic-coding guardrails
Because development is heavily agent-assisted: keep specs as markdown in the repo (`docs/blocks/`, this plan, the execution model §7), require tests with every change, and treat the conformance vectors and golden files as the arbiters that agent-written code must satisfy.

---

## 20. Non-Functional Requirements

| Area | Requirement |
|------|-------------|
| Compatibility | v1.0.0 agents and databases migrate with no data loss; additive migrations; automatic backup |
| Portability | Canonical JSON schema is versioned (`opifex.agent/1`); exporters pin target versions |
| Security | §13, §14: sandbox, grants, redaction, safe codegen, import gating |
| Accessibility | §12: WCAG 2.2 AA for the canvas, inspector, errors, and run states |
| Performance | Canvas ≥ 55 fps panning with 200 nodes; validation < 50 ms at 100 nodes; engine overhead < 5 ms per node excluding I/O; cold start < 2 s |
| Reliability | No data loss on crash (WAL + autosave); every run reaches a terminal status; all child processes cleaned up on exit |
| Offline | Everything except remote providers and granted hosts works offline |
| Platform | Windows 10/11 with WebView2; signed installer |

---

## 21. Tauri Commands

| Group | Commands |
|-------|----------|
| Blocks | `list_block_definitions`, `get_block_definition`, `save_block_definition`, `delete_block_definition`, `export_block_definition`, `import_block_definition` |
| Graphs | `save_agent_graph`, `get_agent_graph`, `list_graph_snapshots`, `create_graph_snapshot`, `restore_graph_snapshot`, `convert_v1_agent` |
| Validation | `validate_agent_graph` (Rust authority; result persisted to `validation_runs`) |
| Runs | `start_run {graphId, mode, scope}`, `cancel_run`, `get_run`, `list_runs`; progress via Tauri events on `run://<id>` (node state, streamed tokens, logs) |
| Pins | `pin_node_data`, `unpin_node_data`, `list_pins` |
| Credentials | `set_credential`, `delete_credential`, `list_credential_slots` (names only), `test_credential` |
| Grants | `list_agent_grants`, `set_agent_grant`, `revoke_agent_grant` |
| Models | `probe_model_capabilities`, `get_model_capabilities` |
| Export | `list_export_targets`, `check_export {agentId, target}` → `CapabilityReport`, `export_agent {agentId, target, options}`, `import_agent_graph` |
| Oppy | `assistant_chat` (streaming), `preview_edit_ops` (diff + validation delta), `apply_edit_ops` (atomic, undoable) |

---

## 22. Milestones

Rev 1's 150-hour estimate is dropped; sizes below are relative (S/M/L/XL) and will be **recalibrated after M0**. Ordering follows risk: de-risk first, build the foundation that everything depends on, then UI, then breadth.

**M0 — Spikes (de-risk).** (M)
- S1: Vue Flow adapter, 200-node performance, keyboard handling, accessibility hooks.
- S2: Model probe suite and tier thresholds on your real Ollama models.
- S3: Sandbox worker with Job Object limits and the escape/resource suite.
- S4: Expression language grammar, function set, static reference checking.
- S5: A 4-node graph exported to LangGraph and OpenAI Agents SDK, running in CI against the fake model, matching the Opifex engine output.
- *Exit:* each spike has a written go/no-go and any plan changes.

**M1 — Core.** (L) `opifex-core`: schema, types, port compatibility + conformance vectors, validator, expression language, migrations, DB layer, graph hashing/signatures. *Exit:* property tests and fuzzers green; v1 DB fixture migrates.

**M2 — Engine.** (XL) Scheduler, run store, cancellation, provider layer, fake provider, CLI. Blocks: Input, LLM Prompt, Condition, Merge, Final Answer. *Exit:* `opifex run` executes graphs deterministically in tests.

**M3 — Canvas shell.** (L) Palette, canvas, nodes/edges, inspector (parameters), Problems panel, undo/redo, autosave, snapshots, outline view, keyboard model. *Exit:* a user can build and validate a graph entirely by keyboard.

**M4 — Run UX.** (L) Per-node run, run to here, run all, mock/live, pinning, stale tracking, run data inspector, run history, credentials UI, grants UI. *Exit:* the full run-pin-iterate loop works on the M2 blocks.

**M5 — Remaining blocks.** (XL) Structured LLM (+repair), HTTP Request (+tool mode), Loop (+drill-in), Agent (+bounds, trace), Code (+sandbox). Each passes the Block Definition of Done.

**M6 — Export.** (L) Canonical JSON export/import, capability report UI, LangGraph and OpenAI exporters, golden + equivalence tests, export gate and "Export draft" (if approved).

**M7 — Oppy.** (L) Context tools, analyze/explain flows, edit-op schema, diff preview, apply/undo, model gating.

**M8 — Ship.** (L) Custom blocks (Path A), v1 conversion and templates, empty states, accessibility audit (NVDA), performance pass, installer and signing, docs.

Critical path: M0 → M1 → M2 → (M3 ∥ M4 after M2 core) → M5 → M6 → M8; M7 can start once M3 and the validator exist.

---

## 23. Deferred (tracked)

- **v2.1:** Webhook and Schedule triggers (need a local host process), Retrieval + Embed + knowledge-base module, MCP tool block, n8n-format export, Path B config-form blocks, parallel for-each, Delay/Set Variable/Log blocks, human-in-the-loop, proactive Oppy hints (opt-in).
- **v3:** multi-tenancy, auth, cloud sync, team collaboration, marketplace, non-technical mode, cross-platform builds.
- Forward-compat hooks already in v2.0.0: schema versioning, block versioning, credential store trait, nullable-ready identifiers, exporter trait.

---

## 24. Open Items (need your confirmation)

1. **Sub-graph editing:** Rev 2 specifies **drill-in with breadcrumbs** instead of the rev 1 modal, to preserve context. Confirm or revert.
2. **Export gate:** keep "clean validation required," and add an explicit "Export draft" action? (§15.4)
3. **"OpenAI" target:** assumed to be the **OpenAI Agents SDK** (Python). Confirm (as opposed to another OpenAI interface).
4. **Code block language:** exports to LangGraph and OpenAI are Python, but the Code block is JS (QuickJS), so Code is non-portable to those targets (stubbed on export). Options: accept that for MVP, or restrict the block to a portable transform language, or choose a Python sandbox (heavier). Spike S3/S5 will inform this.
5. **Single trigger per graph** in v2.0.0 (multiple triggers deferred with Webhook/Schedule).
6. **Model tier thresholds** (k of n probes, minimum context) will be set from Spike S2 data.
7. **Vue Flow acceptance**, pending Spike S1.
8. **RAG Knowledge starter** ships as a placeholder until Retrieval lands in v2.1.
