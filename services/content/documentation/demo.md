---
title: "Demo Guide — 30-Minute Walkthrough"
description: "Step-by-step presenter's guide for the Foodles enterprise demo. Covers platform architecture, agent governance, MCP integration, skills, plugins, and analytics."
author: "systemprompt.io"
slug: "demo"
keywords: "demo, walkthrough, presentation, enterprise, governance"
kind: "guide"
public: true
tags: ["demo", "enterprise", "presentation"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Run the complete 30-minute enterprise demo from start to finish"
  - "Navigate each segment with the correct CLI commands and dashboard routes"
  - "Address Lee's known concerns with specific proof points"
  - "Recover from errors mid-demo using emergency commands"
related_docs:
  - title: "Platform Architecture"
    url: "/documentation/architecture"
  - title: "Scaling Architecture"
    url: "/documentation/scaling"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Rate Limiting & Compliance"
    url: "/documentation/rate-limiting"
  - title: "Cost Tracking & Model Usage"
    url: "/documentation/cost-tracking"
  - title: "Solution Proposal"
    url: "/documentation/proposal"
---

# Demo Guide — 30-Minute Walkthrough

**TL;DR:** This is the presenter's playbook for the Foodles enterprise demo. Seven segments, 30 minutes, covering architecture through analytics. Every segment includes exact CLI commands to run, dashboard routes to navigate, talking points to deliver, and transitions to the next segment. Print this page or keep it on a second screen.

---

## Before You Start (Pre-Demo Checklist)

Complete every item before the meeting starts. Do not skip any of these.

| Check | What to Verify | How to Verify |
|-------|---------------|---------------|
| Platform running | All services healthy | `systemprompt infra services status` |
| Agents loaded | All configured agents running | `systemprompt admin agents list` |
| Browser ready | Logged into /admin/ with admin credentials | Navigate to `/admin/login` and authenticate |
| CLI terminal ready | Terminal open alongside browser | Run `systemprompt --help` to confirm |
| Dashboard populated | Demo data visible in analytics | Navigate to `/admin/` and confirm metric ribbon shows data |
| Documentation pages | All doc pages rendering | Navigate to `/documentation/proposal` and confirm content loads |
| MCP servers running | MCP servers responding on assigned ports | `systemprompt core plugins list` to verify MCP server status |

**If anything fails:** Run `systemprompt infra logs view --level error --since 5m` immediately. Fix before continuing.

---

## Segment 1: Opening Alignment (3 min)

**Purpose:** Show that you understand Foodles's AI direction, then confirm before going deeper.

### What to Show

Navigate to `/documentation/proposal` in the browser. This page introduces the platform's capabilities and the engagement structure.

### What to Say

> "We built this governance layer for exactly the kind of AI deployment Foodles is planning — the back-end infrastructure that sits between your agents and your users. Role-based access, audit trails, cost tracking."

> "Before I go deeper — does this match your current direction, or has the strategy evolved?"

> "Your team is focused on the AI strategy and the front-end experience — we provide the governance layer underneath. This is the back-end control plane."

### Transition

> "Let me show you what's under the hood."

---

## Segment 2: Platform Architecture (5 min)

**Purpose:** Demonstrate that the platform is production-grade infrastructure, not a prototype.

### CLI Commands to Run

Run these in sequence, pausing to explain each:

```bash
# Show CLI-first design — 8 domains, full discoverability
systemprompt --help

# Show running services — everything healthy
systemprompt infra services status

# Show plugin ecosystem — plugins bundle agents, skills, MCP servers
systemprompt core plugins list
```

### Key Points to Make

| Point | Detail |
|-------|--------|
| **Single binary** | One Rust binary, one PostgreSQL database. No microservice sprawl. |
| **CLI-first** | Every operation available from the CLI. `systemprompt <domain> <subcommand>` — 8 domains covering core, infra, admin, cloud, analytics, web, plugins, build. |
| **Stateless** | JWT authentication, no session affinity. Any request can hit any instance. |
| **Profile-based deployment** | Same binary, different YAML configs. Local profile for development, production profile for deployment. |
| **Self-hosted** | Runs in your infrastructure, your compliance boundary, your network. Not a SaaS dependency. |
| **Horizontal scaling** | Stateless design means N replicas behind a load balancer. Tiered rate limiting built in. Connection pooling via SQLx. |
| **Security headers** | HSTS with preload, frame deny, content type sniffing protection, strict referrer policy — all enabled by default. |

### What to Say

> "Single Rust binary. One PostgreSQL database. That's the entire dependency chain. No Redis, no Kafka, no microservice mesh."

> "Stateless means horizontal scaling is trivial — put N replicas behind a load balancer and you're done. We'll cover the scaling story in detail later."

### Links to Reference

- [Platform Architecture](/documentation/architecture)
- [Scaling Architecture](/documentation/scaling)

### Transition

> "Architecture gives you performance. Governance gives you control."

---

## Segment 3: Agent Governance (8 min) — THE CORE SEGMENT

**Purpose:** This is Lee's domain. Agent governance is the primary reason for this meeting. Spend the most time here and go deep.

---

### 3a. RBAC — Role-Based Access Control (2 min)

#### What to Show

1. Navigate to `/admin/` — the main admin dashboard
2. Navigate to the Access Control page from the sidebar
3. Show the three tabs: Plugins, Agents, MCP Servers

#### What to Demo

- Click on `developer_agent` to open the side panel
- Show how it's scoped to specific roles (`admin`, `viewer`)
- Show department-based access — which departments can interact with this agent
- Return to the table and select multiple agents using checkboxes
- Click **Bulk Assign** to show how access rules apply across multiple entities at once

#### What to Say

> "Every agent, plugin, and MCP server has role-based and department-based scoping. developer_agent has admin scope and MCP access. associate_agent has user scope only — no tool access."

> "Bulk assignment means your team can roll out access changes across all agents in one action — not one at a time."

---

### 3b. Audit Trails (2 min)

#### What to Show

1. Navigate to `/admin/events/` in the browser
2. Show the event log with timestamps, actors, and actions

#### CLI Commands to Run

```bash
# View recent platform events
systemprompt infra logs view --level info --since 1h

# Show recent AI requests
systemprompt infra logs request list --limit 5

# Pick a request ID from the output and run a full audit
systemprompt infra logs audit <request-id> --full
```

#### What to Say

> "Every tool call, every session, every prompt — logged and searchable. The full audit command shows you the complete conversation context: what the user asked, what the agent did, which tools it called, and what it returned."

> "This is the audit trail your compliance team needs. Not just 'who logged in' — but 'what did the AI do on behalf of this user.'"

---

### 3c. Secret Encryption (2 min)

#### What to Show

Navigate to secrets management in the admin dashboard.

#### Key Points

| Feature | Detail |
|---------|--------|
| **Encryption** | ChaCha20-Poly1305 AEAD — authenticated encryption with associated data |
| **Key versioning** | Secrets are versioned. Old versions remain decryptable during rotation. |
| **Audit trail** | Every secret access is logged — who accessed what, when, from which service |
| **No plaintext** | Secrets are never stored in plaintext. Encrypted at rest, decrypted only at point of use. |
| **Resolution tokens** | Short-lived, single-use tokens for secret resolution. Not long-lived API keys. |

#### What to Say

> "Secrets never stored in plaintext. ChaCha20-Poly1305 authenticated encryption — the same algorithm Signal uses. Key versioning for zero-downtime rotation. Audit trail on every access."

---

### 3d. Rate Limiting (2 min)

#### Key Points

Explain the tiered rate limiting system using data from the production profile:

| Tier | Multiplier | Description |
|------|-----------|-------------|
| **Admin** | 10x | Platform administrators — highest throughput |
| **User** | 1x | Standard authenticated users — baseline rate |
| **A2A** | 5x | Agent-to-agent communication — elevated for orchestration |
| **MCP** | 5x | MCP server requests — elevated for tool-heavy workflows |
| **Service** | 5x | Internal service communication |
| **Anon** | 0.5x | Unauthenticated requests — most restricted |

Per-endpoint limits with a **3x burst multiplier** for traffic spikes.

#### What to Say

> "Defense in depth — token expiry, scope validation, tiered access. Admin gets 10x throughput because they need it. Anonymous gets half the base rate. Agent-to-agent gets 5x because orchestration patterns generate burst traffic."

> "The burst multiplier of 3x handles traffic spikes without dropping legitimate requests. Token bucket algorithm — battle-tested."

### Links to Reference

- [Access Control](/documentation/access-control)
- [Authentication](/documentation/authentication)
- [Secrets](/documentation/secrets)
- [Rate Limiting & Compliance](/documentation/rate-limiting)

### Transition

> "Now let me show you how we handle MCP — which is what you're standardizing on."

---

## Segment 4: MCP Integration (5 min)

**Purpose:** Demonstrate that the platform is MCP-native, not MCP-bolted-on. This matters because Foodles has committed to MCP as their agent interoperability protocol.

### What to Show

1. Navigate to MCP servers in the admin dashboard
2. Click into an MCP server to show its configuration: binary path, port, endpoint, OAuth config
3. Show the MCP server list with status indicators

### CLI Commands to Run

```bash
# Show all plugins with their bundled MCP servers
systemprompt core plugins list
```

### Key Points

| Point | Detail |
|-------|--------|
| **MCP-native** | Not an adapter layer — MCP is the protocol, not a translation |
| **OAuth per server** | Each MCP server has its own OAuth configuration and scopes |
| **Hooks on every call** | Every MCP tool invocation fires hooks — audit, rate limit, validate |
| **Central registry** | All MCP servers registered centrally, discoverable by agents |
| **Port management** | MCP servers run on ports 5000-5999, managed automatically |

### What to Say

> "Your Platform Agent uses MCP for discovery and data access — our platform is the governance layer that sits between agents and MCP servers. Every tool call is authenticated, rate-limited, and logged."

> "Each MCP server has its own OAuth configuration. Agent A can access Server X but not Server Y. That's not just RBAC — it's per-server, per-agent scope validation."

### Links to Reference

- [MCP Servers](/documentation/mcp-servers)
- [Tool Governance](/documentation/tool-governance)
- [Hooks](/documentation/hooks)

### Transition

> "MCP servers provide tools. Skills and plugins govern who gets what."

---

## Segment 5: Skills & Plugins (5 min)

**Purpose:** Show how the platform organizes agents, skills, and MCP servers into governed bundles.

### What to Show

1. Navigate to the plugins dashboard — show the four tabs: Plugins, Agents, MCP Servers, Skills
2. Expand a plugin to show its bundled resources (agents, skills, MCP servers)
3. Navigate to the Skills page — show system skills vs custom skills
4. Create a custom skill live (this should take about 30 seconds)

### Demo: Create a Custom Skill

1. Navigate to the skills creation interface
2. Create a skill like "Foodles Rate Parity Policy" — a simple knowledge skill
3. Show how it immediately becomes available to the assigned agent
4. Point out that no YAML editing was required — the platform handles it

### What to Say

> "The plugin system organizes agents into governed bundles. Each agent gets the skills it needs — developer_agent gets the code skills and MCP access, associate_agent gets domain skills without tool access. Each bundle has its own RBAC, its own MCP servers, its own audit trail."

> "Skills are the knowledge layer. They're how you inject domain expertise into agents without retraining models. Custom skills take 30 seconds to create and are immediately available."

### Links to Reference

- [Plugins](/documentation/plugins)
- [Skills](/documentation/skills)
- [Agents](/documentation/agents)

### Transition

> "Everything we've shown generates data. Here's how you see it."

---

## Segment 6: Analytics & Observability (2 min)

**Purpose:** Show cost visibility and usage analytics — critical for enterprise budget justification.

### What to Show

1. Navigate to the main admin dashboard (`/admin/`)
2. Highlight the metric ribbon at the top: events, tool uses, prompts, sessions, active users, errors
3. Scroll to the AI usage chart — show token consumption over time
4. Show department activity breakdown — which departments use the most AI
5. Show model usage distribution — which models are being used and where
6. Show popular skills — which skills get the most usage

### What to Say

> "Cost visibility by model, department, user. You can see that the Revenue Management department consumed 40% of tokens last week, primarily through the Revenue Agent using Claude. RBAC governs what each role sees — analysts see their department only."

> "This is the data your finance team needs for chargeback. Export to CSV, filter by date range, break down by any dimension."

### Links to Reference

- [Dashboard](/documentation/dashboard)
- [Cost Tracking & Model Usage](/documentation/cost-tracking)

---

## Segment 7: Close & Next Steps (2 min)

**Purpose:** Move from demo to conversation. Reference the proposal and reduce perceived risk.

### What to Show

Navigate to `/documentation/proposal` in the browser.

### What to Say

> "Perpetual licence, full source code, self-hosted. This is not a vendor dependency — if we disappear tomorrow, you have the binary, the source, and the documentation."

> "The licence is independent of any consulting engagement. You can deploy independently or engage us for integration, skill authoring, and training — scoped to what you need."

> "A focused pilot is the perfect model — start with one agent, prove value, expand."

### Ask

> "What questions do you have?"

### Fallback

If hard questions come up, navigate to `/documentation/objections` — the Q&A page covers data sovereignty, model lock-in, cost justification, and enterprise scale.

---

## Appendix A: Emergency Commands

If something breaks during the demo, run these immediately:

```bash
# Check for recent errors
systemprompt infra logs view --level error --since 5m

# Check all service health
systemprompt infra services status

# Restart services if needed
systemprompt infra services restart
```

**If the platform is down:** Switch to the CLI-only demo path. Every feature shown in the dashboard is also available from the CLI. The demo still works — it just looks different.

**If a specific MCP server is down:** Skip Segment 4 and reference it verbally. Move the time to Segment 3 (governance) which is the most important segment anyway.

---

## Appendix B: Lee's Known Concerns

Use this table to map concerns to demo segments. If a concern comes up out of order, jump to the relevant segment.

| Concern | Where to Address | Key Proof Point |
|---------|-----------------|-----------------|
| **Agent consolidation** | Segment 1 + Capability Brief | Platform agents with plugin governance bundles |
| **Scaling Architecture** | Segment 2 | Stateless Rust binary, horizontal scaling, connection pooling, tiered rate limits |
| **Security and governance** | Segment 3 | RBAC with bulk assignment, full audit trails, ChaCha20-Poly1305 encryption, tiered rate limiting |
| **MCP standardization** | Segment 4 | MCP-native protocol, OAuth per server, hooks on every tool call, central registry |
| **Cost control** | Segment 6 | Per-department, per-model, per-agent cost tracking with CSV export |
| **Vendor risk** | Segment 7 | Perpetual licence, full source code, self-hosted, no SaaS dependency |

---

## Appendix C: Timing Guide

| Segment | Duration | Cumulative | Topic |
|---------|----------|-----------|-------|
| 1 | 3 min | 3 min | Opening Alignment |
| 2 | 5 min | 8 min | Platform Architecture |
| 3 | 8 min | 16 min | Agent Governance (RBAC, Audit, Secrets, Rate Limiting) |
| 4 | 5 min | 21 min | MCP Integration |
| 5 | 5 min | 26 min | Skills & Plugins |
| 6 | 2 min | 28 min | Analytics & Observability |
| 7 | 2 min | 30 min | Close & Next Steps |

**If running long:** Cut Segment 5 to 3 minutes (skip the live skill creation) and Segment 6 to 1 minute (show dashboard, state the key metric, move on). Segments 3 and 7 are non-negotiable — governance is the sale, close is the action.
