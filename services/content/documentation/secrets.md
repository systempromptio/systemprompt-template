---
title: "Secrets"
description: "Manage encrypted environment variables and API keys for your plugins. Secrets are scoped per plugin, encrypted at rest, and resolved at runtime through a secure token-based flow."
author: "systemprompt.io"
slug: "secrets"
keywords: "secrets, environment variables, encryption, API keys, plugin secrets, key rotation, audit log, security"
kind: "guide"
public: true
tags: ["secrets", "security", "plugins", "configuration"]
published_at: "2026-03-02"
updated_at: "2026-03-02"
after_reading_this:
  - "Create and manage encrypted environment variables scoped to specific plugins"
  - "Understand how secrets are encrypted at rest and resolved at runtime"
  - "Rotate encryption keys and review the audit log for secret access"
  - "Configure plugin variable definitions that drive the secrets UI"
related_docs:
  - title: "My Workspace"
    url: "/documentation/my-workspace"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "MCP Servers"
    url: "/documentation/mcp-servers"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Presentation"
    url: "/documentation/presentation"
---

# Secrets

**Secrets let you store encrypted environment variables -- such as API keys, tokens, and credentials -- scoped to individual plugins, with runtime resolution through a secure token-based API.**

> **See this in the presentation:** [Slide 11: Personalization & Ownership](/documentation/presentation#slide-11)

## What You'll See

**URL:** `/admin/my/secrets/`

The Secrets page displays your environment variables grouped by plugin. A stats bar at the top shows:

- **Total count** -- total number of environment variables across all plugins.
- **Secret count** -- how many of those variables are marked as secrets (encrypted at rest).

Below the stats, variables are organized into collapsible groups by plugin ID. Each group shows the plugin name and the number of variables it contains.

## How Secrets Work

Secrets in systemprompt.io use a layered encryption architecture:

1. **Master key** -- a server-side key loaded from the environment at startup.
2. **User DEK (Data Encryption Key)** -- a per-user key encrypted with the master key. Each user gets their own DEK.
3. **Variable encryption** -- when a variable is marked as a secret (`is_secret: true`), its value is encrypted using the user's DEK before being stored in the database.

At no point are secret values stored in plaintext in the database.

## Managing Environment Variables

### Variable Fields

Each environment variable has:

| Field | Description |
|-------|-------------|
| **Plugin ID** | The plugin this variable belongs to |
| **Variable Name** | The environment variable name (e.g., `OPENAI_API_KEY`) |
| **Variable Value** | The value to store. Displayed as-is for non-secrets; masked for secrets. |
| **Is Secret** | When enabled, the value is encrypted at rest |

### Adding Variables

1. Navigate to `/admin/my/secrets/`.
2. Select the target plugin from the plugin dropdown.
3. Enter the variable name and value.
4. Check **Is Secret** if the value should be encrypted.
5. Save the variable.

### Plugin Variable Definitions

Plugins can declare expected variables in their `config.yaml` under `plugin.variables`. When a plugin defines variables, the secrets UI shows which required variables are missing and which are already configured. Each definition can specify:

- **name** -- the variable name
- **required** -- whether the variable must be set for the plugin to function
- **description** -- help text shown in the UI

The API returns a `valid` flag indicating whether all required variables have been set, along with a `missing_required` list of any that are missing.

### Updating Variables

To update a variable, submit the new value through the same form. The system performs an upsert -- if a variable with that name already exists for the plugin, it updates the value. If the `is_secret` flag changes, the encryption state is updated accordingly.

## Runtime Secret Resolution

Plugins resolve their secrets at runtime through a two-step token-based flow:

1. **Request a resolution token** -- the plugin (or its MCP server) sends an authenticated request to create a single-use token that expires in 5 minutes. The JWT must have a `plugin` resource audience and be signed with the platform's JWT secret.
2. **Resolve secrets** -- the plugin uses the token to fetch its decrypted secrets. The system validates and consumes the token, verifies the plugin ID, decrypts the user's DEK with the master key, decrypts all secret values for the plugin, and returns the resolved key-value pairs.

This ensures secrets are only decrypted on demand, tokens cannot be reused, and the plugin ID is verified at every step.

## Key Rotation

You can rotate your personal Data Encryption Key (DEK) at any time. Rotation re-encrypts your DEK with the current master key. This is useful if you suspect your encryption key may have been compromised. The rotation event is recorded in the audit log.

## Audit Log

Every secret-related action is logged in the `secret_audit_log` table.

Each audit entry contains:

| Field | Description |
|-------|-------------|
| **id** | Unique log entry ID |
| **var_name** | The variable that was accessed or modified (or `*` for key rotation) |
| **action** | What happened (e.g., `created`, `updated`, `resolved`, `rotated`) |
| **actor_id** | The user who performed the action |
| **ip_address** | The IP address of the request (if available) |
| **created_at** | Timestamp of the action |

The audit log returns the most recent 100 entries, ordered newest first.

## Security Model

| Aspect | Detail |
|--------|--------|
| **Encryption at rest** | AES-based encryption using per-user DEKs wrapped by a master key |
| **Encryption in transit** | All API calls use HTTPS |
| **Token lifetime** | Resolution tokens expire after 5 minutes |
| **Token reuse** | Tokens are single-use; consumed on first resolution |
| **Plugin isolation** | Tokens are scoped to a specific plugin ID; cross-plugin access is rejected |
| **User isolation** | Each user has their own DEK; one user cannot resolve another's secrets |
| **Audit trail** | All create, update, resolve, and rotate actions are logged |

