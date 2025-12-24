# Authentication & Cloud Setup

Complete guide to authenticating, provisioning tenants, and configuring your SystemPrompt project.

## Command Overview

| Command | Domain | Purpose |
|---------|--------|---------|
| `login` | Auth | Authenticate with SP Cloud |
| `logout` | Auth | Clear credentials |
| `whoami` | Auth | Show current user |
| `tenant` | Tenant | Create or select cloud tenant |
| `init` | Project | Initialize new project (.env + services) |
| `configure` | Profile | Generate profiles (local/production) |
| `secrets` | Secrets | Manage API keys |
| `migrate` | Database | Run database migrations |
| `sync` | Data | Sync content/skills to database |
| `deploy` | Cloud | Push to cloud |
| `status` | Cloud | Check deployment status |

---

## Flow Diagrams

### Complete Setup Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SETUP FLOWS                                     │
└─────────────────────────────────────────────────────────────────────────────┘

     LOCAL DEVELOPMENT                         CLOUD DEPLOYMENT
     ══════════════════                        ══════════════════

                                               ┌─────────────┐
                                               │   login     │
                                               │  (OAuth)    │
                                               └──────┬──────┘
                                                      │
                                                      ▼
                                               ┌─────────────┐
                                               │   tenant    │
                                               │ (checkout)  │
                                               └──────┬──────┘
                                                      │
     ┌─────────────┐                                  │
     │    init     │◄─────────────────────────────────┤
     │  (scaffold) │                                  │
     └──────┬──────┘                                  │
            │                                         ▼
            │                                  ┌─────────────┐
            │                                  │  configure  │
            │                                  │  (wizard)   │
            │                                  └──────┬──────┘
            │                                         │
            ▼                                         ▼
     ┌─────────────┐                           ┌─────────────┐
     │   secrets   │                           │   secrets   │
     │ (API keys)  │                           │ (API keys)  │
     └──────┬──────┘                           └──────┬──────┘
            │                                         │
            ▼                                         ▼
     ┌─────────────┐                           ┌─────────────┐
     │   migrate   │                           │   migrate   │
     │    (db)     │                           │    (db)     │
     └──────┬──────┘                           └──────┬──────┘
            │                                         │
            ▼                                         ▼
     ┌─────────────┐                           ┌─────────────┐
     │    sync     │                           │    sync     │
     │  (content)  │                           │  (content)  │
     └──────┬──────┘                           └──────┬──────┘
            │                                         │
            ▼                                         ▼
     ┌─────────────┐                           ┌─────────────┐
     │    start    │                           │   deploy    │
     └─────────────┘                           └─────────────┘
```

### Credential Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                            CREDENTIAL LIFECYCLE                               │
└──────────────────────────────────────────────────────────────────────────────┘

  ┌─────────┐      OAuth       ┌─────────────┐     API Call    ┌─────────────┐
  │  User   │ ───────────────► │  SP Cloud   │ ◄─────────────► │   Tenant    │
  └─────────┘                  └─────────────┘                 └─────────────┘
       │                              │                              │
       │ 1. login                     │ 2. token                     │
       ▼                              ▼                              ▼
  ┌─────────────────────────────────────────────────────────────────────────┐
  │                        credentials.json                                  │
  │  {                                                                       │
  │    "api_token": "sp_...",           ← From OAuth                        │
  │    "api_url": "https://...",        ← Environment                       │
  │    "user_email": "you@...",         ← From /me endpoint                 │
  │    "tenant_id": "ten_...",          ← From tenant command               │
  │    "tenant_name": "my-app",         ← From tenant command               │
  │    "app_id": "sp-my-app",           ← Fly app name                      │
  │    "hostname": "my-app.fly.dev"     ← Fly hostname                      │
  │  }                                                                       │
  └─────────────────────────────────────────────────────────────────────────┘
```

