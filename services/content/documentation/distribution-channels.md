---
title: "Distribution Channels"
description: "How personal marketplaces are distributed via git repositories and Claude Code plugin format in Foodles."
author: "systemprompt.io"
slug: "distribution-channels"
keywords: "distribution, git, Claude Code, marketplace, plugin format, personal marketplace"
kind: "guide"
public: true
tags: ["marketplace", "distribution", "git"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand how personal marketplaces are served as git repositories"
  - "Know how marketplace inheritance works from organisation to user"
  - "Clone and use marketplace content with standard git tooling"
related_docs:
  - title: "Marketplace"
    url: "/documentation/marketplace"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Forking & Customization"
    url: "/documentation/forking"
  - title: "Marketplace Versions"
    url: "/documentation/marketplace-versions"
---

# Distribution Channels

Every user in your organisation gets a personalised marketplace — a complete collection of the skills, agents, plugins, and MCP servers available to that person. The platform serves each marketplace as a git repository, providing native integration with Claude Code and any git-compatible service.

## Personal Marketplaces

Each user's marketplace is unique to them. It contains:

- The skills, agents, and plugins approved by the organisation
- Any department-level additions or restrictions
- The user's own forked and customised content

The result is that every person has exactly the AI capabilities they need — nothing more, nothing less — managed centrally by the organisation but personalised for the individual.

## Marketplace Inheritance

Marketplaces inherit from higher levels in the organisation:

1. **Organisation baseline** — Administrators define the foundation: approved plugins, skills, agents, and hooks available to everyone
2. **Department scope** — Department-level rules can add or restrict content for specific teams
3. **User personalisation** — Users can fork and customise content within their marketplace, adding their own modifications on top of what the organisation provides

Each level inherits from the one above. Changes at the organisation level propagate down to all users. User-level customisations are preserved independently.

## Git Repository Access

Each user's marketplace is available as a git repository via the smart upload-pack protocol:

```bash
# Clone a user's marketplace
git clone https://foodles.example.com/marketplace/{user_id}/repo.git

# Pull updates
cd repo && git pull
```

Standard git tooling works out of the box — clone, pull, branch, diff. The repository contains the complete marketplace: skills, agents, hooks, MCP server configs, and scripts.

Because the marketplace is a git repository, it integrates natively with any service that speaks git. No proprietary protocols or custom clients required.

## Claude Code Integration

Every plugin in the marketplace includes `.claude-plugin/plugin.json` metadata, making it directly compatible with Claude Code:

```bash
# Run Claude Code with a marketplace plugin
claude --plugin /path/to/plugin

# Combine multiple plugins
claude --plugin plugin-a --plugin plugin-b
```

Plugin bundles include:
- `config.yaml` — Plugin manifest
- `.claude-plugin/plugin.json` — Claude Code metadata
- `skills/` — Skill Markdown files
- `agents/` — Agent system prompt documents
- `hooks/` — Event hook configurations
- `scripts/` — Bash and PowerShell scripts

## Automatic Sync

A background sync job detects changes and rebuilds affected marketplaces:

1. Database triggers mark users as "dirty" when any entity changes (skills, agents, hooks, plugins)
2. The sync job picks up dirty users
3. Marketplace output is rebuilt from current state
4. Content is committed to versioned storage
5. The dirty flag is cleared

Only changed users are reprocessed, keeping sync fast at scale. Changes propagate to the git repository within minutes.

## Version History

Each sync writes a versioned snapshot. This provides:

- **Complete audit trail** of how each user's marketplace evolved over time
- **Rollback capability** — restore any previous version
- **Changelog tracking** — what changed between versions

## Portability

All content uses standard formats:

- Skills are plain Markdown with YAML frontmatter
- Plugin configs are YAML
- Distribution uses git
- No proprietary encoding or vendor-specific schemas

Exported content works outside Foodles without modification — open files in any editor, parse with any toolchain, or feed to any Markdown processor.
