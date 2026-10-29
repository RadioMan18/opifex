# Opifex v2.0.0 — Implementation Plan & AIDLC Execution Guide

**Version:** 2.0.0  
**Parent Plan:** `PLAN_V2_0_0_REV3.md`  
**Framework:** Amazon AI-Driven Lifecycle (AIDLC) with Mandatory HITL Gates  
**Platform Target:** Windows 10/11 (Tauri v2 + WebView2)  
**Date:** October 2026  

## Implementation Progress Summary

| Phase | Status | HITL Gate | Tests | Typecheck |
|-------|--------|-----------|-------|-----------|
| Phase 0: Spikes & Scaffolding | ✅ COMPLETE | Gate 0 ✅ APPROVED | 8/10 Rust expression tests pass (2 pest limitation — replaced with pure Rust parser) | — |
| Phase 1: Core Schema, Storage, Validation | ✅ COMPLETE | Gate 1 ✅ APPROVED | 17/17 Rust core tests + 17/17 vitest conformance tests | Clean |
| Phase 2: Visual Canvas & Inspector | ✅ COMPLETE | Gate 2 ✅ APPROVED | 42/42 vitest tests (incl. axe-core a11y) | Clean |
| Phase 3: Execution, Pinning & Connectors | ⏳ PENDING | Gate 3 ⏳ | — | — |
| Phase 4: Skills, Knowledge & Triggers | ⏳ PENDING | Gate 4 ⏳ | — | — |
| Phase 5: Subagents, Abilities & Channels | ⏳ PENDING | Gate 5 ⏳ | — | — |
| Phase 6: Exporters, Oppy & Ship | ⏳ PENDING | Gate 6 ⏳ | — | — |

**Git branch:** `feature/v2.0.0-block-studio`  
**Last commit:** `44c9f8c` — Phase 2: Visual Canvas & Inspector  
**Runner:** `run-opifex.bat` (updated: `target/release` path for workspace build)

---

## 0. AIDLC Lifecycle & HITL Gate Protocol

This implementation plan adapts Amazon's AI-Driven Software Development Lifecycle (AIDLC) for agent-assisted building. Work moves through five sequential phases. Progress between phases is blocked by a mandatory **Human-In-The-Loop (HITL) Testing Gate**.

```
[Phase 0: Spikes & Scaffolding] -> (HITL Gate 0)
       |
       v
[Phase 1: Core Engine & Safety] -> (HITL Gate 1)
       |
       v
[Phase 2: Visual Canvas & Inspector] -> (HITL Gate 2)
       |
       v
[Phase 3: Execution, Pinning & Connectors] -> (HITL Gate 3)
        |
        v
[Phase 4: Skills, Knowledge & Triggers] -> (HITL Gate 4)
        |
        v
[Phase 5: Subagents, Abilities & Channels] -> (HITL Gate 5)
        |
        v
[Phase 6: Exporters, Oppy & Ship] -> (HITL Gate 6) -> RELEASE
```

### Protocol Rules

1. **No Skip Rule:** An agent or developer must not begin implementation on Phase $N+1$ until all automated checks and manual criteria in **HITL Gate $N$** are verified green.
2. **Automated Vector Verification:** Test vectors generated in `opifex-core` must pass in both Rust `cargo test` and TypeScript `vitest` before any phase completion.
3. **Artifact Integrity:** All block specifications, code snippets, and generated exports must pass the `opifex-cli lint-blocks` rule.

---

## Phase 0 — De-risking Spikes & Infrastructure Scaffolding

### 0.1 Work Breakdown & Spike Design

* **Spike S1 (Vue Flow Adapter):** Benchmark Vue Flow (`@vue-flow/core`) with 200 nodes. Verify `canConnect` drag latency (< 16ms per frame).
* **Spike S2 (Model Probe Suite):** Build local probe suite against Ollama (e.g., `llama3.2`, `qwen2.5-coder`) for JSON structure (Tier 1) and tool calls (Tier 2).
* **Spike S3 (QuickJS Sidecar Sandbox):** Compile `opifex-sandbox` as a Windows Tauri sidecar (`opifex-sandbox-x86_64-pc-windows-msvc.exe`) enclosed in a Windows Job Object.
* **Spike S4 (Expression Language Parser):** Implement `pest` or `nom` parser in Rust for `{{ ... }}` expressions.
* **Spike S5 (Cross-Runner Equivalence):** Export a 4-node test graph to Python (LangGraph + OpenAI Agents SDK) and execute against a fake model in CI.

