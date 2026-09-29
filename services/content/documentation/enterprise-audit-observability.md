---
title: "Audit Trail, Traceability & Observability"
description: "Audit every AI call with actor, cost, and latency; follow one trace id from request to tool to cost; ingest and export OTLP; and catch cost or error spikes automatically."
author: "systemprompt.io"
slug: "enterprise-audit-observability"
keywords: "audit, trace, observability, otlp, sse, anomaly detection, requests, traceability"
kind: "guide"
public: true
tags: ["enterprise", "audit", "admin"]
published_at: "2026-08-25"
updated_at: "2026-09-28"
after_reading_this:
  - "Inspect any AI request's actor, model, tokens, cost, and latency at /admin/requests"
  - "Follow a single trace id through policy decision, model call, tool execution, and cost"
  - "Ingest OTLP telemetry, export the audit trail to a collector, and watch the live audit stream over SSE"
  - "Inspect persisted cost, volume, and error anomalies"
related_docs:
  - title: "Model Gateway, Routing & Data Residency"
    url: "/documentation/enterprise-model-routing"
  - title: "MCP, Tool Governance & Distribution"
    url: "/documentation/enterprise-tool-governance"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# Audit Trail, Traceability & Observability

**TL;DR:** Every model call writes an audit row — actor, model, provider, tokens, cost, latency, trace ids — browsable at `/admin/requests`. One trace id links the whole chain (request → policy decision → model call → tool execution → cost) and resolves at `/admin/traces`. The platform ingests OTLP, can export its audit trail to an OTLP collector, streams live audit events over SSE, and a persisted anomaly job records cost, volume, and error spikes.

## Per-call audit: /admin/requests

Every request through the gateway lands a row with **DB-locked actor attribution**: who made the call, which model and provider served it, token counts in and out, cost, latency, status, and the trace ids that link it onward. `/admin/requests` lists and filters these rows (the older `/admin/entities/requests` path redirects there); the CLI reaches the same data:

```bash
systemprompt infra logs request list --limit 20
systemprompt infra logs request list --since 1h --provider anthropic
systemprompt infra logs request list --since 2026-09-08 --until 2026-09-12 --user <user-id>
systemprompt infra logs request list --before <cursor>      # the last row's `cursor`: strictly older rows
systemprompt infra logs audit <request-id>                  # identity, model, tokens, cost, counts
systemprompt infra logs audit <request-id> --messages --limit 20 --max-content 400
```

`audit <request-id>` reconstructs the context for one call — identity, policy evaluations, tokens and cost, with `message_count` and `tool_call_count`. The transcript is opt-in and paged: `--messages` / `--tools` include the rows, `--offset` / `--limit` walk them, and `--max-content` bounds each body, so a long session never has to come back as one document. The same surface is what the admin MCP server's `request_log` and `conversation_audit` tools call.

## End-to-end traceability: /admin/traces

One **trace id** correlates the entire chain: the inbound request, the governance policy decision, the model call, any MCP or tool executions it triggered, and the resulting cost. `/admin/traces` is the universal chain resolver — paste a trace id and read the whole story, whichever link you started from.

```bash
systemprompt infra logs trace list --limit 20
systemprompt infra logs trace list --agent <name> --status failed
systemprompt infra logs trace show <trace-id>
```

## Observability ingest and live stream

- **OTLP ingest** — the gateway accepts OpenTelemetry traces, logs, and metrics on its credential-gated `/otel` route (spans and log records are persisted; metrics are only summarised), so instrumented clients report into the same place.
- **OTLP export** — the core `otlp_export` job (enabled in `services/scheduler/config.yaml`) ships `ai_requests`, the tool-call ledger and `governance_decisions` as spans, and the logs table as log records, to the collector a profile names under `observability.otlp`. It advances its watermark only once the collector acknowledges a batch, and is a no-op on a profile without that block.
- **Live audit stream** — audit events stream as JSON over **Server-Sent Events**, useful for watching an incident unfold or feeding a live wallboard.

## Anomaly detection

A **persisted anomaly job** watches for cost spikes, volume spikes, and error spikes against recent baselines. The hourly `usage_anomaly` job compares the last complete hour against the trailing week's hourly average; a metric must exceed both a multiplier and an absolute floor. Detections are persisted to `usage_anomalies`, logged on first detection per window, and shown on the analytics Overview. The template configures no external alert delivery. The detectors are threshold-based rather than learned baselines; tuning iterates with real traffic.

## Known caveats

One limitation is stated here once and tracked on the [Enterprise Roadmap](/documentation/enterprise-roadmap):

- **Append-only by convention, not mechanism.** There is no WORM storage or hash-chaining. Database privileges and retention/cleanup operations determine whether rows can be changed or deleted.

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured.
