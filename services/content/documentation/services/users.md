---
title: "Users"
description: "Per-user isolation from day one. User scopes enforced automatically across all operations."
author: "SystemPrompt Team"
slug: "services/users"
keywords: "users, user isolation, scopes, permissions"
image: "/files/images/docs/services-users.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Users

SystemPrompt provides per-user isolation from day one. Every user gets their own isolated workspace. Every operation is scoped to a user. Every query is filtered by user ID. This isn't a feature you enable—it's how the platform works.

When you ship an AI product to your users, each user gets isolated access. User A cannot see User B's conversations, agents, or data. This isolation is enforced through user-scoped database queries and separate database instances per deployment.

## User Isolation Architecture

SystemPrompt uses a single-tenant-per-instance architecture. Each deployment (whether local or cloud) has its own PostgreSQL database, providing complete isolation between deployments. Within a deployment, user data is isolated through user-scoped queries—User A cannot access User B's agents, files, or conversations.

**User hierarchy:**

```
Deployment (your SystemPrompt instance)
└── Users (authenticated principals)
    └── Sessions (active logins)
        └── Tokens (API credentials)
```

Each deployment is an isolated workspace. For cloud deployments, each customer organization gets their own database instance. Within that instance, user-level isolation is enforced through scoped queries. The `systemprompt-users` crate manages this hierarchy.

**Automatic isolation:**

When a request arrives, the system extracts the user ID from the authentication token. This user ID is used to scope all database queries. Every query that touches user-scoped data automatically includes a `WHERE user_id = ?` clause.

```rust
// User-scoped queries in systemprompt-users
let contexts = context_repository
    .list_for_user(ctx.user_id)
    .execute()
    .await?;
```

The `systemprompt-identifiers` crate provides strongly-typed identifiers for users and other entities. Using `UserId` instead of raw strings prevents identifier confusion bugs.

## User Management and Profiles

Users are authenticated principals within a deployment. Each user has a profile containing identity information and preferences.

**User properties:**

| Property | Description |
|----------|-------------|
| `id` | Unique user identifier (UserId type) |
| `email` | Primary email address |
| `display_name` | Human-readable name |
| `avatar_url` | Profile image URL |
| `role` | User role within deployment |
| `created_at` | Registration timestamp |
| `last_login_at` | Most recent authentication |

**CLI user management:**

```bash
# List users in current deployment
systemprompt admin users list

# Show user details
systemprompt admin users show <user_id>

# Create a new user
systemprompt admin users create --email user@example.com --role user

# Update user role
systemprompt admin users update <user_id> --role admin
```

**User roles:**

Roles provide a coarse-grained permission model layered on top of OAuth2 scopes.

| Role | Description | Typical Scopes |
|------|-------------|----------------|
| `viewer` | Read-only access | `user`, `agents:read` |
| `user` | Standard operations | `user`, `agents:read`, `tools:execute` |
| `admin` | Full deployment access | `admin`, all scopes |
| `owner` | Deployment owner | `admin`, billing, user management |

## Per-User Scope Enforcement

Scopes define what operations a user can perform. They are granted during authentication and checked on every protected operation.

**How scope enforcement works:**

1. User authenticates (WebAuthn or OAuth2)
2. System determines user's role and permissions
3. Access token is issued with appropriate scopes
4. Every API call extracts scopes from token
5. Operation requirements are checked against token scopes
6. Request proceeds or returns 403 Forbidden

**Scope assignment:**

Scopes can be assigned at multiple levels:

- **Role-based**: Roles map to default scope sets
- **User-specific**: Individual users can have additional scopes
- **Client-specific**: OAuth2 clients can request specific scopes
- **Context-specific**: Operations can require contextual scopes

```yaml
# Agent configuration with scope requirements
security:
  - oauth2: ["user"]  # Requires at least 'user' scope
```

**Scope verification:**

The authorization layer checks scopes before any protected operation. This happens in middleware, before business logic executes.

```bash
# Check current session scopes
systemprompt admin session show

# Output includes granted scopes
# scopes: ["user", "agents:read", "tools:execute"]
```

## User Isolation Boundaries

User isolation is the security foundation of SystemPrompt. Isolation is enforced at multiple levels.

**Deployment level:**

Each deployment (local or cloud) has its own PostgreSQL database. Cloud deployments are completely isolated from each other—different customers never share a database instance.

**Database level:**

Within a deployment, user-scoped tables include a `user_id` column. Database queries automatically filter by the authenticated user's ID.

**Application level:**

Service methods receive a context object containing the authenticated user. Services use this context for all data access. Cross-user access requires explicit authorization.

**API level:**

API endpoints extract user context from authentication tokens. Requests without valid user context are rejected. Admin endpoints can access cross-user data with appropriate scopes.

**File storage level:**

Uploaded files are stored in user-scoped paths. The file service enforces user boundaries when serving or modifying files.

## Deployment Configuration

Deployments are provisioned through the cloud management system. Each deployment has configuration that controls behavior.

**Deployment settings:**

```yaml
# Deployment configuration (managed via cloud API)
deployment:
  id: "tenant_abc123"
  name: "Acme Corp"
  plan: "professional"
  settings:
    max_users: 50
    max_agents: 10
    storage_quota_gb: 100
  features:
    - custom_domains
    - advanced_analytics
```

**Provisioning a deployment:**

```bash
# Create a new cloud deployment
systemprompt cloud tenant create --region iad

# List all deployments
systemprompt cloud tenant list

# Switch to a deployment
systemprompt cloud tenant select <tenant_id>
```

## User Sessions

Sessions track active logins. A user can have multiple active sessions (different devices). Sessions have expiration times and can be revoked.

**Session management:**

```bash
# List active sessions
systemprompt admin sessions list

# Revoke a specific session
systemprompt admin sessions revoke <session_id>

# Revoke all sessions for a user
systemprompt admin sessions revoke-all --user <user_id>
```

**Session properties:**

| Property | Description |
|----------|-------------|
| `id` | Session identifier |
| `user_id` | Owning user |
| `created_at` | Login timestamp |
| `expires_at` | Session expiration |
| `ip_address` | Client IP (for audit) |
| `user_agent` | Client description |

## Configuration Reference

| Item | Location | Description |
|------|----------|-------------|
| User profiles | Database | User identity and preferences |
| Deployment settings | Cloud API | Deployment configuration |
| Session data | Database | Active login sessions |
| Credentials | `.systemprompt/credentials.json` | Cloud authentication |

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt admin users list` | List users with pagination and filtering |
| `systemprompt admin users show <id>` | Show detailed user information |
| `systemprompt admin users search <query>` | Search users by name, email, or full name |
| `systemprompt admin users create` | Create a new user |
| `systemprompt admin users update <id>` | Update user fields |
| `systemprompt admin users delete <id>` | Delete a user |
| `systemprompt admin users count` | Get total user count |
| `systemprompt admin users export` | Export users to JSON |
| `systemprompt admin users stats` | Show user statistics dashboard |
| `systemprompt admin users merge <source> <target>` | Merge source user into target user |
| `systemprompt admin users bulk` | Bulk operations on users |
| `systemprompt admin users role` | Role management commands |
| `systemprompt admin users session` | Session management commands |
| `systemprompt admin users ban` | IP ban management commands |

See `systemprompt admin users <command> --help` for detailed options.