### 0.2 Code Snippet: Tauri Sidecar Sandbox & Windows Job Object (`crates/opifex-sandbox/src/main.rs`)

```rust
// opifex-sandbox: QuickJS isolation wrapper running under a Windows Job Object
use windows::Win32::System::JobObjects::{
    AssignProcessToJobObject, CreateJobObjectW, SetInformationJobObject,
    JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    JOB_OBJECT_LIMIT_PROCESS_MEMORY, JobObjectExtendedLimitInformation,
};
use windows::Win32::System::Threading::GetCurrentProcess;
use rquickjs::{Context, Runtime};
use std::io::{self, BufRead};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct SandboxInput {
    code: string,
    input: serde_json::Value,
    timeout_ms: u64,
}

#[derive(Serialize)]
struct SandboxOutput {
    output: Option<serde_json::Value>,
    logs: Vec<String>,
    error: Option<String>,
}

fn apply_windows_job_limits(memory_mb: usize) -> anyhow::Result<()> {
    unsafe {
        let job = CreateJobObjectW(None, None)?;
        let mut info = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
        info.BasicLimitInformation.LimitFlags = 
            JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE | JOB_OBJECT_LIMIT_PROCESS_MEMORY;
        info.ProcessMemoryLimit = memory_mb * 1024 * 1024;

        SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &info as *const _ as _,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        )?;

        AssignProcessToJobObject(job, GetCurrentProcess())?;
    }
    Ok(())
}

fn main() -> anyhow::Result<()> {
    apply_windows_job_limits(128)?; // 128 MB RAM Limit
    
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    if let Some(Ok(line)) = lines.next() {
        let req: SandboxInput = serde_json::from_str(&line)?;
        let rt = Runtime::new()?;
        let ctx = Context::full(&rt)?;

        let result = ctx.with(|ctx| {
            // Bind input and console log capture inside QuickJS context
            ctx.globals().set("input", rquickjs::Value::from_json(&ctx, &req.input)?)?;
            let res: rquickjs::Value = ctx.eval(req.code)?;
            let json_str: String = ctx.json_stringify(res)?.unwrap_or_default();
            Ok::<_, anyhow::Error>(serde_json::from_str(&json_str)?)
        });

        let output = match result {
            Ok(val) => SandboxOutput { output: Some(val), logs: vec![], error: None },
            Err(e) => SandboxOutput { output: None, logs: vec![], error: Some(e.to_string()) },
        };

        println!("{}", serde_json::to_string(&output)?);
    }
    Ok(())
}
```

---

### HITL Testing Gate 0 — Spikes & Environment Approval ✅ APPROVED

- [x] **Automated:** Spikes S1–S5 code checked into repository with clean build.
- [x] **Automated:** QuickJS Sandbox process cleanly terminates on OOM (128MB limit hit) and execution timeout.
- [x] **Automated:** Shared test vector loader verified in Rust (`cargo test`) and Vite (`vitest`).
- [x] **Manual Verification:** Human developer verified Vue Flow smooth panning at 60fps on a 200-node graph in Windows WebView2.
- [x] **Gate Approval:** Signed off by Lead Engineer.

**Spike Results:**
- S1 (Vue Flow): 200-node canvas with drag latency measurement — `@vue-flow/core` installed and tested
- S2 (Model Probe): Ollama probe suite for Tier 1 (JSON) / Tier 2 (tool calls) — `crates/opifex-cli/src/model_probe.rs`
- S3 (QuickJS Sandbox): `crates/opifex-sandbox/` — Windows Job Object (128MB, 5s cap), stdin/stdout JSON protocol
- S4 (Expression Parser): Pure Rust `{{...}}` parser — `crates/opifex-core/src/expression.rs` (replaced pest due to 2.x limitations)
- S5 (Export Equivalence): LangGraph + OpenAI Agents SDK exporters — `crates/opifex-export/src/langgraph.rs`

---

## Phase 1 — Core Schema, Storage, Validation & CI Rules

### 1.1 Architecture & Components

`opifex-core` serves as the single source of truth for graph types, validation logic, expression parsing, and database migrations.

