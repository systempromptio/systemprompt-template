---
title: "Enterprise Roadmap & Known Limitations"
description: "What is coming next — semantic caching, A/B model testing, prompt versioning — and one honest consolidated table of known limitations."
author: "systemprompt.io"
slug: "enterprise-roadmap"
keywords: "roadmap, limitations, semantic caching, ab testing, prompt versioning, scim"
kind: "guide"
public: true
tags: ["enterprise", "roadmap", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-28"
after_reading_this:
  - "Know which enterprise capabilities are planned but not yet available"
  - "Check the consolidated known-limitations table before relying on a feature"
  - "Understand the current state and interim workaround for each gap"
  - "Find the delivered walkthrough page for everything that already ships"
related_docs:
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "User & Access Management"
    url: "/documentation/enterprise-user-access"
---

# Enterprise Roadmap & Known Limitations

**TL;DR:** This page is the single honest home for what is *not* yet available. Three larger capabilities are coming — tenant-isolated semantic caching, A/B model testing, and prompt versioning with rollback — and a consolidated table lists every other known limitation with its current state. This list is not exhaustive; consult each capability's guide and deployment configuration before relying on it.

## Coming soon

### Tenant-isolated semantic caching

**What it will do:** serve semantically similar requests from a cache instead of re-billing the provider, with strict isolation so one project's cached responses can never leak into another's.

**Current state:** no semantic cache exists in the gateway today. Building one needs embedding infrastructure, similarity thresholds, and a tenant-isolation proof — a project in its own right.

**Honest note:** not yet available; today every request goes to the provider (provider-side prompt caching still applies where the provider offers it).

### A/B model testing

**What it will do:** split live traffic on a route between models by percentage, with stable experiment assignment, so model changes can be evaluated on real usage before a full switch.

**Current state:** route resolution is strictly first-match; no percentage split or experiment-assignment machinery exists.

**Honest note:** not yet available; comparisons today are done by switching routes for a cohort (for example, one project) and comparing the [cost and usage reports](/documentation/enterprise-cost-management).

### Prompt versioning & rollback

**What it will do:** a first-class prompt-template registry with versions, parameters, pinning, and one-click rollback, distributed through the same signed catalog as skills and plugins.

**Current state:** no prompt registry exists; the nearest primitive (system-prompt overrides) keeps no version history. The signed distribution channel described in [Tool Governance](/documentation/enterprise-tool-governance) can carry prompt content, so the transport exists — the lifecycle object does not.

**Honest note:** not yet available; prompt content today is versioned the way the rest of your configuration is — in git.

### Enterprise knowledge platform (RAG)

**What it will do:** governed retrieval over enterprise knowledge — issue trackers, wikis, and source control — with source-system permissions respected at retrieval time.

**Current state:** not shipped. Knowledge sources are reached today through their own MCP servers, when a deployment connects them, under the same governance as any other tool.

**Honest note:** not yet available.

### ADFS SSO

**What it does:** authenticates through an organisation's AD FS, with Active Directory groups mapping to platform roles, groups and projects.

**Current state:** shipped as an optional login mechanism beside the default WebAuthn passkey sign-in. The SAML 2.0 relying party against AD FS and the `group` and `project` access dimensions are in the tree. It is off until an installation adds `services/web/config/adfs.yaml` (the template ships none) with its IdP metadata and group→role map. Trust is pinned to the IdP's published signing certificate — there is no client secret.

### OpenAI-compatible IDE endpoint

**What it will do:** an OpenAI-compatible chat-completions endpoint so IDE clients such as OpenCode route traffic through platform governance to approved models, with per-role model access and monthly budgets.

**Current state:** shipped for OpenCode. The gateway serves `/v1/chat/completions`, and the bridge registers it as an OpenCode provider — see [Connect OpenCode](/documentation/connect-opencode). Monthly budgets are configuration rather than code (a calendar-month quota window), but the shipped quota windows run in warn mode and none is monthly.

## Known limitations

| Limitation | Current state | Where it is discussed |
|---|---|---|
| **SCIM provisioning** | Deferred: the supported SSO path, AD FS, does not push standards-based SCIM, so an endpoint would have no caller. Revisit if an IdP such as Okta or Entra fronts the instance. | [User & Access Management](/documentation/enterprise-user-access) |
| **WORM audit immutability** | Audit rows are append-only by convention, not mechanism — no WORM storage or hash-chaining. Planned hardening options: revoked UPDATE/DELETE grants, hash-chaining, or export to WORM storage. | [Audit & Observability](/documentation/enterprise-audit-observability) |
| **PHI taxonomy** | PII scanning covers email, credit card, SSN, and phone; health-identifier categories are not yet in the set. | [Content Safety & Guardrails](/documentation/enterprise-safety-guardrails) |
| **Per-key budgets** | An API key or PAT inherits its owner's scope — it is not its own governance subject, so no per-key budget, rate, or model scope. | [Model Gateway & Routing](/documentation/enterprise-model-routing) |
| **Tab-acceptance metric** | Not currently measurable: Claude Code emits no accept/reject signal and no manual-LOC baseline exists. Needs an IDE-level integration. | [Analytics](/documentation/enterprise-analytics) |
| **Per-project budgets** | The shipped spend thresholds warn per user and instance-wide. A `project` quota window with a calendar-month period can be declared, but none ships, and a hard cap needs `quota_mode: enforce`. | [Cost Management](/documentation/enterprise-cost-management) |

## Checking availability

Roadmap entries describe gaps and dependencies, not release commitments. Verify configured behavior with the relevant guide: [User & Access Management](/documentation/enterprise-user-access), [Analytics](/documentation/enterprise-analytics), [Cost Management](/documentation/enterprise-cost-management), [Model Gateway & Routing](/documentation/enterprise-model-routing), [Audit & Observability](/documentation/enterprise-audit-observability), [Safety & Guardrails](/documentation/enterprise-safety-guardrails), and [Tool Governance](/documentation/enterprise-tool-governance).
