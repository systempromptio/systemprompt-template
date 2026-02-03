---
title: "Config Service"
description: "The config service aggregates all service configurations into a unified hub, managing includes, global settings, and startup validation."
author: "SystemPrompt Team"
slug: "services/config"
keywords: "config, configuration, includes, aggregation, validation, settings, services"
image: "/files/images/docs/services-config.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Config Service

**TL;DR:** The config service is the central aggregation point that includes all other service configurations. It defines global settings like port ranges and validation modes, and ensures all configuration is validated at startup.

## The Problem

A SystemPrompt application has many services, each with its own configuration file. Agents, skills, AI providers, MCP servers, scheduler jobs, and web settings all need to work together. Without coordination, these configurations could conflict or become inconsistent.

The config service solves this by providing a single entry point that aggregates all service configurations. When the application starts, it reads the config service first, which then pulls in all other services through includes. This creates a unified configuration hub that can be validated as a whole.

## How Config Works

The config service lives at `services/config/config.yaml`. It uses the includes pattern to pull in configurations from other service directories. The application loads this file first, then follows the includes to build the complete configuration tree.

The includes pattern has several benefits. Each service keeps its configuration in its own directory, making it easy to find and modify. The config service just references these files rather than duplicating their contents. Changes to individual services automatically appear when the application reloads.

When the application starts, it validates the entire configuration tree. If any service has invalid configuration, the application fails to start with a clear error message. This catches problems early rather than at runtime.

## Configuration Location

The config service file is located at:

```
services/config/config.yaml
```

This file aggregates all other services through relative path includes.

## Includes Pattern

The includes section lists all service configuration files that should be aggregated:

<details>
<summary>Example includes configuration</summary>

```yaml
includes:
  - ../agents/welcome.yaml
  - ../skills/config.yaml
  - ../ai/config.yaml
  - ../web/config.yaml
  - ../scheduler/config.yaml
  - ../mcp/systemprompt.yaml
```

</details>

Each include path is relative to the config.yaml file. The system resolves these paths and loads each file in order. Later files can override settings from earlier ones if there are conflicts.

To add a new agent, create its YAML file in `services/agents/` and add an include line. To add a new MCP server, create its YAML file in `services/mcp/` and add an include line. The pattern is consistent across all service types.

## Global Settings

The settings section defines application-wide configuration that applies to all services:

<details>
<summary>Settings configuration</summary>

```yaml
settings:
  agentPortRange: [9000, 9999]
  mcpPortRange: [5000, 5999]
  autoStartEnabled: true
  validationStrict: true
  schemaValidationMode: "skip"
```

</details>

### Port Ranges

The `agentPortRange` and `mcpPortRange` settings define which ports agents and MCP servers can use. When an agent or MCP server starts, it picks an available port within its range.

The default ranges are:
- Agents: 9000-9999
- MCP servers: 5000-5999

These ranges prevent port conflicts between services and make firewall configuration predictable.

### Auto Start

When `autoStartEnabled` is true, agents and MCP servers start automatically when the application starts. When false, you must start them manually through the CLI or API.

### Validation Modes

The `validationStrict` setting controls how the application handles configuration errors. When true, any validation error prevents startup. When false, the application logs warnings but continues with valid configuration.

The `schemaValidationMode` setting controls JSON schema validation:
- `strict` - All configuration must match schemas exactly
- `warn` - Schema mismatches log warnings but allow startup
- `skip` - Schema validation is disabled

During development, you might use `warn` or `skip` for faster iteration. In production, use `strict` to catch configuration errors early.

## Empty Service Collections

The config file includes empty placeholder objects for agents and mcp_servers:

```yaml
agents: {}
mcp_servers: {}
```

These placeholders are populated at runtime when the includes are processed. They provide a place to define inline agents or servers if needed, though using separate files through includes is the recommended pattern.

## Validating Configuration

The application validates configuration at startup. You can also validate manually:

```bash
# Check configuration syntax
systemprompt infra services validate

# View loaded configuration
systemprompt infra services show
```

Common validation errors include:
- Missing required fields in agent or MCP definitions
- Invalid port numbers outside configured ranges
- Circular includes (file A includes file B which includes file A)
- YAML syntax errors in included files

## Service Relationships

The config service is the foundation that makes all other services work together:

- **Agents** are loaded through includes and started based on autoStartEnabled
- **Skills** are loaded and made available to agents
- **AI providers** are loaded and configured for agent use
- **MCP servers** are loaded and started within the configured port range
- **Scheduler jobs** are loaded and scheduled according to their cron expressions
- **Web configuration** is loaded and applied to the web interface

Changes to the config service require an application restart since it is the first file loaded. Changes to included files may be hot-reloaded depending on the service type.

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt admin config show` | Show configuration overview |
| `systemprompt admin config rate-limits` | Rate limit configuration |
| `systemprompt admin config server` | Server configuration |
| `systemprompt admin config runtime` | Runtime configuration |
| `systemprompt admin config security` | Security configuration |
| `systemprompt admin config paths` | Paths configuration |

See `systemprompt admin config <command> --help` for detailed options.

## Troubleshooting

**Application fails to start with validation error** -- Check the error message for the specific file and field causing the problem. Fix the configuration and restart.

**Includes not being loaded** -- Verify the paths are correct and relative to config.yaml. Check that included files exist and have valid YAML syntax.

**Port conflicts on startup** -- Adjust the port ranges or ensure no other processes are using ports in the configured ranges.