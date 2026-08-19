---
title: "Authentication"
description: "The three ways into the platform: Salesforce SSO, domain-gated passkey self-registration, and CLI-provisioned accounts. Covers sessions, JWTs, and route protection."
author: "Astound Digital"
slug: "authentication"
keywords: "authentication, login, salesforce sso, passkey, webauthn, session, JWT, security, provisioning"
kind: "guide"
public: true
tags: ["authentication", "security", "login"]
published_at: "2026-03-02"
updated_at: "2026-08-19"
after_reading_this:
  - "Sign in as an organization user with Salesforce SSO"
  - "Provision a platform operator from the CLI and enrol their passkey"
  - "Understand how session cookies and JWTs govern authenticated access"
  - "Know which admin routes are public and which require a session"
related_playbooks:
  - title: "Start Here — Standing Up the Gateway"
    url: "/documentation/use-case-admin"
  - title: "Salesforce Integration Overview"
    url: "/documentation/salesforce"
  - title: "Users, Seats and Roles"
    url: "/documentation/salesforce-provisioning"
  - title: "Rolling Out the Bridge"
    url: "/documentation/salesforce-bridge-rollout"
---

# Authentication

**TL;DR:** There are three ways into the platform, and none of them creates or
stores a password. Organization users sign in with **Salesforce SSO**, which
provisions their account on first login. Users on an allow-listed email domain
can **self-register with a passkey** from the login page — no Salesforce
required. And platform operators can be created from the **CLI**, enrolling a
passkey through a one-shot setup link.

Every door is gated by the same authority: the allow-listed email domains your
admin configures. Salesforce decides who is an employee; the domain allow-list
decides who may register; a platform operator decides who operates the
platform.

## Path 1 — Organization Users: Salesforce SSO

This is how everyone who is not a platform operator signs in.

The user clicks **Sign in with Salesforce** on `/admin/login`. The platform runs
an OAuth 2.0 authorization-code flow with PKCE against your Salesforce org, so
your org's own login policies — including MFA — apply unchanged.

On the callback, the platform:

1. Exchanges the code and reads verified `userinfo` claims.
2. Gates them: the email must be present, `email_verified`, and on an
   allow-listed domain. A failure never creates an account.
3. Resolves the identity — returning login, merge onto an existing account by
   verified email, or just-in-time provisioning against the organization's seat
   allocation.
4. Records the Salesforce Username, which later authorizes per-user Salesforce
   tool calls.
5. Mints a session JWT and sets it as the `access_token` cookie.

First login creates the account with no admin step, provided the domain is
allow-listed and a seat is free. Failures return to
`/admin/login?sso=<reason>`; the reasons are tabulated in
[Salesforce App Setup](/documentation/salesforce-app-setup).

A Salesforce-linked account is also what unlocks the Salesforce tooling: the
Salesforce MCP server and the Salesforce marketplace plugins are granted
through a derived `salesforce` access dimension that holds `linked` exactly
while the account has a Salesforce identity. Sign in another way — or
disconnect Salesforce from the profile page — and those entries disappear from
the marketplace and the bridge manifest; everything else on the platform is
unaffected. Linking Salesforce from the profile restores them.

All sign-in paths link a desktop bridge equally: the device-link approval
resumes after either a Salesforce or a passkey sign-in.

## Path 2 — Self-Registration: Email + Passkey

For teams evaluating the platform, or organizations that do not use Salesforce
as their identity source, the login page at `/admin/login` offers **Create an
account** directly:

1. Enter your work email and name.
2. The email domain is checked against the allow-list your admin configured
   (`allowed_email_domains` in `services/web/config/salesforce.yaml`). An
   address outside those domains is refused and no account is created.
3. Complete the browser's passkey prompt. The passkey — not a password — is
   the credential from then on.

Registration provisions the account against the organization's seat allocation,
exactly as SSO just-in-time provisioning does. An account that already holds a
passkey cannot re-register; recovery goes through an operator (see below). New
accounts start with the plain `user` role — see
[Create & Manage Users](/documentation/admin-user-management) for promotion.

## Path 3 — Platform Operators: CLI + Passkey

Operators are created out-of-band. There is no way to self-provision one.

```bash
# 1. Create the account
systemprompt admin users create --name "Jane" --email jane@astounddigital.com

# 2. Grant the admin role
systemprompt admin users role promote jane@astounddigital.com admin

# 3. Mint a one-shot passkey setup link
systemprompt admin users webauthn generate-setup-token --email jane@astounddigital.com
```

