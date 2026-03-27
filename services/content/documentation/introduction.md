---
title: "Introduction to the Platform"
description: "What this site is, how the white-label demo becomes a licensed production deployment, and how to navigate the documentation."
author: "systemprompt.io"
slug: "introduction"
keywords: "introduction, enterprise demo, white label, systemprompt, AI governance, living documentation, licensing"
kind: "guide"
public: true
tags: ["introduction", "enterprise", "overview"]
published_at: "2026-03-27"
updated_at: "2026-03-27"
after_reading_this:
  - "Understand what this site is and how it relates to a production deployment"
  - "Know how the engagement model turns a demo into a licensed product"
  - "Navigate to the right page for your area of interest"
related_docs:
  - title: "Platform Overview"
    url: "/documentation/platform-overview"
  - title: "Anthropic Partnership"
    url: "/documentation/anthropic-partnership"
  - title: "Integration: Claude Code"
    url: "/documentation/integration-claude-code"
  - title: "Integration: Claude Cowork"
    url: "/documentation/integration-claude-cowork"
  - title: "Integration: Claude AI"
    url: "/documentation/integration-claude-ai"
  - title: "Partnership Proposal"
    url: "/documentation/proposal"
---

# Introduction to the Platform

**This is a live enterprise demo of the systemprompt.io AI governance platform. It is not a mockup or a slide deck. Every page you are reading, every API endpoint, every governance decision, and every analytics dashboard on this site is running on the same codebase that would be deployed in your production environment. This is enterprise AI governance software — a narrow waist between your AI deployments and your company data — built on Rust, designed for on-premise deployment, and fully integrated with the Anthropic Claude ecosystem.**

---

## What You Are Looking At

This site is a white-label implementation of the systemprompt.io governance platform, configured and branded for Foodles. The platform is running. The governance pipeline is active. The documentation you are reading is served by the same content system that manages skills, plugins, and agent configurations.

This is how the platform works in practice — not a demo environment with synthetic data, but the actual software operating as it would inside your organisation.

## How the Codebase Is Structured

The platform is built from three distinct layers. Understanding this structure is important because it defines what you own, what is licensed, and what you can customise.

```
┌─────────────────────────────────────────────────────────────┐
│  Services (YAML & Markdown)                                  │
│  Configuration you control: agents, skills, plugins,         │
│  content, profiles, branding. No code — just config.         │
├─────────────────────────────────────────────────────────────┤
│  Extensions (Rust code)                                      │
│  Custom functionality built on top of Core: web dashboard,   │
│  MCP management, CLI, and your own extensions.               │
│  THIS IS YOUR IP after engagement.                           │
├─────────────────────────────────────────────────────────────┤
│  Core (Rust library — git submodule)                         │
│  The underlying platform IP: governance primitives, access   │
│  control, analytics, hooks, job scheduling, extension        │
│  system. Licensed for production use.                        │
└─────────────────────────────────────────────────────────────┘
```

### Core — Licensed IP

The core is a Rust library provided as a git submodule. It contains all the governance primitives: access control, analytics, hooks, job scheduling, content management, and the extension system itself. Core is **read-only** — you never modify it directly. It is licensed for production use as part of the engagement. See [Licensing](/documentation/proposal-licensing) for terms.

### Extensions — Your IP

Extensions are Rust code built on top of Core. This is where the platform's functionality lives: the web dashboard, MCP server management, the CLI, and any custom functionality specific to your organisation. **After engagement, the extension code is yours.** You own it, you can modify it, and you can extend it. This is the layer where the platform is customised to your specific requirements.

### Services — Your Configuration

The services layer is YAML and Markdown only — no Rust code. This is where you configure agents, skills, plugins, content, deployment profiles, and branding. Your team can modify services without touching any Rust code, making day-to-day configuration changes straightforward.

### What This Means in Practice

- **This codebase is yours.** On engagement, this very repository is adapted to your organisation — your branding, your agents, your governance policies, your documentation.
- **Living documentation.** The documentation you are reading now becomes your internal documentation. As the platform is customised during engagement, the docs evolve with it.
- **Full source access.** The extension code (Rust) and the configuration (YAML/Markdown) are part of the deliverable. You own and control the code that runs in your infrastructure.
- **Core is licensed, extensions are owned.** The core library is licensed for production use. The extension and services layers are yours to modify and extend without restriction.

