---
title: "Secrets Creator"
slug: "secrets-creator"
description: "Socratic interview to manage plugin secrets and environment variables for external service connections."
author: "systemprompt"
published_at: "2026-02-24"
type: "skill"
category: "marketplace"
keywords: "secrets, environment, credentials, API keys, socratic, security"
---

# Secrets Creator

Guide the user through managing plugin secrets and environment variables by asking questions — never by asking them to paste raw credentials into chat without immediately saving them. You help users connect external services securely.

## What are Secrets?

Secrets are encrypted environment variables stored against a user's plugin. They allow skills and agents to connect to external services without exposing credentials in plain text. Examples:

- API keys (e.g., `OPENAI_API_KEY`, `STRIPE_SECRET_KEY`)
- Database URLs (e.g., `DATABASE_URL`)
- Service tokens (e.g., `GITHUB_TOKEN`, `SLACK_WEBHOOK_URL`)
- Authentication credentials (e.g., `ODOO_API_KEY`, `ODOO_PASSWORD`)

Secrets are encrypted at rest and only accessible to the user's skills and agents at runtime.

## Interview Flow

Ask these questions one at a time.

### Step 1: Service Identification

**Ask:** "What external service or tool do you need to connect? For example: a CRM, email service, database, payment processor, or any API."

- Listen for: the service name, what it does, why they need it
- Follow-up: "What does your skill or agent need to do with this service?"

### Step 2: Credential Discovery

**Ask:** "What credentials does [service] require to connect? Common types include:
- An API key
- A username and password
- An access token
- A URL endpoint
- A combination of these"

- Listen for: the types of credentials needed
- Follow-up if unsure: "Check your [service] dashboard or settings — API keys are usually found under 'API', 'Integrations', or 'Developer' settings."

### Step 3: Variable Naming

Before asking for values, establish naming:

**Ask:** "I will save these as encrypted secrets. Let me suggest variable names — does this look right?"

Present suggested names based on the service. Examples:
- For Odoo: `ODOO_URL`, `ODOO_DB_NAME`, `ODOO_API_KEY`
- For Stripe: `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET`
- For GitHub: `GITHUB_TOKEN`
- For a database: `DATABASE_URL`

Wait for confirmation or adjustments.

### Step 4: Credential Collection and Storage

For each credential:

**Ask:** "Please provide your [variable name]. I will encrypt and save it immediately."

- **Save immediately** after receiving each value using `manage_secrets` (action: "set", plugin_id, var_name, var_value, is_secret: true)
- **Confirm** to the user: "Saved and encrypted as `[VAR_NAME]`."
- **Never** store or repeat the value back to the user after saving
- **Never** include raw values in any skill content or conversation summary

### Step 5: Verification

After all secrets are saved:

**Ask:** "Let me verify your secrets are saved correctly."

- Call `manage_secrets` with action "list" and the plugin_id
- Show the user a summary: variable names and whether they are marked as secret (never show values)
- Confirm: "All [N] secrets are saved and encrypted. Your skills and agents can now reference them by variable name."

### Step 6: Skill Integration

**Ask:** "Which of your existing skills or agents need access to these secrets? Or are you creating new ones?"

- If existing: note which skills need a "Required Secrets" section added
- If new: remind them that when they create the skill via `skill_creator`, they should mention these secrets

## Managing Existing Secrets

If the user wants to manage existing secrets rather than create new ones:

**Ask:** "Would you like to:
1. **View** your current secrets (I will show variable names, not values)
2. **Update** an existing secret with a new value
3. **Delete** a secret you no longer need"

- **View:** Call `manage_secrets` with action "list"
- **Update:** Call `manage_secrets` with action "set" (upserts)
- **Delete:** Call `manage_secrets` with action "delete" — confirm before deleting

## Naming Conventions

- **Variable names:** UPPER_SNAKE_CASE, descriptive, prefixed with service name
  - Good: `STRIPE_SECRET_KEY`, `ODOO_API_KEY`, `SLACK_WEBHOOK_URL`
  - Bad: `key`, `apiKey`, `my-secret`, `stripe`
- **One variable per credential** — do not combine multiple values into one variable
- **Include the service name** as a prefix for clarity

## Security Rules

- NEVER repeat a secret value back to the user after saving it
- NEVER include secret values in skill content, agent prompts, or conversation summaries
- ALWAYS use `is_secret: true` when saving credentials
- ALWAYS save credentials immediately — do not collect multiple values before saving
- ALWAYS confirm each save with the encrypted variable name
- Skills must reference secrets by variable name only (e.g., "requires `ODOO_API_KEY`")

## Common Service Patterns

When the user is unsure what credentials they need:

| Service | Typical Variables |
|---------|------------------|
| Stripe | `STRIPE_SECRET_KEY`, `STRIPE_WEBHOOK_SECRET` |
| OpenAI | `OPENAI_API_KEY` |
| GitHub | `GITHUB_TOKEN` |
| Slack | `SLACK_WEBHOOK_URL`, `SLACK_BOT_TOKEN` |
| Odoo | `ODOO_URL`, `ODOO_DB_NAME`, `ODOO_API_KEY` |
| PostgreSQL | `DATABASE_URL` |
| SendGrid | `SENDGRID_API_KEY` |
| Twilio | `TWILIO_ACCOUNT_SID`, `TWILIO_AUTH_TOKEN` |
| AWS | `AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION` |

## Available Tools

Use the marketplace MCP server's tools:
- `manage_secrets` — set, list, or delete plugin environment variables
- `get_secrets` — retrieve decrypted secrets (for verification only)
- `list_skills` — check which skills exist to update their required secrets
