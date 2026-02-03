---
title: "Schema Extension"
description: "Add database schemas and migrations to your extension."
author: "SystemPrompt Team"
slug: "extensions/traits/schema-extension"
keywords: "schema, database, migrations, sql"
image: ""
kind: "reference"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Schema Extension

Extensions add database tables via the `schemas()` and `migrations()` methods.

## Schema Definition

```rust
fn schemas(&self) -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::inline("users", include_str!("../schema/001_users.sql")),
        SchemaDefinition::file("sessions", "schema/002_sessions.sql"),
    ]
}
```

### Inline vs File

```rust
// Embed SQL at compile time
SchemaDefinition::inline("table_name", include_str!("../schema/table.sql"))

// Load from file at runtime
SchemaDefinition::file("table_name", "schema/table.sql")
```

### Required Columns

Validate that columns exist after schema creation:

```rust
SchemaDefinition::inline("users", include_str!("../schema/users.sql"))
    .with_required_columns(vec!["id".into(), "email".into(), "created_at".into()])
```

## Migration Weight

Controls execution order. Lower values run first:

```rust
fn migration_weight(&self) -> u32 {
    10  // Runs before extensions with higher weights
}
```

Typical weights:
- `1-10`: Core infrastructure (database, users)
- `10-50`: Domain extensions (content, files)
- `50-100`: Feature extensions
- `100+`: Optional/plugin extensions

## Versioned Migrations

For schema evolution after initial deployment:

```rust
fn migrations(&self) -> Vec<Migration> {
    vec![
        Migration::new(1, "add_email_column",
            "ALTER TABLE users ADD COLUMN IF NOT EXISTS email TEXT"),
        Migration::new(2, "add_email_index",
            "CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)"),
    ]
}
```

Each migration runs once, tracked by version number.

## SQL Patterns

Use idempotent patterns:

```sql
-- Tables
CREATE TABLE IF NOT EXISTS my_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_my_items_name ON my_items(name);

-- Columns (in migrations)
ALTER TABLE my_items ADD COLUMN IF NOT EXISTS description TEXT;
```

## Typed Extension

For compile-time type safety:

```rust
use systemprompt::extension::prelude::SchemaExtensionTyped;

impl SchemaExtensionTyped for MyExtension {
    fn schemas(&self) -> Vec<SchemaDefinitionTyped> {
        vec![
            SchemaDefinitionTyped::inline("users", include_str!("../schema/users.sql")),
        ]
    }

    fn migration_weight(&self) -> u32 {
        50
    }
}
```