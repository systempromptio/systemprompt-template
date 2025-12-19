# E2E Deployment Flow

Complete guide to deploying a SystemPrompt template to the cloud.

## Prerequisites

- Git installed
- Rust toolchain (cargo)
- Docker installed and running
- `just` command runner (`cargo install just`)
- Browser for OAuth authentication

## Quick Start

```bash
# 1. Clone template
git clone --recursive https://github.com/systempromptio/systemprompt-template my-project
cd my-project

# 2. Local setup
just setup

# 3. Login to cloud
just login

# 4. Setup tenant (creates free VM via Paddle checkout)
just cloud-setup

# 5. Deploy
just cloud-deploy

# 6. Verify
just cloud-status
```

## Detailed Steps

### Step 1: Clone Template

```bash
git clone --recursive https://github.com/systempromptio/systemprompt-template my-project
cd my-project
```

**What happens:**
- Downloads the template repository
- `--recursive` fetches the `core/` submodule from systemprompt-core

**Expected output:**
```
Cloning into 'my-project'...
Submodule 'core' registered for path 'core'
Cloning into 'core'...
```

**Troubleshooting:**
- If submodule fails: `git submodule update --init --recursive`

---

### Step 2: Local Setup

```bash
just setup
```

**What happens:**
1. Builds the `systemprompt` CLI binary
2. Runs database migrations (if DB configured)
3. Builds web assets

**Expected output:**
```
Building core binary...
    Compiling systemprompt v0.x.x
    Finished `dev` profile
Building workspace...
Running migrations...
Setup complete!
```

**Troubleshooting:**
- Missing Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Missing just: `cargo install just`

---

### Step 3: Login to SystemPrompt Cloud

```bash
just login
```

**What happens:**
1. Prompts to select OAuth provider (GitHub or Google)
2. Opens browser for authentication at `api.systemprompt.io`
3. After OAuth, saves JWT token to `~/.systemprompt/credentials.json`
4. Displays your account info and existing tenants

**Expected output:**
```
SystemPrompt Cloud Login
════════════════════════════════════════
Environment: Production

? Select authentication provider
> GitHub
  Google

Opening browser for authentication...
✓ Logged in successfully

  User
    Email: you@example.com
    ID:    usr_xxxxx

  No tenants found. Run 'systemprompt cloud setup' to create one.
```

**API calls:**
- `GET api.systemprompt.io/api/v1/auth/oauth/github` (or google)
- `GET api.systemprompt.io/api/v1/auth/me`

---

### Step 4: Setup Tenant

```bash
just cloud-setup
```

**What happens (if no existing tenants):**
1. Fetches available plans from API
2. Prompts for plan selection (Free, Basic, etc.)
3. Prompts for region (iad, lhr, sin, etc.)
4. Creates Paddle checkout session
5. Opens browser for Paddle checkout (even for $0 free plan)
6. Polls API waiting for tenant to be provisioned
7. Saves tenant_id to credentials

**What happens (if tenants exist):**
1. Lists your existing tenants
2. Lets you select one or create new
3. Saves selected tenant_id to credentials

**Expected output (new tenant):**
```
SystemPrompt Cloud Setup
════════════════════════════════════════

No existing tenants found. Creating a new one...

? Select a plan
> Free (256MB RAM, 1GB storage)
  Basic (1GB RAM, 10GB storage)

? Select a region
> iad (US East)
  lhr (London)
  sin (Singapore)

Opening Paddle checkout...
Waiting for tenant provisioning...
✓ Tenant created successfully

  Tenant: my-project
  URL:    https://sp-xxxx.fly.dev

  Next: Run 'systemprompt cloud deploy' to deploy your site.
```

**API calls:**
- `GET api.systemprompt.io/api/v1/checkout/plans`
- `POST api.systemprompt.io/api/v1/checkout` → returns Paddle URL
- `GET api.systemprompt.io/api/v1/tenants` (polling)

---

### Step 5: Deploy to Cloud

```bash
just cloud-deploy
```

**What happens:**
1. Builds release binary (`cargo build --release`)
2. Builds web assets (`npm run build`)
3. Builds Docker image
4. Gets registry token from API
5. Logs into `registry.fly.io`
6. Pushes Docker image
7. Triggers deployment via API
8. Displays deployed URL

