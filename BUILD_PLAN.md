# Opifex — Agent Builder Build Plan

## Vision
A personal (MVP) / multi-tenant SaaS (future) desktop app for businesses to build AI agents via a chat-driven wizard. Left pane: chat with a Wizard LLM. Right pane: a live structured plan that updates after every turn. Agents are saved to a local library; starter templates ship with the app.

## Stack
| Layer | Choice |
|-------|--------|
| Shell | Tauri v2 |
| Frontend | Vue 3 + TypeScript + Vite |
| Backend | Rust (Tauri commands) |
| Storage | SQLite (single local file, `better-sqlite3` or `rusqlite`) |
| LLM | Ollama HTTP API (default), BYOK OpenAI/Anthropic later |
| Ollama model mgmt | CLI (`ollama pull`/`ollama list` via Tauri command) |

## Core Concepts
- **Wizard chat**: User describes what they want the agent to do. The Wizard asks clarifying questions, then produces a structured agent spec (JSON).
- **Live plan**: After every chat turn, the right pane re-renders the current agent plan (goal, steps, tools, knowledge sources, triggers, outputs).
- **Agent library**: Local list of saved agents + 5–6 provided starter templates (cloneable/iterable).
- **Model selection**: Per-agent default model; list installed Ollama models; pull new ones via CLI.

## Data Model (SQLite)
Tables:
- `agents` (id, name, description, spec_json, model, created_at, updated_at, is_starter)
- `conversations` (id, agent_id, title, created_at)
- `messages` (id, conversation_id, role, content, turn_plan_json, created_at)
- `settings` (key, value) — Ollama host, default provider, etc.

## File Structure (Tauri v2)
```
src-tauri/
  src/
    main.rs            -- Tauri app entry, menu, window setup
    commands/
      mod.rs           -- command registration
      agent.rs         -- CRUD for agents
      conversation.rs  -- save/load chat history
      ollama.rs        -- list/pull models, chat completion
      plan.rs          -- generate/update plan from chat turn
    db.rs              -- SQLite connection pool, schema migrations
    state.rs           -- AppState (Mutex<DbPool>)
  Cargo.toml
  tauri.conf.json
src/
  main.ts              -- Vue entry
  App.vue              -- root, split-pane layout
  components/
    ChatPane.vue       -- left: message list + input
    PlanPane.vue       -- right: live agent plan render
    AgentLibrary.vue   -- saved agents + starter templates
    OllamaPanel.vue    -- model list/pull status
  stores/
    agentStore.ts      -- Pinia or Vue reactive state
  services/
    ollama.ts          -- HTTP client to Ollama
    ipc.ts             -- invoke() wrappers for Tauri commands
  types/
    agent.ts           -- Agent, Plan, Message types
```

## Phases

### Phase 1 — MVP (local-only personal tool) — COMPLETE
1. Scaffold Tauri v2 + Vue 3 + Vite project.
2. SQLite schema + Rust commands for agent CRUD.
3. Ollama integration: list models, pull model via CLI, chat completion endpoint.
4. Split-pane UI: ChatPane (left), PlanPane (right).
5. Wizard flow: user chat → LLM returns structured plan JSON → PlanPane re-renders after every turn.
6. Save/load agents to local library with conversation/message persistence.
7. Ship 5 starter agent templates (read-only, cloneable): Customer Support, Research, RAG Knowledge, Code Review, Content Writer.

### Phase 2 — Polish — COMPLETE
- Agent export/import (JSON) — export_agent, import_agent commands + UI
- Agent library management (rename, delete, duplicate) — rename_agent, duplicate_agent commands + UI
- Plan versioning / history per conversation — plan_versions table, save_plan_version, list_plan_versions, restore_plan_version commands + UI
- BYOK provider support (OpenAI, Anthropic) — provider abstraction, API key storage, multi-provider chat routing

### Testing — COMPLETE
- Rust unit tests: 6 passing (agent CRUD, export/import, plan versioning)
- Frontend Vitest: 12 passing (agent store actions, state management)

### Phase 3 — SaaS (future)
- Multi-tenancy, auth, cloud sync.
- Team collaboration on agents.

## Open Discussion Points (resolved)
- Storage: SQLite (local file) — chosen over temp files / file-based JSON for structured queries and SaaS migration path.
- Ollama model download: CLI — chosen over iframe to avoid CSP/CORS complexity.
- Frontend: Vue 3 — chosen for professional UI.

## Next Steps
1. Confirm this plan.
2. Scaffold the Tauri v2 + Vue project.
3. Implement SQLite schema and agent CRUD commands.
4. Build Ollama service (list/pull/chat).
5. Build split-pane UI with wizard → live plan loop.
