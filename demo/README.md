<div align="center">
  <a href="https://systemprompt.io">
    <img src="https://systemprompt.io/logo.svg" alt="systemprompt.io" width="150" />
  </a>
  <p><strong>Production infrastructure for AI agents</strong></p>
  <p><a href="https://systemprompt.io">systemprompt.io</a> • <a href="https://systemprompt.io/documentation">Documentation</a> • <a href="https://github.com/systempromptio/systemprompt-core">Core</a> • <a href="https://github.com/systempromptio/systemprompt-template">Template</a></p>
</div>

---

# Demo Suite

**44 runnable demo scripts** organized into 10 categories, plus 2 setup scripts. Every script is a single command — copy, paste, read the output, move on. All 44 demos have been verified end-to-end against a freshly seeded local stack (see [Verification status](#verification-status) at the bottom).

> **Just want to see it work?** Run the four commands in [Quick Start](#quick-start), then walk down the [Full demo index](#full-demo-index) — every row is a paste-and-run command.

## Quick Start

Prereqs: built binary, services up. From the repo root:

```bash
# 1. Build and start services (skip if already running on :8080)
just build && just start

# 2. Preflight — health-check + admin user + token saved to demo/.token
./demo/00-preflight.sh

# 3. Seed — governance decisions, events, skills, content (powers analytics demos)
./demo/01-seed-data.sh

# 4. Pick any demo from the index below and paste it in.
./demo/governance/01-happy-path.sh
```

The preflight step writes a JWT to `demo/.token`. **Every other script reads it automatically — you never pass a token by hand.**

## Categories at a glance

Organized by the three pillars of [systemprompt.io](https://systemprompt.io): **Infrastructure**, **Capabilities**, and **Integrations**.

| Pillar | Category | Scripts | What it covers | Cost |
|--------|----------|---------|----------------|------|
| Infrastructure | [infrastructure/](infrastructure/) | 5 | Services, database, jobs, logs, configuration | Free |
| Infrastructure | [cloud/](cloud/) | 1 | Auth status, profiles, deployment info | Free |
| Capabilities | [governance/](governance/) | 9 | Audit smoke, scope, secrets, blocklist, rate limit, hooks | Free |
| Capabilities | [mcp/](mcp/) | 3 | MCP server management, access tracking, tool execution | Free |
| Capabilities | [analytics/](analytics/) | 8 | Overview, agents, costs, requests, sessions, content/traffic, conversations, tools | Free |
| Capabilities | [agents/](agents/) | 5 | Agent discovery, config, messaging, tracing, A2A registry | 1 × ~$0.01 |
| Capabilities | [users/](users/) | 4 | User CRUD, roles, sessions, IP bans | Free |
| Integrations | [skills/](skills/) | 5 | Skills, content, files, plugins, contexts | Free |
| Integrations | [web/](web/) | 2 | Content types, templates, sitemaps, validation | Free |
| Integrations | [performance/](performance/) | 2 | Request tracing, 2000-request load test | Free |

**Total: 44 category scripts + 2 setup scripts. 43 free, 1 costs ~$0.01.**

---

## Full demo index

Every row is a single copy-paste command. Run them in any order after `00-preflight.sh` and `01-seed-data.sh`.

### Setup (run these first)

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`00-preflight.sh`](00-preflight.sh) | `./demo/00-preflight.sh` | Service health check, admin user creation, token acquisition → `demo/.token` |
| 2 | [`01-seed-data.sh`](01-seed-data.sh) | `./demo/01-seed-data.sh` | Populate governance decisions, events, skills, content |

### Infrastructure

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`infrastructure/01-services.sh`](infrastructure/01-services.sh) | `./demo/infrastructure/01-services.sh` | Service status and health checks |
| 2 | [`infrastructure/02-database.sh`](infrastructure/02-database.sh) | `./demo/infrastructure/02-database.sh` | Tables, schema, queries, migrations |
| 3 | [`infrastructure/03-jobs.sh`](infrastructure/03-jobs.sh) | `./demo/infrastructure/03-jobs.sh` | Job scheduling and execution history |
| 4 | [`infrastructure/04-logs.sh`](infrastructure/04-logs.sh) | `./demo/infrastructure/04-logs.sh` | Log viewing, search, traces, requests |
| 5 | [`infrastructure/05-config.sh`](infrastructure/05-config.sh) | `./demo/infrastructure/05-config.sh` | Full platform configuration overview |

### Cloud

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`cloud/01-cloud-overview.sh`](cloud/01-cloud-overview.sh) | `./demo/cloud/01-cloud-overview.sh` | Auth status, profiles, deployment info |

### Governance

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 0 | [`governance/00-audit-write-smoke.sh`](governance/00-audit-write-smoke.sh) | `./demo/governance/00-audit-write-smoke.sh` | Smoke-test that audit rows actually land in Postgres |
| 1 | [`governance/01-happy-path.sh`](governance/01-happy-path.sh) | `./demo/governance/01-happy-path.sh` | Governance ALLOWS admin-scope tool call |
| 2 | [`governance/02-refused-path.sh`](governance/02-refused-path.sh) | `./demo/governance/02-refused-path.sh` | Governance DENIES user-scope agent |
| 3 | [`governance/03-audit-trail.sh`](governance/03-audit-trail.sh) | `./demo/governance/03-audit-trail.sh` | Query governance decisions from database |
| 4 | [`governance/04-governance-happy.sh`](governance/04-governance-happy.sh) | `./demo/governance/04-governance-happy.sh` | All 3 governance rules pass for admin agent |
| 5 | [`governance/05-governance-denied.sh`](governance/05-governance-denied.sh) | `./demo/governance/05-governance-denied.sh` | Scope + blocklist deny for user agent |
| 6 | [`governance/06-secret-breach.sh`](governance/06-secret-breach.sh) | `./demo/governance/06-secret-breach.sh` | Secret detection blocks credentials in tool payload |
| 7 | [`governance/07-rate-limiting.sh`](governance/07-rate-limiting.sh) | `./demo/governance/07-rate-limiting.sh` | Rate limit, security, and server configuration |
| 8 | [`governance/08-hooks.sh`](governance/08-hooks.sh) | `./demo/governance/08-hooks.sh` | Hook listing and validation |

### MCP

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`mcp/01-mcp-servers.sh`](mcp/01-mcp-servers.sh) | `./demo/mcp/01-mcp-servers.sh` | Server status and tools-by-server |
| 2 | [`mcp/02-mcp-access-tracking.sh`](mcp/02-mcp-access-tracking.sh) | `./demo/mcp/02-mcp-access-tracking.sh` | OAuth + MCP tool call tracking |
| 3 | [`mcp/03-mcp-tool-execution.sh`](mcp/03-mcp-tool-execution.sh) | `./demo/mcp/03-mcp-tool-execution.sh` | Tool listings, execution logs |

