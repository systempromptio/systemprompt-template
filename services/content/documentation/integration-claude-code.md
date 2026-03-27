---
title: "Integration: Claude Code (CLI)"
description: "How the governance platform integrates with Claude Code through the native plugin system, personalised marketplace distribution, and tool-call-level governance."
author: "systemprompt.io"
slug: "integration-claude-code"
keywords: "Claude Code, CLI, plugin, integration, marketplace, distribution, governance, developer"
kind: "guide"
public: true
tags: ["introduction", "integration", "claude-code", "cli", "plugins"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Understand how the platform integrates with Claude Code via the plugin system"
  - "Know how personalised marketplaces distribute governed capabilities to developers"
  - "See how governance is applied at the tool call level without disrupting developer workflow"
related_docs:
  - title: "Introduction to the Platform"
    url: "/documentation/introduction"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Distribution Channels"
    url: "/documentation/distribution-channels"
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
---

# Integration: Claude Code (CLI)

**Claude Code is Anthropic's command-line interface for Claude, used by developers for coding, DevOps, and automation tasks. The governance platform integrates with Claude Code through its native plugin system — distributing governed skills, agents, and tools to developers via personalised marketplaces. Governance is applied at the tool call level, so developers get the capabilities they need while the organisation maintains control and visibility.**

---

## How It Works

Claude Code supports a plugin system that allows organisations to extend what Claude can do. The governance platform uses this system to distribute governed AI capabilities to every developer in your organisation.

The flow:

1. **Administrators configure plugins** — Skills, agents, MCP servers, and hooks are bundled into plugins and assigned to roles and departments via the admin dashboard.
2. **The marketplace assembles personalised bundles** — Each user gets a marketplace tailored to their role, department, and individual customisations. The marketplace is served as a git repository.
3. **Developers install plugins** — Standard Claude Code plugin installation. No proprietary tooling required.
4. **Governance is applied transparently** — When a developer uses a governed skill or invokes a tool, the governance pipeline handles authentication, authorisation, rate limiting, and audit logging.

## Plugin System

Plugins are the unit of distribution. A plugin bundles:

- **Skills** — Reusable AI capabilities that define what Claude can do (e.g., "summarise a document", "review a pull request")
- **Agents** — AI workers configured with specific instructions, tools, and system prompts
- **MCP Servers** — Tool servers that provide external capabilities (database queries, API calls, file operations)
- **Hooks** — Event-driven triggers that fire when specific actions occur

Plugins are scoped by role. An administrator sees different plugins than a developer, who sees different plugins than an analyst. This is managed centrally — individual developers do not need to configure access.

```bash
# Run Claude Code with a governed plugin
claude --plugin /path/to/plugin

# Combine multiple plugins
claude --plugin plugin-a --plugin plugin-b
```

Every plugin includes `.claude-plugin/plugin.json` metadata, making it directly compatible with Claude Code's plugin system. See [Plugins](/documentation/plugins) for the full dashboard guide and [Create Plugin](/documentation/create-plugin) for creating new plugins.

## Personalised Marketplaces

Every user in your organisation gets a personalised marketplace — a complete collection of the skills, agents, plugins, and MCP servers available to that person.

Marketplaces inherit from three levels:

1. **Organisation baseline** — Approved plugins, skills, and agents available to everyone
2. **Department scope** — Department-level additions or restrictions for specific teams
3. **User personalisation** — Individual customisations and forked content

The marketplace is served as a **git repository**, so it integrates with any git-compatible tooling. Developers clone their marketplace, pull updates, and use standard version control workflows.

See [Distribution Channels](/documentation/distribution-channels) for the technical detail and [Marketplace](/documentation/marketplace) for management.

## Tool-Call-Level Governance

Every tool invocation through a governed plugin passes through the governance pipeline. This happens at the MCP protocol level, so it applies regardless of what the tool does or how the developer invokes it.

| Governance Layer | What Happens |
|-----------------|-------------|
| **Authentication** | The user's identity is verified via the shared auth system |
| **Authorisation** | Access control checks whether this user can invoke this tool |
| **Rate Limiting** | Per-role rate limits prevent resource abuse |
| **Execution Logging** | The tool call, its parameters, and its result are recorded |
| **Event Hooks** | Configured hooks fire (e.g., notify a channel, trigger a workflow) |
| **Cost Tracking** | Token consumption is attributed to the user, department, and agent |

This governance is transparent to the developer. They use Claude Code normally — the governance layer adds control without adding friction.

See [Tool Governance](/documentation/tool-governance) for access control configuration and [Hooks](/documentation/hooks) for event-driven automation.

## What Developers Experience

From a developer's perspective, the integration is straightforward:

1. **Install the plugin** provided by their organisation
2. **Use Claude Code as normal** — governed skills and tools are available alongside standard capabilities
3. **No manual authentication** — the shared identity system handles auth transparently
4. **No configuration required** — governance policies are applied by the organisation, not by the developer

The developer gets the AI capabilities they need. The organisation gets visibility, control, and audit trails. Neither side has to compromise.