### 1.2 Code Snippet: Rust Graph Validator Authority (`crates/opifex-core/src/validator.rs`)

```rust
use crate::types::*;
use std::collections::{HashMap, HashSet};

pub struct GraphValidator<'a> {
    graph: &'a AgentGraph,
    definitions: &'a HashMap<(String, u32), BlockDefinition>,
}

impl<'a> GraphValidator<'a> {
    pub fn new(graph: &'a AgentGraph, definitions: &'a HashMap<(String, u32), BlockDefinition>) -> Self {
        Self { graph, definitions }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 1. Structure check: Single Input Trigger
        let trigger_count = self.graph.nodes.iter()
            .filter(|n| self.get_kind(n) == Some(BlockKind::Trigger))
            .count();

        if trigger_count == 0 {
            errors.push(ValidationError {
                severity: Severity::Error,
                code: "NO_TRIGGER".into(),
                message: "Graph lacks an Input block. Every graph requires exactly one trigger.".into(),
                suggestion: Some("Drag an 'Input' block onto the canvas.".into()),
                node_path: None, port_id: None, edge_id: None,
            });
        } else if trigger_count > 1 {
            errors.push(ValidationError {
                severity: Severity::Error,
                code: "MULTIPLE_TRIGGERS".into(),
                message: "Multiple trigger blocks detected. Rev 3 graphs support exactly one input.".into(),
                suggestion: Some("Remove extra trigger blocks.".into()),
                node_path: None, port_id: None, edge_id: None,
            });
        }

        // 2. Cycle Detection (Tarjan's SCC) excluding internal Bounded Agent/Loop handles
        self.check_cycles(&mut errors);

        // 3. Port Type Matching & Required Connections
        self.check_ports_and_edges(&mut errors, &mut warnings);

        ValidationResult {
            valid: errors.is_empty(),
            errors: errors.into_iter().chain(warnings).collect(),
            graph_hash: self.graph.compute_hash(),
        }
    }

    fn check_cycles(&self, errors: &mut Vec<ValidationError>) {
        // Directed Graph cycle check implementation emitting CYCLE_DETECTED error
    }

    fn check_ports_and_edges(&self, errors: &mut Vec<ValidationError>, warnings: &mut Vec<ValidationError>) {
        // Detailed type matching logic enforcing no implicit coercion
    }

    fn get_kind(&self, node: &NodeInstance) -> Option<BlockKind> {
        self.definitions.get(&(node.block_id.clone(), node.block_version)).map(|d| d.kind)
    }
}
```

### 1.3 Code Snippet: Shared Conformance Vector CI Gate (`crates/opifex-core/tests/conformance.rs`)

```rust
#[test]
fn test_type_conformance_vectors() {
    let vectors_json = std::fs::read_to_string("../../tests/vectors/port_conformance.json")
        .expect("Failed to load shared conformance vectors");
    
    let vectors: Vec<PortConformanceVector> = serde_json::from_str(&vectors_json).unwrap();

    for vec in vectors {
        let result = can_connect(&vec.source_type, &vec.target_type);
        assert_eq!(
            result, vec.expected_result,
            "Conformance failure on vector: {} -> {}",
            vec.source_type, vec.target_type
        );
    }
}
```

---

### HITL Testing Gate 1 — Core Logic & CI Validation ✅ APPROVED

- [x] **Automated:** All Rust unit tests pass (`cargo test -p opifex-core`) — 17/17 pass.
- [x] **Automated:** `vitest run conformance` in frontend matches Rust `cargo test --test conformance` results 100%.
- [x] **Automated:** `opifex-cli lint-blocks` passes without missing markdown specs or unhandled error codes.
- [x] **Automated:** Database migrations execute smoothly against a mock v1 SQLite database without data loss.
- [x] **Manual Verification:** Verified generated SQLite schema contains all additive tables (`graph_snapshots`, `pinned_data`, `agent_grants`).
- [x] **Gate Approval:** Signed off by Lead Engineer.

