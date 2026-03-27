---
title: "Platform Agent"
description: "Reference architecture for a platform agent covering development workflows, code review, CI/CD automation, and security scanning. This configuration would be scoped in the Phase 1 PRD."
author: "systemprompt.io"
slug: "agent-developer"
keywords: "platform agent, code review, ci/cd, security scanning, development"
kind: "guide"
public: true
tags: ["agents", "platform", "enterprise", "security"]
published_at: "2026-03-19"
updated_at: "2026-03-19"
after_reading_this:
  - "Understand the Platform Agent's role as an engineering assistant"
  - "Read the agent's YAML configuration including MCP server connections"
  - "Identify the three skill domains: code assistance, DevOps, and security scanning"
  - "Use CLI commands to interact with and monitor the Platform Agent"
related_docs:
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Tool Governance"
    url: "/documentation/tool-governance"
---

# Platform Agent — Reference Architecture

> **Note:** This is a reference architecture document. It demonstrates how a platform agent would be configured in a Foodles deployment. The specific agent configuration, skills, MCP server access, and RBAC rules would be scoped collaboratively during the Phase 1 PRD.

**TL;DR:** The Platform Agent shows a developer orchestration agent for engineering teams. It uses MCP for discovery and A2A for delegation. This reference covers code review, CI/CD automation, security scanning, and architecture guidance. It connects to the `systemprompt` MCP server for platform management with `admin`-scoped OAuth.

## Overview

The Platform Agent is the developer-facing agent that consolidates code generation, review, debugging, DevOps orchestration, and security scanning into a single conversational interface.

The Platform Agent is unique among the demo agents in two ways:

1. **It has MCP server access.** The `systemprompt` MCP server gives this agent the ability to query and manage the platform itself -- listing agents, checking service status, and interacting with the CLI.

2. **It has the most skills.** Three registered skills (`general_assistance`, `dev_rust_standards`, `dev_architecture_standards`) compared to one for other agents. This reflects the breadth of developer workflows.

### Key Facts

| Property | Value |
|----------|-------|
| Agent ID | `developer_agent` |
| Foodles Name | Platform Agent |
| Port | 9026 |
| Endpoint | `http://localhost:8080/api/v1/agents/developer_agent` |
| Protocol | A2A v0.3.0, JSONRPC |
| Streaming | Enabled |
| OAuth Scope | `admin` |
| OAuth Audience | `a2a` |
| MCP Servers | `systemprompt` |
| Primary Agent | No |

## Configuration

The Platform Agent is defined in `services/agents/developer_agent.yaml`:

```yaml
# Platform Agent Configuration (Foodles Demo)
# Developer orchestration layer for engineering teams

agents:
  developer_agent:
    name: developer_agent
    port: 9026
    endpoint: http://localhost:8080/api/v1/agents/developer_agent
    enabled: true
    dev_only: false
    is_primary: false
    default: false
```

### Port Assignment

Port 9026 hosts the Platform Agent. Developer traffic is lower in volume than guest or revenue traffic but tends to involve longer conversations with more complex tool usage. The port is optimized for sustained connections rather than high-frequency short interactions.

### A2A Protocol Card

```yaml
    card:
      protocolVersion: 0.3.0
      name: Platform Agent
      displayName: Platform Agent
      description: Developer-facing AI agent that interprets developer intent
        and orchestrates execution across the engineering ecosystem via CLI,
        IDE, and chat integrations
      version: 1.0.0
      preferredTransport: JSONRPC
      iconUrl: https://ui-avatars.com/api/?name=PA&background=2d8f3f&color=fff
```

The icon uses a developer-friendly green (`#2d8f3f`), visually differentiating it from other agents in the admin dashboard.

### Skills on the A2A Card

The Platform Agent declares three skills:

```yaml
      skills:
      - id: general_assistance
        name: General Assistance
        description: Help developers with code review, debugging, and
          implementation across the engineering stack
        tags:
        - development
        - code-review
        - debugging
        examples:
        - Review this pull request for security issues
        - Debug a failing integration test
        - Generate boilerplate for a new microservice
      - id: dev_rust_standards
        name: Rust Standards
        description: Rust coding standards, linting, testing, and
          architecture patterns
        tags:
        - development
        - rust
        - standards
        examples:
        - Check code against Rust standards
        - Review module architecture
        - Validate test coverage
      - id: dev_architecture_standards
        name: Architecture Standards
        description: Architecture standards, patterns, and best practices
        tags:
        - architecture
        - standards
        examples:
        - Review system architecture
        - Check for architectural anti-patterns
        - Validate extension design
```

