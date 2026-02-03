---
title: "Config Extension"
description: "Add configuration namespaces and validation to your extension."
author: "SystemPrompt Team"
slug: "extensions/traits/config-extension"
keywords: "config, configuration, validation, schema"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Config Extension

Extensions add configuration via `config_prefix()`, `config_schema()`, and `validate_config()`.

## Configuration Prefix

```rust
fn config_prefix(&self) -> Option<&str> {
    Some("my_extension")
}
```

Configuration is loaded from `profile.yaml`:

```yaml
extensions:
  my_extension:
    enabled: true
    max_items: 100
    api_url: "https://api.example.com"
```

## Configuration Schema

Define JSON Schema for your configuration:

```rust
fn config_schema(&self) -> Option<JsonValue> {
    Some(json!({
        "type": "object",
        "properties": {
            "enabled": {
                "type": "boolean",
                "default": true
            },
            "max_items": {
                "type": "integer",
                "minimum": 1,
                "maximum": 10000,
                "default": 100
            },
            "api_url": {
                "type": "string",
                "format": "uri"
            }
        },
        "required": ["api_url"]
    }))
}
```

## Custom Validation

```rust
fn validate_config(&self, config: &JsonValue) -> Result<(), ConfigError> {
    // Check max_items limit
    if let Some(max) = config.get("max_items").and_then(|v| v.as_i64()) {
        if max > 10000 {
            return Err(ConfigError::InvalidValue {
                key: "max_items".into(),
                message: "Value cannot exceed 10000".into(),
            });
        }
    }

    // Check API URL is reachable
    if let Some(url) = config.get("api_url").and_then(|v| v.as_str()) {
        if !url.starts_with("https://") {
            return Err(ConfigError::InvalidValue {
                key: "api_url".into(),
                message: "API URL must use HTTPS".into(),
            });
        }
    }

    Ok(())
}
```

## ConfigError

```rust
pub enum ConfigError {
    NotFound(String),
    InvalidValue { key: String, message: String },
    ParseError(String),
    SchemaValidation(String),
}
```

## Typed Configuration

Use the type-state pattern for validated configuration:

```rust
use serde::Deserialize;
use std::path::PathBuf;

// Raw config from YAML
#[derive(Debug, Deserialize)]
pub struct MyConfigRaw {
    pub data_path: String,
    pub api_url: String,
    pub max_items: Option<u32>,
}

// Validated config with parsed types
#[derive(Debug, Clone)]
pub struct MyConfigValidated {
    pub data_path: PathBuf,
    pub api_url: url::Url,
    pub max_items: u32,
}

impl MyConfigValidated {
    pub fn validate(raw: MyConfigRaw, base_path: &Path) -> Result<Self, ConfigError> {
        let data_path = base_path.join(&raw.data_path);
        if !data_path.exists() {
            return Err(ConfigError::InvalidValue {
                key: "data_path".into(),
                message: format!("Path does not exist: {}", data_path.display()),
            });
        }

        let api_url = url::Url::parse(&raw.api_url)
            .map_err(|e| ConfigError::InvalidValue {
                key: "api_url".into(),
                message: e.to_string(),
            })?;

        Ok(Self {
            data_path,
            api_url,
            max_items: raw.max_items.unwrap_or(100),
        })
    }
}
```

## Typed Extension

```rust
use systemprompt::extension::prelude::ConfigExtensionTyped;

impl ConfigExtensionTyped for MyExtension {
    fn config_prefix(&self) -> &'static str {
        "my_extension"
    }

    fn validate_config(&self, config: &JsonValue) -> Result<(), ConfigError> {
        // Custom validation
        Ok(())
    }

    fn config_schema(&self) -> Option<JsonValue> {
        Some(json!({ /* schema */ }))
    }
}
```