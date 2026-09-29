---
title: "Content Safety, PII & Guardrails"
description: "Run every request through the enabled governance chain and safety scanners: jailbreak heuristics, credential patterns plus entropy, PII detection, and redaction."
author: "systemprompt.io"
slug: "enterprise-safety-guardrails"
keywords: "safety, guardrails, pii, secrets, jailbreak, redaction, governance, scanners"
kind: "guide"
public: true
tags: ["enterprise", "safety", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-28"
after_reading_this:
  - "Understand what the enabled four-stage governance chain checks on every call"
  - "Know which scanner categories ship in the default set and what each catches"
  - "Distinguish buffered-response blocking from streamed audit-only enforcement"
  - "See how transcripts redact credentials and SSNs at the display layer"
related_docs:
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Content Safety, PII & Guardrails

**TL;DR:** The four-stage governance chain and the gateway safety scanners are **enabled in warn mode** in this repository's configuration. Authentication and authorization remain enforced. Requests pass jailbreak heuristics, credential patterns with an entropy backstop, and PII detection (email, credit card, SSN, phone). In enforce mode, configured categories can block buffered responses; streamed responses are scanned audit-only; transcripts redact credentials and SSNs at display time. The category set and each category's block-vs-audit disposition are configurable per deployment, and the whole plane can be run in warn mode, which records what each category would have blocked without blocking it.

## The governance chain is on

Calls routed through the governed integration run the four-stage synchronous chain — **scope check → secret scan → blocklist → rate limit** — with every decision audited with trace linkage. In this configuration stages warn and continue; an enforcing denial stops execution. The chain itself is covered in depth in [MCP, Tool Governance & Distribution](/documentation/enterprise-tool-governance); this page covers the content-safety scanners layered on the gateway.

## What the scanners check

The shipped default category set:

| Category | What it catches |
|---|---|
| **Jailbreak heuristics** | Prompt patterns attempting to subvert model instructions |
| **Secrets** | Known credential patterns (API keys, tokens, private keys) plus a high-entropy-string backstop for secrets no pattern names |
| **PII** | Email addresses, credit card numbers, US Social Security numbers, phone numbers |

This set is the **sensible default, not a fixed contract** — categories and their block-vs-audit disposition are configurable, so a deployment enforces exactly the policy its owners require. A PHI taxonomy (health identifiers) is not yet part of the set — see the [roadmap](/documentation/enterprise-roadmap).

## Blocking vs. auditing: why streaming differs

The current `safety.mode: warn` records findings without blocking. When `safety.mode: enforce` is configured, enforcement depends on how the response is delivered:

- **Buffered responses** are held until scanning completes, so a detection can **block the response** before the caller sees a byte.
- **Streamed responses** are scanned **audit-only**: tokens are already on the wire as they are generated, so retroactively blocking them is impossible without breaking streaming entirely. Detections are recorded and alertable, but the stream completes.

This is a deliberate design, not a gap: the alternative — buffering every stream — would destroy the latency profile streaming exists to provide. Where blocking matters more than latency, use non-streaming calls.

There is a third posture beside block and audit-only. `safety.mode: warn` in `services/gateway/policies.yaml` leaves every scanner running and every finding persisted and drops only the refusal, so findings written under it carry `blocked = false`. The quota windows in the same file have the matching switch, `quota_mode: warn`: an exhausted window is recorded as a `governance_decisions` warn under policy `quota` instead of answering 429, so all three planes on an inference request can be run non-blocking together. That is how you find out what a block list costs before you enforce it: a category with many findings and no blocks is one warn mode is absorbing. Read it back on the governance dashboard at `/admin/governance` or with `systemprompt infra logs governance report`, and see [MCP, Tool Governance & Distribution](/documentation/enterprise-tool-governance) for the governance chain's matching switch.

Note also that in-flight enforcement **blocks rather than redacts** — a flagged buffered response is refused whole, not rewritten with masked values. In-flight redaction (masking values instead of refusing the response) is planned — see the [roadmap](/documentation/enterprise-roadmap).

## Display-layer redaction

Transcripts shown in the admin UI apply **display-layer redaction**: well-known credential shapes (cloud keys, source-control tokens, and similar) and US Social Security numbers (all but the last four digits) are masked when a conversation is rendered, and the page says how many values were masked. Other PII categories — email, phone, card numbers — are recorded as findings but not masked in the rendered transcript. The redactor works on the text shape, independently of whether a scanner flagged it.

## Where detections go

Every scanner finding is an `ai_safety_findings` row on the same spine as everything else — listed on the **Safety** tab of `/admin/governance`, reachable from `/admin/traces` and `systemprompt infra logs trace list`, attributable to the actor via the trace chain described in [Audit Trail, Traceability & Observability](/documentation/enterprise-audit-observability).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured.
