---
title: "Governance Pipeline Overview"
description: "How pre-tool-use and post-tool-use hooks intercept every tool call for inspection, denial, and tracking."
author: "systemprompt.io"
published_at: "2026-04-01T09:00:00Z"
slug: "governance-pipeline"
keywords: "governance, hooks, pipeline, policy"
kind: "guide"
image: "/files/images/logo.png"
category: "documentation"
tags:
  - governance
  - reference
public: true
---

# Governance Pipeline Overview

Every tool call inside the Enterprise Demo flows through two governance
hooks. The pre-tool-use hook validates the call against scope, secret
detection, and destructive-command blocklists. If any check fails, the
call is denied before it reaches the tool runtime.

The post-tool-use hook runs after a successful tool call and records
latency, outcome, and the originating session. This data feeds the
analytics dashboards under `analytics/costs` and `analytics/tools`.

## Scope resolution

Scopes are resolved from the `agents/` services directory on every
hook invocation. The resolver reads each agent YAML, pulls the first
entry under `oauth.scopes`, and falls back to `card.security.oauth2`
when no explicit scope list is present.
