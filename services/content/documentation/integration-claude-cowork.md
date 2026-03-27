---
title: "Integration: Claude Cowork"
description: "How the governance platform extends to Claude Cowork collaborative AI sessions — same governance pipeline, same audit trail, same policies across all Claude surfaces."
author: "systemprompt.io"
slug: "integration-claude-cowork"
keywords: "Claude Cowork, collaborative AI, integration, governance, agents, tools, enterprise"
kind: "guide"
public: true
tags: ["introduction", "integration", "claude-cowork", "collaborative"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Understand how the platform governs collaborative AI sessions in Claude Cowork"
  - "Know that the same governance pipeline applies regardless of which Claude surface is used"
  - "See how access control, audit trails, and cost tracking work in collaborative contexts"
related_docs:
  - title: "Introduction to the Platform"
    url: "/documentation/introduction"
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Events & Audit Trails"
    url: "/documentation/events"
---

# Integration: Claude Cowork

**Claude Cowork is Anthropic's collaborative AI environment where teams work alongside Claude on shared tasks. The governance platform integrates with Cowork through the same plugin and MCP infrastructure used by Claude Code — meaning the same governance policies, the same audit trail, and the same cost tracking apply whether a user is working alone in a CLI or collaborating with colleagues in Cowork.**

---

## Current Status — Research Preview

> **Important:** Claude Cowork is in research preview. Some integration features described below are temporarily unavailable due to Anthropic restricting external marketplace access.

| Feature | Status |
|---------|--------|
| Plugin export (ZIP download) | Working |
| Governance model | Working |
| MCP connector | Working |
| Claude Code integration | Working — same APIs, fully operational |
| External marketplace sync | Unavailable — restricted by Anthropic |
| Analytics in Cowork | Unavailable — dependent on marketplace access |
| Hooks in Cowork | Unavailable — dependent on marketplace access |

systemprompt.io is an Anthropic partner aligned on using official APIs. We expect full Cowork integration to be restored once Anthropic completes their security review of external marketplace access. There is no official timeline.

**Recommended alternative:** [Integration: Claude Code](/documentation/integration-claude-code) provides full governance integration today.

**Anthropic terms:** [Acceptable Use Policy](https://www.anthropic.com/legal/aup) · [Consumer Terms](https://www.anthropic.com/legal/consumer-terms)

---

## How It Works

Claude Cowork connects to the governance platform through the same mechanisms as every other Claude surface:

- **Plugins** provide governed skills and agents to Cowork sessions
- **MCP servers** expose governed tools that Cowork can invoke
- **OAuth authentication** identifies users and enforces access policies
- **The governance pipeline** handles every tool call — auth, authorisation, logging, rate limiting

There is no separate integration for Cowork. The governance layer operates at the protocol level, so it applies uniformly regardless of which surface initiates the interaction.

## One Governance Layer, Every Surface

The fundamental design principle is that governance should not depend on where the user is working. A tool call made from Claude Cowork passes through the same pipeline as a tool call from Claude Code or claude.ai:

| Step | What Happens |
|------|-------------|
| **User identified** | OAuth + shared identity system determines who is making the request |
| **Access checked** | Role-based and department-scoped policies are evaluated |
| **Tool call executed** | The MCP server processes the request |
| **Audit recorded** | Full context logged — user, tool, parameters, result, timestamp |
| **Cost attributed** | Token consumption tracked to user, department, and session |

This consistency is important for compliance. When auditors ask "what governance controls are in place for AI usage?", the answer is the same regardless of surface. There is one policy engine, one audit trail, and one set of controls.

## Governance in Collaborative Sessions

Collaborative AI sessions introduce multi-user dynamics that the governance layer handles naturally:

**Per-user identity.** Each participant in a Cowork session is individually identified and authenticated. Tool calls are attributed to the specific user who initiated them, not to the session as a whole.

**Per-user access control.** Different participants may have different permissions. If a developer and a manager are in the same Cowork session, each user's tool access is governed by their own role and department — not by the session's shared context.

**Session-level audit.** All activity within a collaborative session is logged with full context. This creates a complete audit trail of who did what, when, and with which tools.

**Cost attribution.** Token consumption and tool usage costs are attributed to individual users within the session, enabling accurate department-level chargeback even in collaborative scenarios.

See [Access Control](/documentation/access-control) for role-based permission configuration, [Events & Audit Trails](/documentation/events) for audit logging, and [Cost Tracking](/documentation/cost-tracking) for chargeback capabilities.

## Agents in Cowork

Agents configured through the governance platform are available in Cowork sessions. Each agent operates under the same governance rules:

- Agent capabilities are defined by the skills assigned to them
- Tool access is governed by the calling user's permissions
- All agent activity is logged and attributed

See [Agents](/documentation/agents) for agent configuration and [Plugins](/documentation/plugins) for how agents are bundled and distributed.

## From Demo to Practice

The demo walkthroughs on this site run through Claude Cowork, showing the governance pipeline in action during collaborative sessions. See [Demo Guide](/documentation/demo) for a walkthrough of the governance features in a live Cowork environment.