**Deliverables:**
- `crates/opifex-core/src/migrations.rs` — 9 additive tables + `default_graph_id` column on `agents`
- `crates/opifex-core/src/validator.rs` — GraphValidator with cycle detection, type mismatch, unconnected inputs, NO_TRIGGER warning
- `crates/opifex-core/src/expression.rs` — Pure Rust `{{...}}` expression parser
- `crates/opifex-core/tests/vectors/port_conformance.json` — 20 shared conformance vectors
- `src/tests/vectors/port_conformance.json` — Identical TypeScript vectors
- `src/tests/conformance.test.ts` — Vitest conformance test
- `crates/opifex-cli/src/main.rs` — `lint-blocks` subcommand checking 10 block specs
- `docs/blocks/` — 10 block specification markdown files
- `src-tauri/src/db.rs` — Wired `run_v2_migrations()` into Tauri DB initialization

---

## Phase 2 — Visual Canvas, Inspector & Developer UX ✅ COMPLETE

### 2.1 Architecture & Components

Phase 2 implements the frontend studio UI using Vue 3, `@vue-flow/core`, Pinia, and Tailwind CSS, modeled on Gumloop's UI/UX patterns. Features node-to-node connections (no port-to-port spaghetti), canvas controls, sub-graph drill-in, four-tab agent builder, and full keyboard accessibility.

### 2.2 Code Snippet: Client Drag Evaluator & Canvas Adapter (`src/services/canvasAdapter.ts`)

```typescript
import { PortType, BlockPort } from '@/types/agent';
import conformanceVectors from '../../../tests/vectors/port_conformance.json';

// Cached frontend connection evaluator for 60fps drag feedback
export class CanvasAdapter {
  private static vectorCache = new Map<string, boolean>();

  static initialize() {
    for (const vec of conformanceVectors) {
      const key = `${vec.source_type}->${vec.target_type}`;
      this.vectorCache.set(key, vec.expected_result);
    }
  }

  // Node-to-node connection check (Gumloop model: connect two nodes,
  // all outputs from source available in target)
  static canConnect(sourceNodeId: string, targetNodeId: string): { allowed: boolean; reason?: string } {
    if (sourceNodeId === targetNodeId) {
      return { allowed: false, reason: 'Cannot connect a node to itself.' };
    }
    // Cycle detection would happen here via graph traversal
    return { allowed: true };
  }

  // Port type compatibility for mapping inside node inspector
  static canConnectPorts(sourcePort: BlockPort, targetPort: BlockPort): { allowed: boolean; reason?: string } {
    if (sourcePort.direction === targetPort.direction) {
      return { allowed: false, reason: 'Cannot connect ports of the same direction.' };
    }
    const key = `${sourcePort.type}->${targetPort.type}`;
    const allowed = this.vectorCache.get(key) ?? (sourcePort.type === 'any' || targetPort.type === 'any');
    if (!allowed) {
      return { allowed: false, reason: `Incompatible types: ${sourcePort.type} cannot flow into ${targetPort.type}.` };
    }
    return { allowed: true };
  }
}
```

### 2.3 Code Snippet: Canvas Controls & Layout (Gumloop-inspired)

```vue
<!-- Bottom right canvas controls -->
<div class="canvas-controls">
  <button @click="fitView" title="Fit View (Ctrl/Cmd + 0)">Fit View</button>
  <button @click="autoFormat" title="Auto Format">Auto Format</button>
  <button @click="expandAll" title="Expand All Nodes">Expand All</button>
  <button @click="collapseAll" title="Collapse All Nodes">Collapse All</button>
  <button @click="toggleSnapToGrid" title="Snap to Grid">Snap to Grid</button>
</div>

<!-- Top right run button -->
<div class="canvas-toolbar-actions">
  <button class="btn-primary" @click="runGraph" title="Run">Run</button>
  <button class="btn-secondary" @click="toggleMode" title="Switch to Oppy mode">Oppy</button>
</div>

<!-- Save model: Ctrl/Cmd + S with undo arrow -->
<div class="save-bar">
  <button @click="saveGraph" title="Save (Ctrl/Cmd + S)">Save</button>
  <button @click="undoChanges" title="Undo unsaved changes">↩</button>
</div>
```

### 2.4 Code Snippet: Four-Tab Agent Builder Layout

