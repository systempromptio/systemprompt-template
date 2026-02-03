---
title: "Runtime Configuration"
description: "Environment type, logging levels, output format, and interactive mode settings."
author: "SystemPrompt Team"
slug: "config/runtime"
keywords: "runtime, environment, logging, output, format, color"
image: "/files/images/docs/config-runtime.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Runtime Configuration

Runtime settings control environment behavior, logging verbosity, output format, and interactive prompts.

## Configuration

```yaml
# .systemprompt/profiles/local/profile.yaml
runtime:
  environment: development
  log_level: verbose
  output_format: text
  no_color: false
  non_interactive: false
```

## Fields

| Field | Default | Description |
|-------|---------|-------------|
| `environment` | `development` | Environment type |
| `log_level` | `normal` | Logging verbosity |
| `output_format` | `text` | CLI output format |
| `no_color` | `false` | Disable colored output |
| `non_interactive` | `false` | Disable interactive prompts |

## Environment Types

| Environment | Description |
|-------------|-------------|
| `development` | Local development |
| `test` | Test suites |
| `staging` | Pre-production |
| `production` | Production deployment |

## Log Levels

| Level | Shows | Use Case |
|-------|-------|----------|
| `quiet` | Errors only | Test suites |
| `normal` | Info and above | Production |
| `verbose` | Debug and above | Development |
| `debug` | Everything | Troubleshooting |

## Output Formats

| Format | Use Case |
|--------|----------|
| `text` | Human interaction |
| `json` | Log aggregation, machine parsing |
| `yaml` | Config inspection |

## Environment-Specific Examples

### Development

```yaml
runtime:
  environment: development
  log_level: verbose
  output_format: text
```

### Production

```yaml
runtime:
  environment: production
  log_level: normal
  output_format: json
  no_color: true
  non_interactive: true
```

### CI/CD

```yaml
runtime:
  environment: staging
  log_level: normal
  output_format: json
  no_color: true
  non_interactive: true
```

## Environment Variables

| Variable | Effect |
|----------|--------|
| `NO_COLOR` | Disables colored output |
| `CI` | Enables non-interactive mode |

See the [Runtime Configuration Playbook](/playbooks/config-runtime) for detailed technical information.