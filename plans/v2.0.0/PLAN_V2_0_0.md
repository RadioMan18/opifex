# Opifex v2.0.0 Plan — "Block Studio"

**Version:** 2.0.0  
**Codename:** Block Studio  
**Status:** Draft  
**Parent release:** Opifex v1.0.0 (MIT, open source)  
**Owner:** Expert Engineer + UI/UX Lead  
**Date:** 2026-10-28

---

## 1. Vision

Transform Opifex from a chat-driven wizard agent builder into a **visual block-based agent studio**. Technical users drag typed "blocks" onto a design surface, wire them together into an agent graph, run an on-demand logic check that surfaces errors with clear explanations, and export a valid agent as portable JSON. Oppy is repurposed as a block-authoring assistant (no longer the default build path).

The end state: Opifex is a **block composition engine** whose canonical artifact is a JSON agent definition that runs anywhere.

---

## 2. Goals & Non-Goals

### Goals
- Default UI is a drag-and-drop canvas; Oppy remains available as a secondary mode.
- Blocks have typed inputs/outputs; invalid connections are rejected by the logic check.
- Two paths to create new blocks: (a) blocks made of blocks (sub-canvas, default), (b) config form. Custom code is opt-in and sandboxed.
- On-demand logic check catches: unconnected blocks, dangling inputs, type mismatches, circular dependencies, unreachable/dead blocks, missing required config.
- Errors displayed per-node with WCAG AA–compliant cues (color + text + icon + live region).
- Built-and-validated agents export as JSON to be used anywhere.
- Backward compatible: v1.0.0 agents (structured plan JSON) load into v2 without data loss.
- **Agent-centric model** inspired by Gumloop: agents use tools to solve open-ended tasks, deciding which tools to use and when.
- **Four-tab agent builder**: Agent / Triggers / Access / Settings (sections expand in place with breadcrumb navigation).
- **Connectors** with account selection (Personal Default / Specific Account / Team Default) and Tool Management approval presets (Always allow / Ask each time / Ask for writes/deletes / Custom).
- **Skills** — teach agents how to do specific work "your way" (Create With AI, Upload Files, Write Skill Instructions, Add Existing Skill), with AI Skill Editing toggle.
- **Knowledge Sources** — RAG with citations (Google Drive, Notion, Slack, GitHub, Confluence, uploaded files).
- **Triggers** — how agents start on their own (App, Webhook, Scheduled, One-Time, Create With AI; AI Managed toggle).
- **Subagents** — delegate to other agents, parallel execution, self-cloning (depth limit 1).
- **Abilities** — toggleable built-in capabilities (Web Search, Web Fetch, Image Generation, Chat Widgets, Ask Question, Tool Discovery, etc.).
- **Channels** — make agents reachable outside Opifex (Email Inbox, Slack, Teams, Hosted Page, Developer/API, MCP).
- **Self-improving instructions** — agent updates its own instructions while working, with approval card for every change.
- **Performance / Evaluations / Reflections** — grade agent responses, catch regressions, agent reviews its own tasks.
- **Artifacts** — files, documents, charts produced by the agent.

