---
title: "Runtime Configuration"
description: "Configure environment type, logging, and output format."
keywords:
  - runtime
  - environment
  - logging
  - output
  - format
  - color
  - interactive
category: config
---

# Runtime Configuration

Configure environment type, logging, and output format.

> **Help**: `{ "command": "admin config runtime show" }` via `systemprompt_help`
> **Requires**: Profile configured -> See [Profiles Playbook](../profiles/index.md)

RuntimeConfig controls environment type, logging, output format, and interactive behavior.

---

## RuntimeConfig Struct

**Source**: `crates/shared/models/src/profile/runtime.rs:6-22`

```rust
pub struct RuntimeConfig {
    #[serde(default)]
    pub environment: Environment,              // Default: Development
    #[serde(default)]
    pub log_level: LogLevel,                   // Default: Normal
    #[serde(default)]
    pub output_format: OutputFormat,           // Default: Text
    #[serde(default)]
    pub no_color: bool,                        // Default: false
    #[serde(default)]
    pub non_interactive: bool,                 // Default: false
}
```

### Field Details

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `environment` | Environment | `Development` | Environment type |
| `log_level` | LogLevel | `Normal` | Logging verbosity |
| `output_format` | OutputFormat | `Text` | CLI output format |
| `no_color` | bool | `false` | Disable colored output |
| `non_interactive` | bool | `false` | Disable interactive prompts |

---

## Environment Type

**Source**: `crates/shared/models/src/profile/runtime.rs`

```rust
pub enum Environment {
    #[default]
    Development,
    Test,
    Staging,
    Production,
}
```

### Environment Methods

```rust
impl Environment {
    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub fn is_test(&self) -> bool {
        matches!(self, Environment::Test)
    }
}
```

### Configuration

```yaml
runtime:
  environment: development  # development, test, staging, production
```

### Environment Behaviors

| Environment | Rate Limits | Validation | Logging |
|-------------|-------------|------------|---------|
| `development` | Often disabled | Warn mode | Verbose |
| `test` | Disabled | Skip mode | Minimal |
| `staging` | Enabled | Strict mode | Normal |
| `production` | Enabled | Strict mode | JSON format |

---

## Log Level

**Source**: `crates/shared/models/src/profile/runtime.rs`

```rust
pub enum LogLevel {
    Quiet,       // Only errors
    #[default]
    Normal,      // Info and above
    Verbose,     // Debug and above
    Debug,       // All logs (trace)
}
```

### Tracing Filter Mapping

```rust
impl LogLevel {
    pub fn as_tracing_filter(&self) -> &'static str {
        match self {
            LogLevel::Quiet => "error",
            LogLevel::Normal => "info",
            LogLevel::Verbose => "debug",
            LogLevel::Debug => "trace",
        }
    }
}
```

### Configuration

```yaml
runtime:
  log_level: verbose  # quiet, normal, verbose, debug
```

### Log Level Reference

| Level | Tracing | Shows |
|-------|---------|-------|
| `quiet` | `error` | Only errors |
| `normal` | `info` | Info, warnings, errors |
| `verbose` | `debug` | Debug, info, warnings, errors |
| `debug` | `trace` | Everything including trace |

### Recommended Settings

| Environment | Log Level | Reason |
|-------------|-----------|--------|
| Development | `verbose` or `debug` | Maximum visibility |
| Test | `quiet` | Clean test output |
| Staging | `normal` | Standard visibility |
| Production | `normal` | Balanced |

---

## Output Format

**Source**: `crates/shared/models/src/profile/runtime.rs`

```rust
pub enum OutputFormat {
    #[default]
    Text,    // Human-readable text
    Json,    // JSON format
    Yaml,    // YAML format
}
```

### Configuration

```yaml
runtime:
  output_format: text  # text, json, yaml
```

### Format Use Cases

