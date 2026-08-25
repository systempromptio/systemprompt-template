---
title: "MCP, Tool Governance & Distribution"
description: "Govern every tool call with a four-stage pre-execution chain, run a declarative MCP server registry with instant revocation, and ship signed skills and plugins."
author: "Astound Digital"
slug: "enterprise-tool-governance"
keywords: "mcp, tools, governance, registry, revocation, signing, plugins, skills, blocklist"
kind: "guide"
public: true
tags: ["enterprise", "mcp", "admin"]
published_at: "2026-08-25"
updated_at: "2026-08-25"
after_reading_this:
  - "Trace a tool call through the four-stage pre-execution governance chain"
  - "Administer MCP servers, auth scopes, and entitlements at /admin/catalog/mcp"
  - "Revoke a server or a token and see it take effect immediately"
  - "Distribute skills and plugins through the Ed25519-signed catalog"
related_docs:
  - title: "Content Safety, PII & Guardrails"
    url: "/documentation/enterprise-safety-guardrails"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# MCP, Tool Governance & Distribution

**TL;DR:** Every MCP tool call passes a synchronous four-stage chain — scope check, secret scan, blocklist, rate limit — first-deny-wins, before the tool runs, with every decision audited under a trace id. MCP servers live in a declarative registry at `/admin/catalog/mcp` with per-server auth, plan entitlement, and immediate revocation; skills and plugins ship through Ed25519-signed manifests.

## Pre-execution governance

The chain runs **before the tool executes**, synchronously, on every call:

1. **Scope check** — does the caller's token carry the scope and audience this server demands?
2. **Secret scan** — do the tool arguments carry credentials that must not leave the boundary?
3. **Blocklist** — is this tool or pattern explicitly denied?
4. **Rate limit** — is the caller within their quota window?

Evaluation is **first-deny-wins**: the first failing stage stops the call, and the tool never runs. Every decision — allow or deny — is audited with trace linkage, so a denied call is as visible as an executed one:

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace show <trace-id>
```

The chain's content-safety companion (what the secret and PII scanners actually detect) is covered in [Content Safety, PII & Guardrails](/documentation/enterprise-safety-guardrails).

## The MCP server registry

`/admin/catalog/mcp` administers the declarative registry. Each server declares:

- **Auth requirements** — the token audience and scopes a caller must hold (for example, audience `mcp` with scope `admin` for the administrative server).
- **Plan entitlement** — which organization plans include the server, so access follows the commercial agreement automatically.
- **Revocation** — a server can be disabled with immediate effect, and **JTI-based token revocation** kills individual issued tokens without waiting for expiry.

## Fail-fast schema validation

Registration validates up front: a server with a **missing or invalid manifest**, or whose declared schema fails to sync to the database, is **refused at registration** rather than discovered broken at call time. Tool input schemas are captured at discovery, so what a tool accepts is on record. (One caveat: captured schemas are not meta-validated against the JSON-Schema spec itself — see the [roadmap](/documentation/enterprise-roadmap).)

## Signed distribution of skills and plugins

Skills and plugins are distributed through `/admin/catalog/plugins` and `/admin/catalog/skills`, carried by **Ed25519-signed manifests**: a client verifies the signature before installing, so nothing unsigned or tampered-with reaches a workstation, and central revocation removes an artifact from circulation immediately.

Signed manifests can carry prompt content, but there is no first-class **versioned prompt-template object** with parameters, pinning, and rollback yet — that lifecycle is on the [Enterprise Roadmap](/documentation/enterprise-roadmap).

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| REQ | What the test proves | Replicate with |
|---|---|---|
| REQ-040 | A signed skill/plugin installs after signature verification and disappears from circulation on central revocation | `just e2e-req REQ-040` |
| REQ-041 | Each of the four chain stages denies its case pre-execution, first-deny-wins, with the decision audited under a trace id | `just e2e-req REQ-041` |
| REQ-042 | The registry enforces per-server auth/scopes/audience and plan entitlement, and JTI revocation kills an issued token | `just e2e-req REQ-042` |
| REQ-043 | Registration of a server with a missing or invalid manifest fails fast; tool schemas are captured at discovery | `just e2e-req REQ-043` |

These REQs' Playwright specs live under `playwright/tests/requirements/`, with gateway-level proofs in the `req_040_*`–`req_043_*` Rust modules under `tests/integration/` (run with `just test-integration`). No screenshots exist for them yet — the pack grows with `just e2e-screens`.
