# Option 4: Manifest-Driven Agent Registry Plan

## Goal
Replace hardcoded starter agents in Rust with file-based seed agents, and scan a user agents directory for `*.json` agent files to load as user agents on startup. The 5 seed files are starter agents; everything else found outside the seed directory is a user agent.

## Current State
- `seed_starter_agents()` in `src-tauri/src/commands/agent.rs` hardcodes 5 agents as `serde_json::json!` macros (lines 277-338).
- `import_agent()` reads JSON from a string argument (user-triggered import via UI dialog).
- `Research Agent.json` sits in the project root but is not referenced by any code.
- `init()` in `src/stores/agentStore.ts` calls `ipcSeedStarterAgents()` then `ipcListAgents(true)`.

## Design Decisions

### Directory layout
| Path | Purpose |
|------|---------|
| `src-tauri/seeds/*.json` | 5 starter agent files (bundled, read via `include_str!`) |
| `src-tauri/seeds/agents.manifest.json` | Manifest listing starter file paths |
| `<appData>/Opifex/agents/*.json` | User agent files (scanned at startup) |

### Manifest format (`src-tauri/seeds/agents.manifest.json`)
```json
{
  "starters": [
    "customer_support.json",
    "research_agent.json",
    "rag_knowledge_agent.json",
    "code_review_agent.json",
    "content_writer_agent.json"
  ]
}
```

### Flow
1. `init()` calls `ipcSeedStarterAgents()` (unchanged signature)
2. `seed_starter_agents()` reads manifest, loads each seed file via `include_str!`, validates, inserts with `is_starter = 1` (skip if name already exists)
3. `init()` calls new `ipcLoadUserAgents()`
4. `load_user_agents()` scans `<appData>/Opifex/agents/*.json`, validates, inserts with `is_starter = 0` (skip if name already exists)
5. `init()` calls `ipcListAgents(true)` to load all agents into the store

### Deduplication
- Starters: skip if an agent with the same `name` and `is_starter = 1` already exists
- User agents: skip if an agent with the same `name` and `is_starter = 0` already exists
- Validation: same as `import_agent` — `goal` must exist and be non-empty; `steps` must exist

### Research Agent.json
Moves from project root to `src-tauri/seeds/research_agent.json` as a starter seed file. The existing hardcoded "Research Agent" in `seed_starter_agents` is removed and replaced by the file-based version.

## Files to Change

### Rust: `src-tauri/src/commands/agent.rs`
1. Replace hardcoded `starters` vec in `seed_starter_agents()` with manifest-driven file loading using `include_str!`
2. Add `load_user_agents(user_agents_dir: String)` command that scans a directory for `*.json` files
3. Add `use std::fs;` and `use std::path::Path;` imports
4. Add `use serde_json::Value;` (already imported via `Agent` struct)

### Rust: `src-tauri/src/commands/mod.rs`
- Add `load_user_agents` to the `pub use agent::{...}` export list

### Rust: `src-tauri/src/lib.rs`
- Add `load_user_agents` to imports and `generate_handler![...]`

### TypeScript: `src/stores/agentStore.ts`
- In `init()`, add call to `ipcLoadUserAgents()` after `ipcSeedStarterAgents()` and before/after `loadAgents()`
- Add `ipcLoadUserAgents` to the import from `@/services/ipc`

### TypeScript: `src/services/ipc.ts`
- Add `export async function loadUserAgents(userAgentsDir: string): Promise<string[]>`

### New files
- `src-tauri/seeds/customer_support.json`
- `src-tauri/seeds/research_agent.json`
- `src-tauri/seeds/rag_knowledge_agent.json`
- `src-tauri/seeds/code_review_agent.json`
- `src-tauri/seeds/content_writer_agent.json`
- `src-tauri/seeds/agents.manifest.json`

### Delete
- `Research Agent.json` from project root (moved to seeds)

## Data Flow
```
init()
  └─> ipcSeedStarterAgents()
        └─> read agents.manifest.json
        └─> for each starter file: include_str!, validate, INSERT is_starter=1
  └─> ipcLoadUserAgents(appDataDir/agents)
        └─> scan *.json files
        └─> for each: read, validate, INSERT is_starter=0
  └─> ipcListAgents(true) -> state.agents
```

## Failure Modes
| Failure | Handling |
|---------|----------|
| Manifest missing or malformed | Return error, starters not loaded (fails fast) |
| Seed file missing or malformed | Skip that file, continue with others, log error |
| User agents dir missing | Treat as empty (no error) |
| User agent file malformed | Skip that file, continue with others |
| Duplicate name (starter or user) | Skip insert, continue |
| DB insert failure | Return error to frontend |

## Validation
- `cargo check` passes
- `npx vitest run` passes (15 tests)
- Manual test: launch app, verify 5 starter agents appear in library
- Manual test: place a valid `*.json` in `<appData>/Opifex/agents/`, relaunch, verify it appears as a user agent
- Manual test: place an invalid `*.json` (missing `goal`), relaunch, verify it is skipped and no crash

## Open Questions
1. **User agents directory**: Use `<appData>/Opifex/agents/` (recommended for production portability) or scan from a path relative to the binary? (Recommended: app data dir)
2. **Research Agent.json at root**: Delete it after moving to seeds, or keep as a backup? (Recommended: delete from root, it lives in seeds now)
3. **Manifest location**: `src-tauri/seeds/agents.manifest.json` or `src-tauri/agents.manifest.json`? (Recommended: alongside seed files)

## Rollback
- If manifest loading fails, fall back to the previous hardcoded vec temporarily
- DB already has `is_starter` column, so no migration needed

---
**Note**: This is a plan file only. No source files have been modified. An implementation-capable agent is needed to execute the file changes.