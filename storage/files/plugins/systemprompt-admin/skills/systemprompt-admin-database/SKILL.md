---
name: "Database Management"
description: "Explore schema, run queries, and manage migrations via the systemprompt CLI"
---

# Database Management

You manage the platform database using the systemprompt CLI. All operations go through the `infra db` domain.

## Schema Exploration

| Command | Purpose |
|---------|---------|
| `systemprompt infra db tables` | List all tables with row counts and sizes |
| `systemprompt infra db tables --filter <pattern>` | Filter tables by name pattern |
| `systemprompt infra db describe <table>` | Show table schema (columns, types, constraints) |
| `systemprompt infra db indexes` | List all indexes |
| `systemprompt infra db indexes --table <table>` | List indexes for a specific table |
| `systemprompt infra db count <table>` | Get row count for a table |
| `systemprompt infra db size` | Show database and table sizes |

## Database Status

| Command | Purpose |
|---------|---------|
| `systemprompt infra db info` | Show database information |
| `systemprompt infra db status` | Show database connection status |
| `systemprompt infra db validate` | Validate database schema against expected tables |

## Querying

| Command | Purpose |
|---------|---------|
| `systemprompt infra db query "<sql>"` | Run a read-only SQL query |
| `systemprompt infra db query "<sql>" --limit <n>` | Limit query results |
| `systemprompt infra db query "<sql>" --offset <n>` | Offset query results |
| `systemprompt infra db query "<sql>" --format <fmt>` | Output in specific format |

## Write Operations

| Command | Purpose |
|---------|---------|
| `systemprompt infra db execute "<sql>"` | Execute a write operation (INSERT, UPDATE, DELETE) |

## Migrations

| Command | Purpose |
|---------|---------|
| `systemprompt infra db migrate` | Run pending migrations |
| `systemprompt infra db migrations status` | Show migration status for all extensions |
| `systemprompt infra db migrations history <extension>` | Show migration history for an extension |

## Admin Shortcuts

| Command | Purpose |
|---------|---------|
| `systemprompt infra db assign-admin <user>` | Assign admin role to a user |
| `systemprompt infra db query "SELECT id, email, role FROM users"` | List users with roles |

## Standard Workflow

1. **Explore schema** -- list tables, describe the one you need
2. **Check status** -- verify database connectivity and schema validity
3. **Query data** -- use read-only queries to understand current state
4. **Modify if needed** -- use `execute` for write operations (with caution)
5. **Verify** -- query again to confirm changes

## Common Tasks

### Explore the Database

```bash
systemprompt infra db info
systemprompt infra db tables
systemprompt infra db size
systemprompt infra db describe users
```

### Run a Custom Query

```bash
systemprompt infra db query "SELECT COUNT(*) FROM users"
systemprompt infra db query "SELECT * FROM users ORDER BY created_at DESC LIMIT 10"
```

### Check Database Health

```bash
systemprompt infra db status
systemprompt infra db validate
systemprompt infra db migrations status
```

### Check and Run Migrations

```bash
systemprompt infra db migrations status
systemprompt infra db migrate
```

### Investigate a Table

```bash
systemprompt infra db describe <table>
systemprompt infra db indexes --table <table>
systemprompt infra db count <table>
systemprompt infra db query "SELECT * FROM <table> LIMIT 5"
```

## Important Notes

- `query` is read-only -- it cannot modify data
- `execute` can modify data and schema -- use with extreme caution
- Always back up before running migrations or schema modifications
- SQL queries use PostgreSQL syntax
- Wrap string values in single quotes in SQL
- Use `--help` on any subcommand for full flag reference
