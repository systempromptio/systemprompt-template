# Deploy the gateway on CapRover

Deploys the `systemprompt-gateway` server on [CapRover](https://caprover.com) as a one-click app (gateway + bundled Postgres from the GHCR image).

## Install

Once merged into the official one-click repo, find **systemprompt** under **Apps** → **One-Click Apps/Databases**. Until then:

1. **Apps** → **One-Click Apps/Databases** → scroll to **Template** (bottom of the list).
2. Paste [`deploy/caprover/systemprompt.yml`](https://github.com/systempromptio/systemprompt-template/blob/main/deploy/caprover/systemprompt.yml).
3. Fill the form:
   - **Postgres password**: pre-generated; keep it (the database volume is initialised with it on first boot).
   - **Anthropic / OpenAI / Gemini API key**: at least one; the gateway refuses to boot without a provider key.
4. Deploy. First boot runs migrations and the publish pipeline, so allow several minutes before `https://<app>.<root-domain>/api/v1/health` returns 200.

## HTTPS

Enable HTTPS on the app in CapRover right after deploying. The template sets `EXTERNAL_URL` to the `https://` URL from the start: the gateway's profile validation rejects plain-http non-localhost origins, and `EXTERNAL_URL` is baked into the persisted profile on first boot, so changing the env var later has no effect.

Docs: https://systemprompt.io/documentation/?utm_source=caprover&utm_medium=install_doc
