---
title: "Enterprise Demo Documentation"
description: "Terminal demo walkthroughs and dashboard usage guide for the Enterprise Demo."
author: "systemprompt.io"
slug: ""
keywords: "enterprise-demo, terminal demo, dashboard, authentication"
kind: "guide"
public: true
tags: ["documentation"]
published_at: "2026-02-18"
updated_at: "2026-04-14"
after_reading_this:
  - "Run the terminal demo end-to-end"
  - "Log in to and navigate the admin dashboard"
---

# Enterprise Demo Documentation

This site covers two things: the **terminal demo** walkthroughs and how to **log in and use the dashboard**. Everything else lives in the code.

## Terminal Demos

*Step-by-step walkthroughs of the live terminal demo.*

- [Setup & Authentication](/documentation/demo-terminal-setup) — Bring the demo up and authenticate
- [Governance Decisions](/documentation/demo-terminal-agents) — Agents making governed tool calls
- [Audit Trails & Costs](/documentation/demo-terminal-audit) — Inspect audit logs and cost attribution
- [Governance API](/documentation/demo-terminal-governance) — Drive policy decisions from the CLI
- [MCP Access Tracking](/documentation/demo-terminal-mcp) — Watch MCP tool access in real time
- [Request Tracing & Benchmark](/documentation/demo-terminal-tracing) — Trace requests end-to-end
- [Agent Tracing](/documentation/demo-terminal-agent-tracing) — Follow a single agent's lifecycle

## Dashboard

*Log in and use the admin dashboard.*

- [Authentication & Login](/documentation/authentication) — Login, passkeys, magic links, and session management
- [Dashboard Usage](/documentation/dashboard) — Real-time metrics, activity feed, and health indicators
- [Access Control](/documentation/access-control) — Who reaches what: bands, precedence, rules.yaml and drift
- [Code ↔ Instance Sync](/documentation/services-sync) — Sources, planes, hashes and the three sync directions

## Connect a client

*Route Claude Code, Cowork or OpenCode through the gateway.*

- [Connect Claude Code](/documentation/connect-claude-code) — Terminal routing on macOS, Linux and Windows, model selection and verification
- [Connect Cowork](/documentation/connect-cowork) — Claude Desktop Cowork through the bridge on macOS and Windows
- [Connect OpenCode](/documentation/connect-opencode) — One installer flag writes the provider block and the key
- [Bridge installation](/documentation/bridge-install) — Install, sign-in and troubleshooting, including Linux and WSL
- [Install on macOS](/documentation/install-macos) and [Install on Windows](/documentation/install-windows) — Per-platform bridge setup
- [Downloads](/documentation/downloads) — Bridge files and checksum verification

## Skills and analysis

*See which skills are used, how conversations went, and what changed between versions.*

- [Skill lifecycle](/documentation/skills-lifecycle) — Authoring, discovery, signed distribution, runtime loading and measurement
- [Analysis overview](/documentation/analysis) — The analysis tabs and the end-to-end path
- [Measure which skills are used](/documentation/analysis-measure-skills)
- [Create a conversation and see it land](/documentation/analysis-test-conversation)
- [Versions: sources, generations and compare](/documentation/analysis-versions)
- [Evaluate plugins](/documentation/analysis-evaluate-plugins)

## Enterprise capabilities

*What the platform does for identity, cost, audit and safety — and what it does not yet.*

- [User and access management](/documentation/enterprise-user-access)
- [Usage and analytics](/documentation/enterprise-analytics)
- [Model routing](/documentation/enterprise-model-routing)
- [Cost management and budgets](/documentation/enterprise-cost-management)
- [Audit and observability](/documentation/enterprise-audit-observability)
- [Conversation history](/documentation/enterprise-conversation-history)
- [Safety and guardrails](/documentation/enterprise-safety-guardrails)
- [Tool governance](/documentation/enterprise-tool-governance)
- [Roadmap](/documentation/enterprise-roadmap)
