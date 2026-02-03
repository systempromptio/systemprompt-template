---
title: "Configuration Playbook"
description: "View and understand system configuration settings."
author: "SystemPrompt"
slug: "cli-config"
keywords: "config, settings, rate-limits, admin, profiles, server"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Configuration Playbook

View system configuration settings (read-only). Configuration is loaded from your profile and cannot be modified at runtime.

---

## Understanding Configuration

SystemPrompt configuration follows a 5-stage bootstrap sequence:

```
ProfileBootstrap → SecretsBootstrap → CredentialsBootstrap → Config → AppContext
```

See the [Bootstrap Sequence Playbook](/playbooks/config-bootstrap) for details.

---

## Configuration Overview

View all configuration settings:

```json
{ "command": "admin config show" }
```

---

## Configuration Domains

### Server Configuration

```json
{ "command": "admin config server show" }
```

Shows:
- Host and port binding
- API URLs (server, internal, external)
- CORS allowed origins
- HTTPS settings

See [Server Configuration Playbook](/playbooks/config-server) for details.

### Security Configuration

```json
{ "command": "admin config security show" }
```

Shows:
- JWT issuer
- Token expiration times
- Allowed audiences

See [Security Configuration Playbook](/playbooks/config-security) for details.

### Paths Configuration

```json
{ "command": "admin config paths show" }
```

Shows:
- System, services, bin paths
- Web path
- Storage and GeoIP paths

See [Paths Configuration Playbook](/playbooks/config-paths) for details.

### Runtime Configuration

```json
{ "command": "admin config runtime show" }
```

Shows:
- Environment type
- Log level
- Output format
- Interactive mode settings

See [Runtime Configuration Playbook](/playbooks/config-runtime) for details.

### Rate Limits

```json
{ "command": "admin config rate-limits show" }
{ "command": "admin config rate-limits list" }
```

Shows:
- Per-endpoint rate limits
- Burst multiplier
- Tier multipliers

See [Rate Limits Playbook](/playbooks/config-rate-limits) for details.

---

## Profile Management

Configuration comes from profiles. To manage profiles:

```json
{ "command": "cloud profile list" }
{ "command": "cloud profile show" }
{ "command": "cloud profile show <profile-name>" }
```

See [Cloud Management Playbook](/playbooks/cli-cloud) for profile commands.

---

## Troubleshooting

**Check current settings**
```json
{ "command": "admin config show" }
```

**Verify paths exist**
```json
{ "command": "admin config paths show" }
```

**Check rate limits**
```json
{ "command": "admin config rate-limits show" }
```

**View active profile**
```json
{ "command": "admin session show" }
```

---

## Related Playbooks

| Playbook | Purpose |
|----------|---------|
| [Bootstrap Sequence](/playbooks/config-bootstrap) | 5-stage initialization sequence |
| [Profile Configuration](/playbooks/config-profiles) | Profile struct and validation |
| [Secrets Management](/playbooks/config-secrets) | Secrets loading and validation |
| [Server Configuration](/playbooks/config-server) | Server and CORS settings |
| [Security Configuration](/playbooks/config-security) | JWT and authentication |
| [Paths Configuration](/playbooks/config-paths) | Directory layout |
| [Runtime Configuration](/playbooks/config-runtime) | Environment and logging |
| [Rate Limits](/playbooks/config-rate-limits) | API throttling |

---

## Quick Reference

| Task | Command |
|------|---------|
| Config overview | `admin config show` |
| Server config | `admin config server show` |
| Security config | `admin config security show` |
| Paths config | `admin config paths show` |
| Runtime config | `admin config runtime show` |
| Rate limits | `admin config rate-limits show` |
| List profiles | `cloud profile list` |
| Show profile | `cloud profile show` |
| Active session | `admin session show` |