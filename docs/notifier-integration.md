# Notifier Integration — Opifex v1.1.0

Opifex delegates human-in-the-loop (HITL) approval gates to **[Notifier](https://github.com/RadioMan18/notifier)**, a lightweight approval gateway maintained as a companion repository. This doc covers installing, configuring, and using Notifier with Opifex.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Opifex (Tauri + Vue)                   │
│  • Agent plans with "requires approval" steps               │
│  • Settings: Notifier URL + connection status               │
│  • Approvals tab: pending/recent requests                   │
└──────────────┬──────────────────────────────────────────────┘
               │ HTTP (existing REST API)
               ▼
┌─────────────────────────────────────────────────────────────┐
│                   Notifier (Node.js/Express)                │
│  POST /request-approval  →  creates task, notifies user     │
│  GET  /status/:taskId    →  polls for decision              │
│  GET  /respond/:taskId/:approve|deny  →  user decides       │
└──────────────┬──────────────────────────────────────────────┘
               │ Notification provider
               ▼
        Console / Browser / Telegram / (MCP server v0.2.0+)
```

## 1. Install Notifier

Notifier is a separate Node.js application. Download it from the [Notifier releases page](https://github.com/RadioMan18/notifier/releases) or build from source:

```powershell
# Clone Notifier
git clone https://github.com/RadioMan18/notifier.git
cd notifier

# Install dependencies
npm install

# Copy .env.example to .env and edit
Copy-Item .env.example .env
notepad .env
```

### Minimum `.env` configuration

| Variable | Description | Example |
|----------|-------------|---------|
| `NOTIFIER_NAME` | Name shown in approval messages | `Opifex` |
| `PORT` | Port Notifier listens on | `3001` |
| `PUBLIC_GATEWAY_URL` | Public URL users visit to approve/deny | `http://localhost:3001` |
| `NOTIFIER_PROVIDER` | Delivery channel: `console`, `browser`, `telegram` | `browser` |
| `TELEGRAM_BOT_TOKEN` | Required if provider is `telegram` | *(optional)* |
| `TELEGRAM_CHAT_IDS` | Comma-separated Telegram chat IDs | *(optional)* |
| `APPROVAL_TTL_MS` | Approval request expiry in ms | `3600000` |

Start Notifier:

```powershell
npm start
```

Verify it's running:

```powershell
Invoke-RestMethod http://localhost:3001/health
# Expected: { "status": "ok" }
```

## 2. Configure Opifex to Use Notifier

In Opifex, open **Settings → Approval Gates**:

| Setting | Default | Description |
|---------|---------|-------------|
| Notifier Base URL | `http://localhost:3001` | Where Notifier is reachable |
| Approval Gates Enabled | `true` | Toggle HITL gates on/off |

Click **Test Connection**. A green status indicator confirms Opifex can reach Notifier.

If Notifier is not running, Opifex works normally but approval-required steps are skipped with a warning (graceful degradation).

## 3. Using Approval Gates

### Building an agent with approval steps

1. Create or select an agent in Opifex.
2. Chat with Oppy to build the agent plan.
3. In the **Plan pane**, mark steps that need human approval by checking **"Approval required"**.
4. Steps flagged for approval show an 🔒 badge.

### At runtime

When an agent reaches a step flagged `requiresApproval`:

1. Opifex posts an approval request to Notifier.
2. You receive a notification via your configured provider (browser popup, Telegram message, or console log).
3. Open the approval link (or the **Approvals** tab in Opifex) and choose **Approve** or **Deny**.
4. Opifex polls for the decision and proceeds or halts accordingly.

## 4. Advanced: Antigravity Integration

Notifier v0.2.0+ exposes a **Model Context Protocol (MCP)** server at `http://localhost:3001/mcp` with an `ask_approval` tool. Antigravity agents can connect via `mcp_config.json`:

```json
{
  "mcpServers": {
    "notifier": {
      "serverUrl": "http://localhost:3001/mcp",
      "headers": { "Authorization": "Bearer NOTIFIER_TOKEN" }
    }
  }
}
```

Add a `PreToolUse` hook in `.agents/hooks.json` to intercept privileged tool calls and delegate to Notifier:

```json
{
  "notifier-approval-gate": {
    "PreToolUse": [
      {
        "matcher": "run_command|write_to_file|replace_file_content|invoke_subagent",
        "hooks": [
          {
            "type": "command",
            "command": "node scripts/notifier-hook.js",
            "timeout": 30
          }
        ]
      }
    ]
  }
}
```

## 5. Advanced: Copilot / Kilo Code

Agents running in VS Code (GitHub Copilot or Kilo Code) can call Notifier's HTTP API directly:

```bash
# Request approval
curl -X POST http://localhost:3001/api/request-approval \
  -H "Content-Type: application/json" \
  -d '{"command":"npm run deploy","context":"Deploying to production","requester":"copilot"}'

# Poll for decision
curl http://localhost:3001/api/status/<taskId>
```

## 6. Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|--------------|-----|
| Opifex shows "Notifier disconnected" | Notifier not running or wrong URL | Start Notifier; verify `NOTIFIER_BASE_URL` in Opifex settings |
| Approval request times out | Notifier unreachable or user hasn't decided | Check Notifier logs; ensure user has access to approve/deny URLs |
| Telegram notifications not arriving | Wrong bot token or chat IDs | Verify `TELEGRAM_BOT_TOKEN` and `TELEGRAM_CHAT_IDS` in Notifier `.env` |
| MCP connection fails | Notifier v0.2.0+ required | Upgrade Notifier; verify `/mcp` endpoint responds |

## 7. Security

- Notifier runs locally by default (`http://localhost:3001`). Do not expose `PUBLIC_GATEWAY_URL` to the internet without authentication.
- Approval request bodies are capped at 4KB. Commands/context longer than that are truncated.
- All approval requests and decisions are logged to Notifier's `data/audit.jsonl`.
- No remote code execution — Opifex agent plans are pure JSON; Notifier only stores approval metadata.

## 8. Version Compatibility

| Opifex | Minimum Notifier | Notes |
|--------|------------------|-------|
| v1.0.0 | None | No HITL gates |
| v1.1.0 | v0.1.0 | HTTP-only approval gates |
| v1.1.0 | v0.2.0 | + MCP server support |
