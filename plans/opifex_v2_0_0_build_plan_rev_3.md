# Opifex v2.0.0 Plan — "Block Studio" (Rev 3)

**Version:** 2.0.0  
**Codename:** Block Studio  
**Status:** Approved Draft, rev 3 (supersedes rev 2)  
**Parent release:** Opifex v1.0.0 (MIT, open source)  
**Owner:** Dave (solo, agent-assisted development)  
**Date:** 2026-10-28  
**Platform:** Windows only (Tauri v2 on WebView2)

---

## 0. Changes from Rev 2 to Rev 3

| Area | Rev 2 | Rev 3 (Updated) |
|------|-------|-----------------|
| **Audience Strategy** | Technical community (MVP) | Built for personal engineering workflow, but polished for **external developer adoption** (gold-standard DX, actionable error suggestions, instant-run exported projects). |
| **Canvas UX (Tool Wiring)** | Canvas edges only for tool ports | **Hybrid tool wiring**: visual canvas edges OR clean inline tool-attachment drawer in node inspector to eliminate spaghetti wires. |
| **Canvas Feedback Latency** | Rust round-trip for drag checks | **Client-side cached compatibility evaluator** in TS backed by shared test vectors for 60fps drag feedback; Rust validator retains final authority on drop/save. |
| **Sandbox Sidecar Packaging** | Generic process child binary | **Tauri v2 Sidecar pattern** (`opifex-sandbox-x86_64-pc-windows-msvc.exe`) configured in `tauri.conf.json` with early validation in Spike S3. |
| **Export Gate & Hatch** | Clean validation required | Clean validation required by default **+ explicit "Export Draft" release hatch** (embeds warning badge and invalid status in metadata/README for active debugging). |
| **CI & Agent Guardrails** | Standard unit/fuzz tests | **Automated CI Enforcement Rules**: Conformance Vector rule, Block Definition/DoD linter, and Golden-File Export Regression checks. |
| **Target Framework Pinned SDKs** | Unpinned SDK targets | Exporters pin explicit target SDK package versions in generated `requirements.txt` to prevent upstream breaking changes. |

---

## 1. Vision and Positioning

Opifex v2 is a **local-first visual agent studio** for technical users and developers. You compose typed blocks on a canvas, **run them node by node against real models and tools**, pin and edit data to iterate quickly, validate the whole graph statically, and **export portable artifacts**: a canonical JSON definition and runnable LangGraph / OpenAI Agents SDK projects.

### Key Differentiators

1. **Local-first and portable.** Everything works offline against Ollama; the canonical artifact is yours.
2. **Static guarantees.** Expressions, ports, loops, and bounds are verified before anything runs.
3. **Compiles to real frameworks**, not just to a proprietary runtime.
4. **Oppy**, an on-demand assistant that sees your design surface, explains it, and proposes structured edits you can preview and undo.
5. **Developer-First DX.** Generated exports are zero-friction, `README.md`-driven directories ready to execute via `pip install -r requirements.txt`.

### Design Principles

- **P1 Static where possible.** If it can be checked before running, it is.
- **P2 Run it for real.** Fast feedback (per-node run, pinning) beats theory.
- **P3 Portable by construction.** Every block declares what it exports to which target.
- **P4 Bulletproof over broad.** Ten blocks that never surprise beat twenty that sometimes do.
- **P5 Accessible by default.** Keyboard and screen-reader paths are first-class, not a final audit.
- **P6 AI on demand.** Oppy never interrupts; it appears when summoned.
- **P7 Safe with untrusted input.** Imported graphs, model output, user code, and generated Python files are isolated and sanitized.

---

## 2. Scope and Cut Line

| Capability | v2.0.0 (MVP) | v2.1 | v3 |
|-----------|:-:|:-:|:-:|
| Canvas, palette, inspector, undo/redo, autosave | ✔ | | |
| Top 10 blocks (§8) | ✔ | | |
| Per-node run, pin, stale tracking, run history | ✔ | | |
| Bounded Agent and Loop | ✔ | | |
| Sandboxed Code block (QuickJS sidecar) | ✔ | | |
| Logic check (Rust authority) + export gate + Draft Export hatch | ✔ | | |
| Export: canonical JSON, LangGraph, OpenAI Agents SDK | ✔ | | |
| Oppy: analyze, explain, propose edits | ✔ | | |
| Custom blocks from sub-graphs (Path A - drill-in breadcrumbs) | ✔ | | |
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

---

## 3. Architecture & Canvas Performance

Business logic lives in Rust crates that are testable without the UI and usable headless.

```
opifex/
  crates/
    opifex-core/      graph schema, types, validator, expression language, migrations
    opifex-engine/    scheduler, node runtime, providers, run store, cancellation
    opifex-sandbox/   Tauri sidecar binary: QuickJS worker (Code block)
    opifex-export/    Exporter trait + json, langgraph, openai exporters
    opifex-cli/       headless: validate, run, export, lint-blocks (CI harness)
  src-tauri/          thin Tauri commands over the crates + sidecar manager
  src/                Vue 3 + TS frontend
```