| Format | Use Case | Example Output |
|--------|----------|----------------|
| `text` | Human interaction | Formatted tables, colors |
| `json` | Machine parsing | `{"status": "ok", ...}` |
| `yaml` | Config inspection | `status: ok\n...` |

### Production Recommendations

```yaml
# Production: JSON for log aggregation
runtime:
  environment: production
  output_format: json
  log_level: normal
```

---

## Color Output

### Disable Colors

```yaml
runtime:
  no_color: true
```

### When to Disable

- CI/CD pipelines without color support
- Log aggregation systems
- Piping output to files
- Accessibility requirements

### Automatic Detection

Colors are also disabled when:
- `NO_COLOR` environment variable is set
- Output is not a TTY

---

## Interactive Mode

### Disable Interactive Prompts

```yaml
runtime:
  non_interactive: true
```

### When to Use

- CI/CD pipelines
- Automated scripts
- Docker containers
- Background processes

### Automatic Detection

Non-interactive mode is also enabled when:
- `CI` environment variable is set
- Stdin is not a TTY

---

## Complete Configuration Examples

### Local Development

```yaml
runtime:
  environment: development
  log_level: verbose
  output_format: text
  no_color: false
  non_interactive: false
```

### Test Environment

```yaml
runtime:
  environment: test
  log_level: quiet
  output_format: text
  no_color: true
  non_interactive: true
```

### CI/CD Pipeline

```yaml
runtime:
  environment: staging
  log_level: normal
  output_format: json
  no_color: true
  non_interactive: true
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

---

## Environment Variables

When using `Profile::from_env()`:

| Env Variable | Maps To | Values |
|--------------|---------|--------|
| `SYSTEMPROMPT_ENV` | `environment` | `development`, `test`, `staging`, `production` |
| `SYSTEMPROMPT_LOG_LEVEL` | `log_level` | `quiet`, `normal`, `verbose`, `debug` |
| `SYSTEMPROMPT_OUTPUT_FORMAT` | `output_format` | `text`, `json`, `yaml` |
| `NO_COLOR` | `no_color` | Any value = true |
| `CI` | `non_interactive` | Any value = true |

### Example

```bash
export SYSTEMPROMPT_ENV=production
export SYSTEMPROMPT_LOG_LEVEL=normal
export SYSTEMPROMPT_OUTPUT_FORMAT=json
export NO_COLOR=1
export CI=true
```

---

## Logging Initialization

Logging is initialized from RuntimeConfig in AppContext:

```rust
fn init_logging(runtime: &RuntimeConfig) -> Result<()> {
    let filter = runtime.log_level.as_tracing_filter();

    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_ansi(!runtime.no_color)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
```

---

## Troubleshooting

**"Too much log output"**
- Reduce log level: `quiet` or `normal`
- Use JSON format for filtering

**"Not enough log output"**
- Increase log level: `verbose` or `debug`
- Check environment isn't `test`

**"Colors garbled"**
- Set `no_color: true`
- Set `NO_COLOR=1` environment variable
- Check terminal supports ANSI colors

**"Interactive prompts hanging"**
- Set `non_interactive: true`
- Set `CI=1` environment variable
- Ensure stdin is available

**"JSON parsing failed"**
- Verify `output_format: json` is set
- Check for mixed output (non-JSON in logs)

---

## Quick Reference

### Log Levels

| Level | Filter | Verbosity |
|-------|--------|-----------|
| `quiet` | `error` | Least |
| `normal` | `info` | Default |
| `verbose` | `debug` | More |
| `debug` | `trace` | Most |

### Output Formats

| Format | Human | Machine |
|--------|-------|---------|
| `text` | Yes | No |
| `json` | No | Yes |
| `yaml` | Partial | Yes |

### Environment Settings

| Environment | Typical Config |
|-------------|----------------|
| Development | verbose, text, colors on, interactive |
| Test | quiet, text, colors off, non-interactive |
| Staging | normal, json, colors off, non-interactive |
| Production | normal, json, colors off, non-interactive |
