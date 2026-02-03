---
title: "Analytics"
description: "Costs, usage, audit trails. Every AI request logged automatically with full observability."
author: "SystemPrompt Team"
slug: "services/analytics"
keywords: "analytics, costs, usage, audit, tracking, metrics"
image: "/files/images/docs/services-analytics.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Analytics

SystemPrompt logs every AI request automatically. Token counts, costs, latencies, error rates—all captured without instrumentation code. The analytics system provides observability into your AI operations, enabling cost management, performance optimization, and compliance auditing.

The analytics architecture follows the "closed loop" principle. Agents can query their own performance data through the same interface that humans use. An agent can ask "what was my average response time today?" and receive actionable data. This self-awareness enables adaptive behavior.

## Analytics Architecture

The analytics system captures data at multiple points in the request lifecycle, stores it efficiently, and provides query interfaces for both real-time and historical analysis.

**Data capture:**

Every AI request flows through instrumented middleware. The middleware captures timing, token counts, and metadata before and after LLM calls. This happens transparently—no application code changes required.

```
Request → Middleware (start timer) → LLM Call → Middleware (capture metrics) → Response
                                         ↓
                                   Analytics Store
```

**Storage:**

Analytics data is stored in PostgreSQL alongside application data. Time-series data uses partitioned tables for efficient querying of recent data while maintaining historical archives. The `systemprompt-analytics` crate manages schema and queries.

**Query interfaces:**

- **CLI**: Direct queries for operators and scripts
- **API**: Programmatic access for dashboards
- **MCP**: Agent access for self-analysis

## Track AI Costs

Cost tracking is automatic for all supported providers. The system knows token pricing for each model and calculates costs in real-time.

**Supported providers:**

| Provider | Models | Cost Tracking |
|----------|--------|---------------|
| Anthropic | Claude Opus, Sonnet, Haiku | Full support |
| OpenAI | GPT-4, GPT-3.5 | Full support |
| Gemini | Gemini Pro, Flash | Full support |

**Cost calculation:**

Costs are calculated from token counts and model-specific pricing:

```
Cost = (input_tokens × input_price) + (output_tokens × output_price)
```

Pricing tables are updated regularly. For custom or fine-tuned models, you can configure custom pricing.

**CLI cost queries:**

```bash
# View costs for today
systemprompt analytics costs --period today

# View costs by provider
systemprompt analytics costs --group-by provider --period week

# View costs by model
systemprompt analytics costs --group-by model --period month

# View costs by agent
systemprompt analytics costs --group-by agent --period week
```

**Example output:**

```
Period: 2026-01-24 to 2026-01-30

Provider     | Requests | Input Tokens | Output Tokens | Cost
-------------|----------|--------------|---------------|--------
anthropic    | 1,247    | 2.4M         | 890K          | $42.15
openai       | 156      | 180K         | 45K           | $3.80
gemini       | 892      | 1.1M         | 320K          | $8.40
-------------|----------|--------------|---------------|--------
Total        | 2,295    | 3.68M        | 1.26M         | $54.35
```

**Budget alerts:**

Configure alerts when spending approaches thresholds:

```yaml
# Profile configuration
analytics:
  alerts:
    - type: daily_cost
      threshold: 100.00
      action: email
    - type: monthly_cost
      threshold: 1000.00
      action: pause_requests
```

## Query Usage Metrics

Beyond costs, the analytics system tracks operational metrics that help optimize performance and identify issues.

**Available metrics:**

| Metric | Description |
|--------|-------------|
| `request_count` | Number of AI requests |
| `token_count` | Input and output tokens |
| `latency_p50` | Median response time |
| `latency_p95` | 95th percentile response time |
| `latency_p99` | 99th percentile response time |
| `error_rate` | Percentage of failed requests |
| `cache_hit_rate` | Response cache efficiency |

**CLI usage queries:**

```bash
# Request volume over time
systemprompt analytics requests --period week --interval day

# Latency percentiles
systemprompt analytics latency --period day --percentiles 50,95,99

# Error analysis
systemprompt analytics errors --period week --group-by error_type

# Traffic by hour
systemprompt analytics traffic --period day --interval hour
```

**Filtering:**

Queries can be filtered by various dimensions:

```bash
# Filter by agent
systemprompt analytics requests --agent welcome --period week

# Filter by user
systemprompt analytics requests --user user_abc123 --period day

# Filter by model
systemprompt analytics requests --model claude-sonnet-4 --period month
```

