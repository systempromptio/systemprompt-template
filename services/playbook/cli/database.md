---
title: "Database Operations Playbook"
description: "Database queries, schema exploration, and administration."
author: "SystemPrompt"
slug: "cli-database"
keywords: "database, sql, queries, migrations, schema"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Database Operations Playbook

Database queries, schema exploration, and administration.

---

## Check Database Status

```json
{ "command": "infra db status" }
```

---

## Explore Schema

```json
{ "command": "infra db tables" }
{ "command": "infra db describe <table>" }
{ "command": "infra db indexes" }
{ "command": "infra db info" }
{ "command": "infra db size" }
```

---

## Query Data (Read-Only)

```json
{ "command": "infra db query \"SELECT COUNT(*) FROM users\"" }
{ "command": "infra db query \"SELECT * FROM users LIMIT 10\"" }
{ "command": "infra db count <table>" }
```

---

## Run Migrations

```json
{ "command": "infra db migrate" }
{ "command": "infra db validate" }
```

---

## Assign Admin Role

```json
{ "command": "infra db assign-admin user@example.com" }
```

---

## Cloud Database Operations

```json
{ "command": "cloud db status --profile <profile-name>" }
{ "command": "cloud db query --profile <profile-name> \"SELECT COUNT(*) FROM users\"" }
{ "command": "cloud db migrate --profile <profile-name>" }
```

---

## Schema Modifications

The `infra db execute` command runs DDL statements (ALTER TABLE, CREATE TABLE, etc.).

### Add a Column

```json
{ "command": "infra db execute \"ALTER TABLE <table> ADD COLUMN IF NOT EXISTS <column> TEXT\"" }
{ "command": "infra db execute \"ALTER TABLE <table> ADD COLUMN IF NOT EXISTS <column> INTEGER NOT NULL DEFAULT 0\"" }
```

### Modify a Column

```json
{ "command": "infra db execute \"ALTER TABLE <table> ALTER COLUMN <column> TYPE JSONB USING <column>::jsonb\"" }
{ "command": "infra db execute \"ALTER TABLE <table> ALTER COLUMN <column> DROP NOT NULL\"" }
{ "command": "infra db execute \"ALTER TABLE <table> ALTER COLUMN <column> SET DEFAULT 'pending'\"" }
```

### Drop a Column

```json
{ "command": "infra db execute \"ALTER TABLE <table> DROP COLUMN IF EXISTS <column>\"" }
```

### Create a Table

```json
{ "command": "infra db execute \"CREATE TABLE IF NOT EXISTS <table> (id TEXT PRIMARY KEY, name TEXT NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT NOW())\"" }
```

### Add Indexes

```json
{ "command": "infra db execute \"CREATE INDEX IF NOT EXISTS idx_<table>_<column> ON <table>(<column>)\"" }
{ "command": "infra db execute \"CREATE UNIQUE INDEX IF NOT EXISTS idx_<table>_<column> ON <table>(<column>)\"" }
```

### Add Foreign Key

```json
{ "command": "infra db execute \"ALTER TABLE <table> ADD CONSTRAINT fk_<name> FOREIGN KEY (<column>) REFERENCES <other_table>(id) ON DELETE CASCADE\"" }
```

### Drop a Table

```json
{ "command": "infra db execute \"DROP TABLE IF EXISTS <table>\"" }
```

### Cloud Schema Modifications

```json
{ "command": "cloud db execute --profile <profile-name> \"ALTER TABLE <table> ADD COLUMN IF NOT EXISTS <column> TEXT\"" }
```

---

## Best Practices

1. **Always use IF EXISTS/IF NOT EXISTS** -- Prevents errors on repeated runs
2. **Test locally first** -- Run on local DB before cloud
3. **Check schema after** -- Use `infra db describe <table>` to verify changes
4. **Use transactions for multi-step** -- Wrap related changes

---

## Troubleshooting

**Connection failed** -- Run `infra db status`. Verify profile has correct `database_url` in secrets.

**Query timeout** -- Break into smaller queries with `LIMIT`.

**Permission denied** -- Assign admin role with `infra db assign-admin user@example.com`.

---

## Quick Reference

| Task | Command |
|------|---------|
| Check status | `infra db status` |
| List tables | `infra db tables` |
| Describe table | `infra db describe <table>` |
| Query data | `infra db query "<SQL>"` |
| Row count | `infra db count <table>` |
| Run migrations | `infra db migrate` |
| Database info | `infra db info` |
| Execute DDL | `infra db execute "<SQL>"` |
| Cloud query | `cloud db query --profile <name> "<SQL>"` |