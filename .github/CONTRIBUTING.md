# Contributing to Opifex

Thank you for your interest in contributing to Opifex! Opifex is an AI agent
builder for businesses — a desktop application built with Tauri v2 (Rust
backend) and Vue 3 (frontend) that lets users design AI agents through a
chat-driven wizard interface.

We welcome contributions of all kinds: bug reports, feature requests, code
changes, documentation improvements, translations, and more.

## Table of Contents

- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Submitting Changes](#submitting-changes)
- [Style Guidelines](#style-guidelines)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [License](#license)

## Getting Started

### Prerequisites

* [Node.js](https://nodejs.org/) (v18 or newer)
* [Rust + Cargo](https://rustup.rs/)
* [Ollama](https://ollama.ai/) (optional, for local LLM usage)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/RadioMan18/opifex.git
cd opifex

# Install frontend dependencies
npm install

# Build frontend assets
npm run build

# Run the app in development mode
cargo tauri dev
```

### Building for Release

```bash
cargo tauri build
```

The release binary and installers (NSIS, MSI on Windows) are produced in
`src-tauri/target/release/`.

## Project Structure

```
opifex/
├── src/                          # Vue 3 frontend
│   ├── App.vue                   # Root component (split-pane layout)
│   ├── main.ts                   # Vue entry point
│   ├── components/               # Vue components
│   │   ├── ChatPane.vue          # Chat interface (left pane)
│   │   ├── PlanPane.vue          # Live agent plan (right pane)
│   │   ├── AgentLibrary.vue      # Agent library list
│   │   ├── ModelSelector.vue     # Ollama/OpenAI/Anthropic selector
│   │   └── Message.vue           # Chat message bubble
│   ├── services/                 # Tauri IPC wrappers
│   │   └── ipc.ts
│   ├── stores/                   # Vue reactive state
│   │   ├── agentStore.ts         # Agent, conversation, plan state
│   │   └── agentStore.test.ts    # Vitest tests
│   ├── types/agent.ts            # TypeScript interfaces
│   └── vite.config.ts            # Vite configuration
├── src-tauri/                    # Rust backend
│   ├── src/
│   │   ├── main.rs               # Tauri entry point
│   │   ├── lib.rs                # App setup and command registration
│   │   ├── db.rs                 # SQLite schema and connection
│   │   └── commands/
│   │       ├── agent.rs          # Agent CRUD commands
│   │       ├── conversation.rs   # Conversation/message commands
│   │       ├── ollama.rs         # Ollama/OpenAI/Anthropic chat
│   │       ├── plan.rs           # Plan versioning commands
│   │       └── mod.rs            # Command exports
│   ├── Cargo.toml                # Rust dependencies
│   └── tauri.conf.json           # Tauri configuration
├── scripts/generate-icons.js     # Icon generation from opifex_icon.png
├── opifex_icon.png               # Brand icon source
├── BUILD_PLAN.md                 # Implementation plan
├── OPEN_SOURCE_CHECKLIST.md      # Open source release checklist
├── README.md                     # Project overview
├── LICENSE                       # MIT License
├── CODE_OF_CONDUCT.md            # Contributor Covenant
└── CONTRIBUTING.md               # This file
```

## Submitting Changes

### Before You Start

1. **Check existing issues** — See if someone has already reported the bug or
   requested the feature you want to work on.
2. **Open an issue first** — For significant changes, discuss your approach
   with the maintainers before writing code.
3. **Create a branch** — Use a descriptive name like `feature/agent-export` or
   `fix/chat-scroll`.

### Making Changes

1. Fork the repository and clone your fork.
2. Create a new branch from `main`.
3. Make your changes.
4. Run tests: `cargo test` (Rust) and `npm test` (frontend/Vitest).
5. Verify the build: `npm run build` and `cargo tauri build`.
6. Commit your changes with a clear, descriptive message.
7. Push to your fork and open a Pull Request against `main`.

### Pull Request Guidelines

* Include a clear description of what changed and why.
* Reference any related issues (e.g., "Fixes #123").
* Ensure all tests pass.
* Keep changes focused — one feature or bug fix per PR.
* Update documentation if your change affects user-facing behavior.

## Style Guidelines

### Rust
* Follow standard Rust formatting (`cargo fmt`).
* Clippy warnings should be resolved (`cargo clippy`).
* Use snake_case for functions and variables, PascalCase for types.

### TypeScript/Vue
* Use TypeScript for all new code.
* Follow Vue 3 Composition API patterns (`<script setup>`).
* Component names use PascalCase (e.g., `ChatPane.vue`).
* File names match component names exactly.

### Commit Messages
* Use imperative mood: "Add feature" not "Added feature".
* Keep the subject line under 50 characters.
* Reference issue numbers when applicable: "Fixes #42".

## Reporting Bugs

When reporting a bug, please include:

* **Opifex version** (check the About menu or `cargo tauri --version`)
* **Operating system** (Windows, macOS, Linux)
* **Steps to reproduce**
* **Expected behavior**
* **Actual behavior**
* **Screenshots** (if applicable)
* **Ollama model** being used (if relevant)

Use the bug report issue template if available.

## Suggesting Features

Feature suggestions are welcome! When proposing a new feature, please describe:

* **The problem** you're trying to solve
* **Your proposed solution**
* **Alternatives** you've considered
* **Any additional context** (screenshots, mockups, etc.)

Use the feature request issue template if available.

## License

By contributing to Opifex, you agree that your contributions will be licensed
under the MIT License (see [LICENSE](LICENSE)).

## Questions?

Open an issue on the
[Opifex GitHub repository](https://github.com/RadioMan18/opifex/issues) and
we'll get back to you.
