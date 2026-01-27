---
title: "Cloud Flow Checklist Playbook"
description: "Manual verification checklist for complete cloud setup flow."
keywords:
  - cloud
  - checklist
  - verification
  - flow
---

# Cloud Flow Test Checklist

Manual verification checklist for the complete cloud setup flow.

> **Help**: `{ "command": "playbook cloud" }` via `systemprompt_help`

---

## Prerequisites

- [ ] Fresh project directory (or use `--force` flags)
- [ ] Docker installed and running
- [ ] Valid SystemPrompt Cloud account
- [ ] At least one AI API key (Anthropic, OpenAI, or Gemini)

---

## Phase 1: Authentication

### `just login`

**Run:**
```bash
just login
```

**Verify:**
- [ ] Browser opens for OAuth (GitHub or Google)
- [ ] After auth, CLI shows "Logged in successfully"
- [ ] User email displayed correctly

**Check files created:**
```bash
cat .systemprompt/credentials.json
```

- [ ] File exists at `.systemprompt/credentials.json`
- [ ] Contains `api_token` (non-empty)
- [ ] Contains `api_url` (matches environment)
- [ ] Contains `user_email` (matches your email)

---

## Phase 2: Tenant Creation

### `just tenant`

**Run:**
```bash
just tenant
```

**Verify:**
- [ ] Shows list of existing tenants (if any)
- [ ] Option to create new tenant
- [ ] If creating: plan selection works
- [ ] If creating: region selection works
- [ ] Selected tenant displayed

**Check files:**
```bash
cat .systemprompt/tenants.json
```

- [ ] File exists at `.systemprompt/tenants.json`
- [ ] Contains `selected` field with tenant ID
- [ ] Contains `tenants` array with at least one entry

---

## Phase 3: Project Initialization

### `just init`

**Run:**
```bash
just init
```

**Verify:**
- [ ] No errors during execution
- [ ] Success message displayed

**Check files created:**
```bash
ls -la services/
ls -la .systemprompt/
```

- [ ] `services/` directory exists
- [ ] `services/agents/` exists
- [ ] `services/content/` exists
- [ ] `services/skills/` exists
- [ ] `.gitignore` updated with `.systemprompt/`

---

## Phase 4: Profile Creation

### `just configure`

**Run:**
```bash
just configure
```

**Verify wizard prompts:**
- [ ] Environment selection (Local / Production / Both)
- [ ] Project name prompt
- [ ] Database configuration prompt
- [ ] API key prompts (at least one required)
- [ ] Docker setup prompt (for local)

**Check files created:**
```bash
ls -la .systemprompt/profiles/local/
cat .systemprompt/profiles/local/profile.yml
cat .systemprompt/profiles/local/secrets.json
```

- [ ] `.systemprompt/profiles/local/` exists
- [ ] `profile.yml` exists and is valid YAML
- [ ] `secrets.json` exists
- [ ] `docker-compose.yml` exists (for local)

---

## Phase 5: Database Setup

### `just db-up`

**Run:**
```bash
just db-up
```

**Verify:**
- [ ] Docker container starts without errors
- [ ] Container is healthy

**Check:**
```bash
docker ps | grep postgres
```

- [ ] Container running
- [ ] "database system is ready" in logs

---

## Phase 6: Migrations

### `just migrate`

**Run:**
```bash
export SYSTEMPROMPT_PROFILE=.systemprompt/profiles/local/profile.yml
just migrate
```

**Verify:**
- [ ] Migrations run without errors
- [ ] "Migrations complete" message

---

## Phase 7: Content Sync

### `just sync`

**Run:**
```bash
just sync content
just sync skills
```

**Verify:**
- [ ] Content sync completes without errors
- [ ] Skills sync completes without errors
- [ ] Item counts displayed

---

## Phase 8: Local Server

### `just start`

**Run:**
```bash
just start
```

**Verify:**
- [ ] Server starts without errors
- [ ] Listening on expected port (usually 8080)

**Check:**
```bash
curl http://localhost:8080/api/v1/health
```

- [ ] Health endpoint returns 200 OK

---

## Phase 9: Deployment (Optional)

### `just deploy`

**Prerequisites:**
- [ ] Production profile exists
- [ ] `cloud.enabled: true` in profile
- [ ] Tenant has active subscription

**Run:**
```bash
export SYSTEMPROMPT_PROFILE=.systemprompt/profiles/production/profile.yml
just deploy
```

**Verify:**
- [ ] Docker image builds successfully
- [ ] Image pushes to registry
- [ ] Deployment triggers
- [ ] Deployment URL displayed

**Check:**
```bash
just status
curl https://<hostname>/api/v1/health
```

- [ ] Status shows "running"
- [ ] Health endpoint accessible

---

## Cleanup Commands

```bash
# Stop local database
just db-down

# Remove all Docker volumes (data loss!)
docker compose -f .systemprompt/profiles/local/docker-compose.yml down -v

# Clear credentials (logout)
just logout
rm -rf .systemprompt/
```

---

## Common Issues

### "Not logged in"
```bash
just login
```

### "No tenant configured"
```bash
just tenant
```

### "Profile not found"
```bash
export SYSTEMPROMPT_PROFILE=.systemprompt/profiles/local/profile.yml
```

### "Database connection failed"
```bash
just db-up
just db-logs
```

### "Cloud features disabled"
Edit profile.yml:
```yaml
cloud:
  enabled: true
```

---

## File State Summary

After complete flow, verify:

```
.systemprompt/
├── credentials.json          # Has api_token, api_url, user_email
├── tenants.json              # Has selected tenant + list
├── Dockerfile                # Valid Dockerfile for deploy
└── profiles/
    ├── local/
    │   ├── profile.yml       # Valid YAML config
    │   ├── secrets.json      # Has DATABASE_URL + API keys
    │   └── docker-compose.yml # Defines postgres service
    └── production/           (if created)
        ├── profile.yml       # Has cloud.enabled: true
        └── secrets.json      # Has production secrets
```

---

## Quick Reference

| Phase | Command | Verify |
|-------|---------|--------|
| 1 | `just login` | credentials.json created |
| 2 | `just tenant` | tenants.json created |
| 3 | `just init` | services/ created |
| 4 | `just configure` | profiles/ created |
| 5 | `just db-up` | Container running |
| 6 | `just migrate` | Tables created |
| 7 | `just sync` | Data synced |
| 8 | `just start` | Server running |
| 9 | `just deploy` | Deployed (optional) |
