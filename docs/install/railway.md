# Deploy the gateway to Railway

Set `ADMIN_EMAIL` to an email address you control before first boot, alongside at least one AI provider key. The gateway requires this administrator identity.

One-click deploy of the `systemprompt-gateway` server on [Railway](https://railway.com).

## Deploy

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/deploy/systempromptio-the-self-owned-ai-control?referralCode=AQ_ePp&utm_medium=integration&utm_source=template&utm_campaign=generic)

The published template ([`systempromptio-the-self-owned-ai-control`](https://railway.com/deploy/systempromptio-the-self-owned-ai-control)) provisions:

- A `gateway` service from `ghcr.io/systempromptio/systemprompt-template:latest`
- A PostgreSQL service, with private `DATABASE_URL` references and generated credentials
- A gateway state volume at `/app/data`, retaining profiles, signing identity and uploads
- `HOST=::` on the gateway: Railway's private network is IPv6-only, so the server must bind the IPv6 wildcard to reach Postgres and accept proxied traffic

Steps:

1. Click the button, sign in to Railway.
2. Set `ADMIN_EMAIL` to an address you control, and set at least one AI provider key when prompted (`ANTHROPIC_API_KEY`, `OPENAI_API_KEY`, or `GEMINI_API_KEY`).
3. Deploy. First boot runs migrations and the publish pipeline, which takes a few minutes; the healthcheck allows up to 900 seconds before marking the deploy failed.

## Custom domain

Railway → service → Settings → Networking → *Generate Domain* or add your own.

## Deploying from a fork instead

The one-click template is the supported path. If you deploy your own fork of this repo as a Railway service instead, [`deploy/railway/railway.json`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/railway/railway.json) holds the build/deploy settings Railway reads from the repo (Dockerfile build, `/api/v1/health` healthcheck, restart policy). You still need to add a Postgres service yourself, reference its `DATABASE_URL`, and set `HOST=::` plus a provider key: Railway's config-as-code file cannot declare sibling services, which is why the template is the canonical route.

Docs: https://systemprompt.io/documentation/?utm_source=railway&utm_medium=install_doc

## Administrator access

Public registration is disabled. In the gateway service shell, run `systemprompt admin users webauthn generate-setup-token --email "$ADMIN_EMAIL"` and privately open the returned link to enroll your passkey. Set `EXTERNAL_URL=https://${{RAILWAY_PUBLIC_DOMAIN}}` in the template so the link and OAuth issuer use the public HTTPS endpoint. This public URL is for browser access; PostgreSQL traffic stays on the private network.

Template overview source: [`deploy/railway/overview.md`](../../deploy/railway/overview.md). Template configuration is managed independently from the repository build settings; both must be updated when publishing template changes.
