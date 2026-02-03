---
title: "Installation Playbook"
description: "Machine-executable steps to install and configure SystemPrompt from the template."
author: "SystemPrompt"
slug: "build-installation"
keywords: "installation, setup, template, configuration"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Installation

Install SystemPrompt from the template repository.

> **Requires**: Rust 1.75+, Git, Docker (for local PostgreSQL) OR systemprompt.io Cloud account

---

## Prerequisites

Verify required tools:

```bash
rustc --version    # 1.75.0+
git --version
gh --version       # Optional but recommended
```

PostgreSQL is the only external dependency. Any PostgreSQL 15+ works.

---

## Clone Template

**Option A: GitHub CLI (Recommended)**

```bash
gh repo create my-ai --template systempromptio/systemprompt-template --clone --private
cd my-ai
```

**Option B: Manual Clone**

```bash
git clone https://github.com/systempromptio/systemprompt-template.git my-ai
cd my-ai
```

---

## Build Binary

```bash
SQLX_OFFLINE=true cargo build --release -p systemprompt-cli
```

The `SQLX_OFFLINE=true` flag is required for first build (no database yet).

---

## Login

**IMPORTANT: Manual command only. Do not run via agents.**

```bash
systemprompt cloud auth login
```

This opens a browser for GitHub or Google OAuth. Registration is:
- **Free** — No payment required
- **Required** — License grant depends on authentication
- **One-time** — Credentials persist in `.systemprompt/credentials.json`

---

## Create Tenant

Choose local or cloud deployment:

**Option A: Local PostgreSQL**

Start PostgreSQL in Docker:

```bash
docker run -d --name systemprompt-db \
  -e POSTGRES_DB=systemprompt \
  -e POSTGRES_USER=systemprompt \
  -e POSTGRES_PASSWORD=systemprompt \
  -p 5432:5432 \
  postgres:16
```

Create tenant with your database URL:

```bash
systemprompt cloud tenant create --database-url postgres://systemprompt:systemprompt@localhost:5432/systemprompt
```

Other database URL examples:

| Host | DATABASE_URL |
|------|--------------|
| Docker | `postgres://systemprompt:systemprompt@localhost:5432/systemprompt` |
| Neon | `postgres://user:pass@ep-xxx.us-east-2.aws.neon.tech/systemprompt` |
| Supabase | `postgres://postgres:pass@db.xxx.supabase.co:5432/postgres` |

**Option B: Cloud (managed PostgreSQL)**

```bash
systemprompt cloud tenant create --region iad
```

Provisions managed PostgreSQL in your chosen region.

---

## Create Profile

```bash
systemprompt cloud profile create local
```

This creates `.systemprompt/profiles/local/profile.yaml` with your configuration.

---

## Run Migrations

```bash
systemprompt infra db migrate
```

---

## Start Services

```bash
just start
```

Or via CLI:

```bash
systemprompt infra services start --all
```

---

## Verify Installation

```bash
systemprompt infra services status
systemprompt infra db status
systemprompt admin agents list
```

Visit `http://localhost:8080` to see the homepage.

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Build fails on macOS | Install OpenSSL: `brew install openssl` |
| Database connection refused | Verify PostgreSQL is running, check `DATABASE_URL` |
| Login fails | Ensure browser access, check network |
| Migrations fail | Check `DATABASE_URL` format, verify database exists |

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `SQLX_OFFLINE=true cargo build --release -p systemprompt-cli` |
| Login | `systemprompt cloud auth login` (manual only) |
| Create local tenant | `systemprompt cloud tenant create --database-url postgres://...` |
| Create cloud tenant | `systemprompt cloud tenant create --region iad` |
| Create profile | `systemprompt cloud profile create <name>` |
| Run migrations | `systemprompt infra db migrate` |
| Start services | `systemprompt infra services start --all` |
| Check status | `systemprompt infra services status` |