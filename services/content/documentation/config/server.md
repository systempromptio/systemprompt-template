---
title: "Server Configuration"
description: "HTTP server settings including host, port, API URLs, CORS, and HTTPS."
author: "SystemPrompt Team"
slug: "config/server"
keywords: "server, host, port, cors, https, api, url"
image: "/files/images/docs/config-server.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Server Configuration

Server settings control how the HTTP server runs, including network binding, API endpoints, and cross-origin requests.

## Configuration

```yaml
# .systemprompt/profiles/local/profile.yaml
server:
  host: "0.0.0.0"
  port: 8080
  api_server_url: "http://localhost:8080"
  api_internal_url: "http://localhost:8080"
  api_external_url: "http://localhost:8080"
  use_https: false
  cors_allowed_origins:
    - "http://localhost:8080"
    - "http://localhost:5173"
```

## Fields

| Field | Required | Description |
|-------|----------|-------------|
| `host` | Yes | Network interface to bind (`127.0.0.1` or `0.0.0.0`) |
| `port` | Yes | TCP port (must be > 0) |
| `api_server_url` | Yes | Primary API endpoint |
| `api_internal_url` | Yes | Internal service-to-service URL |
| `api_external_url` | Yes | Public URL for external clients |
| `use_https` | No | Enable HTTPS (default: false) |
| `cors_allowed_origins` | No | Allowed CORS origins |

## The 3 API URLs

SystemPrompt uses three distinct URLs:

| URL | Purpose | Example |
|-----|---------|---------|
| `api_server_url` | Primary API | `http://localhost:8080` |
| `api_internal_url` | Service-to-service | `http://app.internal:8080` |
| `api_external_url` | Public access, OAuth callbacks | `https://api.example.com` |

## CORS Configuration

Allow frontend applications to make API requests:

```yaml
server:
  cors_allowed_origins:
    - "http://localhost:5173"      # Vite dev server
    - "https://app.example.com"    # Production frontend
```

### CORS Rules

- Each origin must start with `http://` or `https://`
- No trailing slashes
- No wildcards (`*`)

## Production Example

```yaml
server:
  host: "0.0.0.0"
  port: 8080
  api_server_url: "https://api.example.com"
  api_internal_url: "http://app.internal:8080"
  api_external_url: "https://api.example.com"
  use_https: false                    # TLS at load balancer
  cors_allowed_origins:
    - "https://app.example.com"
```

See the [Server Configuration Playbook](/playbooks/config-server) for detailed technical information.