### Drag-and-Drop Feedback vs. Rust Validator Authority
To ensure **60fps UI performance** while maintaining a single source of validation truth:
1. **Frontend TS Evaluator:** A lightweight `canConnect(sourcePort, targetPort)` evaluator runs in TypeScript during edge-dragging. It uses cached rules derived directly from shared JSON conformance vectors generated by `opifex-core`.
2. **Rust Authority:** `opifex-core` runs full validation on drop, parameter edits, on-demand checks, pre-run, and pre-export.

---

## 4. Data Model & Versioning

### 4.1 SQLite Schema (Additive)

```sql
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

ALTER TABLE agents ADD COLUMN default_graph_id TEXT;

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
    status TEXT NOT NULL,
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
    scope_json TEXT NOT NULL,
    trigger_json TEXT,
    status TEXT NOT NULL CHECK (status IN ('running','succeeded','failed','cancelled')),
    error_json TEXT,
    started_at TEXT NOT NULL,
    ended_at TEXT
);

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
    tokens_in INTEGER, tokens_out INTEGER,
    started_at TEXT, ended_at TEXT
);

CREATE TABLE IF NOT EXISTS pinned_data (
    graph_id TEXT NOT NULL REFERENCES agent_graphs(id) ON DELETE CASCADE,
    node_id TEXT NOT NULL,
    port_id TEXT NOT NULL,
    value_json TEXT NOT NULL,
    source_node_run_id TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    PRIMARY KEY (graph_id, node_id, port_id)
);

CREATE TABLE IF NOT EXISTS agent_grants (
    agent_id TEXT NOT NULL REFERENCES agents(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('network')),
    value TEXT NOT NULL,
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

---

## 5. Block System & UX Improvements

### 5.1 Hybrid Tool Wiring (Spaghetti Reduction)
Wiring multiple tool blocks into an `Agent` node can cause visual wire tangling. 
* **Canvas Wiring:** Tool output handles can be connected directly to the `tools` port handle on an Agent node.
* **Inline Inspector Drawer:** Alternatively, selecting an Agent node opens an **Attached Tools Drawer** in the inspector. Users can attach/detach available tool blocks in the graph using a simple toggle/selector, updating the graph structure cleanly without cluttering the canvas visual layer.

### 5.2 Sub-Graph Editing
* Multi-node selections can be collapsed into custom sub-graph blocks (Path A).
* Sub-graph editing uses **drill-in navigation with breadcrumbs** (e.g., `Main Graph > Process Workflow > Parse Subgraph`) rather than modals, retaining spatial awareness.

---

## 6. Block Catalog (Top 10)

| # | Block | Kind | Inputs | Outputs | Side Effects |
|---|-------|------|--------|---------|--------------|
| 1 | **Input** | trigger | none | `message` (string) or `payload` (json) | none |
| 2 | **LLM Prompt** | llm | `message` (string), `system` (string, opt) | `completion` (string) | read |
| 3 | **Structured LLM** | llm | `message` (string) | `result` (json, schema-typed) | read |
| 4 | **Agent** | agent | `prompt` (string), `tools` (tool, multi) | `answer` (string), `trace` (json), `status` (string) | per tools |
| 5 | **HTTP Request** | tool | `url`, `body`, `headers` | `response` (json); also a `tool` output | read/write |
| 6 | **Condition** | control | `value` (any) | `true`, `false` | none |
| 7 | **Loop (For Each)** | control | `items` (array<any>), pass-through | `results` (array<any>), `errors` (json) | per body |
| 8 | **Merge** | control | multiple (skip-tolerant) | `merged` (any) | none |
| 9 | **Code (sandboxed JS)** | utility | `input` (any) | `output` (any); also a `tool` output | none |
| 10 | **Final Answer** | output | `message` (string/json) | none | none |

---

## 7. Sandbox & Security Architecture

### 7.1 QuickJS Windows Sidecar Execution
* The Code block executes JavaScript via an out-of-process worker (`opifex-sandbox-x86_64-pc-windows-msvc.exe`) managed through Tauri v2's sidecar command API.
* The sidecar process runs restricted under a **Windows Job Object**:
  * Memory limit (default 128 MB per worker)
  * CPU execution cap (hard limit 5,000 ms)
  * Single process count limit (prevents child process spawning)
  * Automatic termination on parent exit (`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`).
* Host capabilities are absent by default (no file system access, no direct network sockets).

---

## 8. Export Engineering & Draft Hatch

### 8.1 Export Targets
1. **Canonical JSON (`opifex.agent/1`):** Complete graph definition executable via `opifex-cli run`.
2. **LangGraph (Python):** Native Python project with graph definitions and explicitly pinned `langgraph` dependency versions in `requirements.txt`.
3. **OpenAI Agents SDK (Python):** Native Python project targeting `openai-agents` with pinned dependencies in `requirements.txt`.

*Note on Code Block (JS):* Since exported projects target Python, custom JS Code blocks emit a clean `NotImplementedError` stub with original JS source commented in python docstrings.

### 8.2 Export Validation Gate & "Export Draft" Release Hatch
* **Default Gate:** Exporting requires a clean validation pass (`0` errors).
* **"Export Draft" Hatch:** If validation errors exist, the user can explicitly trigger "Export Draft". The exporter embeds `validation.status: "invalid"` and a structured error summary inside the JSON metadata and generated `README.md`, enabling active external debugging without losing state.

---

## 9. Automated CI Enforcement Rules (Agent-Assisted Development)

To ensure high-velocity, agent-assisted code contributions never break schema contracts or export capabilities, CI enforces three strict automated checks:

```
                  +-----------------------------------+
                  |   Agent/Dev Submits PR / Code     |
                  +-----------------------------------+
                                    |
        +---------------------------+---------------------------+
        |                           |                           |
        v                           v                           v