## System Prompt

The Platform Agent's system prompt emphasizes security and auditability:

```
You are the Platform Agent, an AI-powered engineering assistant that
accelerates developer productivity across the entire software development
lifecycle. You integrate into CLI, IDE, and messaging platforms to meet
developers where they work.

## Capabilities

- **Code Generation & Review**: Generate code from natural language, review
  PRs, suggest improvements
- **Debugging & Diagnostics**: Analyze errors, trace issues, suggest fixes
- **DevOps Orchestration**: Manage CI/CD pipelines, deployments, infrastructure
- **Security Scanning**: Dependency audits, secret detection, OWASP compliance
- **Knowledge Base**: Access to internal documentation, architecture decisions,
  and best practices

## Operating Principles

1. **Security First**: Never expose credentials, audit all generated code
   for vulnerabilities
2. **Least Privilege**: Request only the minimum permissions needed for each task
3. **Auditability**: Log all code modifications, deployments, and
   infrastructure changes
4. **Human-in-the-Loop**: Production deployments require explicit human approval

## MCP Integration

This agent uses Model Context Protocol (MCP) for:
- Tool discovery and registration across the engineering platform
- Agent-to-agent delegation for specialized tasks
- Secure data access through standardized protocols
- Plugin ecosystem for extensibility

## Sub-Agent Orchestration

This agent orchestrates specialized sub-agents for:
- Static analysis and linting
- Automated testing execution
- Documentation generation
- Infrastructure-as-code management
```

### Security-First Prompt Design

The Platform Agent's operating principles are ordered deliberately:

1. **Security First** is the top priority because developers have access to source code, credentials, and production systems. A compromised platform agent could expose the entire engineering infrastructure.

2. **Least Privilege** ensures the agent does not request more access than needed for each specific task. Reviewing a PR does not require production deployment access.

3. **Auditability** creates a complete trail of every code change, deployment, and infrastructure modification the agent facilitates.

4. **Human-in-the-Loop** prevents the agent from autonomously deploying to production. This is non-negotiable for enterprise deployments.

## Skills in Depth

### general_assistance

The broadest skill, covering code review, debugging, and general development tasks. Examples from the A2A card:

- **"Review this pull request for security issues"** -- The agent analyzes PR diffs for common vulnerability patterns, dependency issues, and security anti-patterns.
- **"Debug a failing integration test"** -- The agent reads test output, traces the failure, and suggests fixes.
- **"Generate boilerplate for a new microservice"** -- The agent scaffolds service templates following organizational standards.

### dev_rust_standards

Specific to Rust development workflows:

- **Coding standards**: Enforces formatting, naming conventions, error handling patterns
- **Linting**: Checks against clippy rules and custom organizational lints
- **Testing**: Validates test coverage, suggests test cases for untested paths
- **Architecture patterns**: Ensures modules follow the project's architectural guidelines

This skill references the Rust standards defined in `services/plugins/systemprompt-dev/skills/dev-rust-standards/SKILL.md`.

### dev_architecture_standards

Covers system-level architecture review:

- **Pattern compliance**: Checks that new code follows established architectural patterns
- **Anti-pattern detection**: Identifies circular dependencies, god objects, and other anti-patterns
- **Extension design validation**: Reviews new extensions against the platform's extension architecture

This skill references the architecture standards in `services/plugins/systemprompt-dev/skills/dev-architecture-standards/SKILL.md`.

## MCP Integration

The Platform Agent is the only agent with MCP server access:

```yaml
    metadata:
      mcpServers:
      - systemprompt
      skills:
      - general_assistance
      - dev_rust_standards
      - dev_architecture_standards
```

### Why the Platform Agent Has MCP Access

The `systemprompt` MCP server provides tools for managing the platform itself -- listing agents, checking service status, querying logs, and managing configurations. The Platform Agent needs this access because:

1. **Platform development**: Developers building on the platform need to inspect its state
2. **Debugging**: When something breaks, developers need to query logs and trace requests
3. **Automation**: CI/CD workflows need to interact with the platform programmatically
4. **Self-service**: Developers can check service health without asking an admin

### MCP Tool Discovery

The Platform Agent discovers available tools through the MCP protocol:

```bash
# List tools available to the platform agent
systemprompt plugins mcp tools systemprompt

# Check MCP server status
systemprompt plugins mcp status systemprompt

# View MCP server logs
systemprompt plugins mcp logs systemprompt
```

### MCP Tool Usage Patterns

The Platform Agent uses MCP tools for:

| Task | MCP Tool Category | Example |
|------|-------------------|---------|
| Service health check | Infrastructure | Query service status across the platform |
| Log analysis | Logging | Search logs for error patterns |
| Agent inspection | Administration | List agents, check configurations |
| Skill discovery | Core | List available skills, check skill content |
| Build operations | Build | Trigger builds, check build status |

### Security of MCP Access

MCP access is governed by:

- **OAuth scoping**: The `admin` scope on the platform agent's token limits which MCP tools are available
- **Tool-level authorization**: Individual MCP tools can enforce additional permission checks
- **Audit logging**: Every MCP tool call is logged with the agent context, tool name, parameters, and result
- **Rate limiting**: MCP operations are rate-limited at 200 req/s base rate

## Security Configuration

```yaml
    oauth:
      required: true
      scopes:
        - admin
      audience: a2a
```

### Why Admin Scope

The Platform Agent requires `admin` scope because:

- It has MCP server access that can query and modify platform state
- It handles source code which may contain sensitive business logic
- It can interact with CI/CD systems that deploy to production
- It accesses security scanning results that reveal vulnerabilities

### Human-in-the-Loop for Production

The system prompt mandates human approval for production deployments. This is enforced through:

1. **Prompt-level instruction**: The agent is instructed to always ask for explicit confirmation
2. **MCP tool design**: Production deployment tools require a confirmation parameter
3. **Hook-based verification**: `PostToolUse` hooks can validate that confirmation was obtained before allowing deployment to proceed
4. **Audit trail**: All deployment-related interactions are logged with the approval chain

## Operating Principles in Depth

### Security First

The Platform Agent never:

- Displays credentials, API keys, or secrets in responses
- Generates code with known vulnerability patterns (SQL injection, XSS, etc.)
- Bypasses authentication or authorization in generated code
- Accesses production data without explicit authorization

### Least Privilege

For each task, the agent requests only the minimum permissions:

- Code review: Read access to the repository, no write access
- Debugging: Read access to logs, no access to production data
- Deployment: Write access to the specific deployment target, not to all environments
- Security scanning: Read access to dependency manifests, no access to source code beyond what is needed

### Auditability

Every action is logged:

| Action | What is Logged |
|--------|---------------|
| Code review | Files reviewed, issues found, suggestions made |
| Code generation | Prompt, generated code, language/framework |
| Deployment | Target environment, version, approval chain |
| Security scan | Scanner used, findings, severity levels |
| MCP tool call | Tool name, parameters, result, execution time |

### Human-in-the-Loop

The agent always asks for human confirmation before:

- Deploying to production or staging
- Modifying infrastructure (scaling, configuration changes)
- Deleting resources (repositories, services, data)
- Granting or revoking access permissions

## Sub-Agent Orchestration

### Static Analysis and Linting

Delegates to specialized sub-agents that run:

- Clippy (Rust), ESLint (JavaScript), Pylint (Python)
- Custom organizational linting rules
- Complexity analysis (cyclomatic, cognitive)

### Automated Testing

Orchestrates test execution across:

- Unit tests for individual modules
- Integration tests for service interactions
- End-to-end tests for user workflows
- Performance tests for latency-sensitive paths

### Documentation Generation

Generates and updates:

- API documentation from code annotations
- Architecture decision records (ADRs)
- Runbook updates based on incident patterns
- README files for new modules

### Infrastructure-as-Code

Manages IaC workflows:

- Terraform plan review and approval
- Kubernetes manifest generation
- Helm chart configuration
- Cloud resource provisioning

