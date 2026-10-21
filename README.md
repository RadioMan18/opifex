# Opifex — AI Agent Builder for Businesses

[![License](https://img.shields.io/github/license/RadioMan18/opifex)](https://github.com/RadioMan18/opifex/blob/main/LICENSE)
[![Stars](https://img.shields.io/github/stars/RadioMan18/opifex)](https://github.com/RadioMan18/opifex/stargazers)
[![Issues](https://img.shields.io/github/issues/RadioMan18/opifex)](https://github.com/RadioMan18/opifex/issues)
[![Forks](https://img.shields.io/github/forks/RadioMan18/opifex)](https://github.com/RadioMan18/opifex/network/members)
[![Tauri v2](https://img.shields.io/badge/tauri-v2-blue)](https://tauri.app/)
[![Vue 3](https://img.shields.io/badge/vue-3-green)](https://vuejs.org/)

Opifex is a desktop application (Tauri v2 + Rust backend, Vue 3 frontend) for building AI agents through a chat-driven wizard interface. It helps AI practitioners and corporate teams design, configure, and manage AI agents via natural conversation with "Oppy," the Opifex Wizard.

## Table of Contents

- [Overview](#overview)
- ✨ [Features](#features)
- 🏗️ [Architecture & Stack](#architecture--stack)
- 📦 [Installation](#installation)
- 🚀 [Usage](#usage)
- 🤖 [Starter Agents](#starter-agents)
- 🔧 [Configuration](#configuration)
- 🧪 [Testing](#testing)
- 🗺️ [Roadmap](#roadmap)
- 📄 [License](#license)

## Overview

Opifex mimics the chat-driven agent builder pattern popularized by tools like Claude Artifacts and Microsoft Copilot Studio:

- **Left pane:** Conversation with Oppy (the Wizard), who interviews you about what you want your agent to do
- **Right pane:** A live, structured agent plan (goal, steps, tools, knowledge sources, triggers, outputs) that updates after every turn

The flow is:
1. **Select or create an agent** from the library
2. **Interview mode:** Oppy asks clarifying questions to understand your requirements
3. **Build Agent:** Once you confirm, Oppy generates a structured JSON plan
4. **Edit inline:** Modify goal, steps, tools, knowledge sources, triggers, and outputs directly in the right pane
5. **Save versions:** Save named plan versions and export as JSON files

## ✨ Features

### Core
- **Split-pane UI:** Chat on the left, live agent plan on the right
- **Wizard interview:** Oppy conducts a natural conversation to gather requirements before generating a plan
- **Live plan updates:** The right pane updates after every conversation turn
- **Inline plan editing:** Edit goal, steps, tools, knowledge sources, triggers, and outputs directly
- **Agent library:** Save agents locally with starter templates pre-installed
- **Agent CRUD:** Create, rename, duplicate, delete, export, and import agents
- **Plan versioning:** Save and restore named versions of agent plans
- **Model switching:** Switch between Ollama (local), OpenAI, and Anthropic providers
- **API key management:** Securely store provider API keys in local settings
- **Model download:** Pull new Ollama models from within the app

### Starter Agents (5)
Shipped read-only and cloneable:
1. **Customer Support Agent** — Handles customer inquiries and support tickets, grounded in FAQ/product docs
2. **Research Agent** — Conducts web research and produces structured, cited reports
3. **RAG Knowledge Agent** — Answers questions over internal company documents and knowledge base
4. **Code Review Agent** — Reviews pull requests, identifies issues, suggests fixes
5. **Content Writer Agent** — Drafts on-brand internal and external communications

## 🏗️ Architecture & Stack

| Layer | Technology |
|-------|-----------|
| Shell | Tauri v2 (Rust) |
| Frontend | Vue 3 + TypeScript + Vite |
| Storage | SQLite (local file, app data directory) |
| LLM Provider | Ollama (default), OpenAI, Anthropic |
| Icons | PNG/ICO/ICNS/Android/iOS/Windows Store (generated from `opifex_icon.png`) |

### Data Model (SQLite)
- **agents** — Agent configurations (name, description, spec JSON, model, starter flag, timestamps)
- **conversations** — Chat conversations linked to agents
- **messages** — Individual chat messages with optional turn plan JSON
- **settings** — Key/value store (default model, provider, API keys)
- **plan_versions** — Saved plan snapshots with labels

## 📦 Installation

### Prerequisites
- **Node.js** (for frontend dev/build)
- **Rust + Cargo** (for backend build)
- **Ollama** (recommended local LLM provider, `http://localhost:11434`)

### Build from Source

```bash
# Clone the repository
git clone https://github.com/RadioMan18/opifex.git
cd opifex

# Install frontend dependencies
npm install

# Build frontend
npm run build

# Build and run the Tauri app
cargo tauri dev
```

Or build a release binary:

```bash
cargo tauri build
```

The release binary is produced at `src-tauri/target/release/app.exe` (Windows) along with NSIS and MSI installers.

### Using the Provided Script

```bash
# Windows (PowerShell or cmd)
.\run-opifex.bat
```

This builds the frontend, builds the Tauri release binary, and launches the app.

## 🚀 Usage

### Creating Your First Agent

1. **Launch Opifex** — You'll see the empty agent library
2. **Create a new agent** — Click "+ Agent" or select a starter agent to clone
3. **Name your agent** — Give it a descriptive name and optional description
4. **Chat with Oppy** — Describe what you want your agent to do. Oppy will ask clarifying questions
5. **Click "Build Agent"** — When Oppy says "Should I build this agent now?", confirm to generate the plan
6. **Edit the plan** — Modify goal, steps, tools, knowledge sources, triggers, and outputs in the right pane
7. **Save a version** — Click "Save Version" to save the current plan state and download as JSON

### Model Selection

- **Ollama (Local):** Default, no API key needed. Pull models via the ⬇ button
- **OpenAI:** Requires API key (set via 🔑 button)
- **Anthropic:** Requires API key (set via 🔑 button)

Switch providers in the top bar, then select a model from the dropdown.

### Exporting Agents

Click ⬇ (export) on any agent in the library to download its JSON configuration. You can import this JSON into another Opifex installation.

## 🤖 Starter Agents

| Agent | Purpose |
|-------|---------|
| Customer Support Agent | Resolves inquiries, manages tickets, grounded in FAQ/product docs |
| Research Agent | Web research with structured, cited reports |
| RAG Knowledge Agent | Q&A over internal company knowledge base |
| Code Review Agent | PR review, bug detection, fix suggestions, approval |
| Content Writer Agent | On-brand comms: announcements, newsletters, press releases |

Starter agents are read-only but fully cloneable — clone one to create your own variant.

## 🔧 Configuration

### Environment
No environment variables are required for basic operation. Ollama runs on `http://localhost:11434` by default.

### Data Location
The SQLite database (`opifex.db`) is stored in your platform's app data directory:
- **Windows:** `%APPDATA%\Opifex\opifex.db`
- **macOS:** `~/Library/Application Support/Opifex/opifex.db`
- **Linux:** `~/.local/share/Opifex/opifex.db`

### Icons
Branded icons are generated from `opifex_icon.png` using `scripts/generate-icons.js`:
```bash
npm run generate-icons
```

## 🧪 Testing

### Rust Backend Tests
```bash
cd src-tauri
cargo test
```

### Frontend Tests
```bash
npm test
# or
npx vitest run
```

Both test suites must pass (6 Rust tests + 12 Vitest tests).

## 🗺️ Roadmap

### Phase 1 — MVP (Complete)
- ✅ Tauri v2 + Vue 3 scaffold with branded icons
- ✅ SQLite storage with agent CRUD + 6 starter agents
- ✅ Ollama integration (list/pull/chat/stream)
- ✅ Split-pane wizard UI with live plan updates
- ✅ Conversation/message persistence
- ✅ Plan versioning (save/list/restore)
- ✅ BYOK providers (OpenAI, Anthropic)
- ✅ Interview mode with build confirmation
- ✅ Save plan version as downloadable JSON
- ✅ Home navigation from agent view

### Phase 2 — Polish (Planned)
- Agent export/import improvements
- Plan comparison/diff between versions
- Agent templates marketplace
- Keyboard shortcuts
- Dark mode

### Phase 3 — SaaS (Future)
- Multi-tenancy and authentication
- Cloud sync across devices
- Team collaboration on agents
- Agent marketplace

## 📄 License

This project is licensed under the MIT License — see [LICENSE](LICENSE) for details.

---

Built with ❤️ using Tauri v2, Vue 3, and Rust.
