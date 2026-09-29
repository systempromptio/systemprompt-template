---
title: "MCP, Tool Governance & Distribution"
description: "Govern integrated tool calls with a four-stage pre-execution chain, run a declarative MCP server registry with instant revocation, and ship signed skills and plugins."
author: "systemprompt.io"
slug: "enterprise-tool-governance"
keywords: "mcp, tools, governance, registry, revocation, signing, plugins, skills, blocklist, warn mode"
kind: "guide"
public: true
tags: ["enterprise", "mcp", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-28"
after_reading_this:
  - "Trace a tool call through the four-stage pre-execution governance chain"
  - "Administer MCP servers, auth scopes, and entitlements at /admin/mcp"
  - "Run a stage in warn mode to measure what enforcement would cost before turning it on"
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

**TL;DR:** MCP tool calls routed through platform governance pass a synchronous four-stage chain — scope check, secret scan, blocklist, rate limit — before the tool runs, with every decision audited under a trace id. The shipped stages run in warn mode, recording findings without refusing calls. An enforcing stage stops dispatch on denial. Authentication and authorization are still enforced. MCP servers live in a declarative registry at `/admin/mcp` with per-server auth, role, group and project entitlement, and immediate revocation; skills and plugins ship through Ed25519-signed manifests.

## Pre-execution governance

The chain runs **before the tool executes**, synchronously, on governed calls. Client-local actions require the appropriate bridge/plugin hooks; merely routing model inference does not govern every local tool. The stages evaluate:

1. **Scope check** — does the caller's token carry the scope this tool demands? The shipped config marks every `mcp__systemprompt__` tool admin-only.
2. **Secret scan** — do the tool arguments carry credentials that must not leave the boundary? The shipped list has 32 explicit signatures.
3. **Blocklist** — is this tool or pattern explicitly denied? The shipped patterns are `delete`, `drop` and `destroy`.
4. **Rate limit** — is the caller within their rate (300 calls per 60 seconds as shipped)?

In enforce mode, evaluation is **first-deny-wins**: the first denying stage stops the call. Warn-mode findings are recorded and execution continues. Every decision — allow or deny — is audited with trace linkage, so a denied call is as visible as an executed one:

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace show <trace-id>
```

The chain's content-safety companion (what the secret and PII scanners actually detect) is covered in [Content Safety, PII & Guardrails](/documentation/enterprise-safety-guardrails).

## Warn mode

A stage that is misfiring leaves you two bad options. Retune a threshold you have no data for, or switch the stage off and lose the evidence along with the denials. Warn mode is the third: the stage runs, evaluates, and records exactly what it would have refused, and the call proceeds.

Set it per policy, or once for the whole chain, in `services/governance/config.yaml`. The template ships `mode: warn` at the top level with all four stages enabled; the snippet below shows the override shape:

```yaml
governance:
  mode: warn            # the default for every policy below
  policies:
  - id: secret_scan
    enabled: true
  - id: scope_check
    enabled: true
    mode: enforce       # this one still refuses
```

`mode` is inherited: the top-level value applies to any policy that does not name its own. An unrecognised value fails the boot job rather than falling back, because reading `warnn` as `enforce` would block traffic you believed you had unblocked, and reading it as `warn` would disable enforcement nobody asked to disable.

Warn mode is **not** the same as `enabled: false`. A disabled stage does not run and writes nothing. A warning stage runs and writes a `governance_decisions` row with `decision = 'warn'` carrying the reason it would have denied on, so a warn row and a deny row are directly comparable. It also does not halt the chain, so a call that trips three warning stages is recorded against all three rather than only the first.

The gateway safety scanners are a separate plane with its own switch, `safety.mode: warn` in `services/gateway/policies.yaml`. It leaves every scanner running and every finding persisted, and only drops the refusal — findings written under it carry `blocked = false`, which is what lets a report say how many calls a block list would have cost.

Read both planes back over one window:

```bash
systemprompt infra logs governance report --since 7d
systemprompt infra logs governance report --since 7d --group-by tool
systemprompt infra logs governance report --group-by user --format csv > warnings.csv
systemprompt infra logs trace list --decision warn --since 24h
```

The same data is on the governance dashboard at `/admin/governance` — a **Decisions** tab for the chain, a **Safety** tab for the scanners, and a **Hooks** tab — with a KPI strip spanning both planes and a CSV export at `/admin/governance/warnings.csv`. The two numbers to read there are a category's **findings** count beside its **blocked** count: many findings and no blocks is a rule warn mode is currently absorbing, and no findings at all is a scanner earning nothing.

Warn mode is a measurement window, not a posture. Once the report shows which tunables are wrong, fix those and return the stages that no longer misfire to `mode: enforce`, one at a time.

## The MCP server registry

`/admin/mcp` lists the declarative registry (`services/mcp/*.yaml`). Each server declares:

- **Auth requirements** — the token audience and scopes a caller must hold (for example, audience `mcp` with scope `admin` for the administrative server).
- **Entitlement** — which roles, groups or projects may reach the server, declared as `mcp_server/<id>` rules in `services/access-control/rules.yaml` (the shipped `systemprompt` server is open to the `user` role, and its tools are gated a second time by the admin scope). When SSO maps directory groups into groups and projects, access follows the directory.
- **Revocation** — a server can be disabled with immediate effect, and **JTI-based token revocation** kills individual issued tokens without waiting for expiry.

## Fail-fast schema validation

Registration validates up front: a server with a **missing or invalid manifest**, or whose declared schema fails to sync to the database, is **refused at registration** rather than discovered broken at call time. Tool input schemas are captured at discovery, so what a tool accepts is on record. (One caveat: captured schemas are not meta-validated against the JSON-Schema spec itself — see the [roadmap](/documentation/enterprise-roadmap).)

## Signed distribution of skills and plugins

Skills and plugins are listed at `/admin/plugins` and `/admin/skills` and distributed through marketplaces, carried by **Ed25519-signed manifests**: a client verifies the signature before installing, so nothing unsigned or tampered-with reaches a workstation, and central revocation removes an artifact from circulation immediately.

Signed manifests can carry prompt content, but there is no first-class **versioned prompt-template object** with parameters, pinning, and rollback yet — that lifecycle is on the [Enterprise Roadmap](/documentation/enterprise-roadmap).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured.