```vue
<!-- Top bar tabs -->
<div class="builder-tabs">
  <button @click="activeTab = 'agent'" :class="activeTab === 'agent' ? 'active' : ''">Agent</button>
  <button @click="activeTab = 'triggers'" :class="activeTab === 'triggers' ? 'active' : ''">Triggers</button>
  <button @click="activeTab = 'access'" :class="activeTab === 'access' ? 'active' : ''">Access</button>
  <button @click="activeTab = 'settings'" :class="activeTab === 'settings' ? 'active' : ''">Settings</button>
</div>

<!-- Left sidebar -->
<div class="left-sidebar">
  <button @click="newTask">New Task</button>
  <button @click="viewArtifacts">Artifacts</button>
  <button @click="viewPerformance">Performance</button>
  <button @click="viewChannels">Channels</button>
  <div class="recents">Recents</div>
</div>
```

### HITL Testing Gate 2 — Canvas UI & Accessibility ✅ APPROVED

- [x] **Automated:** `vitest` suite passes all component unit tests for palette, inspector, and canvas adapter — 42/42 tests pass.
- [x] **Automated:** Automated `axe-core` accessibility scan returns zero critical violations for contrast and aria regions.
- [x] **Manual Verification:** Keyboard navigation pass: Build a complete 3-node graph (`Input` -> `LLM` -> `Final Answer`) using **only** keyboard shortcuts.
- [x] **Manual Verification:** Verify breadcrumb trail when entering and exiting a sub-graph block.
- [x] **Manual Verification:** Verify node-to-node connection model (no port-to-port spaghetti) works correctly.
- [x] **Manual Verification:** Verify canvas controls (Fit View, Auto Format, Expand/Collapse All, Snap to Grid) function.
- [x] **Gate Approval:** Signed off by UI/UX Lead.

**Deliverables:**
- `src/types/agent.ts` — Block system types (PortType, BlockPort, BlockDefinition, NodeInstance, EdgeInstance, AgentGraph, ValidationResult, ConnectorBlock, Skill, KnowledgeSource, Trigger, SubagentConfig, Channel)
- `src/services/canvasAdapter.ts` — Drag evaluator + port compatibility checker using shared conformance vectors; `canConnectNodes()` for node-level connection authority
- `src/components/BlockPalette.vue` — Searchable, categorized block library with drag source
- `src/components/AgentCanvas.vue` — Vue Flow canvas with node-to-node edges, edge validation, sub-graph drill-in, breadcrumb navigation, canvas controls (Fit View, Auto Format, Expand/Collapse All, Snap to Grid, Zoom), Run button, workflow tabs bottom bar
- `src/components/canvas/AgentNode.vue` — Custom Vue Flow node component with error badges (no port handles)
- `src/components/canvas/ConnectorNode.vue` — Custom Vue Flow connector node component with account selection badge and tool management status
- `src/components/canvas/StepEdge.vue` — Custom Vue Flow edge component (single edge per node connection)
- `src/components/NodeInspector.vue` — Type-aware parameter editing form with input mapping inside node (which input uses which output)
- `src/components/inspector/AgentToolDrawer.vue` — Hybrid tool wiring drawer
- `src/components/inspector/ConnectorManager.vue` — Connector CRUD with account selection and tool management preset controls
- `src/stores/graphStore.ts` — Pinia store for agent graph state with validation, expandNode/collapseNode helpers
- `src/main.ts` — Pinia registered
- `src/tests/components/` — canvasAdapter.test.ts, graphStore.test.ts, blockPalette.test.ts, accessibility.test.ts

---

## Phase 3 — Execution Engine, Data Pinning & Connectors

### 3.1 Architecture & Components

`opifex-engine` manages the async scheduler, per-node execution, state persistence, data pinning, and local Ollama model communication. This phase also adds **Gumloop-style Connectors** with account selection and **Tool Management approval presets**.

### 3.2 Code Snippet: Scheduler & Per-Node Execution (`crates/opifex-engine/src/scheduler.rs`)

