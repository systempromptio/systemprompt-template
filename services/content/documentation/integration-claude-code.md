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
  - title: "Integration: Claude Cowork"
    url: "/documentation/integration-claude-cowork"
  - title: "Distribution Channels"
    url: "/documentation/distribution-channels"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
  - title: "Introduction to the Platform"
    url: "/documentation/introduction"
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

## Installation

The platform serves each user's marketplace as a git repository over HTTP. Claude Code's native `plugin marketplace add` command clones this repository and registers the plugins automatically.

### One-Line Install

Every user gets a personalised install URL. Find it in the **Share & Install** menu (the share icon in the header of any admin page) or on the [Control Center](/admin/) onboarding panel.

**From your terminal:**

```bash
claude plugin marketplace add https://your-instance.example.com/api/public/marketplace/{user_id}.git
```

**From inside a Claude Code conversation:**

```
/plugin marketplace add https://your-instance.example.com/api/public/marketplace/{user_id}.git
```

Both commands do the same thing. The URL contains the user's ID, so the marketplace content is personalised to that user's role, department, and plugin selections.

### How the Internal Git Server Works

The platform acts as its own git server. When Claude Code runs `plugin marketplace add`, it performs a standard `git clone` against the platform's HTTP endpoint. The platform generates a bare git repository on the fly from the user's current marketplace state:

1. The endpoint receives a git smart protocol request (`info/refs` + `upload-pack`)
2. The platform assembles the user's plugins, skills, agents, hooks, and scripts from the database and file system
3. A temporary bare git repository is created with the assembled content
4. Git serves the repository to the client using the standard upload-pack protocol

This means the install URL works with any git client, not just Claude Code. You can `git clone` the URL directly to inspect the contents.

### Updating

To pull the latest changes after an administrator updates plugins or skills:

```bash
claude plugin marketplace update
```

Or clone the URL again to get a fresh copy.

### Enterprise Deployment

For rolling out to a team, administrators can distribute the install command through any internal channel — Slack, email, onboarding docs, or device management. Each user's URL is unique and contains their personalised marketplace.

For automated deployment, the install URL can be included in developer onboarding scripts or dotfile repositories. The platform's git endpoint requires no authentication beyond the user ID in the URL.

## What Developers Experience

From a developer's perspective, the integration is straightforward:

1. **Run the install command** provided by their organisation (one line)
2. **Use Claude Code as normal** — governed skills and tools are available alongside standard capabilities
3. **No manual authentication** — the shared identity system handles auth transparently
4. **No configuration required** — governance policies are applied by the organisation, not by the developer

The developer gets the AI capabilities they need. The organisation gets visibility, control, and audit trails. Neither side has to compromise.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Failed to clone marketplace repository` | Server error during export | Check the platform logs with `systemprompt infra logs view --level error --since 1h` |
| Empty plugin list after install | User has no plugins selected | Go to [My Plugins](/admin/my/plugins/) and select plugins |
| `The requested URL returned error: 500` | Plugin configuration error | Check that all plugin `config.yaml` files have required fields (`author.email`, `license`) |