**Expected output:**
```
SystemPrompt Cloud Deploy
═══════════════════════════════════════════════════

  Tenant: my-project
  Image:  registry.fly.io/sp-xxxx:deploy-1234567890-abc123

Step 1/4: Building...
         ✓ Build complete
Step 2/4: Building Docker image...
         ✓ Docker image built
Step 3/4: Pushing to registry...
         ✓ Image pushed
Step 4/4: Deploying...
         ✓ Deployed

═══════════════════════════════════════════════════
  ✓ Deployment Complete!
═══════════════════════════════════════════════════

  Status: deployed
  URL:    https://sp-xxxx.fly.dev
```

**API calls:**
- `GET api.systemprompt.io/api/v1/tenants/{id}/registry-token`
- `POST api.systemprompt.io/api/v1/tenants/{id}/deploy`

**Troubleshooting:**
- Docker not running: Start Docker Desktop
- Build fails: Check Rust/npm versions
- Push fails: Check internet connection

---

### Step 6: Verify Deployment

```bash
just cloud-status
```

**What happens:**
- Checks tenant provisioning status
- Shows app URL and health

**Expected output:**
```
Tenant Status: ready
App URL: https://sp-xxxx.fly.dev
```

**API calls:**
- `GET api.systemprompt.io/api/v1/tenants/{id}/status`

---

## Other Commands

### View Cloud Configuration

```bash
just cloud-config
```

Shows current tenant, credentials file location, and API URL.

### Logout

```bash
just logout
```

Clears saved credentials from `~/.systemprompt/credentials.json`.

### Sync Local/Cloud

```bash
just cloud-sync push   # Push local changes to cloud
just cloud-sync pull   # Pull cloud changes to local
```

---

## Architecture

```
┌─────────────────────┐     OAuth      ┌──────────────────────┐
│   Local Machine     │◄──────────────►│   api.systemprompt.io│
│                     │                 │   (systemprompt-db)  │
│  ┌───────────────┐  │                 │                      │
│  │ systemprompt  │  │  API calls      │  ┌────────────────┐  │
│  │     CLI       │──┼────────────────►│  │ Auth/Tenants   │  │
│  └───────────────┘  │                 │  │ Checkout       │  │
│                     │                 │  │ Deploy         │  │
│  ┌───────────────┐  │                 │  └────────────────┘  │
│  │ Docker image  │  │                 │          │           │
│  └───────┬───────┘  │                 │          │ webhook   │
│          │          │                 │          ▼           │
└──────────┼──────────┘                 │  ┌────────────────┐  │
           │                            │  │    Paddle      │  │
           │ push                       │  │   (billing)    │  │
           ▼                            │  └────────────────┘  │
┌─────────────────────┐                 │          │           │
│  registry.fly.io    │                 │          │ provision │
│  (Docker Registry)  │                 │          ▼           │
└──────────┬──────────┘                 │  ┌────────────────┐  │
           │                            │  │    Fly.io      │  │
           │ deploy                     │  │   (hosting)    │  │
           ▼                            │  │                │  │
┌─────────────────────┐                 │  │  sp-xxxx.fly   │  │
│   Fly.io Machine    │◄────────────────┤  │    .dev        │  │
│   sp-xxxx.fly.dev   │                 │  └────────────────┘  │
└─────────────────────┘                 └──────────────────────┘
```

---

## Free Tier Limits

- **1 free tenant** per account
- **256MB RAM**
- **1GB storage**
- Region: Any available

Upgrade to Basic plan for unlimited tenants and more resources.

---

## API Reference

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v1/auth/oauth/{provider}` | GET | Start OAuth flow |
| `/api/v1/auth/me` | GET | Get current user + tenants |
| `/api/v1/checkout/plans` | GET | List available plans |
| `/api/v1/checkout` | POST | Create Paddle checkout |
| `/api/v1/tenants` | GET | List your tenants |
| `/api/v1/tenants/{id}/status` | GET | Check tenant status |
| `/api/v1/tenants/{id}/registry-token` | GET | Get Docker registry credentials |
| `/api/v1/tenants/{id}/deploy` | POST | Deploy Docker image |
| `/api/v1/tenants/{id}/credentials/{token}` | GET | Get DB URL, JWT, admin password |

---

## Troubleshooting

### "No customer record found"

Run `just login` first to authenticate.

### "Free tier limit reached"

You can only have 1 free tenant. Either:
- Delete your existing free tenant
- Upgrade to Basic plan

### "Tenant has no cloud infrastructure"

The tenant was created but Fly.io provisioning failed. Check:
1. Run `just cloud-status` to see current state
2. Contact support if stuck in "provisioning" state

### Build errors

```bash
# Update Rust
rustup update

# Clean and rebuild
cargo clean
just build
```

### Docker push fails

```bash
# Verify Docker is running
docker ps

# Try logging in manually
docker login registry.fly.io
```
