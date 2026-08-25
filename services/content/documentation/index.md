---
title: "Get Started with the AI Salesforce Gateway"
description: "Start here. Two ways to get connected: Claude Code in your terminal, or the desktop bridge with Claude Cowork. Advanced setup for admins and developers below."
author: "Astound Digital"
slug: ""
keywords: "get started, how to use, getting started, salesforce ai, desktop bridge, admin setup, dashboard, authentication"
kind: "guide"
public: true
tags: ["documentation", "getting-started"]
published_at: "2026-02-18"
updated_at: "2026-08-25"
after_reading_this:
  - "Know which of the two setup paths applies to you"
  - "Get connected and ask your first question"
  - "Know which skills to reach for and how to ask"
  - "Find the reference documentation for each step"
---

# Get Started

The AI Salesforce Gateway puts your Salesforce inside Claude. You ask in plain
English. It answers from live data, confirms with you before it changes
anything, and records everything it does.

One distinction decides which docs you need. **Users** — everyone who works
with Claude through the gateway, including developers who build *with* it —
install a small client and sign in. No repository, no Rust toolchain, no
server: the gateway runs hosted, and the two paths below are the whole setup.
**Maintainers** — the people changing this codebase or operating the platform —
are the only ones who ever clone and build; their docs are under
[Advanced](#advanced).

Two ways for users to get connected. Pick yours.

## I want Claude Code in my terminal

**[Connect Claude Code →](/documentation/connect-claude-code)**

One command: get a connect code from your profile page, run the installer, and
`claude` works against the hosted gateway with your organization's skills and
plugins already synced. No checkout, no build. This is also the path for
developers who want to build with the gateway's capabilities — the ready-made
binary and install script cover it.

*Prerequisites: an account on the instance. About two minutes.*

## I'm on Windows and want Claude Cowork

**[Install the Desktop Bridge →](/documentation/bridge-install)**

Download the bridge, sign in with your normal Salesforce credentials, and start
asking. It runs in your system tray and keeps your skills in sync inside Claude
Cowork and Claude Code. macOS and Linux are covered on the same page.

*Prerequisites: your Salesforce login, and an admin to have set up the gateway.
About two minutes.*

## The Short Version

For a user, getting connected is three steps and no account creation:

1. **Install the desktop bridge.** The Windows download is on the
   [homepage](/); other platforms install via the bridge script — see
   [Install the Desktop Bridge](/documentation/bridge-install). It runs
   in your menu bar or system tray and keeps your skills in sync inside Claude
   Code, Cowork, and Codex.
2. **Sign in with Salesforce.** There is no signup form and no password. Your
   Salesforce login is your account — first sign-in creates it, provided your
   admin has claimed your email domain and a seat is free. Platform operators
   are a separate case, created from the CLI with a passkey; see
   [Authentication](/documentation/authentication).
3. **Approve the device link and ask.** Try *"Give me a full briefing on my
   biggest account"* or *"What is in my pipeline this quarter?"* The skills are
   already installed.

## What to Ask

Browse the [full skills catalogue](/skills/) — every skill lists what it does and
example questions covering pipeline, accounts, contacts, leads, activities,
cases, consultancy, brand, and governance.

## Enterprise Administration

The enterprise operations handbook — nine pages covering user and access
management, organizations, analytics, cost and budget controls, model routing
and data residency, audit and observability, safety guardrails, tool
governance, and the roadmap. Every page ends with its verified evidence: the
tagged end-to-end tests and screenshots that prove the capability, and the
commands to replicate them.

- [User & Access Management](/documentation/enterprise-user-access)
- [Organizations, Departments & Hubs](/documentation/enterprise-organizations)
- [Usage, Adoption & Productivity Analytics](/documentation/enterprise-analytics)
- [Cost Management, Budgets & FinOps](/documentation/enterprise-cost-management)
- [Model Gateway, Routing & Data Residency](/documentation/enterprise-model-routing)
- [Audit Trail, Traceability & Observability](/documentation/enterprise-audit-observability)
- [Content Safety, PII & Guardrails](/documentation/enterprise-safety-guardrails)
- [MCP, Tool Governance & Distribution](/documentation/enterprise-tool-governance)
- [Enterprise Roadmap & Known Limitations](/documentation/enterprise-roadmap)

## Advanced

Everything below is for admins standing the gateway up for an organization,
operators running the platform, and maintainers building it from source. If
you just want to use it — even to build applications with it — the two user
paths above are all you need, and nothing here applies.

**Setting up for your organization (admins):**

- [Standing Up the Gateway](/documentation/use-case-admin) — the admin journey:
  create the Salesforce app, apply the spec, hand the bridge to your team.
  *About an hour, once per organization.*
- [Overview](/documentation/salesforce) — the trust chain and what is configured where
- [1. Salesforce App Setup](/documentation/salesforce-app-setup)
- [2. JWT-Bearer & Certificate](/documentation/salesforce-jwt-bearer)
- [3. Hosted MCP Access](/documentation/salesforce-hosted-mcp)
- [4. Users, Seats & Roles](/documentation/salesforce-provisioning)
- [5. Rolling Out the Bridge](/documentation/salesforce-bridge-rollout)

**Running the platform:**

- [Authentication](/documentation/authentication) — Salesforce SSO, operator passkeys, sessions and route protection
- [Dashboard Usage](/documentation/dashboard) — real-time metrics, activity feed, and health indicators
- [Gateway API](/documentation/gateway-api) — the `/v1/messages` endpoint and its governance

**Developing and deploying:**

- [Develop Against a Local Gateway](/documentation/develop-claude-code) — clone, build the binary from source, and verify the connect path from a clean state
- [Create & Manage Users](/documentation/admin-user-management) — admin CLI, bulk creation, roles, and passkey registration
- [Expose Your Instance Remotely](/documentation/remote-access) — take the gateway from `127.0.0.1` to a public HTTPS URL