### Profile Generation Flow

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                           CONFIGURE WIZARD                                    │
└──────────────────────────────────────────────────────────────────────────────┘

                          ┌─────────────────┐
                          │    configure    │
                          └────────┬────────┘
                                   │
                    ┌──────────────┼──────────────┐
                    ▼              ▼              ▼
             ┌───────────┐  ┌───────────┐  ┌───────────┐
             │   Local   │  │   Both    │  │Production │
             │   Only    │  │           │  │   Only    │
             └─────┬─────┘  └─────┬─────┘  └─────┬─────┘
                   │              │              │
                   ▼              │              ▼
         ┌─────────────────┐      │     ┌─────────────────┐
         │ Docker Setup    │      │     │ Verify Tenant   │
         │ (PostgreSQL)    │      │     │ Subscription    │
         └────────┬────────┘      │     └────────┬────────┘
                  │               │              │
                  ▼               ▼              ▼
         ┌─────────────────────────────────────────────────┐
         │              Collect Configuration               │
         │  • Project name                                  │
         │  • Database credentials                          │
         │  • API keys (Anthropic, OpenAI, etc.)           │
         └─────────────────────────┬───────────────────────┘
                                   │
                    ┌──────────────┴──────────────┐
                    ▼                             ▼
         ┌─────────────────────┐       ┌─────────────────────┐
         │ local.profile.yml   │       │ production.profile  │
         │                     │       │      .yml           │
         └─────────────────────┘       └─────────────────────┘
                    │                             │
                    ▼                             ▼
         ┌─────────────────────┐       ┌─────────────────────┐
         │   secrets.json      │       │   (secrets in       │
         │   (local dev)       │       │    cloud KV)        │
         └─────────────────────┘       └─────────────────────┘
```

---

## The `.systemprompt` Directory

### Location

Credentials are **always project-specific**. Each project has its own tenant. Secrets are **per-profile** (environment-specific).

```
project-root/
└── .systemprompt/                      ← Project-level (gitignored)
    ├── credentials.json                ← Auth token + user info
    ├── tenants.json                    ← Tenant cache (selected tenant)
    ├── Dockerfile                      ← Application image (used by deploy)
    └── profiles/                       ← Environment-specific configs
        ├── local/
        │   ├── profile.yml             ← Runtime configuration
        │   ├── secrets.json            ← API keys + DATABASE_URL
        │   └── docker-compose.yml      ← Local services (PostgreSQL, etc.)
        ├── staging/
        │   ├── profile.yml
        │   ├── secrets.json
        │   └── docker-compose.yml      ← Staging services (optional)
        └── production/
            ├── profile.yml
            └── secrets.json            ← (no docker-compose - cloud-managed)
