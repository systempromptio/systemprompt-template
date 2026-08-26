---
title: "Enterprise Roadmap & Known Limitations"
description: "What is coming next — semantic caching, provider failover, A/B model testing, prompt versioning — and one honest consolidated table of known limitations."
author: "Astound Digital"
slug: "enterprise-roadmap"
keywords: "roadmap, limitations, semantic caching, failover, ab testing, prompt versioning, scim"
kind: "guide"
public: true
tags: ["enterprise", "roadmap", "admin"]
published_at: "2026-08-25"
updated_at: "2026-08-25"
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

**TL;DR:** This page is the single honest home for what is *not* yet available. Four larger capabilities are coming — tenant-isolated semantic caching, automatic provider failover, A/B model testing, and prompt versioning with rollback — and a consolidated table lists every other known limitation with its current state. Everything not listed here is delivered and documented in the walkthrough pages.

## Coming soon

### Tenant-isolated semantic caching

**What it will do:** serve semantically similar requests from a cache instead of re-billing the provider, with strict tenant isolation so one organization's cached responses can never leak into another's.

**Current state:** no semantic cache exists in the gateway today. Building one needs embedding infrastructure, similarity thresholds, and a tenant-isolation proof — a project in its own right.

**Honest note:** not yet available; today every request goes to the provider (provider-side prompt caching still applies where the provider offers it).

### Automatic provider failover

**What it will do:** retry a failed or unhealthy provider on a configured secondary automatically, so a provider outage degrades latency rather than availability.

**Current state:** no failover exists in the gateway dispatch path — a route resolves to exactly one provider, and a provider error is relayed to the caller. The design needs secondary-provider fields on routes plus health- and error-triggered retry in the dispatch core.

**Honest note:** not yet available; today failover means an operator switching a route's provider, which is a configuration change.

### A/B model testing

**What it will do:** split live traffic on a route between models by percentage, with stable experiment assignment, so model changes can be evaluated on real usage before a full switch.

**Current state:** route resolution is strictly first-match; no percentage split or experiment-assignment machinery exists.

**Honest note:** not yet available; comparisons today are done by switching routes for a cohort (for example, one organization) and comparing the [cost and usage reports](/documentation/enterprise-cost-management).

### Prompt versioning & rollback

**What it will do:** a first-class prompt-template registry with versions, parameters, pinning, and one-click rollback, distributed through the same signed catalog as skills and plugins.

**Current state:** no prompt registry exists; the nearest primitive (system-prompt overrides) keeps no version history. The signed distribution channel described in [Tool Governance](/documentation/enterprise-tool-governance) can carry prompt content, so the transport exists — the lifecycle object does not.

**Honest note:** not yet available; prompt content today is versioned the way the rest of your configuration is — in git.

### Enterprise knowledge platform (RAG)

**What it will do:** governed retrieval over enterprise knowledge — Jira, Confluence, GitHub, and Salesforce — through a `knowledge-bank` MCP server, with source-system permissions respected at retrieval time.

**Current state:** the MCP server ships today (visible in the [MCP catalog](/documentation/enterprise-tool-governance), disabled by default) with its three-tool contract — `search_knowledge`, `list_knowledge_sources`, `knowledge_index_stats` — pinned by tests. It answers honestly that no retrieval backend is configured; the retrieval backend is the piece being built. [Conversation history search](/documentation/enterprise-conversation-history) is live and is the foundation the conversations-as-knowledge experience builds on.

### ADFS SSO

**What it will do:** authenticate through Astound's ADFS, with Active Directory groups mapping to platform roles and permissions.

**Current state:** the OIDC path is proven end-to-end (the Salesforce identity flow runs on the same primitives) and a generic trusted-issuer seam is ready. The protocol choice and the AD-group → role mapping rules are the inputs still needed before implementation.

### OpenAI-compatible IDE endpoint

**What it will do:** a `/openai/v1/chat/completions` endpoint so OpenCode and VS Code Copilot route IDE traffic through platform governance to approved models (including GCP Vertex AI), with per-role model access and monthly budgets.

**Current state:** the gateway already speaks the OpenAI wire on its existing endpoints; the dedicated OpenAI-path endpoint, the Vertex AI provider, and per-role monthly budget policies are the build items.

## Known limitations

| Limitation | Current state | Where it is discussed |
|---|---|---|
| **SCIM provisioning** | Deliberately deferred: neither Salesforce (as IdP) nor Odoo pushes standards-based SCIM, so an endpoint would have no caller. Revisit if Okta or Entra fronts the instance. | [User & Access Management](/documentation/enterprise-user-access) |
| **WORM audit immutability** | Audit rows are append-only by convention, not mechanism — no WORM storage or hash-chaining. Planned hardening options: revoked UPDATE/DELETE grants, hash-chaining, or export to WORM storage. | [Audit & Observability](/documentation/enterprise-audit-observability) |
| **Observability push egress** | OTLP flows in and SSE streams out to connected clients, but nothing pushes audit data to Datadog, Splunk, or an external OTEL collector yet. | [Audit & Observability](/documentation/enterprise-audit-observability) |
| **PHI taxonomy** | PII scanning covers email, credit card, SSN, and phone; health-identifier categories are not yet in the set. | [Content Safety & Guardrails](/documentation/enterprise-safety-guardrails) |
| **Per-key budgets** | An API key or PAT inherits its owner's scope — it is not its own governance subject, so no per-key budget, rate, or model scope. | [Model Gateway & Routing](/documentation/enterprise-model-routing) |
| **Tab-acceptance metric** | Not currently measurable: Claude Code emits no accept/reject signal and no manual-LOC baseline exists. Needs an IDE-level integration. | [Analytics](/documentation/enterprise-analytics) |
| **Hub analytics dimension** | A Hub is defined as a *geographical* grouping; filtering dashboards and reports by Hub is not yet available. | [Organizations & Departments](/documentation/enterprise-organizations) |

## Verified evidence

Each roadmap item is tracked in the same automated test suite as everything delivered — as an explicitly **skipped placeholder**. The suite cannot pass a test for an unbuilt feature, so a test run visibly reports the gap rather than a false green; the day a capability ships, its placeholder becomes a real test that must pass.

## Everything else

If a capability is not on this page, it is delivered — start from the walkthroughs: [User & Access Management](/documentation/enterprise-user-access), [Organizations](/documentation/enterprise-organizations), [Analytics](/documentation/enterprise-analytics), [Cost Management](/documentation/enterprise-cost-management), [Model Gateway & Routing](/documentation/enterprise-model-routing), [Audit & Observability](/documentation/enterprise-audit-observability), [Safety & Guardrails](/documentation/enterprise-safety-guardrails), and [Tool Governance](/documentation/enterprise-tool-governance).