### Analytics

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`analytics/01-overview.sh`](analytics/01-overview.sh) | `./demo/analytics/01-overview.sh` | Dashboard overview |
| 2 | [`analytics/02-agent-analytics.sh`](analytics/02-agent-analytics.sh) | `./demo/analytics/02-agent-analytics.sh` | Agent stats, trends, deep-dives |
| 3 | [`analytics/03-cost-analytics.sh`](analytics/03-cost-analytics.sh) | `./demo/analytics/03-cost-analytics.sh` | Cost summary, breakdown, trends |
| 4 | [`analytics/04-request-analytics.sh`](analytics/04-request-analytics.sh) | `./demo/analytics/04-request-analytics.sh` | AI request volume, latency, models |
| 5 | [`analytics/05-session-analytics.sh`](analytics/05-session-analytics.sh) | `./demo/analytics/05-session-analytics.sh` | Session stats, trends, real-time |
| 6 | [`analytics/06-content-traffic.sh`](analytics/06-content-traffic.sh) | `./demo/analytics/06-content-traffic.sh` | Content engagement, traffic, geo |
| 7 | [`analytics/07-conversations.sh`](analytics/07-conversations.sh) | `./demo/analytics/07-conversations.sh` | Conversation stats, trends, listing |
| 8 | [`analytics/08-tool-analytics.sh`](analytics/08-tool-analytics.sh) | `./demo/analytics/08-tool-analytics.sh` | Tool usage stats, trends |

