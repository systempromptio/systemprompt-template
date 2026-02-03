---
title: "Database"
description: "Configure PostgreSQL database connection for SystemPrompt. One connection string is all you need."
author: "SystemPrompt Team"
slug: "config/database"
keywords: "database, postgresql, connection, configuration, DATABASE_URL"
image: "/files/images/docs/config-database.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Database

PostgreSQL is SystemPrompt's only external dependency. Configure it with a single `DATABASE_URL` connection string—the binary handles everything else.

## The Simple Truth

You need exactly one thing: a PostgreSQL connection string with credentials.

```
postgres://user:password@host:port/database
```

Store it in your secrets. The SystemPrompt binary uses this URL to manage your tenant's data. It doesn't matter where PostgreSQL runs or who hosts it.

## Profile Configuration

```yaml
# .systemprompt/profiles/local/profile.yaml
database:
  type: postgres
  external_db_access: true    # Allow connections from outside Docker
```

| Field | Default | Description |
|-------|---------|-------------|
| `type` | Required | Database type (`postgres` or `postgresql`) |
| `external_db_access` | `false` | Allow external database connections |

**Note**: Only PostgreSQL is supported. SQLite, MySQL, and other databases are not supported.

## DATABASE_URL Format

```
postgres://USERNAME:PASSWORD@HOST:PORT/DATABASE_NAME?sslmode=require
```

| Component | Description |
|-----------|-------------|
| `USERNAME` | Database user |
| `PASSWORD` | User password (URL-encoded if special chars) |
| `HOST` | Database server hostname |
| `PORT` | PostgreSQL port (default: 5432) |
| `DATABASE_NAME` | Database name |
| `sslmode` | `require` for production, `disable` for local |

### Examples

| Provider | DATABASE_URL |
|----------|--------------|
| Local Docker | `postgres://postgres:postgres@localhost:5432/systemprompt` |
| Local Install | `postgres://systemprompt:localdev@localhost:5432/systemprompt` |
| Neon | `postgres://user:pass@ep-xxx.us-east-2.aws.neon.tech/systemprompt?sslmode=require` |
| Supabase | `postgres://postgres:pass@db.xxx.supabase.co:5432/postgres?sslmode=require` |
| AWS RDS | `postgres://admin:pass@mydb.xxx.us-east-1.rds.amazonaws.com:5432/systemprompt?sslmode=require` |
| SystemPrompt Cloud | Managed automatically |

## Hosting Options

PostgreSQL can run anywhere. Choose based on your needs:

### Development

**Docker (Recommended)**

```bash
docker run -d \
  --name systemprompt-db \
  -e POSTGRES_DB=systemprompt \
  -e POSTGRES_USER=systemprompt \
  -e POSTGRES_PASSWORD=localdev \
  -p 5432:5432 \
  postgres:18-alpine
```

DATABASE_URL: `postgres://systemprompt:localdev@localhost:5432/systemprompt`

**Local Install**

```bash
# macOS
brew install postgresql@16
brew services start postgresql@16

# Ubuntu/Debian
sudo apt install postgresql-16
sudo systemctl start postgresql

# Create database
createdb systemprompt
```

### Production

| Option | Pros | Cons |
|--------|------|------|
| **SystemPrompt Cloud** | Zero config, automatic backups, included in hosting | Paid service |
| **Neon** | Free tier, serverless, auto-scaling | Cold starts on free tier |
| **Supabase** | Free tier, good dashboard | Shared resources on free |
| **AWS RDS** | Enterprise features, reliability | Complex setup, costs |
| **Self-hosted** | Full control | You manage it |

## Configuration Location

DATABASE_URL is stored in your tenant secrets, accessible via the profile:

```
.systemprompt/
├── profiles/
│   └── local/
│       ├── profile.yaml      # References secrets path
│       └── secrets.json      # Contains DATABASE_URL (gitignored)
└── tenants.json              # Tenant registry with credentials reference
```

When you run `systemprompt cloud tenant create --type local`, you're prompted for the DATABASE_URL. It's stored securely and never committed to git.

## Setting DATABASE_URL

**During tenant creation:**

```bash
systemprompt cloud tenant create --type local
# Prompts: Enter DATABASE_URL:
```

**After creation (edit secrets):**

```bash
# View current config
systemprompt cloud profile show local

# Secrets are in .systemprompt/profiles/local/secrets.json
```

## Verify Connection

```bash
# Check database status
systemprompt infra db status

# Test connection directly
systemprompt infra db query "SELECT 1"
```

Expected output:

```
Database: connected
Migrations: up to date
Tables: 42
```

## Run Migrations

Migrations create and update the database schema:

```bash
# Run pending migrations
systemprompt infra db migrate

# Check migration status
systemprompt infra db status
```

Migrations are idempotent—running them multiple times is safe.

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Connection refused | Check PostgreSQL is running, verify host/port |
| Authentication failed | Verify username/password, check URL encoding |
| Database does not exist | Create it: `createdb systemprompt` |
| SSL required | Add `?sslmode=require` to URL |
| Connection timeout | Check firewall rules, network connectivity |

### Special Characters in Password

URL-encode special characters in passwords:

| Character | Encoded |
|-----------|---------|
| `@` | `%40` |
| `:` | `%3A` |
| `/` | `%2F` |
| `#` | `%23` |
| `%` | `%25` |

Example: Password `my@pass:word` becomes `my%40pass%3Aword`