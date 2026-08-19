---
title: "Create & Manage Users"
description: "Every way to create and manage users without Salesforce: admin CLI, bulk creation, passkey self-registration, roles, and conversational admin via the systemprompt MCP server."
author: "Astound Digital"
slug: "admin-user-management"
keywords: "users, create user, bulk users, roles, promote, admin, user management, passkey, registration, systemprompt mcp, csv"
kind: "guide"
public: true
tags: ["documentation", "admin", "users"]
published_at: "2026-08-19"
updated_at: "2026-08-19"
after_reading_this:
  - "Create a user from the CLI and get them connected"
  - "Bulk-create users from a CSV"
  - "Grant and revoke the admin role"
  - "Manage users conversationally through the systemprompt MCP server"
related_playbooks:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Users, Seats and Roles (Salesforce)"
    url: "/documentation/salesforce-provisioning"
---

# Create & Manage Users

Salesforce SSO is one way accounts come to exist — not the only way. This page
covers the paths that need no Salesforce at all: the admin CLI, bulk creation,
domain-gated self-registration, roles, and the conversational admin workflow.

## Three ways an account is created

1. **Admin CLI** — an operator creates the account directly (below). Full
   control, scriptable, works headless.
2. **Passkey self-registration** — the user clicks **Create an account** on
   `/admin/login`; allowed only for email domains on the configured allow-list.
   See [Authentication](/documentation/authentication).
3. **Salesforce SSO just-in-time** — first sign-in creates the account against
   the org's seat allocation. See
   [Users, Seats and Roles](/documentation/salesforce-provisioning).

All three produce the same kind of account; they differ only in who initiates.

## Create one user, end to end

```bash
# 1. Create the account (idempotent with --if-not-exists)
systemprompt admin users create --name jane --email jane@example.com \
  --full-name "Jane Doe" --if-not-exists

# 2. Optional: make them an admin
systemprompt admin users role promote jane@example.com

# 3. Issue a bridge exchange code so they can connect their laptop
systemprompt admin bridge issue-code --user-id jane@example.com
```

Send the code through a channel you trust — it is single-use with a 10-minute
TTL. The user then installs the bridge with it:
[Install the Desktop Bridge](/documentation/bridge-install).

If the user will sign in to the web admin (not just the bridge), mint them a
passkey setup link instead of — or as well as — a code:

```bash
systemprompt admin users webauthn generate-setup-token --email jane@example.com
```

## Bulk creation

`--if-not-exists` makes creation idempotent, so a CSV loop is safe to re-run:

```bash
# users.csv: name,email,full_name
while IFS=, read -r name email full_name; do
  systemprompt admin users create --name "$name" --email "$email" \
    --full-name "$full_name" --if-not-exists
done < users.csv
```

Follow with `role promote` for the rows that need it, and `bridge issue-code`
per user as they onboard.

## The full CLI surface

`systemprompt admin users --help` is the authoritative list. The highlights:

| Command | Purpose |
|---------|---------|
| `list` / `show` / `search` / `count` / `stats` | Inspect the user base |
| `create` / `update` / `delete` | Lifecycle |
| `export` | Dump users for reporting or migration |
| `merge` | Combine duplicate accounts |
| `bulk …` | Batch operations |
| `ban …` | Block access |
| `role promote` / `role demote` / `role assign` | Role management |
| `session …` | Inspect and revoke sessions |
| `webauthn …` | Passkey management and setup tokens |
| `apikey` | API key management |

## Roles

Two roles ship: `user` and `admin`. Admins see the full dashboard, the admin
console, and the `systemprompt` MCP server; users see their profile and their
own connected clients.

```bash
systemprompt admin users role promote jane@example.com   # grant admin
systemprompt admin users role demote jane@example.com    # revoke admin
```

Role checks are read from the user record per request, so revocation is
immediate. The OAuth *scope* in a session token, however, is fixed when the
token is issued — a freshly promoted admin must **sign out and back in** (and
re-link the bridge) before admin surfaces appear.

## Conversational admin: the systemprompt MCP server

Admins who have connected Claude Code through the bridge get the
`systemprompt` MCP server automatically. It exposes the admin surface as
tools, so user management becomes conversational:

> "Create accounts for these 40 people from this spreadsheet, and give the
> team leads admin roles."

Three properties make this safe:

- **Admin-gated** — a non-admin token gets HTTP 403 on every tool; the server
  does not even appear in their client until promotion (and a re-link).
- **Audited** — every tool call lands in the same governance spine as
  inference calls, with identity, arguments, decision, and trace ID.
- **Same surface** — the tools wrap the same CLI commands documented above;
  there is no separate, wider API.

For non-admins the server is simply absent — nothing to misconfigure.

## Where's the web UI for this?

User *creation* is CLI-first (and MCP-conversational) by design; the admin
dashboard shows users, sessions, activity, and costs. See
[Dashboard Usage](/documentation/dashboard). If your team prefers clicking to
typing, the MCP route above is the practical middle ground: type what you
want, review, done.
