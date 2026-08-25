---
title: "Content Safety, PII & Guardrails"
description: "Run every request through the enabled governance chain and safety scanners: jailbreak heuristics, 34 secret patterns plus entropy, PII detection, and redaction."
author: "Astound Digital"
slug: "enterprise-safety-guardrails"
keywords: "safety, guardrails, pii, secrets, jailbreak, redaction, governance, scanners"
kind: "guide"
public: true
tags: ["enterprise", "safety", "admin"]
published_at: "2026-08-25"
updated_at: "2026-08-25"
after_reading_this:
  - "Understand what the enabled four-stage governance chain checks on every call"
  - "Know which scanner categories ship in the default set and what each catches"
  - "Distinguish buffered-response blocking from streamed audit-only enforcement"
  - "See how transcripts redact detected sensitive values at the display layer"
related_docs:
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Audit Trail, Traceability & Observability"
    url: "/documentation/enterprise-audit-observability"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Content Safety, PII & Guardrails

**TL;DR:** The four-stage governance chain and the gateway safety scanners are **enabled** on this deployment. Requests pass jailbreak heuristics, 34 secret patterns with an entropy backstop, and PII detection (email, credit card, SSN, phone). Buffered responses can be blocked outright; streamed responses are scanned audit-only; transcripts redact detected values at display time. The category set is a sensible default Astound can tune.

## The governance chain is on

Every call runs the four-stage synchronous chain — **scope check → secret scan → blocklist → rate limit** — first-deny-wins, with every decision audited with trace linkage. The chain itself is covered in depth in [MCP, Tool Governance & Distribution](/documentation/enterprise-tool-governance); this page covers the content-safety scanners layered on the gateway.

## What the scanners check

The shipped default category set:

| Category | What it catches |
|---|---|
| **Jailbreak heuristics** | Prompt patterns attempting to subvert model instructions |
| **Secrets** | 34 known credential patterns (API keys, tokens, private keys) plus a high-entropy-string backstop for secrets no pattern names |
| **PII** | Email addresses, credit card numbers, US Social Security numbers, phone numbers |

This set is the **sensible default, not a fixed contract** — categories and their block-vs-audit disposition are configurable, and the intended workflow is that Astound signs off the category set it requires. A PHI taxonomy (health identifiers) is not yet part of the set — see the [roadmap](/documentation/enterprise-roadmap).

## Blocking vs. auditing: why streaming differs

Enforcement depends on how the response is delivered:

- **Buffered responses** are held until scanning completes, so a detection can **block the response** before the caller sees a byte.
- **Streamed responses** are scanned **audit-only**: tokens are already on the wire as they are generated, so retroactively blocking them is impossible without breaking streaming entirely. Detections are recorded and alertable, but the stream completes.

This is a deliberate design, not a gap: the alternative — buffering every stream — would destroy the latency profile streaming exists to provide. Where blocking matters more than latency, use non-streaming calls.

Note also that in-flight enforcement **blocks rather than redacts** — a flagged buffered response is refused whole, not rewritten with masked values. A full in-flight redaction pipeline is a scoped design item on the roadmap.

## Display-layer redaction

Transcripts shown in the admin UI apply **display-layer redaction**: values the scanners flagged (secrets, PII) are masked when a conversation is rendered, so reviewing an audit trail does not itself re-expose the sensitive data that triggered the detection.

## Where detections go

Every scanner decision is an audit row on the same spine as everything else — reachable from `/admin/entities/traces` and `systemprompt infra logs trace list`, alertable to Slack, and attributable to the actor via the trace chain described in [Audit Trail, Traceability & Observability](/documentation/enterprise-audit-observability).

## Verified evidence

Every capability on this page is proven by tagged end-to-end tests run against a seeded instance. To replicate: `just start`, then `just e2e-seed --reset`, then the command in the table. Screenshots regenerate with `just e2e-screens`.

| REQ | What the test proves | Replicate with |
|---|---|---|
| REQ-030 | The enabled safety chain flags jailbreak patterns and PII, blocks a flagged buffered response, and audits a streamed one | `just e2e-req REQ-030` |
| REQ-036 | Secret patterns and the entropy backstop catch credential egress, and transcripts render detected values redacted | `just e2e-req REQ-036` |

These REQs' Playwright specs live under `playwright/tests/requirements/`, with gateway-level proofs in the `req_030_*` / `req_036_*` Rust modules under `tests/integration/` (run with `just test-integration`). No screenshots exist for them yet — the pack grows with `just e2e-screens`.
