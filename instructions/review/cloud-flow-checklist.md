# Cloud Flow Test Checklist

Manual verification checklist for the complete cloud setup flow.

---

## Prerequisites

- [ ] Fresh project directory (or use `--force` flags)
- [ ] Docker installed and running
- [ ] Valid SystemPrompt Cloud account
- [ ] At least one AI API key (Anthropic, OpenAI, or Gemini)

---

## Phase 1: Authentication

### `just login` (or `systemprompt cloud auth login`)

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
- [ ] Does NOT contain tenant fields (those go in tenants.json)

**Expected structure:**
```json
{
  "api_token": "sp_...",
  "api_url": "https://api.systemprompt.io",
  "user_email": "you@example.com"
}
```

---

## Phase 2: Tenant Creation

### `just tenant` (or `systemprompt cloud tenant create`)

**Run:**
```bash
just tenant
```

**Verify:**
- [ ] Shows list of existing tenants (if any)
- [ ] Option to create new tenant
- [ ] If creating: plan selection works
- [ ] If creating: region selection works
- [ ] If creating: Paddle checkout opens
- [ ] After checkout: tenant provisioning completes
- [ ] Selected tenant displayed

**Check files created/updated:**
```bash
cat .systemprompt/tenants.json
```

- [ ] File exists at `.systemprompt/tenants.json`
- [ ] Contains `selected` field with tenant ID
- [ ] Contains `tenants` array with at least one entry
- [ ] Selected tenant has `id`, `name`, `app_id`, `hostname`, `region`

**Expected structure:**
```json
{
  "selected": "ten_abc123",
  "tenants": [
    {
      "id": "ten_abc123",
      "name": "my-project",
      "app_id": "sp-my-project-abc",
      "hostname": "my-project.systemprompt.io",
      "region": "iad"
    }
  ]
}
```

---

## Phase 3: Project Initialization

### `just init` (or `systemprompt cloud init`)

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
- [ ] `services/scheduler/` exists (if applicable)
- [ ] `.gitignore` updated with `.systemprompt/`

---

## Phase 4: Profile Creation

### `just configure` (or `systemprompt cloud profile create local`)

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
cat .systemprompt/profiles/local/docker-compose.yml
```

**Profile directory:**
- [ ] `.systemprompt/profiles/local/` exists
- [ ] `profile.yml` exists and is valid YAML
- [ ] `secrets.json` exists
- [ ] `docker-compose.yml` exists

**profile.yml checks:**
- [ ] Contains `database.url` or references secrets
- [ ] Contains `paths.services`
- [ ] Contains `cloud.enabled` (true/false)

**secrets.json checks:**
- [ ] Contains `DATABASE_URL`
- [ ] Contains at least one AI API key
- [ ] All values are non-empty strings

**docker-compose.yml checks:**
- [ ] Defines `postgres` service
- [ ] Uses correct image (`postgres:18-alpine` or similar)
- [ ] Exposes port 5432
- [ ] Has healthcheck configured

**If production profile created:**
```bash
ls -la .systemprompt/profiles/production/
```

- [ ] `.systemprompt/profiles/production/` exists
- [ ] `profile.yml` exists
- [ ] `secrets.json` exists
- [ ] No `docker-compose.yml` (cloud-managed)

**Dockerfile check:**
```bash
cat .systemprompt/Dockerfile
```

- [ ] `.systemprompt/Dockerfile` exists
- [ ] Based on `debian:bookworm-slim` or similar
- [ ] Copies binary, services, web assets
- [ ] Exposes port 8080
- [ ] Has healthcheck

---

## Phase 5: Database Setup

### `just db-up` (or `docker compose ...`)

**Run:**
```bash
just db-up
# or: just db-up local
```

**Verify:**
- [ ] Docker container starts without errors
- [ ] Container is healthy

**Check:**
```bash
docker ps | grep postgres
docker compose -f .systemprompt/profiles/local/docker-compose.yml logs
```

- [ ] Container running
- [ ] No error logs
- [ ] "database system is ready" message

**Test connection:**
```bash
# Using psql if available:
psql "$(jq -r .DATABASE_URL .systemprompt/profiles/local/secrets.json)" -c "SELECT 1"
```

- [ ] Connection successful

---

## Phase 6: Migrations

### `just migrate` (or `systemprompt db migrate`)

**Run:**
```bash
export SYSTEMPROMPT_PROFILE=.systemprompt/profiles/local/profile.yml
just migrate
```

**Verify:**
- [ ] Migrations run without errors
- [ ] "Migrations complete" or similar message

**Check:**
```bash
# Connect to DB and verify tables exist
psql "$(jq -r .DATABASE_URL .systemprompt/profiles/local/secrets.json)" -c "\dt"
```

- [ ] Tables created (users, content, skills, etc.)

---

## Phase 7: Content Sync

### `just sync` (or `systemprompt cloud sync`)

**Run:**
```bash
just sync content
just sync skills
```

**Verify:**
- [ ] Content sync completes without errors
- [ ] Skills sync completes without errors
- [ ] Item counts displayed

**Check database:**
```bash
psql "$(jq -r .DATABASE_URL .systemprompt/profiles/local/secrets.json)" \
  -c "SELECT COUNT(*) FROM content"
psql "$(jq -r .DATABASE_URL .systemprompt/profiles/local/secrets.json)" \
  -c "SELECT COUNT(*) FROM skills"
```

- [ ] Content count matches files in `services/content/`
- [ ] Skills count matches files in `services/skills/`

---

## Phase 8: Local Server

### `just start` (or `systemprompt services start`)

**Run:**
```bash
export SYSTEMPROMPT_PROFILE=.systemprompt/profiles/local/profile.yml
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
- [ ] Response indicates healthy status

---

## Phase 9: Deployment (Optional)

### `just deploy` (or `systemprompt cloud deploy`)

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
- [ ] Deployment triggers on Fly.io
- [ ] Deployment URL displayed

**Check:**
```bash
just status
curl https://<hostname>/api/v1/health
```

- [ ] Status shows "running"
- [ ] Health endpoint accessible via public URL

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
just db-logs  # Check for errors
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
├── credentials.json          ✓ Has api_token, api_url, user_email
├── tenants.json              ✓ Has selected tenant + list
├── Dockerfile                ✓ Valid Dockerfile for deploy
└── profiles/
    ├── local/
    │   ├── profile.yml       ✓ Valid YAML config
    │   ├── secrets.json      ✓ Has DATABASE_URL + API keys
    │   └── docker-compose.yml ✓ Defines postgres service
    └── production/           (if created)
        ├── profile.yml       ✓ Has cloud.enabled: true
        └── secrets.json      ✓ Has production secrets
```