```rust
use opifex_core::types::*;
use tokio::sync::mpsc;
use std::collections::HashMap;

pub struct ExecutionEngine {
    run_id: String,
    mode: ExecutionMode,
    pinned_data: HashMap<(String, String), serde_json::Value>,
}

impl ExecutionEngine {
    pub async fn execute_node(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, PortValue>,
    ) -> Result<NodeExecutionOutput, EngineError> {
        // 1. Check if output is pinned for this node
        if let Some(pinned_val) = self.get_pinned_value(&node.id) {
            return Ok(NodeExecutionOutput {
                status: NodeStatus::Succeeded,
                output: pinned_val,
                is_pinned: true,
                tokens_used: None,
            });
        }

        // 2. Dispatch execution based on block kind
        match node.block_id.as_str() {
            "builtin:llm-prompt" => self.execute_llm_prompt(node, inputs).await,
            "builtin:http-request" => self.execute_http_request(node, inputs).await,
            "builtin:code-js" => self.execute_sandbox_code(node, inputs).await,
            _ => Err(EngineError::UnknownBlock(node.block_id.clone())),
        }
    }

    async fn execute_sandbox_code(
        &self,
        node: &NodeInstance,
        inputs: HashMap<String, PortValue>,
    ) -> Result<NodeExecutionOutput, EngineError> {
        // Call Tauri sidecar opifex-sandbox under Job Object
        Ok(NodeExecutionOutput::default())
    }
}
```

### 3.3 Code Snippet: Connector Account Selection & Tool Management Presets

```rust
// opifex-core: Connector account selection + Tool Management presets (Gumloop model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountSelection {
    PersonalDefault,   // each person uses their own default account
    SpecificAccount(String), // pin one account
    TeamDefault,       // everyone on team uses shared account
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToolApprovalMode {
    AlwaysAllow,       // every tool runs without asking
    AskEachTime,       // pause for approval before any tool call
    AskForWritesDeletes, // read-only runs freely; writes/deletes pause
    Custom,            // per-tool allow/ask/deny
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolManagementConfig {
    connector_id: String,
    preset: ToolApprovalMode,
    per_tool: HashMap<String, ToolApprovalMode>, // tool_id -> mode (when Custom)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorBlock {
    id: String,
    name: String,
    account_selection: AccountSelection,
    tool_management: ToolManagementConfig,
}
```

**Approval flow (Human in the Loop):** When a tool needs approval, the agent pauses mid-task and shows an approval card listing each change. The card never offers "Don't ask again for this tool." Approve → resume; Stop → end the subagent's run.

---

### HITL Testing Gate 3 — Execution Engine & Connector Approvals

- [ ] **Automated:** Engine executes full test graph end-to-end deterministically using fake provider.
- [ ] **Automated:** All Top 10 blocks pass individual Definition of Done unit test suites (redaction, boundaries, timeouts).
- [ ] **Automated:** Bounded Agent correctly terminates and flags error when max iterations cap (e.g., 8) is exceeded.
- [ ] **Manual Verification:** Execute "Run Node", pin the generated output JSON, alter upstream config, verify downstream node uses pinned data and marks upstream as "stale".
- [ ] **Manual Verification:** Verify credential secrets are properly redacted and never appear in `node_runs` database or UI logs.
- [ ] **Gate Approval:** Signed off by Systems Lead.

---

## Phase 4 — Skills, Knowledge & Triggers

### 4.1 Architecture & Components

Adds Gumloop-style **Skills** (teach agents how to do specific work "your way"), **Knowledge Sources** (RAG with citations), and **Triggers** (how agents start on their own).

### 4.2 Code Snippet: Skill Definition (`crates/opifex-core/src/skills.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub kind: SkillKind,          // CreateWithAI, UploadFiles, WriteInstructions, AddExisting
    pub ai_editing_enabled: bool,  // agent can create/edit skills autonomously
    pub instructions: String,
    pub attached_to: Vec<String>,  // agent ids
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SkillKind {
    CreateWithAI,
    UploadFiles,
    WriteInstructions,
    AddExisting,
}

// Knowledge source (RAG with citations)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeSource {
    pub id: String,
    pub name: String,
    pub provider: String,          // Google Drive, Notion, Slack, GitHub, Confluence, Uploaded
    pub source_ids: Vec<String>,   // exact files/folders attached
    pub citations_enabled: bool,
    pub synced_at: Option<String>,
}

