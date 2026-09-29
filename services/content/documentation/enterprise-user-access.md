---
title: "User & Access Management"
description: "Manage enterprise users from the admin UI: search, roles, session and PAT revocation, passkey sign-in, and optional AD FS SSO with AD groups as entitlements."
author: "systemprompt.io"
slug: "enterprise-user-access"
keywords: "users, access, roles, passkeys, sso, adfs, active directory, pat, sessions, deprovisioning"
kind: "guide"
public: true
tags: ["enterprise", "admin", "access"]
published_at: "2026-08-25"
updated_at: "2026-09-28"
after_reading_this:
  - "Manage users end to end from /admin/users without touching the CLI or database"
  - "Provision users through operator administration or optional SSO"
  - "Understand the passkey sign-in and recovery paths"
  - "Revoke sessions and personal access tokens for any user immediately"
  - "Understand how SSO, when configured, provisions accounts and how AD groups revoke access"
related_docs:
  - title: "Access Control: Who Reaches What, and Why"
    url: "/documentation/access-control"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Enterprise Roadmap & Known Limitations"
    url: "/documentation/enterprise-roadmap"
---

# User & Access Management

The admin UI at `/admin/users` manages accounts, roles, status, sessions, and personal access tokens. Browser sign-in uses WebAuthn passkeys by default; AD FS SSO is an optional mechanism an installation switches on; operators can provision and recover access through the CLI.

## Managing users from the admin UI

Open `/admin/users` as an admin (the older `/admin/access/users` path redirects there). From this one page you can:

- **Search and list** users by name or email, with pagination.
- **Create** a user directly.
- **Edit roles and status** — grant or remove roles (`user`, `developer`, `knowledge_worker`, `project_manager`, `admin`, `platform_admin`, `super_admin`), activate or suspend an account.
- **Disable or delete** an account. Disabling keeps history; deleting removes the account.
- **Revoke sessions** — force sign-out everywhere with one action.
- **Revoke personal access tokens** — from `/admin/devices`, which lists tokens across the instance.
- **See last activity** — each user row shows last-active time, so stale accounts are visible at a glance.

No CLI or database access is required for any of these operations. The CLI equivalent for scripting exists under `systemprompt admin users` (see `systemprompt admin users --help`).

## Signing in and provisioning

The login page at `/admin/login` signs in with a **passkey**. Someone on a new device, or who has lost their passkey, can ask for a time-limited, single-use email link that lets them create a passkey on that device, where the installation has email delivery configured. Operators create accounts from `/admin/users` or with `systemprompt admin users create`. On a development, non-cloud profile, `just dev-login <email>` prints a single-use login link; the route is not mounted anywhere else. See [Authentication](/documentation/authentication).

## Optional AD FS SSO and deprovisioning

AD FS SSO (SAML 2.0) is built in and **off by default**: it switches on when an installation adds `services/web/config/adfs.yaml`, which the template does not ship. The relying party is served at `/admin/auth/adfs/start` and `/admin/auth/adfs/acs`, and trust is pinned to the IdP's published signing certificate — there is no client secret. When it is configured:

- A verified assertion must carry an allowed email domain, and a group claim when `deny_without_group` is on (the default).
- `auto_provision` (default off) decides whether SSO may create an account; off, SSO only signs in existing accounts.
- `group_roles` maps an AD group to roles, and `group_role_patterns` does the same for a one-`*` glob, so a family of regional groups (for example `Systemprompt-ProjectManagers-*`) maps to one role without enumerating them. An unmapped group signs in as `user`. Directory role grants are re-projected at every sign-in, while roles an admin granted by hand survive.

Deprovisioning then follows the directory: disabling the AD account stops sign-in, and removing someone from a mapped group strips the role it granted at their next sign-in. Mid-session revocation is bounded by the session lifetime; an admin can revoke sessions and PATs immediately from `/admin/users` and `/admin/devices`.

SCIM provisioning is not offered — AD FS does not push standards-based SCIM, so an endpoint would have no caller. See the [Enterprise Roadmap](/documentation/enterprise-roadmap) for when this changes.

## The project-manager role

`project_manager` is a read-only admin. It reaches every console page of the dashboard — users, access, analytics, conversations, traces, the catalog — across every project rather than one, and it changes nothing: writes stay with `admin` and `platform_admin`, and directory-shaped controls (AD mappings, granting `platform_admin`) with `platform_admin` alone. Nor does it grant the admin control plane: the `systemprompt` MCP server requires the `admin` OAuth scope, and the governance scope check marks its tools admin-only (in the shipped warn mode that check records rather than refuses).

With SSO configured, the role can be granted from the directory through `group_role_patterns`. Project membership is independent of this role; users are not restricted to exactly one project.

## Personal access tokens and devices

`/admin/devices` lists every personal access token on the instance: owner, prefix (so a leaked token can be identified without exposing it), expiry, and creation time. Any token can be revoked immediately, and revocation takes effect on the next request. The same page can issue a token for the signed-in account (`POST /admin/devices/pats`) and shows the secret exactly once. A token may be issued with an expiry; the template sets no instance-wide maximum token lifetime.

## Time-bound access

Share tokens honour an expiry timestamp, and setup tokens, JWTs and personal access tokens can all be time-bound.

Access itself can be time-bound too. A group membership, a project membership, a manual role grant, an access-control rule and a device certificate each have a validity window in the schema, and the hourly `access_expiry` job makes it bind. The template's console does not yet offer a picker for these windows; the declared path is `valid_until:` (an RFC 3339 instant) on an entity in `services/access-control/rules.yaml`. How an expiry binds differs by what it is on, because some of these rows are read by core code this extension does not own:

| What expires | Takes effect | How |
|---|---|---|
| Group or project membership | Immediately | Every membership read, including the `group`/`project` authorization dimensions, hides a row outside its window. The `access_expiry` sweep then stamps the row `revoked_at` so it remains as the audit trail, and recomputes the person's primary scope. |
| Manual role | Within the hour | Roles are enforced from the effective set on the user record, which the sweep rewrites without the expired grant. If that removes a manage role, the sweep revokes the person's live sessions, tokens and certificates — the same teardown a demotion in the role editor performs. |
| Access-control rule | Within the hour | Core's resolver reads the rule table directly, so the sweep deletes the expired rule. A `rules.yaml` declaration already past its `valid_until` is treated as not declared, the export writes the window back, and a window that differs between code and database is reported as drift on the Sync page. |
| Device certificate | Within the hour | The sweep stamps the certificate revoked, which is what the device gate reads. |

For contractors, combine a membership or rule expiry with a token expiry; account-level disable and revocation remain available for the immediate case.

## Context-aware access control

What a session can reach — marketplaces, plugins, skills, MCP servers, gateway routes — is decided per request from the person's **current context**: their role, the groups and projects they belong to (operator-managed, or placed by SSO when it is configured), and the MCP servers they hold a ready connection to. The narrowest band that names them decides, a deny beats an allow inside it, and an entity with rules is closed to everyone they do not name. Every rule is declared once in `services/access-control/rules.yaml` with a stated reason and enforced from the database; `/admin/access-control` shows who reaches what and why, and its Sync page reconciles code and console in either direction. The full model, worked examples and how-tos are in [Access Control](/documentation/access-control).

## Verification

Use the commands and UI checks above against the configured instance. Repository maintainers can run `just test-unit` and `just test-integration`; a passing fixture test does not establish that a deployed integration is configured.
