# Changelog

All notable changes to Opifex will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-10-21

### 🚀 First Public Release

**Opifex** — A desktop AI agent builder (Tauri v2 + Rust + Vue 3) that lets you design AI agents through a chat-driven wizard interface.

### Added

- **Oppy chat wizard** — Conversational interview that generates structured agent plans (goal, steps, tools, knowledge sources, triggers, outputs)
- **Agent library** — Save, rename, duplicate, delete, export, and import agents locally
- **Plan versioning** — Save named plan versions and restore previous versions
- **Starter agents** — 5 built-in agents: Customer Support, Research, RAG Knowledge, Code Review, Content Writer
- **Model switching** — Switch between Ollama (local), OpenAI, and Anthropic providers
- **API key management** — Securely store provider API keys in local settings
- **Agent export/import** — Export agents as JSON files; import from JSON
- **Plan versioning UI** — View and restore saved plan versions in the agent plan pane
- **Manifest-driven agent registry** — Seed agents loaded from `src-tauri/seeds/`; user agents scanned from `<appData>/Opifex/agents/`
- **Dark mode** — WCAG AA compliant light/dark theme support with Aether color palette
- **CI/CD** — GitHub Actions workflow running `cargo fmt` and frontend tests

### Changed

- **Light mode UI** — Buttons now have visible 3px borders and white backgrounds on cream bar
- **Starter vs user agents** — Agents now correctly separated into "Starter Agents" (isStarter=true) and "Your Agents" (isStarter=false)
- **Tauri version** — Bumped from 0.1.0 to 1.0.0
- **CSS variables** — Replaced hardcoded hex values with Aether theme variables throughout
- **Serde field naming** — Added `#[serde(rename_all = "camelCase")]` to command input/output structs for TypeScript compatibility

### Fixed

- **Chat entry disappearing on Enter** — `createNewConversation()` now wrapped in try/catch so user messages are pushed even if conversation creation fails
- **Tauri build error** — Removed unsupported `windows` property from `build` section in `tauri.conf.json`
- **Button visibility in light mode** — All top bar buttons now have dark borders and readable text on light backgrounds
- **Build Agent button** — Changed from low-contrast white-on-green to white button with dark text and border
- **Rust warnings** — Fixed snake_case naming, unused imports, and borrow issues

### Removed

- `Research Agent.json` from project root — moved to `src-tauri/seeds/research_agent.json` as a starter seed file

### Community

- **CODE_OF_CONDUCT.md** — Moved to `.github/` (Contributor Covenant v2.1)
- **CONTRIBUTING.md** — Moved to `.github/` (how to report bugs, suggest features, submit PRs)
- **SECURITY.md** — Added responsible disclosure process
- **ISSUE_TEMPLATE/** — Bug report and feature request templates
- **PULL_REQUEST_TEMPLATE.md** — Standard PR template with checklist

## [Unreleased]

- Nothing yet. Check back after the v1.0.0 launch.