+---------------+           +---------------+           +---------------+
| Rule 1:       |           | Rule 2:       |           | Rule 3:       |
| Shared Vector |           | Block Spec    |           | Golden Export |
| Conformance   |           | & DoD Linter  |           | Regression    |
+---------------+           +---------------+           +---------------+
        |                           |                           |
        +---------------------------+---------------------------+
                                    |
                                    v
                  +-----------------------------------+
                  |     CI Pass: Merge Allowed        |
                  +-----------------------------------+
```

### Rule 1: Shared Conformance Vector Test (`opifex-core` & Frontend)
* Any changes to port matching logic or expression parsing must be accompanied by JSON test vectors in `crates/opifex-core/tests/vectors/`.
* `cargo test --test conformance` and `vitest run conformance` run against the exact same test vector files. CI fails if Rust and TypeScript logic diverge.

### Rule 2: Block Spec & DoD Linter
* `opifex-cli lint-blocks` runs in CI.
* Validates that every registered block has:
  1. A corresponding markdown specification in `docs/blocks/<block-id>.md`.
  2. Executable unit tests covering happy path, boundary validation, and redaction.

### Rule 3: Golden-File Export Regression
* Exporters undergo golden-file snapshot testing (`opifex-export`).
* CI fails if generated LangGraph or OpenAI SDK Python output changes without an explicit golden snapshot commit update.

---

## 10. Milestones & Execution Plan

### M0 — Spikes (De-risk Critical Foundations)
* **S1: Vue Flow Adapter & Canvas Spike:** Verify performance with 200+ nodes, verify keyboard accessibility hooks, test `canConnect` drag evaluator.
* **S2: Model Probe & Tier Benchmarking:** Benchmarking Ollama model suite for Tier 1 (structured JSON) and Tier 2 (agent tool call) thresholds.
* **S3: QuickJS Tauri Sidecar Sandbox:** Package `opifex-sandbox-x86_64-pc-windows-msvc.exe` inside Tauri v2 sidecar layout; run Job Object escape/resource test suite.
* **S4: Expression Language Grammar:** Parser tests for `{{ ... }}` reference checking and function whitelist in Rust.
* **S5: End-to-End Export & Cross-Runner Equivalence:** Export a 4-node test graph to LangGraph / OpenAI SDK and run executed code in CI against fake model fixtures.

### M1 — Core (`opifex-core`)
Schema, types, port compatibility, shared conformance vectors, Rust validator authority, expression parser, SQLite migrations.

### M2 — Engine (`opifex-engine`)
Scheduler, run store, cancellation, fake provider, CLI harness (`opifex-cli`).

### M3 — Canvas Shell (Frontend)
Palette, Vue Flow adapter, node inspector, hybrid tool drawer, breadcrumbs sub-graph drill-in, outline view, accessibility/keyboard model.

### M4 — Run & Pin UX
Per-node run, run to here, live/mock mode, data pinning, stale tracking, credentials manager.

### M5 — Remaining Block Catalog
Complete all top 10 blocks adhering to the Definition of Done.

### M6 — Export System
LangGraph and OpenAI Agents SDK exporters, capability reports, draft export hatch, golden export CI integration.

### M7 — Oppy Assistant
Context tools, diff previews, atomic edit-operation apply/undo, model tier gating.

### M8 — Packaging & Developer Ship
Custom block definition exports, v1 conversion scripts, installer builds, developer documentation.
```

---

### Summary of Changes in Rev 3
1. **External Developer Polish:** Emphasized gold-standard developer experience, actionable validator error messages, and zero-friction export setups (`requirements.txt`, `.env.example`).
2. **Hybrid Tool Wiring:** Solved potential "spaghetti wire" issues by offering an inline tool-attachment drawer inside the Agent inspector alongside traditional visual edges.
3. **60fps Canvas Feedback:** Combined client-side TS port-matching evaluation for drag interactions with Rust's absolute validator authority on drops and saves.
4. **Tauri v2 Sandbox Packaging:** Specified sidecar executable naming and Tauri configuration in Spike S3.
5. **Draft Export Hatch:** Added an explicit release hatch allowing developers to export invalid draft graphs for external debugging.
6. **CI Enforcement Rules:** Standardized three automated CI guardrails (Shared Conformance Vectors, Block Specification Linter, and Golden-File Export Regression).

Plan Rev 3 is finalized. We are ready to initialize **M0 (Spikes S1–S5)**.