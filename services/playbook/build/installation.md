---
title: "Installation Playbook"
description: "Interactive setup steps for SystemPrompt from the template."
keywords:
  - installation
  - setup
  - template
  - configuration
category: build
---

# Installation

Install SystemPrompt from the template repository.

> **Requires**: Rust 1.75+, Git, Docker (for local PostgreSQL) OR systemprompt.io Cloud account
> **Note**: These are interactive commands requiring terminal access. See [Justfile Playbook](../cli/justfile.md)

---

## Prerequisites

Verify required tools:

```bash
rustc --version    # 1.75.0+
git --version
gh --version       # Optional but recommended
```

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

## Build

```bash
just build --release
```

First build auto-detects offline mode (no database yet).

---

## Login

**IMPORTANT: Manual command only. Do not run via agents.**

```bash
just login
```

Opens browser for GitHub or Google OAuth. Registration is:
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
  postgres:18-alpine
```

Create tenant:

```bash
just tenant create --database-url postgres://systemprompt:systemprompt@localhost:5432/systemprompt
```

**Option B: Cloud (managed PostgreSQL)**

```bash
just tenant create --region iad
```

---

## Create Profile

```bash
just profile create local
```

Creates `.systemprompt/profiles/local/profile.yaml` with your configuration.

---

## Run Migrations

```bash
just migrate
```

---

## Start Services

```bash
just start
```

---

## Verify

Visit `http://localhost:8080` to see the homepage.

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Build fails on macOS | Install OpenSSL: `brew install openssl` |
| Database connection refused | Verify PostgreSQL is running |
| Login fails | Ensure browser access, check network |
| Migrations fail | Check database URL, verify database exists |

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `just build --release` |
| Login | `just login` (manual only) |
| Create local tenant | `just tenant create --database-url postgres://...` |
| Create cloud tenant | `just tenant create --region iad` |
| Create profile | `just profile create <name>` |
| Run migrations | `just migrate` |
| Start services | `just start` |

-> See [Justfile Playbook](../cli/justfile.md) for all interactive commands.
