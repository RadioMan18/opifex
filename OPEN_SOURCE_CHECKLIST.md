# Open Source Release Checklist

## Phase 1: GitHub Repository Visibility

- [x] Make repository public (Settings → General → Danger Zone → Change repository visibility)
- [x] Confirm repository is public by viewing in an incognito/private browser window

## Phase 2: Repository Metadata

- [ ] Add repository topics/tags: `tauri`, `vue`, `ai-agent`, `agent-builder`, `desktop-app`, `ollama`, `open-source`, `windows`, `rust`
- [ ] Add a repository description: "Agent Builder for businesses — chat-driven AI agent creation with Oppy Wizard"
- [ ] Enable Issues and Discussions in repository settings
- [ ] Add CODEOWNERS file if specific people should be notified for certain files

## Phase 3: Package.json Metadata

- [ ] Add `repository` field:
  ```json
  "repository": {
    "type": "git",
    "url": "git+https://github.com/RadioMan18/opifex.git"
  }
  ```
- [ ] Add `author` field with name and email
- [ ] Add `license`: `"MIT"`
- [ ] Add `homepage`: `"https://github.com/RadioMan18/opifex"`
- [ ] Add `bugs`: `"https://github.com/RadioMan18/opifex/issues"`
- [ ] Add keywords: `["tauri", "vue", "ai-agent", "agent-builder", "desktop"]`

## Phase 4: Community Health Files

- [ ] `.github/CODE_OF_CONDUCT.md` — Contributor Covenant
- [ ] `.github/CONTRIBUTING.md` — How to report bugs, suggest features, submit PRs
- [ ] `.github/SECURITY.md` — Responsible disclosure process
- [ ] `.github/ISSUE_TEMPLATE/bug_report.md`
- [ ] `.github/ISSUE_TEMPLATE/feature_request.md`
- [ ] `.github/PULL_REQUEST_TEMPLATE.md`

## Phase 5: README.md Badges

- [ ] License badge: `![License](https://img.shields.io/github/license/RadioMan18/opifex)`
- [ ] Stars badge: `![Stars](https://img.shields.io/github/stars/RadioMan18/opifex)`
- [ ] Issues badge: `![Issues](https://img.shields.io/github/issues/RadioMan18/opifex)`
- [ ] Forks badge: `![Forks](https://img.shields.io/github/forks/RadioMan18/opifex)`
- [ ] Tauri badge: `![Tauri](https://img.shields.io/badge/tauri-v2-blue)`
- [ ] Vue badge: `![Vue](https://img.shields.io/badge/vue-3-green)`

## Phase 6: CI/CD (Optional but Recommended)

- [x] Add `.github/workflows/ci.yml` — runs `cargo test` + `npm test` on push/PR *(PR #1 open, awaiting merge)*
- [ ] Enable Dependabot for dependency updates

## Phase 7: Announce

- [ ] Post on LinkedIn
- [ ] Post on Twitter/X
- [ ] Post on Reddit (r/selfhosted, r/taurijs, r/vuejs)
- [ ] Submit to Hacker News
- [ ] Submit to Product Hunt
- [ ] Add to "Awesome Tauri" or "Awesome Vue" lists

## Status

| Item | Status |
|------|--------|
| Repo made public | ✅ |
| Topics/tags added | ⬜ *(GitHub web UI)* |
| package.json metadata | ⬜ |
| Community health files | ✅ |
| README badges | ✅ |
| CI workflow | ✅ PR #1 merged |
| Announcement | ⬜ |

> **Phase 2 items** (topics/tags, repository description, Issues/Discussions settings) are GitHub web UI actions — visit https://github.com/RadioMan18/opifex/settings to complete them. CODEOWNERS is skipped per user preference.
>
> **Community health files** (CODE_OF_CONDUCT.md, CONTRIBUTING.md) exist at project root but need to be moved to `.github/` per Phase 4.
>
> **CI workflow** ✅ PR #1 merged — workflow active on `main`