For the full technical architecture, see [Platform Architecture](/documentation/architecture).

## This Is an Evaluation Shell

What you are looking at is an **enterprise demo extension** — a generic template that is shared with prospective clients and then personalised for each installation. It is designed to give a comprehensive overview of the platform's capabilities, but as a template there are parts that may not be fully adapted or fully operational for your specific context.

**Please treat this as an evaluation environment.** If something catches your interest — a specific governance feature, an integration surface, a technical capability — the team can double-click on it and provide a deep dive. If there is a feature that appears incomplete or generic, that is expected for a demo extension. The production engagement customises everything to your requirements.

- **Want to see something in more detail?** Contact the team and we can walk through it live.
- **Interested in a specific technical surface?** We can deep-dive into the architecture, the extension code, or the integration layer.
- **Found something that does not look right?** It may be part of the generic template that has not been adapted yet. Flag it and we will clarify.

## What the Software Does

This is governance software for enterprise AI deployments. The platform acts as the **narrow waist** between all AI clients (Claude Code, Claude Cowork, claude.ai, custom agents) and all backend services (LLM providers, MCP servers, databases, APIs). All use of AI flows through this system.

The platform provides three things:

1. **Insight** — Know what people are doing with AI across your organisation. Activity tracking, cost tracking, analytics dashboards, and engagement metrics.
2. **Governance** — Business logic to control AI usage. Access control, tool governance, hooks, rate limiting, audit trails, and compliance reporting.
3. **Integration** — Full integration into the surfaces where AI is used, primarily the Claude ecosystem. Plugin distribution, marketplace management, and MCP server governance.

Read the full breakdown in [Platform Overview](/documentation/platform-overview).

## Built for the Anthropic Ecosystem

systemprompt.io is an Anthropic partner. The platform is built to work with Claude, not against it. Native integration with Claude Code's plugin system, the Model Context Protocol (MCP) as the tool interface standard, and alignment with Anthropic's safety and governance goals.

Read more in [Anthropic Partnership](/documentation/anthropic-partnership).

> **Note:** Claude Cowork is currently in research preview with some features temporarily restricted by Anthropic. See [Integration: Claude Cowork](/documentation/integration-claude-cowork) for current status. [Claude Code integration](/documentation/integration-claude-code) is fully operational.

## User Identification and Authentication

The platform maintains a full user registry with WebAuthn passkey-based authentication. This means:

- **Passwordless login** — Users authenticate with passkeys (biometrics, hardware keys) via the WebAuthn standard
- **Shared identity** — The same authentication is used across all platform surfaces: the admin dashboard, MCP servers, AI agents, and the API
- **Third-party IDP integration** — The OAuth 2.0 + PKCE authentication flow can be fully integrated with your existing identity provider (Okta, Azure AD, Google Workspace, or any OIDC-compliant provider)
- **Single source of truth** — User roles, departments, and permissions are managed centrally and enforced consistently across all AI interactions

See [Authentication](/documentation/authentication) for the full technical flow.

## What to Read Next

| If you want to... | Read |
|-------------------|------|
| Understand what the software does in detail | [Platform Overview](/documentation/platform-overview) |
| Learn about the Anthropic partnership | [Anthropic Partnership](/documentation/anthropic-partnership) |
| See how it integrates with Claude Code | [Integration: Claude Code](/documentation/integration-claude-code) |
| See how it integrates with Claude Cowork | [Integration: Claude Cowork](/documentation/integration-claude-cowork) (Research Preview) |
| See how it integrates with claude.ai | [Integration: Claude AI](/documentation/integration-claude-ai) |
| Review the engagement model and pricing | [Partnership Proposal](/documentation/proposal) |
| Read the full technical architecture | [Architecture Overview](/documentation/architecture) |

## From Demo to Production

The engagement follows a two-phase model. Phase 1 is a collaborative PRD and development process — the platform is customised to your requirements on systemprompt cloud infrastructure. Phase 2 deploys the platform on your infrastructure with a production license and ongoing support.

Phase 2 is conditional on Phase 1 sign-off. There is no production commitment until both sides agree the platform meets the agreed requirements.

Read the full details in [Partnership Proposal](/documentation/proposal) and [Engagement Summary](/documentation/engagement-summary).
