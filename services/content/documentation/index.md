---
title: "Get Started with the AI Salesforce Gateway"
description: "Start here. Two paths: setting the Gateway up for your organization, or using it day to day in Salesforce. Pick the one that describes you."
author: "Astound Digital"
slug: ""
keywords: "get started, how to use, getting started, salesforce ai, desktop bridge, admin setup, dashboard, authentication"
kind: "guide"
public: true
tags: ["documentation", "getting-started"]
published_at: "2026-02-18"
updated_at: "2026-08-19"
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

There are two ways to arrive here. Pick yours.

## I need to set this up for my organization

**[Standing Up the Gateway →](/documentation/use-case-admin)**

You own the Salesforce org and the deployment. You will create a Salesforce app,
point the repository at your org, apply the spec, and hand the bridge to your
team.

*Prerequisites: Salesforce admin access and shell access to the deployment.
About an hour, once per organization.*

## I am a developer running this locally

**[Connect Claude Code →](/documentation/connect-claude-code)**

Clone, build, start the gateway, register at `/admin/login`, then one command
with the one-shot code from your profile page. Includes the clean-state
verification procedure for the connect path.

*Prerequisites: Docker, just, a Rust toolchain, and one provider API key.*

## I want to use Salesforce in plain English

**[Your Salesforce Day →](/documentation/use-case-salesforce-user)**

You work in Salesforce — sales, service, account management. You will install
the bridge, sign in with your normal Salesforce credentials, and start asking.

*Prerequisites: your Salesforce login, and someone to have done the path above.
About two minutes.*

## The Short Version

For a user, getting connected is three steps and no account creation:

1. **Install the desktop bridge.** Downloads are on the [homepage](/). It runs
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

## Reference

**Setting up the Salesforce connector** — the five steps in full:

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

**Setting up your team:**

- [Connect Claude Code](/documentation/connect-claude-code) — from an empty machine to a gateway-routed Claude Code session
- [Install the Desktop Bridge](/documentation/bridge-install) — Windows, macOS, Linux, and WSL
- [Create & Manage Users](/documentation/admin-user-management) — admin CLI, bulk creation, roles, and passkey registration
- [Expose Your Instance Remotely](/documentation/remote-access) — take the gateway from `127.0.0.1` to a public HTTPS URL
