---
title: "Error Handling"
description: "Error types for extension loading, configuration, and runtime."
author: "SystemPrompt Team"
slug: "extensions/internals/error-handling"
keywords: "errors, handling, loader, config, extensions"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Error Handling

The extension system defines two primary error types: `LoaderError` for extension loading and `ConfigError` for configuration validation.

## LoaderError

Errors during extension discovery, validation, and initialization:

```rust
#[derive(Debug, Error)]
pub enum LoaderError {
    #[error("Extension '{extension}' requires dependency '{dependency}' which is not registered")]
    MissingDependency {
        extension: String,
        dependency: String,
    },

    #[error("Extension with ID '{0}' is already registered")]
    DuplicateExtension(String),

    #[error("Failed to initialize extension '{extension}': {message}")]
    InitializationFailed {
        extension: String,
        message: String,
    },

    #[error("Failed to install schema for extension '{extension}': {message}")]
    SchemaInstallationFailed {
        extension: String,
        message: String,
    },

    #[error("Migration failed for extension '{extension}': {message}")]
    MigrationFailed {
        extension: String,
        message: String,
    },

    #[error("Configuration validation failed for extension '{extension}': {message}")]
    ConfigValidationFailed {
        extension: String,
        message: String,
    },

    #[error("Extension '{extension}' uses reserved API path '{path}'")]
    ReservedPathCollision {
        extension: String,
        path: String,
    },

    #[error("Extension '{extension}' has invalid base path '{path}': must start with /api/")]
    InvalidBasePath {
        extension: String,
        path: String,
    },

    #[error("Circular dependency detected: {chain}")]
    CircularDependency {
        chain: String,
    },
}
```

## Handling LoaderError

```rust
match ExtensionRegistry::discover().validate() {
    Ok(()) => {
        // Proceed with startup
    }
    Err(LoaderError::MissingDependency { extension, dependency }) => {
        eprintln!("Extension '{}' requires '{}' - ensure it's linked", extension, dependency);
        std::process::exit(1);
    }
    Err(LoaderError::CircularDependency { chain }) => {
        eprintln!("Circular dependency: {}", chain);
        eprintln!("Break the cycle by restructuring dependencies");
        std::process::exit(1);
    }
    Err(e) => {
        eprintln!("Extension error: {}", e);
        std::process::exit(1);
    }
}
```

## ConfigError

Errors during configuration validation:

```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Configuration key '{0}' not found")]
    NotFound(String),

    #[error("Invalid configuration value for '{key}': {message}")]
    InvalidValue {
        key: String,
        message: String,
    },

    #[error("Failed to parse configuration: {0}")]
    ParseError(String),

    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),
}
```

## Creating ConfigError

```rust
fn validate_config(&self, config: &JsonValue) -> Result<(), ConfigError> {
    // Key not found
    let api_key = config.get("api_key")
        .ok_or_else(|| ConfigError::NotFound("api_key".to_string()))?;

    // Invalid value
    let max_items = config.get("max_items")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| ConfigError::InvalidValue {
            key: "max_items".to_string(),
            message: "must be an integer".to_string(),
        })?;

    if max_items > 10000 {
        return Err(ConfigError::InvalidValue {
            key: "max_items".to_string(),
            message: "cannot exceed 10000".to_string(),
        });
    }

    // Parse error
    let url = config.get("api_url")
        .and_then(|v| v.as_str())
        .ok_or_else(|| ConfigError::ParseError("api_url must be a string".to_string()))?;

    url::Url::parse(url).map_err(|e| ConfigError::ParseError(e.to_string()))?;

    Ok(())
}
```

## MissingDependency

For typed dependency checking:

```rust
#[derive(Debug, Error)]
#[error("Missing dependency: {id} (required by {required_by})")]
pub struct MissingDependency {
    pub id: &'static str,
    pub required_by: &'static str,
}
```

## Extension Error Handling

Extensions should define their own error types:

```rust
use thiserror::Error;
use axum::http::StatusCode;
use systemprompt::extension::ExtensionError;

#[derive(Error, Debug)]
pub enum MyExtensionError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Validation failed: {0}")]
    Validation(String),

    #[error("External service error: {0}")]
    External(String),
}

impl MyExtensionError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NOT_FOUND",
            Self::Database(_) => "DATABASE_ERROR",
            Self::Validation(_) => "VALIDATION_ERROR",
            Self::External(_) => "EXTERNAL_ERROR",
        }
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::External(_) => StatusCode::BAD_GATEWAY,
        }
    }

    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Database(_) | Self::External(_))
    }
}
```

## Error Context

Add context to errors:

```rust
use anyhow::Context;

async fn load_extension(id: &str) -> Result<Arc<dyn Extension>, LoaderError> {
    let ext = registry.get(id)
        .ok_or_else(|| LoaderError::MissingDependency {
            extension: "current".to_string(),
            dependency: id.to_string(),
        })?;

    ext.validate()
        .context("validation failed")
        .map_err(|e| LoaderError::InitializationFailed {
            extension: id.to_string(),
            message: e.to_string(),
        })?;

    Ok(ext.clone())
}
```

## Debugging

### Enable Debug Logging

```bash
RUST_LOG=systemprompt=debug systemprompt serve
```

### Common Error Messages

**MissingDependency**:
```
Extension 'oauth' requires dependency 'users' which is not registered
```
→ Ensure 'users' extension is linked in `src/lib.rs`

**CircularDependency**:
```
Circular dependency detected: a -> b -> c -> a
```
→ Restructure dependencies to break the cycle

**ReservedPathCollision**:
```
Extension 'my-ext' uses reserved API path '/api/v1/users'
```
→ Change to a non-reserved path like `/api/v1/my-ext`

**SchemaInstallationFailed**:
```
Failed to install schema for extension 'users': relation "users" already exists
```
→ Use `CREATE TABLE IF NOT EXISTS`

**ConfigValidationFailed**:
```
Configuration validation failed for extension 'my-ext': api_url must use HTTPS
```
→ Fix the value in profile.yaml