### Agents

| # | Script | Command | What it does | Cost |
|---|--------|---------|--------------|------|
| 1 | [`agents/01-list-agents.sh`](agents/01-list-agents.sh) | `./demo/agents/01-list-agents.sh` | Agent discovery — admin and core views | Free |
| 2 | [`agents/02-agent-config.sh`](agents/02-agent-config.sh) | `./demo/agents/02-agent-config.sh` | Validation, MCP tool access, status | Free |
| 3 | [`agents/03-agent-messaging.sh`](agents/03-agent-messaging.sh) | `./demo/agents/03-agent-messaging.sh` | **Full agent pipeline with a live Anthropic call** | ~$0.01 |
| 4 | [`agents/04-agent-tracing.sh`](agents/04-agent-tracing.sh) | `./demo/agents/04-agent-tracing.sh` | Traces, artifacts, cost attribution | Free |
| 5 | [`agents/05-agent-registry.sh`](agents/05-agent-registry.sh) | `./demo/agents/05-agent-registry.sh` | A2A gateway and agent logs | Free |

### Users

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`users/01-user-listing.sh`](users/01-user-listing.sh) | `./demo/users/01-user-listing.sh` | User listing, counts, stats, search |
| 2 | [`users/02-role-management.sh`](users/02-role-management.sh) | `./demo/users/02-role-management.sh` | User details and role inspection |
| 3 | [`users/03-session-management.sh`](users/03-session-management.sh) | `./demo/users/03-session-management.sh` | Current session, available profiles |
| 4 | [`users/04-ip-ban.sh`](users/04-ip-ban.sh) | `./demo/users/04-ip-ban.sh` | Add/remove IP bans with verification |

### Skills

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`skills/01-skill-lifecycle.sh`](skills/01-skill-lifecycle.sh) | `./demo/skills/01-skill-lifecycle.sh` | Skill listing, details, sync status |
| 2 | [`skills/02-content-management.sh`](skills/02-content-management.sh) | `./demo/skills/02-content-management.sh` | Content listing, search, popularity |
| 3 | [`skills/03-file-management.sh`](skills/03-file-management.sh) | `./demo/skills/03-file-management.sh` | File listing, config, storage stats |
| 4 | [`skills/04-plugin-management.sh`](skills/04-plugin-management.sh) | `./demo/skills/04-plugin-management.sh` | Plugins, hooks, extensions, capabilities |
| 5 | [`skills/05-contexts.sh`](skills/05-contexts.sh) | `./demo/skills/05-contexts.sh` | Context CRUD — create, show, edit, delete |

### Web

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`web/01-web-config.sh`](web/01-web-config.sh) | `./demo/web/01-web-config.sh` | Content types, templates, assets |
| 2 | [`web/02-sitemap-validate.sh`](web/02-sitemap-validate.sh) | `./demo/web/02-sitemap-validate.sh` | Sitemap config, web validation |

### Performance

| # | Script | Command | What it does |
|---|--------|---------|--------------|
| 1 | [`performance/01-request-tracing.sh`](performance/01-request-tracing.sh) | `./demo/performance/01-request-tracing.sh` | Typed data, flow maps, micro-benchmarks |
| 2 | [`performance/02-load-test.sh`](performance/02-load-test.sh) | `./demo/performance/02-load-test.sh` | 2000-request load test (free, ~1–2 min) |

---

## Run them all

Replay the entire suite in one go (skips the costly `agents/03-agent-messaging.sh` by default):

