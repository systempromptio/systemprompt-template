<div align="center">
  <a href="https://systemprompt.io">
    <img src="https://systemprompt.io/logo.svg" alt="systemprompt.io" width="150" />
  </a>
  <p><strong>Production infrastructure for AI agents</strong></p>
  <p><a href="https://systemprompt.io">systemprompt.io</a> • <a href="https://systemprompt.io/documentation">Documentation</a> • <a href="https://github.com/systempromptio/systemprompt-core">Core</a> • <a href="https://github.com/systempromptio/systemprompt-template">Template</a></p>
</div>

---

# Demo Suite

**44 runnable demo scripts** organised into 10 categories, plus 2 setup scripts, plus 2 orchestrated multi-container scenarios that demonstrate the air-gap and horizontal-scaling claims in the factsheet.

Every section below is a fenced code block — open this file in your editor, click into the block, copy, paste into your terminal.

## Contents

- [Quick start](#quick-start)
- [Categories at a glance](#categories-at-a-glance)
- [Demos by category](#demos-by-category)
  - [Setup](#setup-run-these-first)
  - [Infrastructure](#infrastructure) · [Cloud](#cloud) · [Governance](#governance) · [MCP](#mcp) · [Analytics](#analytics) · [Agents](#agents) · [Users](#users) · [Skills](#skills) · [Web](#web) · [Performance](#performance)
- [Run them all at once](#run-them-all-at-once)
- [Scenarios — factsheet proofs](#scenarios--factsheet-proofs)
  - [Air-gap scenario](#air-gap-scenario)
  - [Scaled scenario](#scaled-scenario)
- [Connecting the Bridge desktop app](#connecting-the-bridge-desktop-app)
- [Troubleshooting](#troubleshooting)
- [Verification status](#verification-status)

---

## Quick start

Prereqs: built binary, services up. From the repo root, run these four blocks in order:

Build and start services (skip if already running on :8080):

```bash
just build && just start
```

Preflight — health check, admin user, token written to `demo/.token`:

```bash
./demo/00-preflight.sh
```

Seed — governance decisions, events, skills, content (powers the analytics demos):

```bash
./demo/01-seed-data.sh
```

First demo — governance allows an admin-scope tool call:

```bash
./demo/governance/01-happy-path.sh
```

The preflight step writes a JWT to `demo/.token`. **Every other script reads it automatically — you never pass a token by hand.**

---

## Categories at a glance

Organised by the three pillars of [systemprompt.io](https://systemprompt.io): **Infrastructure**, **Capabilities**, and **Integrations**.

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

**Total: 44 category scripts + 2 setup scripts. 43 free, 1 costs ~$0.01.** Plus two multi-container scenarios — see [Scenarios](#scenarios--factsheet-proofs).

---

## Demos by category

Every demo below is a single fenced command. Run them in any order once preflight + seed are done.

### Setup (run these first)

[`00-preflight.sh`](00-preflight.sh) — service health check, admin user creation, token written to `demo/.token`.
```bash
./demo/00-preflight.sh
```

[`01-seed-data.sh`](01-seed-data.sh) — populate governance decisions, events, skills, content.
```bash
./demo/01-seed-data.sh
```

### Infrastructure

[`infrastructure/01-services.sh`](infrastructure/01-services.sh) — service status and health checks.
```bash
./demo/infrastructure/01-services.sh
```

[`infrastructure/02-database.sh`](infrastructure/02-database.sh) — tables, schema, queries, migrations.
```bash
./demo/infrastructure/02-database.sh
```

[`infrastructure/03-jobs.sh`](infrastructure/03-jobs.sh) — job scheduling and execution history.
```bash
./demo/infrastructure/03-jobs.sh
```

[`infrastructure/04-logs.sh`](infrastructure/04-logs.sh) — log viewing, search, traces, requests.
```bash
./demo/infrastructure/04-logs.sh
```

[`infrastructure/05-config.sh`](infrastructure/05-config.sh) — full platform configuration overview.
```bash
./demo/infrastructure/05-config.sh
```

### Cloud

[`cloud/01-cloud-overview.sh`](cloud/01-cloud-overview.sh) — auth status, profiles, deployment info.
```bash
./demo/cloud/01-cloud-overview.sh
```

### Governance

[`governance/00-audit-write-smoke.sh`](governance/00-audit-write-smoke.sh) — smoke-test that audit rows actually land in Postgres.
```bash
./demo/governance/00-audit-write-smoke.sh
```

[`governance/01-happy-path.sh`](governance/01-happy-path.sh) — governance ALLOWS admin-scope tool call.
```bash
./demo/governance/01-happy-path.sh
```

[`governance/02-refused-path.sh`](governance/02-refused-path.sh) — governance DENIES user-scope agent.
```bash
./demo/governance/02-refused-path.sh
```

[`governance/03-audit-trail.sh`](governance/03-audit-trail.sh) — query governance decisions from the database.
```bash
./demo/governance/03-audit-trail.sh
```

[`governance/04-governance-happy.sh`](governance/04-governance-happy.sh) — all 3 governance rules pass for admin agent.
```bash
./demo/governance/04-governance-happy.sh
```

[`governance/05-governance-denied.sh`](governance/05-governance-denied.sh) — scope + blocklist deny for user agent.
```bash
./demo/governance/05-governance-denied.sh
```

[`governance/06-secret-breach.sh`](governance/06-secret-breach.sh) — secret detection blocks credentials in a tool payload.
```bash
./demo/governance/06-secret-breach.sh
```

[`governance/07-rate-limiting.sh`](governance/07-rate-limiting.sh) — rate limit, security, and server configuration.
```bash
./demo/governance/07-rate-limiting.sh
```

[`governance/08-hooks.sh`](governance/08-hooks.sh) — hook listing and validation.
```bash
./demo/governance/08-hooks.sh
```

### MCP

[`mcp/01-mcp-servers.sh`](mcp/01-mcp-servers.sh) — server status and tools-by-server.
```bash
./demo/mcp/01-mcp-servers.sh
```

[`mcp/02-mcp-access-tracking.sh`](mcp/02-mcp-access-tracking.sh) — OAuth + MCP tool call tracking.
```bash
./demo/mcp/02-mcp-access-tracking.sh
```

[`mcp/03-mcp-tool-execution.sh`](mcp/03-mcp-tool-execution.sh) — tool listings, execution logs.
```bash
./demo/mcp/03-mcp-tool-execution.sh
```

### Analytics

[`analytics/01-overview.sh`](analytics/01-overview.sh) — dashboard overview.
```bash
./demo/analytics/01-overview.sh
```

[`analytics/02-agent-analytics.sh`](analytics/02-agent-analytics.sh) — agent stats, trends, deep-dives.
```bash
./demo/analytics/02-agent-analytics.sh
```

[`analytics/03-cost-analytics.sh`](analytics/03-cost-analytics.sh) — cost summary, breakdown, trends.
```bash
./demo/analytics/03-cost-analytics.sh
```

[`analytics/04-request-analytics.sh`](analytics/04-request-analytics.sh) — AI request volume, latency, models.
```bash
./demo/analytics/04-request-analytics.sh
```

[`analytics/05-session-analytics.sh`](analytics/05-session-analytics.sh) — session stats, trends, real-time.
```bash
./demo/analytics/05-session-analytics.sh
```

[`analytics/06-content-traffic.sh`](analytics/06-content-traffic.sh) — content engagement, traffic, geo.
```bash
./demo/analytics/06-content-traffic.sh
```

[`analytics/07-conversations.sh`](analytics/07-conversations.sh) — conversation stats, trends, listing.
```bash
./demo/analytics/07-conversations.sh
```

[`analytics/08-tool-analytics.sh`](analytics/08-tool-analytics.sh) — tool usage stats, trends.
```bash
./demo/analytics/08-tool-analytics.sh
```

### Agents

[`agents/01-list-agents.sh`](agents/01-list-agents.sh) — agent discovery (admin and core views).
```bash
./demo/agents/01-list-agents.sh
```

[`agents/02-agent-config.sh`](agents/02-agent-config.sh) — validation, MCP tool access, status.
```bash
./demo/agents/02-agent-config.sh
```

[`agents/03-agent-messaging.sh`](agents/03-agent-messaging.sh) — **full agent pipeline with a live Anthropic call (~$0.01)**.
```bash
./demo/agents/03-agent-messaging.sh
```

[`agents/04-agent-tracing.sh`](agents/04-agent-tracing.sh) — traces, artifacts, cost attribution.
```bash
./demo/agents/04-agent-tracing.sh
```

[`agents/05-agent-registry.sh`](agents/05-agent-registry.sh) — A2A gateway and agent logs.
```bash
./demo/agents/05-agent-registry.sh
```

### Users

[`users/01-user-listing.sh`](users/01-user-listing.sh) — user listing, counts, stats, search.
```bash
./demo/users/01-user-listing.sh
```

[`users/02-role-management.sh`](users/02-role-management.sh) — user details and role inspection.
```bash
./demo/users/02-role-management.sh
```

[`users/03-session-management.sh`](users/03-session-management.sh) — current session, available profiles.
```bash
./demo/users/03-session-management.sh
```

[`users/04-ip-ban.sh`](users/04-ip-ban.sh) — add/remove IP bans with verification.
```bash
./demo/users/04-ip-ban.sh
```

### Skills

[`skills/01-skill-lifecycle.sh`](skills/01-skill-lifecycle.sh) — skill listing, details, sync status.
```bash
./demo/skills/01-skill-lifecycle.sh
```

[`skills/02-content-management.sh`](skills/02-content-management.sh) — content listing, search, popularity.
```bash
./demo/skills/02-content-management.sh
```

[`skills/03-file-management.sh`](skills/03-file-management.sh) — file listing, config, storage stats.
```bash
./demo/skills/03-file-management.sh
```

[`skills/04-plugin-management.sh`](skills/04-plugin-management.sh) — plugins, hooks, extensions, capabilities.
```bash
./demo/skills/04-plugin-management.sh
```

[`skills/05-contexts.sh`](skills/05-contexts.sh) — context CRUD (create, show, edit, delete).
```bash
./demo/skills/05-contexts.sh
```

### Web

[`web/01-web-config.sh`](web/01-web-config.sh) — content types, templates, assets.
```bash
./demo/web/01-web-config.sh
```

[`web/02-sitemap-validate.sh`](web/02-sitemap-validate.sh) — sitemap config, web validation.
```bash
./demo/web/02-sitemap-validate.sh
```

### Performance

[`performance/01-request-tracing.sh`](performance/01-request-tracing.sh) — typed data, flow maps, micro-benchmarks.
```bash
./demo/performance/01-request-tracing.sh
```

[`performance/02-load-test.sh`](performance/02-load-test.sh) — 2000-request load test (free, ~1–2 min).
```bash
./demo/performance/02-load-test.sh
```

---

## Run them all at once

Replay the entire free suite in one shot. Skips the paid `agents/03-agent-messaging.sh` — paste that one separately if you want a live Anthropic call.

```bash
./demo/00-preflight.sh && ./demo/01-seed-data.sh && \
for f in demo/infrastructure/*.sh demo/cloud/*.sh demo/governance/*.sh \
         demo/mcp/*.sh demo/analytics/*.sh \
         demo/agents/01-*.sh demo/agents/02-*.sh demo/agents/04-*.sh demo/agents/05-*.sh \
         demo/users/*.sh demo/skills/*.sh demo/web/*.sh demo/performance/*.sh; do
  echo "=== $f ==="; ./"$f" || { echo "FAIL: $f"; break; }
done
```

The paid demo:

```bash
./demo/agents/03-agent-messaging.sh
```

---

## Scenarios — factsheet proofs

The two scenarios below back the air-gap and horizontal-scaling claims in the factsheet. Each spins up its own Docker Compose stack. Treat them like a separate environment — they do **not** share state with the local single-node stack from the Quick start.

### Air-gap scenario

**What it proves.** The platform runs as a genuinely closed system: a network proof (no outbound connections from any container in the sealed subnet) and an application proof (governance denies un-listed models before any upstream call). Backs the factsheet claim that the gateway can be deployed in fully air-gapped environments with no egress.

Architecture and per-script walkthrough: [`scenarios/airgap/architecture.md`](scenarios/airgap/architecture.md).

**One-time setup.** Build and bring up the sealed stack (postgres + mock-inference + app + monitor + ingress, all on an `internal: true` Docker network — the only ingress is a socat proxy on host `:8090`). First build is ~10 min from source.

```bash
just airgap-up
```

**Run the assertions.** All three pass with exit `0`:

```bash
just airgap-test
```

That recipe runs the three scripts in order, stopping on first failure. To run them individually:

[`scenarios/airgap/01-egress-assert.sh`](scenarios/airgap/01-egress-assert.sh) — two independent proofs of closure: (1) `ss`/`conntrack` from inside the sealed network shows no remote address outside the internal subnet; (2) a burst of governance-denied `/v1/messages` calls leaves the mock-inference request counter unchanged — denial precedes any upstream call.
```bash
./demo/scenarios/airgap/01-egress-assert.sh
```

[`scenarios/airgap/02-load.sh`](scenarios/airgap/02-load.sh) — drives the core loadtest harness against the air-gap ingress in two scenarios (`gateway-inference` and `governance-only`) and asserts the air-gap performance thresholds.
```bash
./demo/scenarios/airgap/02-load.sh
```

[`scenarios/airgap/03-governance.sh`](scenarios/airgap/03-governance.sh) — end-to-end governance assertions inside the sealed network: allow-listed model → 200 via the mock; un-listed model → 403 before any upstream call.
```bash
./demo/scenarios/airgap/03-governance.sh
```

**Reproducibility from a clean DB.** Tears down with volumes, brings back up reusing the cached image, runs the full suite — prints wall-clock time so a reviewer can see "from zero state" in front of them:

```bash
just airgap-fresh-test
```

**Tear it down.**

```bash
just airgap-down
```

### Scaled scenario

**What it proves.** The platform scales horizontally: N stateless API replicas behind nginx round-robin, one dedicated scheduler replica with cron isolation (no double execution), event bus crosses replicas via Postgres `LISTEN/NOTIFY`, and the whole thing holds p95 under sustained load. Backs the factsheet claim that the gateway scales horizontally with no shared mutable state on the API tier.

Architecture and per-script walkthrough: [`scenarios/scaled/architecture.md`](scenarios/scaled/architecture.md).

**One-time setup.** Bring up the multi-replica stack (postgres primary + read replica + 3 app replicas + 1 scheduler + nginx LB). Adjust `REPLICAS=` to fan out further.

```bash
just scaled-up REPLICAS=3
```

**Run the assertions.** Runs `01-load.sh`, `03-replica-distribution.sh`, `04-scheduler-isolation.sh` in order — skips the long soak test:

```bash
just scaled-test
```

Or run them individually:

[`scenarios/scaled/01-load.sh`](scenarios/scaled/01-load.sh) — load test through the nginx LB. Asserts p95 ≤ 500 ms and error rate ≤ 2 % across all replicas.
```bash
./demo/scenarios/scaled/01-load.sh
```

[`scenarios/scaled/02-soak.sh`](scenarios/scaled/02-soak.sh) — long sustained run (~1 hour) with periodic latency windows and a memory sampler. Asserts no p95 drift > 5 % and no growing memory. **Not** included in `just scaled-test`.
```bash
./demo/scenarios/scaled/02-soak.sh
```

[`scenarios/scaled/03-replica-distribution.sh`](scenarios/scaled/03-replica-distribution.sh) — (a) `lb-fairness` scenario buckets responses by `x-served-by` header to prove every replica registered with the LB; (b) subscribes to SSE on replica B, publishes on replica A, asserts cross-replica delivery via `PostgresEventBridge`.
```bash
./demo/scenarios/scaled/03-replica-distribution.sh
```

[`scenarios/scaled/04-scheduler-isolation.sh`](scenarios/scaled/04-scheduler-isolation.sh) — asserts that scheduled jobs execute exactly once across the fleet. API replicas mount a `scheduler-disabled` config; only the dedicated scheduler container runs cron. Verified via `infra jobs history` row counts and per-container log markers.
```bash
./demo/scenarios/scaled/04-scheduler-isolation.sh
```

[`scenarios/scaled/05-quick-proof.sh`](scenarios/scaled/05-quick-proof.sh) — fast end-to-end proof useful for screencasts: hits the LB N times, prints the replica distribution, fires a cross-replica SSE event, confirms a job ran exactly once.
```bash
./demo/scenarios/scaled/05-quick-proof.sh
```

**Tear it down.**

```bash
just scaled-down
```

---

## Connecting the Bridge desktop app

The `enterprise-demo` plugin, its skills, agents, and MCP servers ship with the template, but plugin assignment is **per-user** — `setup-local` populates the global registry and `01-seed-data.sh` forks `enterprise-demo` into the active session's user. Whoever is authenticated when you run the seed gets the plugin.

Bring up the template:

```bash
just setup-local <anthropic_key> [openai_key] [gemini_key]
just start
```

Preflight + seed (also forks `enterprise-demo` for the current admin session):

```bash
./demo/00-preflight.sh
./demo/01-seed-data.sh
```

Issue a PAT: open <http://localhost:8080/admin/devices>, sign in, create device, copy the `sp-live-...` token. If you need to fork for a different user than the one seeded, hit the fork endpoint with their JWT:

```bash
curl -X POST http://localhost:8080/api/public/admin/user/fork/plugin \
  -H "Authorization: Bearer <their-JWT>" \
  -H 'Content-Type: application/json' \
  -d '{"org_plugin_id":"enterprise-demo"}'
```

Build and wire up the bridge helper (from systemprompt-core):

```bash
cargo build --release --manifest-path bin/bridge/Cargo.toml
BIN=bin/bridge/target/release/systemprompt-bridge
"$BIN" login sp-live-... --gateway http://localhost:8080
"$BIN" install --apply
"$BIN" sync
```

Cmd+Q Bridge and relaunch. It now reads `inferenceProvider=gateway` from the MDM keys and routes every request through `http://localhost:8080` with a JWT minted by the helper.

Verify:

```bash
"$BIN" status
"$BIN" whoami
```

If `sync` reports `0 plugins / 0 skills / 0 agents / 0 MCP`, the authenticated user has no forked plugin — re-run the fork POST above with the correct user's token.

Revert Bridge to direct Anthropic:

```bash
systemprompt-bridge uninstall
```

then Cmd+Q and relaunch.

---

## Troubleshooting

Token expired or missing:

```bash
./demo/00-preflight.sh
```

Services down:

```bash
systemprompt infra services cleanup --yes
systemprompt infra services start --kill-port-process
```

Wait for `All services started successfully`, then re-run the demo.

Empty analytics / governance tables:

```bash
./demo/01-seed-data.sh
```

---

## Verification status

All 44 category demos + 2 setup scripts + both scenario stacks were run end-to-end on **2026-05-21** against template `v0.11.0`. Every script exited `0`.

| Set | Count | Result |
|-----|------:|--------|
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
| scenarios/airgap (just airgap-test) | 3 | All pass |
| scenarios/scaled (just scaled-test) | 3 | All pass — long soak (`02-soak.sh`) excluded by design |
| **Total** | **52** | **All pass** |

---

## Recording

SVG terminal recordings and video recording infrastructure are in `recording/`. See `recording/RECORDING-GUIDE.md` for the video production workflow.

---

## License

MIT — see [LICENSE](https://github.com/systempromptio/systemprompt-template/blob/main/LICENSE) for details.
