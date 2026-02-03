---
title: "Add Database Schema"
description: "Add database tables and migrations to your extension."
author: "SystemPrompt"
slug: "build-02-library-extensions-add-schema"
keywords: "schema, database, migrations, sql"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Add Database Schema

Add database tables to your extension. Reference: `extensions/web/schema/` for examples.

> **Help**: `{ "command": "core playbooks show build_add-schema" }`

---

## Structure

```
extensions/my-extension/
└── schema/
    └── 001_tables.sql
```

---

## Write SQL

File: `schema/001_tables.sql`

```sql
CREATE TABLE IF NOT EXISTS my_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_my_items_name ON my_items(name);
```

---

## Register Schema

In `src/extension.rs`. See `extensions/web/src/extension.rs:45-55` for reference.

```rust
fn schemas(&self) -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::inline("my_items", include_str!("../schema/001_tables.sql")),
    ]
}

fn migration_weight(&self) -> u32 {
    50
}
```

---

## Add Migration

For schema changes after initial deployment. In `src/extension.rs`:

```rust
fn migrations(&self) -> Vec<Migration> {
    vec![
        Migration::new(1, "add_status",
            "ALTER TABLE my_items ADD COLUMN IF NOT EXISTS status TEXT DEFAULT 'active'"),
    ]
}
```

---

## SQL Patterns

| Pattern | SQL |
|---------|-----|
| Idempotent table | `CREATE TABLE IF NOT EXISTS my_table (...)` |
| Idempotent index | `CREATE INDEX IF NOT EXISTS idx_name ON table(column)` |
| Idempotent column | `ALTER TABLE my_table ADD COLUMN IF NOT EXISTS col TEXT` |
| Foreign key | `FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE` |

---

## Foreign Keys

```sql
CREATE TABLE IF NOT EXISTS my_items (
    id UUID PRIMARY KEY,
    user_id TEXT NOT NULL,
    CONSTRAINT fk_my_items_user
        FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);
```

---

## Extending Core Tables

Use companion tables, not ALTER:

```sql
CREATE TABLE IF NOT EXISTS content_metadata (
    id TEXT PRIMARY KEY,
    content_id TEXT NOT NULL UNIQUE,
    custom_field JSONB DEFAULT '{}',
    CONSTRAINT fk_content
        FOREIGN KEY (content_id) REFERENCES markdown_content(id) ON DELETE CASCADE
);
```

---

## Checklist

- [ ] Schema file in `schema/` directory
- [ ] File numbered: `001_name.sql`
- [ ] Uses `IF NOT EXISTS` patterns
- [ ] Indexes on query columns
- [ ] Foreign keys with `ON DELETE CASCADE`
- [ ] Registered in `schemas()`
- [ ] `migration_weight()` set appropriately

---

## Quick Reference

| Task | Command/Action |
|------|----------------|
| Run migrations | `systemprompt infra db migrate` |
| Check status | `systemprompt infra db status` |
| Query data | `systemprompt infra db query "SELECT * FROM my_items"` |

---

## Related

-> See [Create Library Extension](create-extension.md) for full extension setup
-> See [Schema Extension](../../documentation/extensions/traits/schema-extension.md) for trait reference
-> See [Rust Standards](../06-standards/rust-standards.md) for code style