The third command prints a copy-paste URL of the form
`{api_external_url}/auth/link-passkey?token=…`, valid for 15 minutes by default
(`--expires-minutes` to change). Send it to the operator through a channel you
already trust; they open it, create a passkey, and sign in with that passkey
from then on.

Roles are **not** carried in the session token. They are read from the user
record on every request, so promoting or demoting an operator takes effect on
their next request — no sign-out, no waiting for a token to refresh. That is
deliberate: revocation is only worth having if it is immediate. The OAuth
*scope* minted into the JWT is fixed at issue time, so a change that widens the
scope itself still needs a fresh sign-in.

### Passkey Sign-In

Passkey authentication uses public-key cryptography. The browser generates a key
pair bound to this domain; the private key stays on the device or in the user's
password manager. The server stores only the public key, verifies a signed
challenge, and issues an OAuth 2.0 session token via PKCE.

### Lost Passkey

There is no self-service recovery — magic links were removed, and no email
service is configured to deliver them. An operator who loses passkey access
needs another operator to mint a fresh setup link with the same
`generate-setup-token` command. Keep more than one operator account so this is
never a single point of failure.

## Session Management

| Property | Value |
|----------|-------|
| **Cookie name** | `access_token` |
| **Token format** | JWT |
| **Default expiry** | 3600 seconds (1 hour) |
| **Cookie flags** | `path=/`, `HttpOnly`, `SameSite=Lax`, `Secure` on HTTPS |
| **Required scopes** | `user` or `admin` |

Every admin request passes through two middleware layers. **User context
middleware** extracts and validates the JWT, then loads the user's roles and
department into a `UserContext`. **Auth check middleware** rejects protected
routes without a valid user ID, returning HTTP 401.

`UserContext` carries `user_id`, `username`, `email`, `roles`, `department`, and
`is_admin`.

To sign out, clear the `access_token` cookie.

## Public vs. Protected Routes

| Route | Access |
|-------|--------|
| `/admin/login` | Public |
| `/admin/auth/salesforce/*` | Public — the SSO start and callback |
| `/auth/link-passkey` | Public — consumes a one-shot setup token |
| `/admin/profile`, `/admin/settings`, `/admin/setup` | Any valid session, including a plain `user` |
| `/admin/*` (everything else) | Requires a valid session **and** the `admin` role |
| `/admin/enterprises*`, `/admin/reports/internal` | Requires platform admin |
| `/bridge-auth/*` | Requires a valid session |

Anonymous requests to a protected route are redirected to
`/admin/login?redirect=…`. A signed-in user **without** the `admin` role is not
shown a 403 for console pages — they are redirected to `/admin/profile`, which
is the only part of the console addressed to them. JSON admin API routes return
HTTP 403 instead, and the platform-admin routes return an HTML 403.

## System-originated actions

Every action recorded by the platform — including scheduled jobs, hooks, and MCP-server invocations — traces to a real `users` row. There is no separate "system user" or synthesized principal. The platform refuses to attribute work to an invented identity.

### How ownership is declared

Each scheduled job in `services/scheduler/config.yaml` carries an explicit `owner:` field naming an existing admin user:

```yaml
- name: publish_pipeline
  extension: web
  owner: admin
  schedule: "0 */15 * * * *"
  enabled: true
```

At startup the scheduler resolves `owner:` to a `users.id`. If the named user does not exist or is inactive, startup fails loudly — the platform refuses to run with unowned jobs. To change ownership, edit the YAML and restart.

### How attribution flows

The resolved owner becomes `JobContext.actor` for every `execute()` call. Job implementations consume it through `ctx.actor()` and pass it to any audit-row write. Governance audit rows carry three fields that together give full forensic clarity:

| Column | Meaning |
|--------|---------|
| `user_id` | The accountable principal — a real `users.id`. |
| `actor_kind` | The surface that ran the action: `user`, `job`, `mcp`. |
| `actor_id` | A label for that surface (job name, MCP server name, etc.). |

A direct human action shows as `(user_id = alice, actor_kind = 'user', actor_id = 'alice')`. A scheduled job owned by Alice shows as `(user_id = alice, actor_kind = 'job', actor_id = 'publish_pipeline')`. Same accountability column, different surface, queryable separately:

```sql
SELECT actor_kind, user_id, COUNT(*)
FROM governance_decisions
GROUP BY actor_kind, user_id;
```

### Why no separate "system" user

A dedicated "system" identity would be either a synthesized principal (impersonation) or a backdoor account with no real human accountability. Neither passes the "every action traces to a real user" bar. The designated owner is a normal admin who legitimately authorized the platform's existence by installing it — same accountability model as a unix crontab. Compromising the designated owner is exactly as bad as compromising that admin's credentials directly; there is no additional power and no amplification path.
