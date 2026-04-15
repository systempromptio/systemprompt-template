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

| Category | Scripts | What it covers | Cost |
|----------|---------|---------------|------|
| [governance/](governance/) | 8 | Tool access control, scope enforcement, secret detection, audit trails, hooks | Free |
| [agents/](agents/) | 5 | Agent discovery, configuration, messaging, tracing, A2A registry | 2 @ ~$0.01 |
| [mcp/](mcp/) | 3 | MCP server management, access tracking, tool execution | Free |
| [skills/](skills/) | 5 | Skills, content, files, plugins, contexts | Free |
| [infrastructure/](infrastructure/) | 5 | Services, database, jobs, logs, configuration | Free |
| [analytics/](analytics/) | 8 | Overview, agents, costs, requests, sessions, content/traffic, conversations, tools | Free |
| [users/](users/) | 4 | User CRUD, roles, sessions, IP bans | Free |
| [web/](web/) | 2 | Content types, templates, sitemaps, validation | Free |
| [cloud/](cloud/) | 1 | Auth status, profiles, deployment info | Free |
| [performance/](performance/) | 2 | Request tracing, benchmarks, load testing | Free |

**Total: 42 category scripts + preflight + seed = 44 scripts. 41 free, 2 cost ~$0.01 each.**

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

### Governance
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

### Agents
| Script | Description |
|--------|-------------|
| `agents/01-list-agents.sh` | Agent discovery — admin and core views |
| `agents/02-agent-config.sh` | Validation, MCP tool access, status |
| `agents/03-agent-messaging.sh` | Full agent pipeline with AI (~$0.01) |
| `agents/04-agent-tracing.sh` | Traces, artifacts, cost attribution |
| `agents/05-agent-registry.sh` | A2A gateway, agent logs |

### MCP Servers
| Script | Description |
|--------|-------------|
| `mcp/01-mcp-servers.sh` | Server status, tools by server |
| `mcp/02-mcp-access-tracking.sh` | OAuth + MCP tool call tracking |
| `mcp/03-mcp-tool-execution.sh` | Tool listings, execution logs |

### Skills & Content
| Script | Description |
|--------|-------------|
| `skills/01-skill-lifecycle.sh` | Skill listing, details, sync status |
| `skills/02-content-management.sh` | Content listing, search, popularity |
| `skills/03-file-management.sh` | File listing, config, storage stats |
| `skills/04-plugin-management.sh` | Plugins, hooks, extensions, capabilities |
| `skills/05-contexts.sh` | Context CRUD — create, show, edit, delete |

### Infrastructure
| Script | Description |
|--------|-------------|
| `infrastructure/01-services.sh` | Service status and health checks |
| `infrastructure/02-database.sh` | Tables, schema, queries, migrations |
| `infrastructure/03-jobs.sh` | Job scheduling, execution history |
| `infrastructure/04-logs.sh` | Log viewing, search, traces, requests |
| `infrastructure/05-config.sh` | Full platform configuration overview |

### Analytics
| Script | Description |
|--------|-------------|
| `analytics/01-overview.sh` | Dashboard overview |
| `analytics/02-agent-analytics.sh` | Agent stats, trends, deep-dives |
| `analytics/03-cost-analytics.sh` | Cost summary, breakdown, trends |
| `analytics/04-request-analytics.sh` | AI request volume, latency, models |
| `analytics/05-session-analytics.sh` | Session stats, trends, real-time |
| `analytics/06-content-traffic.sh` | Content engagement, traffic, geo |
| `analytics/07-conversations.sh` | Conversation stats, trends, listing |
| `analytics/08-tool-analytics.sh` | Tool usage stats, trends |

### Users & Auth
| Script | Description |
|--------|-------------|
| `users/01-user-crud.sh` | User listing, counts, stats, search |
| `users/02-role-management.sh` | User details and role inspection |
| `users/03-session-management.sh` | Current session, available profiles |
| `users/04-ip-ban.sh` | Add/remove IP bans with verification |

### Web Generation
| Script | Description |
|--------|-------------|
| `web/01-web-config.sh` | Content types, templates, assets |
| `web/02-sitemap-validate.sh` | Sitemap config, web validation |

### Cloud
| Script | Description |
|--------|-------------|
| `cloud/01-cloud-overview.sh` | Auth status, profiles, deployment info |

### Performance
| Script | Description |
|--------|-------------|
| `performance/01-request-tracing.sh` | Typed data, flow maps, benchmarks |
| `performance/02-load-test.sh` | 2000-request load test |

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