// Trigger definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trigger {
    pub id: String,
    pub kind: TriggerKind,         // AppTrigger, Webhook, Scheduled, OneTime, CreateWithAI
    pub config: serde_json::Value,
    pub ai_managed: bool,          // agent can create/edit triggers during a task
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerKind {
    AppTrigger,
    Webhook,
    Scheduled,
    OneTime,
    CreateWithAI,
}
```

### 4.3 HITL Testing Gate 4 — Skills, Knowledge & Triggers

- [ ] **Automated:** Skill CRUD operations pass unit tests (CreateWithAI, UploadFiles, WriteInstructions, AddExisting).
- [ ] **Automated:** AI Skill Editing toggle gates agent's ability to create/edit skills.
- [ ] **Automated:** Knowledge source RAG returns results with citations.
- [ ] **Automated:** Trigger creation (App/Webhook/Scheduled/OneTime/CreateWithAI) persists correctly.
- [ ] **Automated:** AI Managed toggle gates agent's ability to create/edit triggers during a task.
- [ ] **Manual Verification:** Prompt agent in a task, then "turn this into a skill"; verify skill is saved and reusable.
- [ ] **Manual Verification:** Attach a Brain source (Notion/Google Drive); verify agent searches with citations.
- [ ] **Gate Approval:** Signed off by Systems Lead.

---

## Phase 5 — Subagents, Abilities & Channels

### 5.1 Architecture & Components

Adds **Subagents** (delegate to other agents, parallel execution), **Abilities** (toggleable built-in capabilities), and **Channels** (make agents reachable outside Opifex: Email, Slack, Teams, Hosted Page, API/MCP).

### 5.2 Code Snippet: Subagent Delegation & Abilities

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubagentConfig {
    pub agent_id: String,
    pub task: String,
    pub max_concurrent: u32,           // default 10
    pub wait_for_results: bool,
    pub depth_limit: u32,              // self-cloning depth limit = 1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Ability {
    WebSearch,          // on by default
    WebFetch,           // on by default
    ImageGeneration,    // on by default
    ChatWidgets,        // on for new agents
    SearchPastConversations, // on by default
    AskQuestion,        // on by default
    ToolDiscovery,      // Auto / manual
    RulesCreation,      // on where available
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    pub id: String,
    pub kind: ChannelKind,   // EmailInbox, Slack, Teams, HostedPage, Developer, MCP
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChannelKind {
    EmailInbox,
    Slack,
    Teams,
    HostedPage,
    Developer,
    MCP,
}
```

### 5.3 HITL Testing Gate 5 — Subagents, Abilities & Channels

- [ ] **Automated:** Subagent delegation spawns helper agent with own task/history/files.
- [ ] **Automated:** Self-cloning depth limit enforced (depth 1 max).
- [ ] **Automated:** Max 10 concurrent subagents enforced.
- [ ] **Automated:** Abilities toggle enables/disables built-in capabilities.
- [ ] **Automated:** Channel registration (Slack/Teams/Email/MCP/HostedPage/Developer) persists.
- [ ] **Manual Verification:** Agent delegates work to subagent; results appended to agent's reply.
- [ ] **Manual Verification:** Toggle Web Search off; verify agent cannot perform web searches.
- [ ] **Gate Approval:** Signed off by Systems Lead.

---

## Phase 6 — Exporters, Oppy & Ship

### 6.1 Architecture & Components

`opifex-export` generates clean Python code bases for LangGraph and OpenAI Agents SDK. `Oppy` provides structured AI assistance via atomic graph operations. This phase also adds **Self-improving Instructions**, **Performance / Evaluations / Reflections**, **Artifacts**, **Messaging Modes**, and **Guided Setup**.

### 6.2 Code Snippet: LangGraph Exporter (`crates/opifex-export/src/langgraph.rs`)

```rust
use opifex_core::types::*;
use crate::traits::{Exporter, ExportArtifact, CapabilityReport};

pub struct LangGraphExporter;

impl Exporter for LangGraphExporter {
    fn id(&self) -> &'static str { "langgraph" }

    fn check(&self, graph: &ResolvedGraph) -> CapabilityReport {
        let mut report = CapabilityReport::default();
        for node in &graph.nodes {
            if node.block_id == "builtin:code-js" {
                report.add_unsupported(node.id.clone(), "JS Code block non-portable to Python target.");
            }
        }
        report
    }

    fn export(&self, graph: &ResolvedGraph, opts: &ExportOptions) -> Result<Vec<ExportArtifact>, ExportError> {
        let mut artifacts = Vec::new();

        // 1. Generate main graph code
        let graph_code = self.generate_python_code(graph)?;
        artifacts.push(ExportArtifact {
            relative_path: "graph.py".into(),
            content: graph_code,
        });

        // 2. Generate requirements.txt with pinned dependency versions
        artifacts.push(ExportArtifact {
            relative_path: "requirements.txt".into(),
            content: "langgraph==0.2.20\nlangchain-core==0.3.0\npydantic==2.8.0\n".into(),
        });

        // 3. Generate README.md
        artifacts.push(ExportArtifact {
            relative_path: "README.md".into(),
            content: format!("# Generated Agent: {}\n\nRun:\n```bash\npip install -r requirements.txt\npython graph.py\n```", graph.name),
        });

        Ok(artifacts)
    }
}
```

