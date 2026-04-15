---
title: "Launching Enterprise Governance for AI Agents"
description: "How the Enterprise Demo shows policy hooks, scope restriction, and secret detection working together to keep AI agents inside the rails."
author: "systemprompt.io"
published_at: "2026-04-01T09:00:00Z"
slug: "governance-launch"
keywords: "governance, ai, policy, hooks, enterprise"
kind: "blog"
image: "/files/images/logo.png"
category: "blog"
tags:
  - governance
  - launch
  - enterprise
public: true
---

# Launching Enterprise Governance for AI Agents

The Enterprise Demo ships with a policy pipeline that blocks dangerous tool
calls before they reach the model. Pre-tool-use hooks validate every
invocation against scope, secret detection, and destructive-command
blocklists. Post-tool-use hooks record latency and outcomes so analytics can
surface trends across agents and sessions.

This post walks through the three denial classes you will see in the seed
data: `scope_restriction`, `secret_injection`, and `tool_blocklist`.
