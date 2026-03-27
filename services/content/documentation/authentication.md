---
title: "Authentication"
description: "Understand how systemprompt.io authenticates users via OAuth 2.0 with PKCE, session cookies, magic links, and the add-passkey onboarding flow."
author: "systemprompt.io"
slug: "authentication"
keywords: "authentication, login, OAuth, PKCE, passkey, magic link, session, JWT, security"
kind: "guide"
public: true
tags: ["authentication", "security", "login"]
published_at: "2026-03-02"
updated_at: "2026-03-02"
after_reading_this:
  - "Understand the OAuth 2.0 + PKCE login flow used by the admin dashboard"
  - "Know how session cookies and JWT tokens manage authenticated access"
  - "Use magic links as an alternative sign-in method"
  - "Complete the add-passkey onboarding flow for new users"
related_docs:
  - title: "Access Control"
    url: "/documentation/access-control"
  - title: "Users"
    url: "/documentation/users"
  - title: "Profile"
    url: "/documentation/profile"
  - title: "Getting Started"
    url: "/documentation/getting-started"
---

# Authentication

**TL;DR:** systemprompt.io uses OAuth 2.0 with PKCE for secure, passwordless authentication. When you visit the login page you are automatically redirected to the OAuth authorization flow, and upon success a JWT access token is stored as a session cookie. Magic links provide an email-based alternative for marketplace access, and the add-passkey page lets new users register their credentials.

## What You'll See

When you navigate to `/admin/login`, the login page displays a brief "Redirecting to sign in..." message with a loading spinner. There is no username/password form -- the page immediately begins the OAuth authorization flow in the background. If something goes wrong, an error message appears along with a "Try again" link.

The page has three visual states:

| State | What You See |
|-------|-------------|
| **Loading** | A spinner with "Processing..." while the OAuth flow executes |
| **Error** | A red error message describing what went wrong |
| **Retry** | A "Try again" link that restarts the login flow |

## OAuth 2.0 + PKCE Flow

The login page implements a full OAuth 2.0 Authorization Code flow with Proof Key for Code Exchange (PKCE). This is the primary authentication method for the admin dashboard.

### How It Works

1. **Start** -- The login page generates a random PKCE code verifier (64 characters) and computes a SHA-256 code challenge. Both are stored in `sessionStorage`.
2. **Authorize** -- The browser redirects to `/api/v1/core/oauth/authorize` with the code challenge, a CSRF state parameter, client ID (`marketplace-admin`), and scope (`user`).
3. **User authenticates** -- The OAuth provider handles the actual credential verification (this is delegated to the configured identity provider).
4. **Callback** -- The OAuth provider redirects back to `/admin/login?code=...`. The login page extracts the authorization code from the URL.
5. **Token exchange** -- The page sends the authorization code and the original PKCE code verifier to exchange for tokens.
6. **Session created** -- The returned JWT access token is stored as an `access_token` cookie, and a server-side session is established.
7. **Redirect** -- The user is sent to the admin dashboard (or to a custom redirect URL if one was specified via the `?redirect=` query parameter).

### Token Details

| Property | Value |
|----------|-------|
| **Cookie name** | `access_token` |
| **Token format** | JWT (JSON Web Token) |
| **Default expiry** | 3600 seconds (1 hour), configurable via OAuth response |
| **Cookie flags** | `path=/`, `SameSite=Lax`, `Secure` (HTTPS only) |
| **Required scopes** | `user` or `admin` |

The JWT payload includes the user ID, username, email, scope, and expiration timestamp. The token is validated server-side on every request using the configured JWT secret and issuer.

### Automatic Token Check

If you visit `/admin/login` while already holding a valid access token, the page skips the OAuth flow and redirects you directly to the dashboard. The token is validated client-side by checking the JWT expiry and scope before redirecting.

## Session Management

Once authenticated, every request to the admin dashboard passes through two middleware layers:

1. **User context middleware** -- Extracts the JWT from the `access_token` cookie, validates it, and loads the user's roles and department from the database. This information is made available to all page handlers as a `UserContext`.
2. **Auth check middleware** -- For protected routes, verifies that a valid (non-empty) user ID exists. Returns HTTP 401 if not authenticated.

The `UserContext` contains:

| Field | Description |
|-------|-------------|
| `user_id` | Unique user identifier |
| `username` | Display name |
| `email` | User email address |
| `roles` | Array of assigned roles (e.g., `admin`, `developer`, `analyst`, `viewer`) |
| `department` | User's department |
| `is_admin` | Whether the user has the `admin` role |

### Signing Out

To sign out, clear the `access_token` cookie. The login page does this automatically when starting a new OAuth flow.

## Magic Link Authentication

Magic links provide an email-based authentication alternative, primarily used for marketplace access. Instead of going through the OAuth flow, users receive a one-time link via email.

### How Magic Links Work

1. **Request** -- A POST request is sent to the magic link endpoint with the user's email address.
2. **Token generation** -- If the email belongs to an existing user, a random 32-byte token is generated and its SHA-256 hash is stored in the database with a 15-minute expiry.
3. **Email delivery** -- The raw token is included in a link sent to the user's email.
4. **Validation** -- When the user clicks the link, the token is validated and consumed (single-use). The response includes the user's email for session establishment.

### Rate Limiting

Magic link requests are rate-limited to 3 tokens per email address within a 15-minute window. Requests beyond this limit return a success response (to prevent email enumeration) but do not generate new tokens.

### Security Properties

| Property | Detail |
|----------|--------|
| **Token length** | 32 random bytes (64 hex characters) |
| **Storage** | SHA-256 hash only (raw token never stored) |
| **Expiry** | 15 minutes from creation |
| **Usage** | Single-use (consumed on validation) |
| **Enumeration protection** | Same response regardless of whether the email exists |

## Add Passkey

The add-passkey page (`/admin/add-passkey`) is a public route (no authentication required) used during user onboarding to register a new passkey or credential. This page is accessible without an existing session, allowing invited users to set up their authentication method before their first login.

The add-passkey route is one of only two public admin routes -- the other being the login page itself. All other admin pages require an authenticated session.

## Public vs. Protected Routes

The admin dashboard enforces authentication at the routing level:

| Route | Access |
|-------|--------|
| `/admin/login` | Public -- no authentication required |
| `/admin/add-passkey` | Public -- no authentication required |
| `/admin/*` (all other pages) | Requires valid session (JWT cookie) |
| `/admin/api/*` (write operations) | Requires valid session + admin role for admin-only endpoints |

Pages that require admin privileges (such as Access Control and user management) perform an additional `is_admin` check and return HTTP 403 with an "Admin access required" message if the user lacks the admin role.