## Audit Trails

Every operation in SystemPrompt generates audit events. These events create a complete record of who did what, when, and how.

**Audit event types:**

| Category | Events |
|----------|--------|
| Authentication | login, logout, token_issued, token_revoked |
| Authorization | scope_granted, permission_denied |
| AI Operations | request_started, request_completed, request_failed |
| Data Access | file_accessed, content_created, content_deleted |
| Administration | user_created, config_changed, agent_modified |

**Audit event structure:**

```json
{
  "id": "evt_abc123",
  "timestamp": "2026-01-30T14:23:45Z",
  "tenant_id": "tenant_xyz",
  "user_id": "user_123",
  "event_type": "ai.request_completed",
  "trace_id": "trace_789",
  "metadata": {
    "provider": "anthropic",
    "model": "claude-sonnet-4",
    "input_tokens": 1250,
    "output_tokens": 480,
    "latency_ms": 2340
  }
}
```

**Querying audit events:**

```bash
# View recent audit events
systemprompt analytics audit --limit 100

# Filter by event type
systemprompt analytics audit --type "ai.*" --period day

# Filter by user
systemprompt analytics audit --user user_123 --period week

# Export for compliance
systemprompt analytics audit --period month --format json > audit.json
```

**Trace correlation:**

Every request receives a trace ID that links all related events. You can follow a single user action through authentication, authorization, AI calls, and responses.

```bash
# Follow a trace
systemprompt analytics trace <trace_id>

# Output shows all events with this trace ID in chronological order
```

## Agent Self-Analysis

The closed-loop architecture enables agents to query their own analytics. An agent can assess its performance, identify patterns, and adapt behavior.

**MCP analytics tools:**

When connected via MCP, agents have access to analytics queries:

- `analytics_costs`: Query cost data
- `analytics_requests`: Query request metrics
- `analytics_errors`: Query error patterns
- `analytics_audit`: Query audit events

**Self-analysis patterns:**

An agent might ask:
- "What was my average response time in the last hour?"
- "Which of my responses had the highest token counts?"
- "How many errors did I encounter today?"

This data enables agents to optimize their behavior. An agent noticing high latency might switch to a faster model. An agent seeing repeated errors might adjust its approach.

## Dashboard Integration

Analytics data is available through the API for custom dashboards and integrations.

**API endpoints:**

| Endpoint | Description |
|----------|-------------|
| `/api/v1/analytics/costs` | Cost metrics |
| `/api/v1/analytics/requests` | Request metrics |
| `/api/v1/analytics/latency` | Latency percentiles |
| `/api/v1/analytics/audit` | Audit events |

**Response format:**

```json
{
  "period": {
    "start": "2026-01-24T00:00:00Z",
    "end": "2026-01-30T23:59:59Z"
  },
  "data": [
    {
      "timestamp": "2026-01-24",
      "requests": 342,
      "cost": 7.84,
      "latency_p50": 1240
    }
  ]
}
```

## Configuration

Analytics configuration is in the profile settings:

```yaml
# Profile configuration
analytics:
  enabled: true
  retention_days: 90
  sampling_rate: 1.0  # 100% sampling

  costs:
    currency: USD
    custom_pricing:
      my-custom-model:
        input_per_1k: 0.01
        output_per_1k: 0.03

  alerts:
    - type: daily_cost
      threshold: 100.00
      action: email
```

## Configuration Reference

| Item | Location | Description |
|------|----------|-------------|
| Enable analytics | Profile (`analytics.enabled`) | Turn analytics on/off |
| Retention | Profile (`analytics.retention_days`) | How long to keep data |
| Sampling | Profile (`analytics.sampling_rate`) | Percentage of requests to track |
| Custom pricing | Profile (`analytics.costs.custom_pricing`) | Model-specific costs |
| Alerts | Profile (`analytics.alerts`) | Budget notifications |

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt analytics overview` | Dashboard overview of all analytics |
| `systemprompt analytics conversations` | Conversation analytics |
| `systemprompt analytics agents` | Agent performance analytics |
| `systemprompt analytics tools` | Tool usage analytics |
| `systemprompt analytics requests` | AI request analytics |
| `systemprompt analytics sessions` | Session analytics |
| `systemprompt analytics content` | Content performance analytics |
| `systemprompt analytics traffic` | Traffic analytics |
| `systemprompt analytics costs` | Cost analytics |

See `systemprompt analytics <command> --help` for detailed options.