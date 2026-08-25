---
title: "Model Gateway, Routing & Data Residency"
description: "Route all AI traffic through one provider-neutral gateway with ACL-enforced model access, quotas, latency SLOs, private endpoints, and residency guarantees."
author: "Astound Digital"
slug: "enterprise-model-routing"
keywords: "gateway, routing, models, providers, acl, quotas, residency, no-retain, shadow ai, slo"
kind: "guide"
public: true
tags: ["enterprise", "gateway", "admin"]
published_at: "2026-08-25"
updated_at: "2026-08-25"
after_reading_this:
  - "Route any Anthropic- or OpenAI-compatible client through the provider-neutral gateway"
  - "Grant and revoke model access per role, organization, or department via gateway_route ACLs"
  - "Enforce data-residency and no-retain requirements declaratively on routes"
  - "Set quota windows and latency SLO thresholds and read the breach reporting"
  - "Add a private or self-hosted endpoint as a configuration entry"
related_docs:
  - title: "Cost Management, Budgets & FinOps"
    url: "/documentation/enterprise-cost-management"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
---

# Model Gateway, Routing & Data Residency

**TL;DR:** All AI traffic flows through one provider-neutral gateway (`/v1/messages`, `/v1/responses`, four wire protocols) that rewrites models, enforces per-role access with 403s, refuses unlisted models, applies quota windows, tracks latency against an SLO, and — via governance metadata on providers and routes — refuses to send classified traffic to a provider that retains data.

## One gateway, any provider

The gateway speaks **four wire protocols** behind `/v1/messages` and `/v1/responses`, so any Anthropic-SDK or OpenAI-compatible client points at it unchanged. Routes match on model pattern and can rewrite via `upstream_model` — a client asking for `claude-*` can be transparently served by a different backend without any application change. Substitution across Cerebras, OpenAI, and Gemini is proven in live deployments.

## Model access control

Access to models is a **`gateway_route` ACL matrix**, administered in the admin UI and granted per role, organization, or department. A request for a route the caller is not entitled to is refused with **HTTP 403**, the denial is audited, and grants are revocable with immediate effect.

## Credentials

Personal access tokens carry an optional **expiry**, a **prefix** (identify a leaked token without exposing it), and immediate **revocation** from `/admin/devices/pats`. Note that a key is not its own governance subject — budgets and quotas bind to the owning user and organization, not to the individual key (see the [roadmap](/documentation/enterprise-roadmap)).

## Shadow-AI posture

The gateway runs with **`allow_unlisted_models: false`**: a model that is not explicitly configured cannot be called, whoever asks. Combined with route entitlements and the full audit trail, everything that touches the gateway is governed. Traffic that never touches the gateway — someone calling a provider directly from their laptop — is a network-control matter for corporate IT, outside any application platform's reach.

## Latency SLOs

A **configurable latency SLO threshold** is tracked per the reporting views: breach percentage over the period, alongside **p50 and p95** latency and per-model latency data. Per-use-case SLO taxonomies and error-budget alerting await agreed SLO definitions.

## Quota windows

Rate limiting ships **configured and enabled with sensible defaults**: quota windows on requests, tokens, and cost, applied per user or per organization, with role-tier multipliers and per-organization caps. The defaults are a starting point Astound can tune; sizing sign-off is an open decision.

## Private and self-hosted endpoints

Any OpenAI- or Anthropic-compatible private endpoint is **a configuration entry, not a code change**. Declaring a provider with the `Backend` surface keeps it un-advertised — it serves routes without appearing in any public model listing — and every route is validated at boot.

## Data classification & residency

Provider and model YAML carries a **`governance:` block** with two booleans: `european` (data stays in the EU) and `no_retain` (the provider contractually does not retain prompts or completions). Routes declare what they demand via a **`requires:` block**:

```yaml
providers:
- name: anthropic
  governance:
    european: false
    no_retain: true
gateway:
  routes:
  - model_pattern: claude-*
    provider: anthropic
    requires:
      no_retain: true
```

Enforcement happens twice:

- **At boot** — the server refuses to start a route whose provider or model does not satisfy the route's `requires:` block. A misconfiguration is caught before any traffic flows.
- **At dispatch** — a request that would land on a non-satisfying provider is denied, with a policy audit row whose descriptor records the failed `requires:` condition, so the denial is explainable after the fact.

This turns residency and no-retain from convention into a declared, machine-checked property. Classifying *data* (rather than routes) still needs Astound's classification scheme — tracked on the [roadmap](/documentation/enterprise-roadmap).

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| REQ | What the test proves | Replicate with |
|---|---|---|
| REQ-018 | Declarative routing matches on request shape and the `RouteSelector` seam is exercised | `just e2e-req REQ-018` |
| REQ-020 | The same client is served by a substituted upstream provider via `upstream_model` with no application change | `just e2e-req REQ-020` |
| REQ-021 | A caller without a `gateway_route` grant is refused with 403, audited, and admitted once the grant is added | `just e2e-req REQ-021` |
| REQ-022 | PATs honour expiry, are identifiable by prefix, and stop working immediately on revocation | `just e2e-req REQ-022` |
| REQ-024 | A request for an unlisted model is refused under `allow_unlisted_models: false` and the refusal is audited | `just e2e-req REQ-024` |
| REQ-029 | Latency is reported against the configured SLO threshold with breach %, p50, and p95 | `just e2e-req REQ-029` |
| REQ-033 | Quota windows on requests, tokens, and cost enforce per user and per org, with role-tier multipliers | `just e2e-req REQ-033` |
| REQ-034 | A private OpenAI/Anthropic-compatible endpoint serves traffic as a `Backend` provider, validated at boot | `just e2e-req REQ-034` |
| REQ-037 | A route with a `requires:` block refuses to boot on a non-satisfying provider and denies at dispatch with an audit row | `just e2e-req REQ-037` |
| REQ-038 | A `requires: {no_retain: true}` route only ever dispatches to a provider whose governance metadata satisfies it | `just e2e-req REQ-038` |

These REQs are gateway-level: their Playwright specs live under `playwright/tests/requirements/` and their protocol proofs in the `req_018_*`–`req_038_*` Rust modules under `tests/integration/` (run with `just test-integration`). No screenshots exist for them yet — the pack grows with `just e2e-screens`.
