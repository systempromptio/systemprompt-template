---
title: "Anthropic Partnership"
description: "systemprompt.io is built to extend and govern the Claude ecosystem — working with Anthropic's tools and protocols, not around them. We have applied for the Anthropic partner programme and are actively engaging."
author: "systemprompt.io"
slug: "anthropic-partnership"
keywords: "Anthropic, partnership, Claude, ecosystem, MCP, Model Context Protocol, governance, enterprise"
kind: "guide"
public: true
tags: ["introduction", "partnership", "anthropic"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Understand systemprompt.io's relationship with Anthropic and the current state of the partnership"
  - "Know how the platform is built with the Claude ecosystem, not around it"
  - "Understand what this alignment means for enterprise customers"
related_docs:
  - title: "Introduction to the Platform"
    url: "/documentation/introduction"
  - title: "Platform Overview"
    url: "/documentation/platform-overview"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Distribution Channels"
    url: "/documentation/distribution-channels"
  - title: "Plugins"
    url: "/documentation/plugins"
---

# Anthropic Partnership

**systemprompt.io is purpose-built to govern and extend the Claude ecosystem — Claude Code, Claude Cowork, and Claude AI. This is not a third-party tool that wraps or replaces Anthropic's products. It is governance infrastructure that makes Claude deployments enterprise-ready, built in alignment with Anthropic's protocols, plugin systems, and safety goals.**

---

## Our Approach: Build With Anthropic, Not Around It

We have applied to the Anthropic partner programme and are actively engaging with their team. Our goal is straightforward: build **with** Anthropic, not around them.

Anthropic is growing extraordinarily fast — they are scaling their team, their products, and their partner ecosystem simultaneously. Formal partnership takes time, and we respect that. In the meantime, we are building on every public protocol and integration point Anthropic provides, ensuring that when the partnership formalises, the platform is already fully aligned.

This is not a wait-and-see position. We are shipping production governance infrastructure today, built entirely on Anthropic's own standards:

- The platform uses Anthropic's **Model Context Protocol (MCP)** as the standard interface for tool access. MCP servers are first-class citizens, not an afterthought.
- Plugin distribution uses **Claude Code's native plugin system**. Developers install governed plugins the same way they install any Claude Code plugin — no proprietary client, no custom protocol.
- The marketplace distributes content through **standard git repositories**, compatible with Claude Code and any git-compatible tooling.
- Governance policies are enforced **at the protocol level**, so they apply regardless of which Claude surface the user is working in.

---

## The Principle: Govern, Don't Replace

Enterprise organisations adopting Claude need governance — access control, audit trails, cost tracking, tool management. But they do not need another AI platform competing with the one they have chosen. The governance layer should extend and strengthen the Claude ecosystem, not create an alternative to it.

## What This Means for Enterprise Customers

When you adopt the systemprompt.io governance platform, you are not introducing a competing system alongside Claude. You are adding a governance layer that is:

**Aligned with your AI provider.** The platform is built on Anthropic's public protocols and integration points, with deep knowledge of Claude's architecture and roadmap. Governance capabilities are designed to complement Claude's own safety features, not duplicate or override them.

**Protocol-native.** MCP is the tool integration standard used by Claude Code, Claude Cowork, and claude.ai. The platform governs MCP tool calls natively — every tool invocation passes through the governance pipeline with full authentication, authorisation, and audit logging.

**Forward-compatible.** As Anthropic evolves Claude's capabilities and surfaces, the governance platform evolves with them. Plugin formats, protocol versions, and integration patterns are maintained in alignment with Anthropic's releases. This means updates to Claude do not break your governance — they are supported by it.

**Not a lock-in risk.** The platform governs AI usage; it does not host AI models or proxy AI requests. Your relationship with Anthropic is direct. The governance layer adds control and visibility without inserting itself between you and your AI provider.

## Integration Points

The platform integrates with three Claude surfaces, each through Anthropic's own integration mechanisms:

| Surface | Integration Mechanism | What It Provides |
|---------|----------------------|-----------------|
| **Claude Code** | Native plugin system | Governed skills, agents, and tools distributed to developers via personalised marketplaces |
| **Claude Cowork** | MCP + plugin system | Same governance pipeline for collaborative AI sessions |
| **Claude AI** | MCP servers with OAuth | Governed tool access from claude.ai with unified authentication |

Each integration is documented in detail:

- [Integration: Claude Code](/documentation/integration-claude-code) — Plugin system and marketplace distribution
- [Integration: Claude Cowork](/documentation/integration-claude-cowork) — Governance for collaborative sessions
- [Integration: Claude AI](/documentation/integration-claude-ai) — MCP servers bridging claude.ai to governed tools

## The MCP Standard

The Model Context Protocol (MCP) is central to how the platform operates. MCP defines how AI clients discover and invoke tools — and the governance platform sits at this boundary.

Every MCP tool call in your organisation passes through the governance layer. This means:

- **Per-tool access control** — Define which roles and departments can use which tools
- **Execution logging** — Every tool invocation is recorded with full context
- **Event hooks** — Trigger automation when specific tools are called
- **OAuth per server** — Each MCP server has its own authentication, managed centrally

See [MCP Servers](/documentation/mcp-servers) for configuration details and [Tool Governance](/documentation/tool-governance) for access control and audit capabilities.
