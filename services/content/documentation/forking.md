---
title: "Forking & Customization"
description: "How to fork skills, plugins, agents, and MCP servers in Foodles. Create customised versions without affecting the organisation baseline."
author: "systemprompt.io"
slug: "forking"
keywords: "forking, customization, skills, plugins, agents, baseline, origin tracking"
kind: "guide"
public: true
tags: ["marketplace", "forking", "customization"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand how forking works for skills, plugins, agents, and MCP servers"
  - "Know how origin tracking preserves lineage from fork to baseline"
  - "Use forking to enable department-specific AI behaviour"
related_docs:
  - title: "Skills"
    url: "/documentation/skills"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Distribution Channels"
    url: "/documentation/distribution-channels"
  - title: "Marketplace"
    url: "/documentation/marketplace"
---

# Forking & Customization

Forking lets users create their own customised version of any skill, plugin, agent, or MCP server without affecting the organisation baseline. The original entity is preserved and independently maintained. Changes to the fork do not propagate back to the baseline, and changes to the baseline do not overwrite fork customisations.

## How Forking Works

When a user forks an entity:

1. A copy is created in the user's personal workspace (`user_skills`, `user_agents`, `user_hooks` tables)
2. The copy inherits all content from the original
3. The base entity ID is preserved on the fork for origin tracking
4. The fork is independently editable — changes affect only the user's copy

## What Can Be Forked

| Entity | Source Table | Fork Table | What Transfers |
|--------|-------------|------------|----------------|
| **Skills** | `skills` | `user_skills` | Content, tags, metadata |
| **Agents** | `agents` | `user_agents` | System prompt, configuration |
| **Hooks** | `hooks` | `user_hooks` | Event triggers, actions |
| **Plugins** | `plugins` | `user_plugins` | All bundled skills, agents, hooks |
| **MCP Servers** | `mcp_servers` | *(per-user config)* | Server configuration |

## Origin Tracking

Every fork records its `base_entity_id` — the ID of the original entity it was forked from. This enables:

- **Lineage visibility:** Administrators can see the full fork tree for any entity
- **Update awareness:** Users can compare their fork against the current baseline
- **Audit trails:** The relationship between fork and original is always traceable

## Use Cases

### Department-Specific Agent Behaviour

The customer support team forks the shared customer agent to customise responses for their domain. The development team forks the same agent with different tool permissions. Each team maintains their own version while the baseline remains the organisation standard.

### Skill Localisation

A user forks a platform-wide skill to add department-specific policies or procedures. The platform skill continues to receive updates, but the fork preserves the user's customisations independently.

### Personal Workspace

Users can assemble a personal workspace by forking plugins they use frequently, then customising tool configurations, adding notes, or adjusting agent instructions to match their workflow.

## Managing Forks

Forks are visible in the user's personal workspace (**My Skills**, **My Agents**, **My Hooks** in the admin interface). Each fork shows:

- The fork's current content
- The base entity it was forked from
- When the fork was created and last modified

Administrators with admin bypass can see all user forks across the organisation, regardless of access control rules.

## Fork vs. Original Updates

Forks are independent copies. When the original entity is updated:

- The fork is **not** automatically updated
- Users can manually re-fork to get the latest baseline
- The origin tracking link is preserved even after re-forking

This design ensures that customisations are never unexpectedly overwritten by upstream changes.