### Non-Goals (v2.0.0)
- No built-in agent runtime/execution inside Opifex (export-first; runtime is the consumer's responsibility).
- No multi-tenancy, auth, or cloud sync.
- No template marketplace (deferred from v1.1.0).
- No arbitrary server-side code execution from user-authored blocks.

---

## 3. Stack (unchanged from v1, with additions)

| Layer | Choice |
|-------|--------|
| Shell | Tauri v2 (Rust) |
| Frontend | Vue 3 + TypeScript + Vite |
| Canvas library | @vue-flow/core |
| State | Vue reactive store (existing `agentStore`) extended + Pinia |
| Storage | SQLite (local file, app data dir) — existing schema + new tables |
| LLM providers | Ollama (default), OpenAI, Anthropic — existing |
| Testing | Vitest (frontend) + Rust `cargo test` |
| Lint/Typecheck | `vue-tsc --noEmit`, `cargo clippy`, `cargo fmt` |

New frontend dependency: `@vue-flow/core` for node/edge rendering.

---

## 4. Data Model

### 4.1 New SQLite tables (additive migrations)

```sql
-- Block definitions (reusable block templates, incl. user-authored)
CREATE TABLE IF NOT EXISTS block_definitions (
    id TEXT NOT NULL,
    version INTEGER NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    category TEXT NOT NULL,
    kind TEXT NOT NULL CHECK (kind IN ('trigger','llm','agent','tool','control','output','utility','subgraph')),
    origin TEXT NOT NULL CHECK (origin IN ('user','imported')),
    spec_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (id, version)
);

-- Agent block graphs (canonical agent form, replaces/augments spec_json)
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

-- Graph snapshots (auto/manual/pre-migration/pre-ai-edit)
CREATE TABLE IF NOT EXISTS graph_snapshots (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
    kind TEXT NOT NULL CHECK (kind IN ('auto','manual','pre-migration','pre-ai-edit')),
    label TEXT,
    graph_json TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Validation runs (audit trail of logic checks)
CREATE TABLE IF NOT EXISTS validation_runs (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
    graph_hash TEXT NOT NULL,
    status TEXT NOT NULL,
    error_count INTEGER NOT NULL DEFAULT 0,
    warning_count INTEGER NOT NULL DEFAULT 0,
    errors_json TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Runs (per-node execution history)
CREATE TABLE IF NOT EXISTS runs (
    id TEXT PRIMARY KEY,
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id),
    graph_hash TEXT NOT NULL,
    mode TEXT NOT NULL CHECK (mode IN ('live','mock')),
    scope_json TEXT NOT NULL,
    trigger_json TEXT,
    status TEXT NOT NULL CHECK (status IN ('running','succeeded','failed','cancelled')),
    error_json TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT
);

-- Node runs (individual node execution within a run)
CREATE TABLE IF NOT EXISTS node_runs (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES runs(id) ON DELETE CASCADE,
    parent_node_run_id TEXT,
    node_path TEXT NOT NULL,
    attempt INTEGER NOT NULL DEFAULT 1,
    status TEXT NOT NULL CHECK (status IN ('queued','running','succeeded','failed','skipped','cancelled')),
    input_json TEXT,
    output_json TEXT,
    error_json TEXT,
    truncated INTEGER NOT NULL DEFAULT 0,
    tokens_in INTEGER,
    tokens_out INTEGER,
    started_at TEXT,
    ended_at TEXT
);

-- Pinned data (user-pinned node outputs for iterative development)
CREATE TABLE IF NOT EXISTS pinned_data (
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL,
    port_id TEXT NOT NULL,
    value_json TEXT NOT NULL,
    source_node_run_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (graph_id, node_id, port_id)
);

-- Agent grants (capability grants, e.g. network access)
CREATE TABLE IF NOT EXISTS agent_grants (
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('network')),
    value TEXT NOT NULL,
    granted_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (agent_id, kind, value)
);

-- Model capabilities (probed tier per provider/model)
CREATE TABLE IF NOT EXISTS model_capabilities (
    provider TEXT NOT NULL,
    model TEXT NOT NULL,
    probe_json TEXT NOT NULL,
    tier INTEGER NOT NULL,
    probed_at TEXT NOT NULL,
    PRIMARY KEY (provider, model)
);

-- Add default_graph_id to agents (additive, nullable)
ALTER TABLE agents ADD COLUMN default_graph_id TEXT;
```

### 4.2 TypeScript types (`src/types/agent.ts` additions)

```ts
export type BlockType = 'trigger' | 'llm' | 'tool' | 'knowledge' | 'control' | 'output' | 'utility' | 'sub-graph';

export interface BlockPort {
  id: string;
  name: string;
  type: string;
  direction: 'input' | 'output';
  required: boolean;
  defaultValue?: unknown;
  description?: string;
}

export interface BlockParameter {
  id: string;
  name: string;
  type: 'string' | 'number' | 'boolean' | 'select' | 'multi-select' | 'textarea' | 'code' | 'credential';
  label: string;
  required: boolean;
  defaultValue?: unknown;
  options?: string[];
  description?: string;
  sensitive: boolean;
}

export interface BlockDefinition {
  id: string;
  name: string;
  description?: string;
  category: string;
  type: BlockType;
  icon?: string;
  color?: string;
  inputs: BlockPort[];
  outputs: BlockPort[];
  parameters: BlockParameter[];
  isBuiltin: boolean;
  isCustom: boolean;
  subGraph?: AgentGraph;
  createdAt: string;
  updatedAt: string;
}

export interface NodeInstance {
  id: string;
  blockId: string;
  position: { x: number; y: number };
  parameters: Record<string, unknown>;
  subGraph?: AgentGraph;
  isCollapsed: boolean;
}

export interface EdgeInstance {
  id: string;
  source: string;
  target: string;
}

export interface AgentGraph {
  id: string;
  name: string;
  description?: string;
  nodes: NodeInstance[];
  edges: EdgeInstance[];
  viewport?: { x: number; y: number; zoom: number };
  metadata?: Record<string, unknown>;
  createdAt: string;
  updatedAt: string;
}

export interface ValidationError {
  nodeId?: string;
  portId?: string;
  severity: 'error' | 'warning';
  code: string;
  message: string;
  suggestion?: string;
}

export interface ValidationResult {
  valid: boolean;
  errors: ValidationError[];
  nodeCount: number;
  edgeCount: number;
}

export interface ConnectorBlock {
  id: string;
  name: string;
  accountSelection: 'personal-default' | 'specific-account' | 'team-default';
  toolManagement: {
    preset: 'always-allow' | 'ask-each-time' | 'ask-for-writes-deletes' | 'custom';
    perTool: Record<string, 'allow' | 'ask' | 'deny'>;
  };
}

export interface Skill {
  id: string;
  name: string;
  kind: 'create-with-ai' | 'upload-files' | 'write-instructions' | 'add-existing';
  aiEditingEnabled: boolean;
  instructions: string;
}

export interface KnowledgeSource {
  id: string;
  name: string;
  provider: string;
  sourceIds: string[];
  citationsEnabled: boolean;
}

export interface Trigger {
  id: string;
  kind: 'app-trigger' | 'webhook' | 'scheduled' | 'one-time' | 'create-with-ai';
  aiManaged: boolean;
}

export interface SubagentConfig {
  agentId: string;
  task: string;
  maxConcurrent: number;
  depthLimit: number;
}

export interface Channel {
  id: string;
  kind: 'email-inbox' | 'slack' | 'teams' | 'hosted-page' | 'developer' | 'mcp';
}
```

### 4.3 Agent table extension

`agents.spec_json` is retained for backward compatibility (v1 plan format). A new `agents.defaultGraphId` points to the canonical block graph. v1 agents without a graph are treated as "legacy" and can be auto-converted (goal → Entry Point + Prompt Weaver + Result Terminal) or edited in Oppy mode.

---

## 5. Block System

### 5.1 Built-in block catalog

| Category | Block | Inputs | Outputs | Description |
|----------|-------|--------|---------|-------------|
| Trigger | Entry Point | none | message (string), payload (json) | Starts agent on user message |
| Trigger | Webhook | none | payload (object), headers (object) | HTTP trigger |
| Trigger | Schedule | none | tick (string) | Cron/interval trigger |
| LLM | Prompt Weaver | message (string), system (string) | completion (string) | LLM call via selected provider |
| LLM | Schema Weaver | message (string), schema (object) | result (object) | LLM with JSON schema output |
| Tool | Web Courier | url (string), body (any), headers (object) | response (json) | Generic HTTP call |
| Knowledge | Retrieval | query (string), source (string) | chunks (string[]) | RAG over knowledge base |
| Knowledge | Embed | text (string) | vector (object) | Generate embedding |
| Control | Logic Gate | value (any), condition (string) | true / false (any) | Branch |
| Control | Item Iterator | items (array<any>), body (sub-graph) | results (array<any>), errors (json) | Iterate |
| Control | Delay | duration (number) | done (string) | Wait |
| Output | Result Terminal | message (string/json) | none | Return result to user |
| Output | Log | message (string) | none | Emit to console/log |
| Utility | Set Variable | value (any) | variable (any) | Pass-through / assign |
| Utility | Script Vault | input (any) | output (any); also a `tool` output | Safe expression (opt-in dev mode) |

### 5.2 Creating new blocks

**Path A — Blocks made of blocks (default):** User opens a sub-canvas, composes existing blocks into a mini-graph, saves it as a block definition. The sub-graph becomes a reusable block with auto-derived input/output ports from its boundary nodes.

**Path B — Config form:** User fills a form (name, category, icon, parameters). Produces a "utility" or "tool" block backed by a fixed runtime behavior (e.g. Web Courier, set variable).

**Path C — Custom code (opt-in, sandboxed):** User writes a safe expression/function. Requires explicit "Developer mode" enable + warning dialog. Runs in a WebWorker iframe with no file system or network access unless credentials are granted. Custom code blocks are stored separately and flagged `isCustom: true`.

All three paths produce the same `BlockDefinition` JSON, stored in `block_definitions`.

### 5.3 Block registry

The Rust backend exposes commands to:
- `list_block_definitions` / `get_block_definition`
- `save_block_definition` (builtin + custom)
- `delete_block_definition`
- `export_block_definition` / `import_block_definition` (JSON)

Frontend loads the registry into a palette (searchable, categorized) on app init.

---
---

## 6. Connectors, Account Selection & Tool Management

### 6.1 Connectors

Connectors are integrations the agent connects to (Gmail, Salesforce, Slack, HTTP, etc.). The connectors added determine what the agent can access — this is the most important part of configuration.

Each connector block has:
- **Account selection:** Personal Default (each person uses their own default account), Specific Account (pin one account), Team Default (everyone on team uses shared account).
- **Tool Management panel:** controls which of a connector's tools the agent can call, and which need approval first. Tools are grouped into Read-only and Write/delete.
- **Approval presets:**
  - **Always allow** — every tool runs without asking.
  - **Ask each time** — agent pauses for approval before any tool call.
  - **Ask for writes/deletes** — read-only tools run freely; write/delete/unknown-risk tools pause for approval.
  - **Custom** — set mode per tool: allow always, ask each time, or never allow (deny).

When a tool needs approval, the agent pauses mid-task and shows an approval card listing each change. The card never offers "Don't ask again for this tool."

---

## 7. Skills

Skills teach your agent how to do specific work *your way*: multi-step processes, templates, and domain knowledge that load only when relevant.

**Create paths:**
- **Create With AI:** describe the skill and the agent generates it.
- **Upload Files:** turn a document or `.zip` into a skill.
- **Write Skill Instructions:** enter a name and description yourself.
- **Add Existing Skill:** attach a skill you already created.

**AI Skill Editing** toggle lets the agent create and edit skills on its own. When an agent gets something wrong, give it feedback and ask it to save the correction in its instructions or a skill. Saving gives future tasks a reusable starting point.

---

## 8. Knowledge Sources

Give your agent a searchable memory of what your company knows. Attach Brain sources (Google Drive, Notion, Slack, GitHub, Confluence, or uploaded files) so the agent answers from real documents with citations, instead of guessing.

When you ask about internal knowledge, the agent searches automatically and can open a full document for more context.

---

## 9. Triggers

Agents can run on their own, without anyone starting a task. The Triggers tab lists triggers on the agent; **AI Managed** in its header lets the agent create, edit, and manage its own triggers during a task.

**Five ways to add a trigger:**
- **App Trigger** — runs when an event happens in another app (new email, new Slack message, record changes).
- **Webhook** — runs when another service sends a request.
- **Scheduled Trigger** — runs on a recurring schedule (e.g., every weekday at 9 AM).
- **One-Time Trigger** — runs once at a specific time.
- **Create With AI** — describe what you want and the agent builds a custom trigger.

Owners manage every trigger; Users can create/edit their own triggers while Create triggers is allowed in User Permissions. Mine/Team filter switches between views.

---

## 10. Subagents

Subagents let your agent delegate to other agents. Instead of doing everything in one task, it can spin up focused helpers that work in parallel, then collect results and continue.

**Two ways to delegate:**
- **Self-cloning:** agent starts copies of itself with same tools and instructions. Each copy handles a separate part of work. Copies cannot make further copies (depth limit of 1).
- **Invoking other agents:** add a specialized agent to the Subagents list. These can delegate further without depth limit.

Each subagent has its own task, history, files, and computer. Max 10 concurrent subagents per task.

---

## 11. Abilities

Abilities are the agent's built-in capabilities. Each can be toggled from the Agent tab's Abilities section:

| Ability | Default | What it does |
|---------|---------|-------------|
| Web Search | On | Searches the web for current information |
| Web Fetch | On | Reads content of a specific URL |
| Image Generation | On | Creates images from text prompts |
| Chat Widgets | On for new agents | Adds compact visual (chart, comparison) to replies |
| Search Past Conversations | On | Searches/retrieves earlier tasks for context |
| Ask Question | On | Lets agent pause and ask structured multiple-choice question |
| Tool Discovery | Auto | Loads tool schemas on demand |
| Rules Creation | On where available | Lets agent propose App Rules for review |

---

## 12. Self-improving Instructions

Your agent can update its own instructions while it works. Correct it once ("always check Salesforce first," "keep emails under 100 words") and it edits its instructions so the same mistake doesn't happen again. Changes take effect on the next step and persist across future tasks.

**Rules:**
- **Edit Own Instructions** toggle must be on (default: on).
- Every change needs your approval — approval card lists each change; never offers "Don't ask again."
- Agent makes the change as you (same access rules apply).
- Changes apply from your next message, not partway through current turn.
- Agent can't delete itself or turn off its own tool approvals.

---

## 13. Channels

Use Channels to make your agent reachable outside Opifex:

| Channel | What it gives you |
|---------|-----------------|
| Email Inbox | Agent gets its own email address |
| Slack | Talk to agent in Slack channels and DMs |
| Microsoft Teams | Talk to agent in Teams |
| Hosted Page | Public or private chat page to link or embed |
| Developer | API and SDK details for calling agent from your code |
| MCP | Expose agent as MCP server so other AI tools can call it |

---

## 14. Canvas UI/UX (replaces split-pane as default)

### 14.1 Layout (four-tab agent builder, Canvas is default)

+-----------------------------------------------------------------------+
| Top bar: Opifex | Agent / Triggers / Access / Settings tabs | Save    |
+-------------------------------------------------------------------------+
| Left sidebar: New Task | Artifacts | Performance | Channels | Recents  |
+-------------------+------------------------------------------------------------+
|                   |                                                                    |
|                   |               MAIN CANVAS (Gumloop-style workbook)             |
|                   |  [nodes]  ===>  [nodes]  ===>  [nodes]                         |
|                   |  (node-to-node edges, mapping inside node)                     |
|                   |                                                                    |
+-------------------+------------------------------------------------------------+
| Bottom bar: Workflow tabs | Canvas controls: Fit View | Auto Format | Expand All |
|               | Collapse All | Snap to Grid | Zoom                  |
+-------------------------------------------------------------------------+
| Bottom panel (collapsible): Logic Check results | Error list | Inspector |
+-------------------------------------------------------------------------+
| Right drawer (slide-out): Execution log — per-node status, duration, output |
+-------------------------------------------------------------------------+

**Top bar:** Opifex logo, four agent-builder tabs (Agent / Triggers / Access / Settings), Save button (`Ctrl/Cmd + S`) with undo arrow.

**Left sidebar:** New Task, Artifacts, Performance, Channels, Recents. Pending action request appears above Recents when the agent waits for tool approval.

**Agent tab:** model picker, instructions box, connectors, skills, knowledge sources, subagents, secrets, abilities. Sections expand in place with breadcrumb navigation (`Agent > Connectors`).

**Triggers tab:** how the agent starts on its own — App Trigger, Webhook, Scheduled Trigger, One-Time Trigger, Create With AI. Mine/Team filter. AI Managed toggle.

**Access tab:** Owners, Who Can Use, Artifact Sharing, Incognito, User Permissions.

**Settings tab:** Identity, Suggested Prompts, Voice, Advanced AI (max steps, parallel tool calls, max output tokens), Chat Summarization, Model Fallback, Messaging Mode (Classic/Conversational), Destructive.

**Save model:** Nothing saves until you press **Save** (`Ctrl/Cmd + S`); undo arrow next to it reverts unsaved changes.

**- Block Palette (slide-out drawer from left, overlays canvas):** Searchable list of block definitions, grouped by category. Drag onto canvas to create a node instance. Drawer does not remember open/closed state across sessions.

**- Canvas:** Infinite pan/zoom, **node-to-node edges** (connect two nodes directly — all outputs from source become available in target; no port-to-port "spaghetti"). Mapping — saying which input to use — happens **inside the node's inspector**, not on the edge. Nodes show a colored header (category color), title, and a small error badge when invalid.

**- Canvas controls (bottom right):** Fit View, Auto Format, Expand All Nodes, Collapse All Nodes, Snap to Grid, Zoom controls.

**- Run button (top right of canvas):** Execute the workflow. Run report appears showing real-time progress through each node.

**- Resume button (`Cmd/Ctrl + Shift + Enter`):** Jump to a specific node and continue from there — saves time iterating on specific parts.

**- Workflow tabs (bottom bar):** Multiple workflows as tabs (like Excel sheets in a workbook). Each tab is a separate workflow that can run independently. Right-click to rename, duplicate, delete. `+` to add new.

**- Subflows:** Convert groups of nodes into modular subflows (Shift+drag to select, then "Make Subflow"). Selected nodes move to a new tab with inputs/outputs auto-configured. Sub-flows appear in a separate sub-library tab in the block palette.

**- Node inspector (bottom panel):** When a node is selected, show its parameters as form controls matching `BlockParameter` types. Mapping (which input to use) happens here — no port handles on nodes. Credential values are masked and stored separately from the graph JSON.

**- Execution drawer (right slide-out panel):** Auto-opens when a run starts. Shows per-node execution status (green checkmark success, red X failure), execution duration, output values, and total credits used. Tabs: Current Run vs Previous Runs.

**- Node context menu (right-click):** Duplicate, Delete, Convert to sub-graph, Collapse/Expand, Open documentation, Error details.

**- Edges:** Single edge between two nodes (no port handles). Edge color matches source block category. Multiple edges from one node allowed (fan-out).

**- Default canvas:** Every new workflow opens pre-populated with an **Entry Point** node and a **Result Terminal** node.

### 14.2 Oppy mode (secondary)

Accessible via a mode toggle. Oppy is Clippy-style — on demand only and never appears without the user requesting it. Oppy's new role: user describes a block they want, Oppy proposes a block definition (sub-graph structure or config-form parameters), user reviews and saves to the block library. Existing v1 agent-building flow is preserved under a "Legacy / Oppy wizard" label.

### 14.3 Connections & Mapping (replaces "Edges & ports")

- **Node-to-node edges only:** Connect two nodes directly. All outputs from the source node become available in the target node — no port-to-port mapping on edges.
- **Mapping inside nodes:** Mapping — "I want this input to use that output" — happens in the target node's inspector, not on the edge. This eliminates "spaghetti" when a flow has 50+ connections.
- **Multiple edges from one node (fan-out) allowed.** No fan-in restriction on edges themselves (mapping handles it inside nodes).
- **Edge color** matches source block category. Dashed edges indicate warnings/type adapters.
- **DAG requirement:** Cycles are rejected. Topological sort gives deterministic execution order.

---

## 15. Logic Check (Validation)

### 15.1 When it runs
- **On demand:** "Validate" button in the canvas toolbar (primary trigger).
- **Continuous (lightweight):** A debounced, non-blocking pass that flags obvious issues (dangling edges, unconnected required inputs) without preventing edits. Does not run heavy cycle detection continuously.
- **Pre-export:** Validation must pass before JSON export; invalid agents can be saved as drafts but flagged "unbuildable".

### 15.2 Validation checks (graph-level + per-node)

| Check | Scope | Severity | Message example |
|-------|-------|----------|-----------------|
| CYCLE_DETECTED | Graph | error | "Cycle detected: Block A → B → C → A. Agent graphs must be acyclic." |
| UNCONNECTED_INPUT | Node | error | "Required input 'message' on 'Prompt LLM' is not connected." |
| TYPE_MISMATCH | Edge | error | "Output 'completion' (string) cannot connect to input 'count' (number) on 'Delay'." |
| DANGLING_EDGE | Edge | warning | "Edge references a deleted block." |
| UNREACHABLE_NODE | Node | warning | "Block 'Log' is not reachable from any trigger." |
| MISSING_REQUIRED_PARAM | Node | error | "Parameter 'API Key' is required on 'HTTP Request' but not set." |
| DUPLICATE_NODE_ID | Graph | error | "Duplicate node id detected." |
| EMPTY_GRAPH | Graph | warning | "Agent graph has no nodes." |
| NO_TRIGGER | Graph | warning | "No trigger block found; agent will never start." |
| NO_OUTPUT | Graph | warning | "No output block found; agent produces no result." |

### 15.3 Validation algorithm
1. Build adjacency list from edges.
2. Detect cycles via DFS (DAG requirement).
3. Topological sort to determine reachability from trigger nodes.
4. Per-node: check required inputs connected, required parameters set, port type compatibility on each edge.
5. Return `ValidationResult` with `errors` array, each error carrying `nodeId`, `portId`, `code`, `message`, `suggestion`.

### 15.4 Error persistence
Each validation run is stored in `validation_runs` with status + serialized errors, for audit and "compare with last run" UX.

---

## 16. Accessible Error Display (WCAG AA)

Flashing red alone is **not** sufficient for WCAG AA (1.4.1 Use of Color). Errors must not rely on color alone.

### Required cues (all present together)
1. **Color:** Red border / error badge background on the offending node and edge (`--danger`).
2. **Icon:** ✖️ (error) or ⚠️ (warning) next to the node title and in the error list.
3. **Text:** Error list panel shows each error with node name, port, code, message, and suggestion. Screen-reader accessible (semantic `<ul>`/`<li>`, `aria-label`).
4. **Live region:** An `aria-live="polite"` region announces the error count and first error on validation completion.
5. **Focus management:** Clicking an error in the list focuses/pans the canvas to the offending node (`aria-describedby` linking node to its errors).
6. **Motion:** Flashing red respects `prefers-reduced-motion` — flash is replaced by a steady pulse or static highlight when reduced motion is enabled.
7. **Contrast:** Error text and icons meet 4.5:1 contrast against background; focus indicators visible.

### Error list panel (bottom panel)
- Header: "Logic Check — N errors, M warnings" (announced to screen readers).
- Filter tabs: All / Errors only / Warnings only.
- Each item: icon + node name + port + message + "Fix" button (pans to node + opens inspector).
- "Clear / Re-run validation" buttons.

---

## 17. Safe Custom Code (Developer Mode)

Custom code blocks (Path C) are **opt-in** and isolated:

1. **Developer mode toggle** in settings, off by default. Enabling shows a one-time warning: "Custom code blocks run with full JS access. Only use code you trust."
2. Custom code executes in a **WebWorker** inside a sandboxed iframe (no `fs`, no `net`, no `process`, no `eval` of user strings beyond the block's own source).
3. The block's source code is stored in `block_definitions.spec_json` but **not** exported with the agent JSON unless the user explicitly includes custom blocks (separate export flag).
4. Agents containing custom code blocks are flagged `containsCustomCode: true` and show a shield icon in the library.
5. Import of agents with custom code blocks requires explicit user confirmation.

---

## 18. Oppy Repurposing

- Oppy is no longer the default build path; it's a **block authoring assistant**.
- User tells Oppy: "make a block that summarizes web pages and outputs bullet points."
- Oppy proposes: a sub-graph (Webhook → HTTP Request → LLM Prompt → Final Answer) with parameter defaults, OR a config-form block definition.
- User reviews the proposed block in a side panel, tweaks parameters/ports, then clicks "Save to library."
- Existing v1 agents remain editable via Oppy mode (legacy chat flow), preserving backward compatibility.

---

## 19. Export

- **Export Agent JSON:** Serializes `AgentGraph` + referenced `BlockDefinition`s + metadata into a single portable JSON file. Validated agents only (invalid = draft warning).
- **Export options:** Include custom code blocks? (yes/no, with warning). Include credentials? (no — credentials stored separately, never in export).
- Import validates the graph on load and runs the logic check automatically, surfacing any import-time errors.

---

## 20. Migration & Backward Compatibility

- v1.0.0 `agents.spec_json` (AgentPlan) is preserved.
- Auto-conversion (optional, user-triggered): goal → [Chat Input] → [Prompt LLM] → [Final Answer], with steps/tools/knowledge mapped to intermediate blocks where sensible.
- Agents without a graph open in Oppy/legacy mode by default; user can "Convert to block graph" at any time.
- No destructive DB changes; all new tables additive.

---

## 21. New Tauri Commands

| Command | Input | Output | Description |
|---------|-------|--------|-------------|
| `list_block_definitions` | none | `BlockDefinition[]` | List all block definitions |
| `get_block_definition` | `{ id }` | `BlockDefinition \| null` | Get one block definition |
| `save_block_definition` | `BlockDefinition` | `boolean` | Upsert a block definition |
| `delete_block_definition` | `{ id }` | `boolean` | Delete a block definition |
| `export_block_definition` | `{ id }` | `string` | JSON string of one block |
| `import_block_definition` | `{ json }` | `BlockDefinition` | Import and validate one block |
| `save_agent_graph` | `{ agentId, graph }` | `string` (graphId) | Save/create agent graph |
| `get_agent_graph` | `{ agentId }` | `AgentGraph \| null` | Get default graph for agent |
| `validate_agent_graph` | `{ graph }` | `ValidationResult` | Run logic check |
| `export_agent_graph` | `{ agentId }` | `string` | Export agent + blocks as JSON |
| `import_agent_graph` | `{ json }` | `Agent` | Import agent from JSON |

---

## 22. Task Breakdown

| # | Task | Area | Est. Hours | Deps |
|---|------|------|------------|------|
| T1 | DB migrations: block_definitions, agent_graphs, validation_runs | Backend | 3 | None |
| T2 | TS types: BlockDefinition, NodeInstance, EdgeInstance, AgentGraph, ValidationResult | Frontend | 2 | None |
| T3 | Rust commands: block_definitions CRUD | Backend | 6 | T1 |
| T4 | Rust commands: agent_graphs save/get | Backend | 4 | T1,T2 |
| T5 | Rust commands: validate_agent_graph | Backend | 8 | T1,T2 |
| T6 | Rust commands: export/import agent graph | Backend | 6 | T1,T2,T4 |
| T7 | IPC service wrappers for new commands | Frontend | 4 | T3-T6 |
| T8 | Canvas scaffold: Vue Flow wrapper, pan/zoom, mini-map | Frontend | 10 | T2 |
| T9 | Block palette: searchable, categorized, drag-to-canvas | Frontend | 8 | T2,T3 |
| T10 | Node rendering: typed ports, error badges, category colors | Frontend | 8 | T2,T9 |
| T11 | Edge rendering: node-to-node edges, type-colored, fan-out | Frontend | 6 | T2,T9,T10 |
| T12 | Node inspector: parameter form by BlockParameter type | Frontend | 8 | T2 |
| T13 | Context menu: duplicate, delete, sub-graph, collapse | Frontend | 4 | T8 |
| T14 | Logic check engine: cycles, types, required inputs, reachability | Frontend+Backend | 10 | T2,T5 |
| T15 | Error list panel + WCAG AA cues (color+icon+text+live region) | Frontend | 8 | T14 |
| T16 | Validate button + continuous lightweight check | Frontend | 4 | T14,T15 |
| T17 | Oppy repurpose: block authoring assistant side panel | Frontend | 8 | T2,T3,T9 |
| T18 | Legacy mode: preserve v1 split-pane Oppy wizard | Frontend | 4 | None |
| T19 | Auto-convert v1 AgentPlan → block graph (optional) | Frontend+Backend | 6 | T2,T4 |
| T20 | Export/Import agent JSON with custom-code gating | Frontend | 6 | T6,T7 |
| T21 | Developer mode toggle + sandboxed custom code block | Frontend+Backend | 10 | T2,T3 |
| T22 | Tests: Rust validation + Vitest canvas/store/error display | All | 8 | All above |
| T23 | Polish: animations, empty states, keyboard shortcuts, a11y audit | Frontend | 6 | T8-T17 |
| | **Total** | | **~150** | |

Critical path: T1 → T2 → T3/T4 → T5 → T6 → T7 → T8 → T9 → T10/T11 → T12 → T14 → T15 → T16 → T20 → T22.

---

## 23. Non-Functional Requirements

| Area | Requirement |
|------|-------------|
| **Backward compatibility** | v1.0.0 agents load unchanged; Oppy/legacy mode preserved; additive DB migrations only. |
| **Portability** | Agent export is pure JSON; runs anywhere the consumer implements a runner. |
| **Security** | No arbitrary server-side code execution. Custom code is opt-in, sandboxed, flagged, and gated on import. Credentials never exported. |
| **Accessibility** | WCAG AA for error display (color + icon + text + live region + focus management + reduced-motion respect). |
| **Performance** | Canvas renders 100+ nodes at 60fps; validation completes < 2s for graphs < 50 nodes. |
| **Offline** | Canvas, validation, export/import all work offline. No network required except LLM/provider calls (at runtime, not build time). |
| **Tests** | Rust unit tests for validation rules; Vitest tests for canvas store, port-type matching, error rendering, WCAG cues. |

---

## 24. Versioning & Release

Release as **v2.0.0** (breaking UI change: canvas is default; Oppy is secondary). v1.0.0 databases migrate without data loss. Semantic versioning: block graph format is v1 of the agent graph schema.

---

## 25. Decisions (closed)

1. **Canvas library:** `@vue-flow/core` is primary (selected per Gumloop research — Vue-native, no React interop). Check license status if cost/licensing is unacceptable, fall back to **custom SVG** despite higher build complexity.

2. **Sub-graph editing:** **Drill-in breadcrumbs** (not modal). Multi-node selections can be collapsed into custom sub-graph blocks. Drill-in navigation with breadcrumbs retains spatial awareness (`Main Graph > Sub-flow > Process`).

3. **Export with errors:** **Blocked.** An agent with validation errors must not export. Export requires a clean validation run. Invalid graphs are saved as drafts only.

4. **Default block catalog:** **Ship all 10 built-in blocks** from §5.1 at v2.0.0 launch.

5. **Connection model:** **Node-to-node edges** (Gumloop model). Connect two nodes directly — all outputs from source become available in target. Mapping happens *inside* the node inspector, not on edges. Eliminates "spaghetti" on complex flows.

6. **Canvas controls:** Fit View, Auto Format, Expand/Collapse All, Snap to Grid (bottom right). Run button top right. Resume button (`Cmd/Ctrl + Shift + Enter`).

7. **Save model:** `Ctrl/Cmd + S` with undo arrow. Nothing saves until explicitly saved.

8. **Workbook tabs:** Multiple workflows as tabs at bottom of canvas (like Excel sheets).

9. **Theme:** Cool blue sky/cloud palette. Sky blue (`#3B82F6`) accents, white primary surfaces, light slate text. No magenta/pink (avoid Gumloop copyright similarity).

10. **Font:** Inter or system-ui — clean sans-serif with compact line heights.

11. **Triggers v2.0.0:** Entry Point (Chat Input), Webhook, Schedule (full Gumloop config — minute/hour/day/month/day-of-week/timezone). App triggers (Gmail, Slack, etc.) deferred to v2.1.

12. **Subflow library:** Separate sub-library tab in the block palette — sub-flows are a distinct category of block, not mixed with regular blocks.

13. **Default canvas:** Every new workflow opens pre-populated with an **Entry Point** node and a **Result Terminal** node.

14. **Oppy:** Clippy-style, on-demand only. Oppy never appears without the user requesting it. Oppy proposes block definitions (sub-graph structure or config-form parameters); user reviews and saves to block library.

Save this plan. It is gitignored under `plans/` and ready to implement once v1.0.0 is where you want it.