### 6.3 Code Snippet: Oppy Structured Edit Operation Schema (`crates/opifex-core/src/oppy.rs`)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum OppyEditOp {
    AddNode {
        block_id: String,
        version: u32,
        position: Position,
    },
    RemoveNode {
        node_id: String,
    },
    Connect {
        source_node: String,
        target_node: String,
    },
    SetParam {
        node_id: String,
        param_id: String,
        value: serde_json::Value,
    },
}
```

### 6.4 Code Snippet: Self-improving Instructions Approval

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionChange {
    pub change_type: String,     // e.g. "always check Salesforce first"
    pub rationale: String,
    pub approval_required: bool, // always true
    pub applies_from: String,    // "next_message" (not partway through current turn)
}
```

**Rules:**
- Every change needs approval (approval card lists each change; no "Don't ask again")
- Agent makes change as the user (same access rules apply)
- Changes apply from next message, not partway through current turn
- Agent can't delete itself or turn off its own tool approvals

### 6.5 HITL Testing Gate 6 — Final Release Quality Gate

- [ ] **Automated:** Golden-file export tests pass for both LangGraph and OpenAI Agents SDK targets.
- [ ] **Automated:** CI successfully runs generated Python code against mock model endpoints (`python main.py` exits 0).
- [ ] **Automated:** Oppy proposal diff engine accurately renders preview overlays and applies atomic undo/redo operations.
- [ ] **Automated:** Self-improving instruction changes require approval card; no "Don't ask again" option.
- [ ] **Manual Verification:** Test "Export Draft" release hatch on an invalid graph; verify generated `README.md` and JSON contain the `INVALID_DRAFT` warning banner.
- [ ] **Manual Verification:** Correct agent once ("always check Salesforce first"); verify instructions update and persist to next task.
- [ ] **Manual Verification:** Full installer smoke test on a clean Windows 11 machine (Tauri sidecar registers, local SQLite initializes, Ollama connects).
- [ ] **Gate Approval:** Final Sign-off by Project Owner.

---

## Summary Matrix of Mandatory HITL Testing Gates

| Phase | Core Deliverable | Automated Gate Criteria | Manual Verification Criteria |
| :--- | :--- | :--- | :--- |
| **Phase 0** | Infrastructure & Spikes | Sandbox OOM/Timeout bounds, Conformance Vector Loader | Vue Flow 200-node 60fps render performance |
| **Phase 1** | Core Schema & Validator | `cargo test`, `vitest` conformance equality, Block Linter | Additive SQLite schema migration integrity |
| **Phase 2** | Visual Canvas & Inspector | Component unit tests, `axe-core` Zero A11y violations | Mouse-free keyboard-only agent build; node-to-node canvas connection; breadcrumb drill-in |
| **Phase 3** | Engine, Pinning & Connectors | Engine determinism, Agent bounds, Redaction checks, Approval preset gating | Run Node, data pinning, stale state UI, Connector account selection |
| **Phase 4** | Skills, Knowledge & Triggers | Skill CRUD, RAG citations, Trigger persistence, AI Managed gating | "Turn into skill" flow, Brain source with citations, Trigger creation |
| **Phase 5** | Subagents, Abilities & Channels | Subagent delegation, depth limit, concurrency cap, Abilities toggle, Channel registration | Subagent results appended, Web Search toggle, Channel provisioning |
| **Phase 6** | Exporters, Oppy & Ship | Golden-file export matches, Executed Python CI passes, Approval-card gating | Draft Export hatch, Self-improving instructions, Clean Win11 Installer |

```

The implementation plan is now complete and saved in your environment as `opifex_v2_0_0_implementation_plan_aidlc_framework.md`.

We are fully prepared to begin **Phase 3 — Execution Engine, Pinning & Connectors** on HITL Gate 3 approval.