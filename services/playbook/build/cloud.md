---
title: "Cloud Infrastructure Playbook"
description: "Understanding DNS, SSL certificates, multi-tenant routing, and cloud infrastructure management."
---

# Cloud Infrastructure Playbook

Understanding DNS, SSL certificates, and multi-tenant routing for SystemPrompt Cloud.

---

## Architecture Overview

SystemPrompt Cloud uses a multi-tenant architecture with a central Management API that handles SSL termination and routes requests to individual tenant applications.

```
┌─────────────────────────────────────────────────────────────────────────┐
│                              INTERNET                                    │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────────────┐
│                         DNS (Cloudflare)                                 │
│                                                                          │
│   *.systemprompt.io  ──────────────────►  Management API IP              │
│                                                                          │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────────────┐
│                      Management API (Proxy)                              │
│                                                                          │
│   • Wildcard SSL certificate (*.systemprompt.io)                         │
│   • Extracts subdomain from Host header                                  │
│   • Looks up tenant by subdomain                                         │
│   • Routes request to tenant app via internal replay                     │
│                                                                          │
└────────────────────────────────┬────────────────────────────────────────┘
                                 │
         ┌───────────────────────┼───────────────────────┐
         │                       │                       │
┌────────▼────────┐    ┌────────▼────────┐    ┌────────▼────────┐
│   sp-tenant-a   │    │   sp-tenant-b   │    │   sp-tenant-c   │
│                 │    │                 │    │                 │
│  Own IP address │    │  Own IP address │    │  Own IP address │
│  Own resources  │    │  Own resources  │    │  Own resources  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

---

## DNS Configuration

### Wildcard DNS

All tenant subdomains use a **wildcard DNS record** that points to the Management API:

```
*.systemprompt.io  →  Management API IP
```

### How DNS Resolution Works

1. User requests `abc123.systemprompt.io`
2. Cloudflare resolves `*.systemprompt.io` wildcard
3. Request arrives at Management API
4. Management API reads `Host: abc123.systemprompt.io`
5. Proxy routes to `sp-abc123` tenant app

### Verifying DNS

```bash
# Check wildcard resolution
dig +short '*.systemprompt.io' A

# Check specific subdomain (should match wildcard)
dig +short {tenant-id}.systemprompt.io A

# Compare - both should return the same IP
```

---

## SSL Certificates

### Certificate Architecture

**Critical Rule**: All SSL certificates for `*.systemprompt.io` subdomains **must** be configured on the **Management API**, not on individual tenant apps.

```
┌─────────────────────────────────────────────────────────────┐
│                    Management API                            │
│                                                              │
│   Certificates:                                              │
│   ├── *.systemprompt.io (wildcard) ✓ REQUIRED               │
│   ├── api.systemprompt.io ✓ Optional (explicit)             │
│   └── {tenant}.systemprompt.io ✓ Auto-added as needed       │
│                                                              │
└─────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────┐
│                    Tenant App (sp-xxx)                       │
│                                                              │
│   Certificates:                                              │
│   └── (NONE for *.systemprompt.io) ✗ Never add here         │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Why This Matters

If a certificate is added to both the Management API and a tenant app for the same hostname:

1. The edge router doesn't know which app should handle TLS
2. SSL handshake fails with "unexpected eof"
3. Requests never reach either app

### Certificate Commands

```bash
# List certificates on Management API
systemprompt cloud certs list

# Check specific certificate status
systemprompt cloud certs show {subdomain}.systemprompt.io

# Add certificate (Management API only)
systemprompt cloud certs add {subdomain}.systemprompt.io

# Remove certificate
systemprompt cloud certs remove {subdomain}.systemprompt.io -y
```

### Certificate States

| Status | Meaning | Action |
|--------|---------|--------|
| Ready | Certificate issued and active | None |
| Awaiting certificates | Let's Encrypt is issuing | Wait 30-60 seconds |
| Awaiting configuration | DNS not pointing to correct IP | Fix DNS or remove conflicting cert |

---

## Troubleshooting

### Site Unreachable (SSL Error)

**Symptoms:**
- `curl: (35) error:0A000126:SSL routines::unexpected eof while reading`
- Browser shows "This site can't be reached" or SSL error

**Diagnosis:**

```bash
# 1. Check if tenant app responds directly (bypasses proxy)
curl -sI https://sp-{tenant-id}.fly.dev/

# 2. Check certificate status
systemprompt cloud certs list

# 3. Look for certificate on tenant app (should be empty)
fly certs list -a sp-{tenant-id}
```

**Fix:**

```bash
# Remove certificate from tenant app
fly certs remove {subdomain}.systemprompt.io -a sp-{tenant-id} -y

# Add certificate to Management API
fly certs add {subdomain}.systemprompt.io -a management-api-prod

# Wait for issuance (check status)
fly certs show {subdomain}.systemprompt.io -a management-api-prod
```

### 502 Bad Gateway

**Symptoms:**
- Site loads but shows 502 error
- SSL works (HTTPS connection established)

**Diagnosis:**

```bash
# Check tenant app status
systemprompt cloud status

# Check if tenant app is running
systemprompt cloud logs -f
```

**Causes:**
- Tenant app crashed or not started
- Tenant app name mismatch (proxy looking for wrong app)
- Internal network issue

### DNS Mismatch

**Symptoms:**
- Certificate shows "Awaiting configuration"
- Error: "A Record does not match app's IP"

**Diagnosis:**

```bash
# Check where DNS points
dig +short {subdomain}.systemprompt.io A

# Check Management API IP
fly ips list -a management-api-prod
```

**Fix:**
- Ensure wildcard DNS points to Management API IP
- Do not create individual DNS records that override the wildcard

---

## Request Flow Debugging

### Tracing a Request

```bash
# 1. Verify DNS resolution
dig +short {tenant-id}.systemprompt.io A

# 2. Test direct connection to Management API
curl -v https://{tenant-id}.systemprompt.io/ 2>&1 | head -30

# 3. Check Management API logs for routing
fly logs -a management-api-prod | grep {tenant-id}

# 4. Check tenant app logs
systemprompt cloud logs
```

### Expected Log Flow

```
# Management API log (successful routing)
INFO Replaying request to tenant app subdomain={tenant-id} fly_app=sp-{tenant-id}

# Tenant app log (request received)
INFO Request received path=/ method=GET
```

---

## Post-Deployment Checklist

After deploying a new tenant:

| Check | Command | Expected |
|-------|---------|----------|
| Tenant app running | `systemprompt cloud status` | Status: started |
| SSL certificate | `fly certs show {subdomain} -a management-api-prod` | Status: Ready |
| Site accessible | `curl -sI https://{subdomain}.systemprompt.io/` | HTTP/2 200 |
| No conflicting certs | `fly certs list -a sp-{tenant-id}` | Empty |

---

## Quick Reference

| Task | Command |
|------|---------|
| Check cloud status | `systemprompt cloud status` |
| View logs | `systemprompt cloud logs -f` |
| List certificates | `fly certs list -a management-api-prod` |
| Add certificate | `fly certs add {subdomain}.systemprompt.io -a management-api-prod` |
| Remove certificate | `fly certs remove {subdomain}.systemprompt.io -a management-api-prod -y` |
| Check DNS | `dig +short {subdomain}.systemprompt.io A` |
| Test connectivity | `curl -sI https://{subdomain}.systemprompt.io/` |
