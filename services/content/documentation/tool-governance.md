---
title: "Tool Governance"
description: "How MCP tool usage is governed through OAuth scoping, execution logging, event hooks, and role-based access control for enterprise AI deployments."
author: "systemprompt.io"
slug: "tool-governance"
keywords: "tool governance, mcp, oauth, audit, hooks, security"
kind: "guide"
public: true
tags: ["mcp", "governance", "security", "enterprise"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand how every MCP tool call is governed, logged, and auditable"
  - "Configure OAuth scoping for MCP servers and individual tools"
  - "Use CLI commands to monitor tool execution and analyze usage patterns"
  - "Set up event hooks for tool call notifications and compliance checks"
related_docs:
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Hooks"
    url: "/documentation/hooks"
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Rate Limiting"
    url: "/documentation/rate-limiting"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Tool Governance

**TL;DR:** Every MCP tool call in the platform is governed, logged, and auditable. OAuth scoping controls which agents can use which tools. Execution logging captures every invocation with status, timing, and context. Event hooks fire on tool success and failure, enabling real-time alerts and compliance checks. Rate limiting prevents abuse. This page covers the full governance lifecycle of a tool call from authorization to audit.

> **See this in the presentation:** [Slide 14: Every Tool Call Governed](/documentation/presentation#slide-14)

## Overview

In an enterprise deployment with multiple agents, dozens of MCP servers, and tens of thousands of users, ungoverned tool access is a security and compliance risk. A customer agent should not be able to deploy code. A developer agent should not be able to access supplier contracts. A partner agent should not be able to view employee schedules.

Tool governance solves this by placing controls at every stage of a tool call:

1. **Authorization**: Can this agent use this tool? (OAuth scoping)
2. **Execution**: What happened when the tool ran? (Execution logging)
3. **Notification**: Who needs to know about this tool call? (Event hooks)
4. **Analysis**: How are tools being used over time? (Usage analytics)
5. **Throttling**: Is this usage within acceptable limits? (Rate limiting)

## The Governance Lifecycle of a Tool Call

When an agent invokes an MCP tool, the platform executes the following governance steps:

```
1. Agent decides to call a tool
   │
2. OAuth validation
   ├── Does the agent's token include the required scope?
   ├── Does the token audience match the MCP server's audience?
   └── Is the token expired? → Reject if any check fails
   │
3. Rate limit check
   ├── Is the agent/user within their rate limit?
   └── Is the MCP server within its capacity? → 429 if exceeded
   │
4. Tool execution
   ├── MCP server processes the request
   ├── Execution time is measured
   └── Result (success or failure) is captured
   │
5. Execution logging
   ├── Tool name, parameters, result status
   ├── Execution time, agent context, user identity
   └── Written to the audit log
   │
6. Event hooks fire
   ├── PostToolUse → on success
   └── PostToolUseFailure → on failure
   │
7. Response returned to agent
```

Every step in this lifecycle is auditable. There is no path for a tool call to bypass governance.

## OAuth Scoping

### How OAuth Controls Tool Access

Each MCP server has its own OAuth configuration:

```yaml
# Example MCP server OAuth config
oauth:
  required: true
  scopes:
    - admin
  audience: mcp
```

When an agent calls a tool on this server, the platform validates:

| Check | What it validates |
|-------|-------------------|
| `required: true` | A token must be present (no anonymous access) |
| `scopes: [admin]` | The token must include the `admin` scope |
| `audience: mcp` | The token's audience must be `mcp` |

If any check fails, the tool call is rejected before execution.

### Scope Patterns for Enterprise Deployments

| Scope | Typical Use | Agents That Use It |
|-------|-------------|-------------------|
| `admin` | Full platform management tools | Developer Agent, Partner Agent |
| `user` | Standard operational tools | Associate Agent |
| `tools:read` | Read-only tool access | Monitoring agents, reporting agents |
| `tools:write` | Tool execution with write permissions | Agents that modify data |
| `tools:deploy` | Production deployment tools | Developer Agent (with human approval) |

### Per-Agent Tool Access

Each agent declares which MCP servers it connects to in its YAML configuration:

```yaml
# agent configuration example
metadata:
  mcpServers:
  - systemprompt
```

This declaration limits the agent's tool access:

| Agent Scope | MCP Servers | Tool Access |
|-------------|-------------|-------------|
| Admin-scoped agents | `systemprompt`, `skill-manager` | Platform management and skill tools |
| User-scoped agents | `skill-manager` only | User-level skill management |
| No MCP agents | None | No external tool access |

An agent without MCP servers cannot call any tools. This is a secure default -- tools are explicitly added, not inherited.

In addition to agent-level MCP mapping, the enterprise-demo plugin uses **HTTP governance hooks** to evaluate tool call content at runtime. The PreToolUse hook inspects every tool input for policy violations (secret detection, scope restrictions, blocklist checks, rate limits) and can deny a tool call even when the agent has MCP access.

### Plugin-Level Tool Control

The enterprise-demo plugin bundles MCP servers and governance hooks:

```yaml
# enterprise-demo/config.yaml
skills:
  - example_web_search
  - use_dangerous_secret
mcp_servers:
  - systemprompt
```

MCP servers listed in the plugin are available to agents within that plugin -- but only if the agent also declares the server in its own configuration. The plugin provides the server; the agent opts in to using it. The governance hooks apply to all tool calls regardless of MCP server mapping.

## Tool Execution Logging

### What Gets Logged

Every tool call generates a log entry containing:

| Field | Description | Example |
|-------|-------------|---------|
| Timestamp | UTC time of the call | `2026-03-19T14:32:01.234Z` |
| Tool name | The MCP tool that was called | `list_agents` |
| MCP server | Which server handled the call | `systemprompt` |
| Agent | Which agent initiated the call | `platform_agent` |
| User | The authenticated user context | `dev-user-42` |
| Parameters | Input parameters (sanitized) | `{limit: 10}` |
| Status | Success or failure | `success` |
| Duration | Execution time in milliseconds | `142ms` |
| Error | Error message if failed | `null` |
| Request ID | Correlation ID for the full request chain | `req-abc-123` |

### Querying Tool Execution Logs

```bash
# List recent tool executions
systemprompt infra logs tools list

# Filter by status
systemprompt infra logs tools list --status error

# Filter by MCP server
systemprompt infra logs tools list --server systemprompt

# Filter by agent
systemprompt infra logs tools list --agent <agent-name>

# Combine filters
systemprompt infra logs tools list --agent <agent-name> --status error --since 1h

# View detailed log for a specific request
systemprompt infra logs audit <request-id> --full
```

### Log Retention and Compliance

Tool execution logs are retained according to the platform's log retention policy. For enterprise deployments supporting SOC2 and regulatory compliance:

- **Minimum retention**: 90 days for operational logs
- **Extended retention**: 1 year for compliance-sensitive tool calls (data access, deployments)
- **Immutable logging**: Logs cannot be modified or deleted during the retention period
- **Export capability**: Logs can be exported in standard formats for external audit tools

## Agent-Tool Mapping

### Viewing Which Tools Each Agent Can Access

```bash
# List all tools available to a specific agent
systemprompt admin agents tools <agent-name> --detailed

# List all tools across all agents
systemprompt admin agents tools --all

# Show which agents can access a specific tool
systemprompt admin agents tools --tool list_agents
```

### How Agent-Tool Mapping Works

The mapping is determined by three layers:

```
Layer 1: Plugin declares MCP servers
  └── enterprise-demo plugin includes "systemprompt" server

Layer 2: Agent declares MCP server connections
  └── Admin-scoped agent includes "systemprompt" in mcpServers

Layer 3: OAuth scoping filters available tools
  └── Admin scope grants access to admin-scoped tools

Layer 4: PreToolUse governance hook (HTTP)
  └── Evaluates tool input content (secrets, blocklist, rate limits)
```

All three layers must align for a tool to be accessible to an agent. This defense-in-depth approach ensures that:

- A plugin cannot grant tool access to agents that do not opt in
- An agent cannot access tools from MCP servers not in its plugin
- OAuth scoping provides the final authorization check
- The PreToolUse governance hook provides content-based policy enforcement as the last line of defence

### Modifying Agent-Tool Access

To change which tools an agent can access:

1. **Add/remove MCP servers from the agent's YAML** -- Controls which servers the agent connects to
2. **Modify the plugin's MCP server list** -- Controls which servers are available within the plugin
3. **Adjust OAuth scopes** -- Controls the permission level for tool access

Example: To give an agent access to a new MCP server:

```yaml
# 1. Add to the agent's config (services/agents/<agent-name>.yaml)
metadata:
  mcpServers:
  - product-search

# 2. Add to the plugin (services/plugins/enterprise-demo/config.yaml)
plugin:
  mcp_servers:
  - systemprompt
  - product-search
```

## Event Hooks on Tool Calls

### PostToolUse

Fires after every successful tool call. Use cases:

- **Audit logging**: Write tool usage to an external audit system
- **Compliance checks**: Verify that the tool call result meets compliance requirements
- **Notifications**: Alert administrators when sensitive tools are used
- **Analytics**: Feed tool usage data to analytics pipelines

```yaml
# Example hook configuration
hooks:
  - event: PostToolUse
    handler: audit_tool_usage
    config:
      notify_on:
        - deploy_production
        - delete_resource
        - modify_access
      channels:
        - slack
        - audit_log
```

### PostToolUseFailure

Fires when a tool call fails. Use cases:

- **Alerting**: Immediately notify the ops team when critical tools fail
- **Retry logic**: Automatically retry transient failures
- **Circuit breaking**: Disable an MCP server after repeated failures
- **Incident creation**: Auto-create incidents for persistent tool failures

```yaml
# Example failure hook
hooks:
  - event: PostToolUseFailure
    handler: alert_tool_failure
    config:
      severity_threshold: error
      channels:
        - pagerduty
        - slack
      auto_retry:
        max_attempts: 3
        backoff: exponential
```

### Hook Execution Order

When a tool call completes:

1. Execution log is written (always, regardless of hooks)
2. Event hooks fire asynchronously (do not block the response to the agent)
3. Hook results are logged separately from the tool call

Hooks never block the tool call response. A slow hook does not delay the agent's response to the user.

### Debugging Hooks

```bash
# View hook execution history
systemprompt infra logs view --source hooks --since 1h

# Check for hook failures
systemprompt infra logs view --source hooks --level error --since 1h

# View hook configuration
systemprompt core hooks list
systemprompt core hooks show <hook-id>
```

## Tool Usage Analytics

### Overview Metrics

```bash
# High-level tool usage statistics
systemprompt analytics tools stats
```

This shows:

| Metric | Description |
|--------|-------------|
| Total calls | Total tool invocations across all agents |
| Success rate | Percentage of calls that completed successfully |
| Average latency | Mean execution time across all tools |
| Most-used tools | Top 10 tools by invocation count |
| Error rate | Percentage of calls that failed |
| Unique users | Number of distinct users who triggered tool calls |

### Per-Tool Analytics

```bash
# Detailed analytics for a specific tool
systemprompt analytics tools show list_agents
```

This shows:

| Metric | Description |
|--------|-------------|
| Invocation count | How many times this tool was called |
| Success/failure ratio | Reliability of this specific tool |
| p50/p95/p99 latency | Latency distribution for this tool |
| Top callers | Which agents use this tool most |
| Error patterns | Common error messages and their frequency |
| Usage trend | Invocations over time (hourly, daily) |

### Cross-Agent Tool Comparison

```bash
# Compare tool usage across agents
systemprompt analytics tools stats --compare-agents

# Tool usage by agent
systemprompt analytics tools stats --agent <agent-name>
```

### Usage Patterns to Watch

| Pattern | What It Means | Action |
|---------|--------------|--------|
| High error rate on one tool | Tool or its backend is failing | Check MCP server logs |
| One agent dominates tool usage | Possible misconfiguration or abuse | Review agent configuration |
| Latency increasing over time | Backend degradation | Scale MCP server or optimize tool |
| Tool never used | Dead configuration | Consider removing to reduce attack surface |
| Burst of calls from one user | Possible automation or abuse | Check rate limiting configuration |

## MCP Server Configuration

### Port Ranges

MCP servers are assigned ports in the 5000-5999 range:

| Range | Purpose |
|-------|---------|
| 5000-5099 | Platform MCP servers (systemprompt, admin tools) |
| 5100-5199 | Integration MCP servers (EDI, WMS, POS) |
| 5200-5299 | Custom MCP servers (domain-specific tools) |
| 5300-5999 | Reserved for future expansion |

Agent ports (9026-9029) are separate from MCP server ports. This separation ensures that agent traffic and tool traffic are independently monitorable and scalable.

### Auto-Discovery

MCP servers register with the platform's server registry:

```bash
# List all registered MCP servers
systemprompt plugins mcp list

# Check a specific server's status
systemprompt plugins mcp status systemprompt

# View server's available tools
systemprompt plugins mcp tools systemprompt
```

The registry endpoint (`/api/v1/mcp/registry`) returns all available MCP servers with their capabilities, enabling agents to discover tools dynamically.

### Timeout and Retry Settings

Each MCP server supports configurable timeout and retry behavior:

| Setting | Default | Description |
|---------|---------|-------------|
| Connection timeout | 5s | Time to establish connection to MCP server |
| Request timeout | 30s | Maximum time for a tool call to complete |
| Retry attempts | 3 | Number of retries on transient failure |
| Retry backoff | Exponential | Backoff strategy between retries |
| Circuit breaker threshold | 5 failures in 60s | Failures before circuit opens |
| Circuit breaker recovery | 30s | Time before trying again after circuit opens |

```bash
# View MCP server configuration including timeouts
systemprompt plugins mcp show systemprompt --detailed
```

## Rate Limiting for Tools

### Base Rates

MCP tool calls are rate-limited to prevent abuse and ensure fair resource allocation:

| Tier | Rate Limit | Burst Allowance |
|------|-----------|-----------------|
| Standard (user) | 200 req/s | 50 additional burst |
| Service | 1,000 req/s (5x base) | 250 additional burst |
| MCP | 1,000 req/s (5x base) | 250 additional burst |
| Admin | 500 req/s | 100 additional burst |

### Rate Limit Dimensions

Rate limits are applied across multiple dimensions:

| Dimension | What it limits |
|-----------|---------------|
| Per-user | A single user cannot monopolize tool capacity |
| Per-agent | Each agent has independent rate limits |
| Per-MCP-server | Each MCP server has its own capacity limits |
| Per-tool | Individual tools can have custom rate limits |
| Global | Total platform-wide tool call capacity |

### Rate Limit Headers

Tool call responses include rate limit headers:

```
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 187
X-RateLimit-Reset: 1710849600
```

### Monitoring Rate Limits

```bash
# Check current rate limit status
systemprompt infra services rate-limits

# View rate-limited requests
systemprompt infra logs view --level warn --source rate-limiter --since 1h

# Adjust rate limits (admin only)
systemprompt admin config set rate_limit.mcp.base 300
```

## Compliance Patterns

### SOC2 Support

Tool governance supports SOC2 compliance through:

| SOC2 Control | How Tool Governance Supports It |
|-------------|-------------------------------|
| CC6.1 (Logical access) | OAuth scoping restricts tool access to authorized agents |
| CC6.2 (Authentication) | Every tool call requires a valid OAuth token |
| CC6.3 (Authorization) | Scope-based authorization controls tool permissions |
| CC7.1 (System monitoring) | Tool execution logging captures all activity |
| CC7.2 (Anomaly detection) | Usage analytics identify unusual tool usage patterns |
| CC8.1 (Change management) | Deployment tools require human-in-the-loop approval |

### Audit Requirements

For regulated industries, tool governance provides:

- **Complete audit trail**: Every tool call logged with full context
- **Immutable logs**: Logs cannot be modified during retention period
- **Access logging**: Who accessed which data through which tool, when
- **Export capability**: Logs exportable in standard formats for external audit tools
- **Separation of duties**: Different OAuth scopes for different responsibilities

### Building a Compliance Report

```bash
# Generate a tool usage report for a date range
systemprompt analytics tools stats --since 30d --format json > tool_usage_report.json

# List all tool calls with compliance-relevant details
systemprompt infra logs tools list --since 30d --format json > tool_audit_log.json

# List all failed tool calls (potential security events)
systemprompt infra logs tools list --status error --since 30d --format json > tool_failures.json
```

## Best Practices

### Principle of Least Privilege

- Start agents with no MCP servers and add only what they need
- Use the most restrictive OAuth scope that still allows the agent to function
- Regularly review agent-tool mappings and remove unused connections

### Monitor Before You Need To

- Set up `PostToolUseFailure` hooks before tool failures become a problem
- Establish baseline usage patterns during normal operations
- Configure alerts for deviations from baseline

### Defense in Depth

- Do not rely on a single layer of governance (OAuth alone is not enough)
- Combine OAuth scoping + agent-level MCP declarations + plugin configuration
- Use hooks for additional runtime checks beyond what OAuth provides

### Regular Audits

- Review tool usage analytics monthly
- Identify unused tools and remove them (reduced attack surface)
- Check for agents with excessive tool access
- Verify that rate limits are appropriate for current traffic patterns
