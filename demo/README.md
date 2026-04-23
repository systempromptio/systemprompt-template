<div align="center">
  <a href="https://systemprompt.io">
    <img src="https://systemprompt.io/logo.svg" alt="systemprompt.io" width="150" />
  </a>
  <p><strong>Production infrastructure for AI agents</strong></p>
  <p><a href="https://systemprompt.io">systemprompt.io</a> • <a href="https://systemprompt.io/documentation">Documentation</a> • <a href="https://github.com/systempromptio/systemprompt-core">Core</a> • <a href="https://github.com/systempromptio/systemprompt-template">Template</a></p>
</div>

---

# Demo Suite

42 runnable demo scripts organized into 10 categories. Each demonstrates a different aspect of the platform via CLI commands.

## Quick Start

```bash
# 1. Build and start services
just build && just start

# 2. Run preflight (acquires token, checks services, creates admin user if needed)
./demo/00-preflight.sh

# 3. Seed data for analytics and trace demos
./demo/01-seed-data.sh

# 4. Pick any category
./demo/governance/01-happy-path.sh
./demo/infrastructure/02-database.sh
./demo/analytics/01-overview.sh
```

## Categories

Organized by the three pillars of [systemprompt.io](https://systemprompt.io): **Infrastructure**, **Capabilities**, and **Integrations**.

| Pillar | Category | Scripts | What it covers | Cost | Recording |
|--------|----------|---------|---------------|------|-----------|
| Infrastructure | [infrastructure/](infrastructure/) | 5 | Services, database, jobs, logs, configuration | Free | `infra-self-hosted`, `infra-control-plane` |
| Infrastructure | [cloud/](cloud/) | 1 | Auth status, profiles, deployment info | Free | `infra-deploy-anywhere` |
| Capabilities | [governance/](governance/) | 8 | Scope enforcement, secret detection, audit trails, hooks | Free | `cap-governance`, `cap-secrets` |
| Capabilities | [mcp/](mcp/) | 3 | MCP server management, access tracking, tool execution | Free | `cap-mcp` |
| Capabilities | [analytics/](analytics/) | 8 | Overview, agents, costs, requests, sessions, content/traffic | Free | `cap-analytics` |
| Capabilities | [agents/](agents/) | 5 | Agent discovery, messaging, tracing, A2A registry | 2 @ ~$0.01 | `cap-agents` |
| Capabilities | [users/](users/) | 4 | User CRUD, roles, sessions, IP bans | Free | `cap-compliance` |
| Integrations | [skills/](skills/) | 5 | Skills, content, files, plugins, contexts | Free | `int-cowork`, `int-extensions` |
| Integrations | [web/](web/) | 2 | Content types, templates, sitemaps, validation | Free | `int-web-publisher` |
| Integrations | [performance/](performance/) | 2 | Request tracing, benchmarks, load testing | Free | `int-benchmark` |

**Total: 43 category scripts + preflight + seed = 45 scripts. 41 free, 2 cost ~$0.01 each.**

## Prerequisites

- Built binary: `just build`
- Running services: `just start`
- Preflight completed: `./demo/00-preflight.sh`
- Seed data (recommended): `./demo/01-seed-data.sh`

## Script Index

### Setup
| Script | Description |
|--------|-------------|
| `00-preflight.sh` | Service health check, admin user creation, token acquisition |
| `01-seed-data.sh` | Populate governance decisions, events, skills, and content |

### Infrastructure

| Script | Description |
|--------|-------------|
| `infrastructure/01-services.sh` | Service status and health checks |
| `infrastructure/02-database.sh` | Tables, schema, queries, migrations |
| `infrastructure/03-jobs.sh` | Job scheduling, execution history |
| `infrastructure/04-logs.sh` | Log viewing, search, traces, requests |
| `infrastructure/05-config.sh` | Full platform configuration overview |
| `cloud/01-cloud-overview.sh` | Auth status, profiles, deployment info |

### Capabilities

| Script | Description |
|--------|-------------|
| `governance/01-happy-path.sh` | Governance ALLOWS admin-scope tool call |
| `governance/02-refused-path.sh` | Governance DENIES user-scope agent |
| `governance/03-audit-trail.sh` | Query governance decisions from database |
| `governance/04-governance-happy.sh` | All 3 rules pass for admin agent |
| `governance/05-governance-denied.sh` | Scope + blocklist deny for user agent |
| `governance/06-secret-breach.sh` | Secret detection blocks credentials |
| `governance/07-rate-limiting.sh` | Rate limit, security, and server configuration |
| `governance/08-hooks.sh` | Hook listing and validation |
| `mcp/01-mcp-servers.sh` | Server status, tools by server |
| `mcp/02-mcp-access-tracking.sh` | OAuth + MCP tool call tracking |
| `mcp/03-mcp-tool-execution.sh` | Tool listings, execution logs |
| `analytics/01-overview.sh` | Dashboard overview |
| `analytics/02-agent-analytics.sh` | Agent stats, trends, deep-dives |
| `analytics/03-cost-analytics.sh` | Cost summary, breakdown, trends |
| `analytics/04-request-analytics.sh` | AI request volume, latency, models |
| `analytics/05-session-analytics.sh` | Session stats, trends, real-time |
| `analytics/06-content-traffic.sh` | Content engagement, traffic, geo |
| `analytics/07-conversations.sh` | Conversation stats, trends, listing |
| `analytics/08-tool-analytics.sh` | Tool usage stats, trends |
| `agents/01-list-agents.sh` | Agent discovery — admin and core views |
| `agents/02-agent-config.sh` | Validation, MCP tool access, status |
| `agents/03-agent-messaging.sh` | Full agent pipeline with AI (~$0.01) |
| `agents/04-agent-tracing.sh` | Traces, artifacts, cost attribution |
| `agents/05-agent-registry.sh` | A2A gateway, agent logs |
| `users/01-user-crud.sh` | User listing, counts, stats, search |
| `users/02-role-management.sh` | User details and role inspection |
| `users/03-session-management.sh` | Current session, available profiles |
| `users/04-ip-ban.sh` | Add/remove IP bans with verification |

### Integrations

| Script | Description |
|--------|-------------|
| `skills/01-skill-lifecycle.sh` | Skill listing, details, sync status |
| `skills/02-content-management.sh` | Content listing, search, popularity |
| `skills/03-file-management.sh` | File listing, config, storage stats |
| `skills/04-plugin-management.sh` | Plugins, hooks, extensions, capabilities |
| `skills/05-contexts.sh` | Context CRUD — create, show, edit, delete |
| `performance/01-request-tracing.sh` | Typed data, flow maps, benchmarks |
| `performance/02-load-test.sh` | 2000-request load test |
| `web/01-web-config.sh` | Content types, templates, assets |
| `web/02-sitemap-validate.sh` | Sitemap config, web validation |

## Connecting the Cowork desktop app

The `enterprise-demo` plugin, its skills, agents, and MCP servers ship with the
template, but plugin assignment is **per-user** — `setup-local` populates the
global registry and `01-seed-data.sh` forks `enterprise-demo` into the active
session's user. Whoever is authenticated when you run the seed gets the plugin.

Full flow to route Cowork through a locally-running template:

```bash
# 1. Bring the template up (from systemprompt-template)
just setup-local <anthropic_key> [openai_key] [gemini_key]
just start

# 2. Seed — this ALSO forks enterprise-demo for the current admin session.
./demo/00-preflight.sh        # admin session + /demo/.token
./demo/01-seed-data.sh        # + fork enterprise-demo for that user

# 3. Issue a PAT for the user you want Cowork to connect as.
#    Open http://localhost:8080/admin/devices → sign in → create device →
#    copy the sp-live-... PAT. (CLI PAT issuance isn't wired yet.)
#    If this is a different user than the seed target, fork for them too:
#      curl -X POST http://localhost:8080/api/public/admin/user/fork/plugin \
#        -H "Authorization: Bearer <their-JWT>" \
#        -H 'Content-Type: application/json' \
#        -d '{"org_plugin_id":"enterprise-demo"}'

# 4. Wire up the cowork helper (from systemprompt-core)
cargo build --release --manifest-path bin/cowork/Cargo.toml
BIN=bin/cowork/target/release/systemprompt-cowork
"$BIN" login sp-live-... --gateway http://localhost:8080
"$BIN" install --apply    # writes MDM keys for com.anthropic.claudefordesktop
"$BIN" sync               # pulls manifest → ~/Library/Application Support/Claude/org-plugins/

# 5. Cmd+Q Cowork and relaunch. It now reads inferenceProvider=gateway from
#    the MDM keys and routes every request through http://localhost:8080 with
#    a JWT minted by the helper.
```

Verify the gateway is wired:

```bash
"$BIN" status     # paths + gateway + last-sync timestamp
"$BIN" whoami     # identity JSON from the gateway
```

If `sync` reports `0 plugins / 0 skills / 0 agents / 0 MCP`, the authenticated
user has no forked plugin. Re-run the fork POST in step 3 with the correct
user's token.

To revert Cowork to direct Anthropic: `systemprompt-cowork uninstall` then
Cmd+Q and relaunch.

## Troubleshooting

```bash
# If services are down
systemprompt infra services cleanup --yes
systemprompt infra services start --kill-port-process

# Wait for "All services started successfully" then retry
```

## Recording

SVG terminal recordings and video recording infrastructure are in `recording/`.
See `recording/RECORDING-GUIDE.md` for video production workflow.


---

## License

MIT - See [LICENSE](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE) for details.
