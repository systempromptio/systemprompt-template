---
title: "Integration: Claude Cowork"
description: "How the governance library extends to Claude Cowork collaborative AI sessions — same governance pipeline, same audit trail, same policies across all Claude surfaces."
author: "systemprompt.io"
slug: "integration-claude-cowork"
keywords: "Claude Cowork, collaborative AI, integration, governance, agents, tools, enterprise"
kind: "guide"
public: true
tags: ["introduction", "integration", "claude-cowork", "collaborative"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Understand how the library governs collaborative AI sessions in Claude Cowork"
  - "Know that the same governance pipeline applies regardless of which Claude surface is used"
  - "Know that current restrictions are due to Anthropic's research preview, not systemprompt.io"
  - "See how access control, audit trails, and cost tracking work in collaborative contexts"
related_docs:
  - title: "Introduction"
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

**Claude Cowork is Anthropic's collaborative AI environment where teams work alongside Claude on shared tasks. The governance library integrates with Cowork through the same plugin and MCP infrastructure used by Claude Code — meaning the same governance policies, the same audit trail, and the same cost tracking apply whether a user is working alone in a CLI or collaborating with colleagues in Cowork.**

---

## Current Status

> **Important:** Claude Cowork is currently in **research preview**. As part of this preview phase, Anthropic has temporarily restricted certain capabilities — including external marketplace access — that affect all third-party integrations globally. This is not a systemprompt.io restriction. It is a side effect of the research preview status and applies to every organisation building on Cowork.

We know that companies are already relying on Claude Cowork in production workflows, and we support that as fully as possible. Our integration was **fully functional** before the research preview restrictions were applied — the governance pipeline, plugin delivery, and MCP connectivity all operated as described below. We have no control over the research preview timeline, but we are actively engaged with Anthropic and expect these restrictions to be lifted as Cowork moves toward general availability.

systemprompt.io is fully compatible with Anthropic's APIs. We have applied to the partner programme, are actively engaging with their team, and aim to be as natively integrated as possible. This is a temporary restriction imposed by the research preview, not a compatibility issue.

| Feature | Status |
|---------|--------|
| Plugin export (ZIP download) | Working |
| Governance model | Working |
| MCP connector | Working |
| Claude Code integration | Working — same APIs, fully operational |
| External marketplace sync | Temporarily unavailable — restricted by Anthropic's research preview, affects all third parties globally |
| Analytics in Cowork | Temporarily unavailable — dependent on marketplace access |
| Hooks in Cowork | Temporarily unavailable — dependent on marketplace access |

**Recommended alternative:** [Integration: Claude Code](/documentation/integration-claude-code) provides full governance integration today and is unaffected by the research preview restrictions.

**Anthropic terms:** [Acceptable Use Policy](https://www.anthropic.com/legal/aup) · [Consumer Terms](https://www.anthropic.com/legal/consumer-terms)

---

## How It Works

Claude Cowork connects to the governance library through the same mechanisms as every other Claude surface:

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

Agents configured through the governance library are available in Cowork sessions. Each agent operates under the same governance rules:

- Agent capabilities are defined by the skills assigned to them
- Tool access is governed by the calling user's permissions
- All agent activity is logged and attributed

See [Agents](/documentation/agents) for agent configuration and [Plugins](/documentation/plugins) for how agents are bundled and distributed.

## From Demo to Practice

The demo walkthroughs on this site run through Claude Cowork, showing the governance pipeline in action during collaborative sessions. See [Demo Guide](/documentation/demo) for a walkthrough of the governance features in a live Cowork environment.
