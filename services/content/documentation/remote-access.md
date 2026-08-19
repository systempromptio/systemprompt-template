---
title: "Expose Your Instance Remotely"
description: "Take the gateway from 127.0.0.1 to a public HTTPS URL: profile changes, nginx with TLS, and why you can stay on the local profile."
author: "Astound Digital"
slug: "remote-access"
keywords: "remote access, 0.0.0.0, bind address, nginx, TLS, https, reverse proxy, profile, local, production, cors, jwt issuer, trusted proxies"
kind: "guide"
public: true
tags: ["documentation", "deployment", "configuration"]
published_at: "2026-08-19"
updated_at: "2026-08-19"
after_reading_this:
  - "Make the gateway listen beyond 127.0.0.1"
  - "Know every profile.yaml key that must change for a remote URL"
  - "Put nginx with TLS in front of the instance"
  - "Know why staying on the local profile is correct"
related_playbooks:
  - title: "Install the Desktop Bridge"
    url: "/documentation/bridge-install"
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
---

# Expose Your Instance Remotely

A fresh install binds `127.0.0.1:8080` — reachable only from the machine it
runs on. To use the web admin and connect Claude Code from your laptops, the
instance needs a public HTTPS URL. This page covers exactly what changes.

## Do I need to switch profiles?

**No.** Stay on the `local` profile. Profiles are just named configuration
directories under `.systemprompt/profiles/<name>/`; nothing about the `local`
profile is inherently local-only — its `server:` block simply defaults to
loopback. "Production" is not a mode you switch into; it is another profile
directory with different values. Edit `local` in place.

## The change set

All of it lives in `.systemprompt/profiles/local/profile.yaml`. With nginx
terminating TLS on the same VM (the recommended setup), the changes are:

```yaml
server:
  host: 0.0.0.0                                # was 127.0.0.1
  port: 8080
  api_server_url: https://gateway.example.com   # public URL
  api_internal_url: http://localhost:8080       # stays loopback — in-process calls
  api_external_url: https://gateway.example.com # public URL
  use_https: true
  cors_allowed_origins:
    - https://gateway.example.com
  trusted_proxies:
    - 127.0.0.0/8                               # nginx on the same host
security:
  jwt_issuer: https://gateway.example.com       # must match api_external_url
```

Replace `gateway.example.com` with your hostname. Note:

- `api_internal_url` **stays** `http://localhost:8080` — it is what the server
  uses to call itself.
- `jwt_issuer` must be the same public URL as `api_external_url`, or issued
  tokens will not validate.
- `trusted_proxies` lists the CIDRs your reverse proxy connects from. Left
  empty, `X-Forwarded-For` headers are ignored and every request appears to
  come from the proxy. An invalid entry fails profile load — deliberately.
- If nginx sits on a different machine, bind `host` to the interface it
  reaches and put that machine's CIDR in `trusted_proxies`. If you bind
  `0.0.0.0` directly to the internet with no proxy, firewall port 8080 so only
  the proxy — or nothing — reaches it.

Most of this is also editable from the CLI, no hand-edited YAML:

```bash
systemprompt admin config server show
systemprompt admin config server set --host 0.0.0.0 --use-https true \
  --api-server-url https://gateway.example.com \
  --api-external-url https://gateway.example.com
systemprompt admin config server cors add https://gateway.example.com
systemprompt admin config security set --jwt-issuer https://gateway.example.com
```

`trusted_proxies` is YAML-only. Restart the server after either route.

## HTTPS is mandatory, not cosmetic

The sign-in cookie is set with the `Secure` flag. Over plain HTTP the browser
drops it, so login appears to succeed and then immediately forgets you.
Passkeys (WebAuthn) likewise require a secure context. Do not skip TLS and
expect a degraded-but-working setup — authentication simply will not stick.

## nginx with TLS

A minimal server block, with certificates from certbot/Let's Encrypt:

```nginx
server {
    listen 443 ssl http2;
    server_name gateway.example.com;

    ssl_certificate     /etc/letsencrypt/live/gateway.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/gateway.example.com/privkey.pem;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto https;
        proxy_read_timeout 300s;      # streaming inference responses
        proxy_buffering off;          # do not buffer SSE streams
    }
}

server {
    listen 80;
    server_name gateway.example.com;
    return 301 https://$host$request_uri;
}
```

With nginx proxying to loopback you may keep `host: 127.0.0.1` — the URL,
CORS, issuer, and `trusted_proxies` changes are still required.

## Verify

```bash
# From the VM
systemprompt infra services status

# From a laptop
curl -s https://gateway.example.com/v1/
```

Then open `https://gateway.example.com/admin/login` from a laptop, sign in,
and confirm the session survives a page reload — that proves the cookie,
issuer, and CORS settings agree. From there, hand out the bridge:
[Install the Desktop Bridge](/documentation/bridge-install).

## Troubleshooting

| Symptom | Cause |
|---------|-------|
| Login succeeds, then you are signed out on the next page | Serving over plain HTTP — the `Secure` cookie was dropped. Fix TLS. |
| Token or SSO errors after the move | `jwt_issuer` does not match `api_external_url`, or a stale session predates the change. Align them, restart, sign in fresh. |
| Browser console CORS errors | Public origin missing from `cors_allowed_origins`. |
| Every request logs the proxy's IP | `trusted_proxies` does not include the proxy's address. |
| Passkey registration fails remotely | WebAuthn needs a secure context and a stable hostname — register on the final HTTPS URL, not an IP. |