```

> **Important:** Never use global `~/.systemprompt/`. Each project connects to its own tenant with its own credentials.

### Structure Rationale

| Path | Scope | Purpose |
|------|-------|---------|
| `credentials.json` | Project | Auth token + user info (from login) |
| `tenants.json` | Project | Selected tenant + cached tenant list |
| `Dockerfile` | Project | Application image built once, deployed to any environment |
| `profiles/<env>/profile.yml` | Environment | Runtime configuration for that environment |
| `profiles/<env>/secrets.json` | Environment | API keys and secrets for that environment |
| `profiles/<env>/docker-compose.yml` | Environment | Local services for that environment (local/staging only) |

**Why separate credentials vs tenants?**
- **credentials.json**: Auth identity (who you are) — changes rarely
- **tenants.json**: Selected tenant (where you deploy) — may switch between tenants

**Why separate Dockerfile vs docker-compose?**
- **Dockerfile** at root: Same image for all envs. Build once → deploy anywhere.
- **docker-compose.yml** per profile: Different services per env (local needs postgres, staging needs redis, production uses cloud-managed).

Each profile folder is **self-contained** — everything needed to run that environment lives together.

### File Specifications

#### `credentials.json` — Cloud Authentication

**Created by:** `cloud auth login`
**Location:** `.systemprompt/credentials.json`

```json
{
  "api_token": "sp_live_abc123...",
  "api_url": "https://api.systemprompt.io",
  "user_email": "developer@example.com"
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `api_token` | Yes | OAuth access token |
| `api_url` | Yes | API endpoint URL |
| `user_email` | Yes | Authenticated user's email |

#### `tenants.json` — Tenant Selection

**Created by:** `cloud tenant create` or `cloud tenant list`
**Updated by:** `cloud tenant` (select)
**Location:** `.systemprompt/tenants.json`

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

| Field | Required | Description |
|-------|----------|-------------|
| `selected` | Yes | Currently active tenant ID |
| `tenants` | Yes | Cached list of user's tenants |
| `tenants[].id` | Yes | Tenant identifier |
| `tenants[].name` | Yes | Human-readable name |
| `tenants[].app_id` | For deploy | Fly.io application name |
| `tenants[].hostname` | For deploy | Public hostname |
| `tenants[].region` | Yes | Deployment region |

#### `Dockerfile` — Application Image

**Created by:** `configure` command (or manually)
**Used by:** `deploy` command
**Location:** `.systemprompt/Dockerfile`

Builds the production image for cloud deployment. Same image deploys to all environments — runtime config comes from profile.yml and secrets.json.

#### `secrets.json` — API Keys & Secrets (per profile)

**Created by:** `configure` or `secrets` command
**Location:** `.systemprompt/profiles/<env>/secrets.json`

Each environment has its own secrets file. This allows different API keys, database URLs, and configurations per environment.

```json
{
  "ANTHROPIC_API_KEY": "sk-ant-...",
  "OPENAI_API_KEY": "sk-...",
  "GEMINI_API_KEY": "AI...",
  "DATABASE_URL": "postgresql://...",
  "CUSTOM_SECRET": "value"
}
```

| Field | Required | Description |
|-------|----------|-------------|
| `ANTHROPIC_API_KEY` | One AI key required | Claude API access |
| `OPENAI_API_KEY` | One AI key required | GPT API access |
| `GEMINI_API_KEY` | One AI key required | Gemini API access |
| `DATABASE_URL` | Yes | PostgreSQL connection string (env-specific) |
| `*` | Optional | Any custom secrets |

**Example per-environment secrets:**

| Environment | DATABASE_URL | API Keys |
|-------------|--------------|----------|
| `local` | `postgresql://localhost:5432/myapp_dev` | Dev/test keys |
| `staging` | `postgresql://staging-db:5432/myapp` | Staging keys |
| `production` | `postgresql://prod-db:5432/myapp` | Production keys (or via cloud KV) |

#### `docker-compose.yml` — Local Services (per profile)

**Created by:** `configure` command
**Used by:** `just db-up`, `just db-down`
**Location:** `.systemprompt/profiles/<env>/docker-compose.yml`

Defines services needed for that environment (PostgreSQL, Redis, etc.). Production typically omits this file since it uses cloud-managed services.

---

## Directory State by Stage

### After `init`

```
project-root/
├── .env                        ← Environment variables (legacy, optional)
├── .gitignore                  ← Updated with .systemprompt/
└── services/
    ├── agents/
    │   └── default.yml
    ├── content/
    │   └── .gitkeep
    ├── scheduler/
    │   └── .gitkeep
    └── skills/
        └── .gitkeep
```

### After `cloud auth login`

```
project-root/
├── .systemprompt/
│   └── credentials.json        ← api_token, api_url, user_email
└── ...
```

### After `cloud tenant create`

```
project-root/
├── .systemprompt/
│   ├── credentials.json
│   └── tenants.json            ← selected + tenants list
└── ...
```

### After `cloud profile create local`

```
project-root/
├── .systemprompt/
│   ├── credentials.json
│   ├── tenants.json
│   ├── Dockerfile                  ← Generated for deployment
│   └── profiles/
│       └── local/
│           ├── profile.yml         ← Runtime configuration
│           ├── secrets.json        ← API keys + DATABASE_URL
│           └── docker-compose.yml  ← PostgreSQL container
└── ...
```

### After `cloud profile create production`

```
project-root/
├── .systemprompt/
│   ├── credentials.json
│   ├── tenants.json
│   ├── Dockerfile                  ← Same image for all envs
│   └── profiles/
│       ├── local/
│       │   ├── profile.yml
│       │   ├── secrets.json
│       │   └── docker-compose.yml
│       └── production/
│           ├── profile.yml         ← Cloud runtime configuration
│           └── secrets.json        ← Production secrets
└── ...
```

### After `migrate` + `sync`

```
project-root/
├── .systemprompt/
│   ├── credentials.json
│   ├── tenants.json
│   ├── Dockerfile
│   └── profiles/
│       └── local/
│           ├── profile.yml
│           ├── secrets.json
│           └── docker-compose.yml  ← Running via `just db-up`
└── services/
    ├── content/
    │   ├── blog/
    │   │   └── *.md                ← Synced to database
    │   └── legal/
    │       └── *.md                ← Synced to database
    └── skills/
        └── *.yml                   ← Synced to database
```

---

## Command Reference

### `login`

Authenticate with SystemPrompt Cloud via OAuth.

```bash
just login                    # Production (default)
just login staging            # Staging environment
```

**Flow:**
1. Opens browser for OAuth (GitHub or Google)
2. Local server receives callback
3. Exchanges code for token
4. Fetches user info from `/me`
5. Saves to `credentials.json`

**Produces:** `~/.systemprompt/credentials.json`

---

### `logout`

Clear saved credentials.

```bash
just logout
```

**Removes:** `credentials.json`

---

### `whoami`

Display current authenticated user and tenant.

```bash
just whoami
```

**Output:**
```
User: developer@example.com
Tenant: my-project (ten_abc123)
URL: https://my-project.systemprompt.io
```

---

### `tenant`

Create or select a cloud tenant.

```bash
just tenant                   # Interactive selection
just tenant --region syd      # Create in Sydney
```

**Flow (new tenant):**
1. Fetch available plans
2. Select plan (Starter, Pro, etc.)
3. Select region
4. Open Paddle checkout
5. Poll for tenant provisioning
6. Update `credentials.json`

**Flow (existing tenant):**
1. List existing tenants
2. Select one
3. Update `credentials.json`

**Updates:** `credentials.json` with `tenant_id`, `app_id`, `hostname`

---

### `init`

Initialize a new project with boilerplate.

```bash
just init                     # Interactive
just init --force             # Overwrite existing
just init --hard-reset        # Remove and regenerate everything
```

**Prompts for:**
- `DATABASE_URL`
- `GITHUB_LINK`
- At least one AI API key

**Produces:**
- `.env` file
- `services/` directory structure

---

### `configure`

Generate validated profile and secrets files.

```bash
just configure
```

**Wizard steps:**
1. Environment selection (Local / Production / Both)
2. Subscription verification (for production)
3. Configuration collection (project name, database, API keys)
4. Docker PostgreSQL setup (for local)
5. Connection validation
6. Profile + secrets generation
7. Optional: Run migrations

**Produces (per environment selected):**
```
.systemprompt/profiles/<env>/
├── profile.yml     ← Runtime configuration
└── secrets.json    ← API keys + DATABASE_URL
```

**Example:** Selecting "Both" creates:
- `.systemprompt/profiles/local/profile.yml`
- `.systemprompt/profiles/local/secrets.json`
- `.systemprompt/profiles/production/profile.yml`
- `.systemprompt/profiles/production/secrets.json`

---

### `secrets`

Manage API keys and secrets for a specific profile.

```bash
just secrets                          # List all (current profile)
just secrets --profile local          # List secrets for local profile
just secrets set KEY VALUE            # Set a secret (current profile)
just secrets set KEY VALUE --profile production  # Set for specific profile
just secrets import                   # Import from .env to current profile
```

**Updates:** `.systemprompt/profiles/<env>/secrets.json`

**Profile selection:**
- Uses `SYSTEMPROMPT_PROFILE` environment variable by default
- Override with `--profile <env>` flag

---

### `migrate`

Run database migrations.

```bash
just migrate
```

**Requires:** `SYSTEMPROMPT_PROFILE` or `DATABASE_URL`

---

### `sync`

Synchronize content and skills with database.

```bash
just sync                     # Sync all (content + skills)
just sync content             # Sync markdown → database
just sync skills              # Sync skills → database
```

**Requires:** Database connection

---

### `deploy`

Build and deploy to cloud.

```bash
just deploy                   # Full deploy
just deploy --skip-push       # Build only, don't push
just deploy --tag v1.0.0      # Custom image tag
```

**Requires:**
- `credentials.json` with `tenant_id` and `app_id`
- Profile with `cloud.enabled: true`
- Built binary and web dist

**Flow:**
1. Validate credentials and profile
2. Build Docker image
3. Push to Fly.io registry
4. Trigger deployment
5. Report status

---

### `status`

Check cloud deployment status.

```bash
just status
```

**Shows:**
- Deployment state
- Running instances
- Recent logs
- Health checks

---

## Quickstart Recipes

### Local Development Only

```bash
# 1. Initialize project
just init

# 2. Start PostgreSQL
just db-up

# 3. Run migrations
just migrate

# 4. Sync content
just sync

# 5. Start server
just start
```

### Cloud Deployment

```bash
# 1. Authenticate
just login

# 2. Create/select tenant
just tenant

# 3. Configure profiles
just configure

# 4. Start local PostgreSQL
just db-up

# 5. Run migrations
just migrate

# 6. Sync content
just sync

# 7. Deploy to cloud
just deploy
```

---

## Troubleshooting

### "Not logged in"

```bash
just login
```

### "No tenant configured"

```bash
just tenant
```

### "Profile required"

```bash
export SYSTEMPROMPT_PROFILE=services/profiles/local.profile.yml
# or
just configure
```

### "Cloud features disabled"

Edit your profile and set:
```yaml
cloud:
  enabled: true
```

### "Database connection failed"

```bash
# Start PostgreSQL
just db-up

# Check connection
just db-status
```

---

## Environment Variables

| Variable | Description | Set By |
|----------|-------------|--------|
| `SYSTEMPROMPT_PROFILE` | Path to active profile | User |
| `DATABASE_URL` | PostgreSQL connection | Profile or .env |
| `ANTHROPIC_API_KEY` | Claude API key | secrets.json |
| `OPENAI_API_KEY` | OpenAI API key | secrets.json |
| `GEMINI_API_KEY` | Gemini API key | secrets.json |

---

## Security Notes

1. **Never commit secrets**: `.systemprompt/` is gitignored
2. **Project-specific credentials**: Never use global `~/.systemprompt/` — each project has its own tenant
3. **Environment isolation**: Each profile has its own secrets — don't share API keys across environments
4. **Token refresh**: Tokens expire; re-run `login` if API calls fail with 401
5. **Production secrets**: Consider using cloud KV store instead of local `secrets.json` for production

---

## Open Questions

> **Note:** The refactoring plan in core needs clarification on the following:

### 1. Dockerfile Generation

The plan shows flat structure without `Dockerfile`. This doc assumes:
- `Dockerfile` lives at `.systemprompt/Dockerfile` (project root, not per-profile)
- Generated by `cloud profile create` or `cloud init`?
- Or manually created by user?

**Question:** Which command generates the Dockerfile, and should it be templated based on project type?

### 2. docker-compose.yml Generation

The plan shows flat structure (`local/`, `staging/`) without `profiles/` parent or `docker-compose.yml`.

This doc assumes:
- `docker-compose.yml` lives at `.systemprompt/profiles/<env>/docker-compose.yml`
- Generated by `cloud profile create <env>`
- Different per environment (local has postgres, staging might have redis, production has none)

**Question:** Should `cloud profile create` generate docker-compose.yml? What services should be included by default?

### 3. Flat vs Nested Structure

Plan shows:
```
.systemprompt/
├── local/
├── staging/
└── production/
```

This doc shows:
```
.systemprompt/
└── profiles/
    ├── local/
    ├── staging/
    └── production/
```

**Question:** Do we need the `profiles/` parent folder, or should environments be at root level?