```bash
./demo/00-preflight.sh && ./demo/01-seed-data.sh && \
for f in demo/infrastructure/*.sh demo/cloud/*.sh demo/governance/*.sh \
         demo/mcp/*.sh demo/analytics/*.sh \
         demo/agents/01-*.sh demo/agents/02-*.sh demo/agents/04-*.sh demo/agents/05-*.sh \
         demo/users/*.sh demo/skills/*.sh demo/web/*.sh demo/performance/01-*.sh; do
  echo "=== $f ==="; ./"$f" || { echo "FAIL: $f"; break; }
done
```

Then run the paid demo and the 2000-request load test on their own:

```bash
./demo/agents/03-agent-messaging.sh   # ~$0.01 on Anthropic
./demo/performance/02-load-test.sh    # free, ~1–2 min
```

---

## Advanced scenarios

Two larger orchestrated scenarios live under `scenarios/`. Each spins up its own Compose stack via `just` — they are **not** part of the 44-script sweep above.

### Air-gap — [`scenarios/airgap/`](scenarios/airgap/)

Sealed-network proof that governance denies before any upstream call.

```bash
just airgap-up
./demo/scenarios/airgap/01-egress-assert.sh   # no outbound + denial counter unchanged
./demo/scenarios/airgap/02-load.sh            # loadtest inside the sealed net
./demo/scenarios/airgap/03-governance.sh      # governance assertions in air-gap
just airgap-down
```

See [`scenarios/airgap/architecture.md`](scenarios/airgap/architecture.md).

### Scaled — [`scenarios/scaled/`](scenarios/scaled/)

Replica fan-out, soak, and scheduler isolation under load.

```bash
just scaled-up
./demo/scenarios/scaled/01-load.sh
./demo/scenarios/scaled/02-soak.sh
./demo/scenarios/scaled/03-replica-distribution.sh
./demo/scenarios/scaled/04-scheduler-isolation.sh
./demo/scenarios/scaled/05-quick-proof.sh
just scaled-down
```

See [`scenarios/scaled/architecture.md`](scenarios/scaled/architecture.md).

---

## Connecting the Cowork desktop app

The `enterprise-demo` plugin, its skills, agents, and MCP servers ship with the template, but plugin assignment is **per-user** — `setup-local` populates the global registry and `01-seed-data.sh` forks `enterprise-demo` into the active session's user. Whoever is authenticated when you run the seed gets the plugin.

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

If `sync` reports `0 plugins / 0 skills / 0 agents / 0 MCP`, the authenticated user has no forked plugin. Re-run the fork POST in step 3 with the correct user's token.

To revert Cowork to direct Anthropic: `systemprompt-cowork uninstall` then Cmd+Q and relaunch.

---

## Troubleshooting

```bash
# Token expired or missing
./demo/00-preflight.sh

# Services down
systemprompt infra services cleanup --yes
systemprompt infra services start --kill-port-process
# wait for "All services started successfully", then re-run the demo

# Empty analytics / governance tables
./demo/01-seed-data.sh
```

---

## Verification status

All 44 category demos + 2 setup scripts were run end-to-end against a freshly seeded local stack on **2026-05-21** against template `v0.11.0`. Every script exited `0`.

| Category | Count | Result |
|----------|------:|--------|
| Setup (preflight, seed) | 2 | All pass |
| infrastructure/ | 5 | All pass |
| cloud/ | 1 | Pass |
| governance/ | 9 | All pass |
| mcp/ | 3 | All pass |
| analytics/ | 8 | All pass |
| agents/ | 5 | All pass (incl. paid `03-agent-messaging.sh`) |
| users/ | 4 | All pass |
| skills/ | 5 | All pass |
| web/ | 2 | All pass |
| performance/ | 2 | All pass (incl. 2000-request load test) |
| **Total** | **46** | **All pass** |

Advanced `scenarios/airgap` and `scenarios/scaled` were **not** included in this sweep — they require dedicated Compose stacks (`just airgap-up` / `just scaled-up`).

---

## Recording

SVG terminal recordings and video recording infrastructure are in `recording/`. See `recording/RECORDING-GUIDE.md` for the video production workflow.

---

## License

MIT — see [LICENSE](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE) for details.
