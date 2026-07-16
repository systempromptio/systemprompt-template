# systemprompt on Portainer

Install systemprompt on any Docker host managed by Portainer CE or BE
(tested on CE 2.39). Two paths: the App Templates feed (recommended) or a
manual repository stack.

## Option A — App Templates feed (recommended)

1. In Portainer, go to **Settings → App Templates** and set the URL to:

   ```
   https://raw.githubusercontent.com/systempromptio/systemprompt-template/main/deploy/portainer/templates.json
   ```

   Save. (If you already use the community feed at
   `lissy93/portainer-templates`, systemprompt is included there once the
   listing is merged — no URL change needed.)

2. Open **App Templates** on your environment and select **systemprompt**.

3. Fill in the environment variables:
   - `POSTGRES_PASSWORD` — set before first deploy; the database volume is
     initialised with it and it cannot be changed afterwards by redeploying.
   - `ANTHROPIC_API_KEY` (or `OPENAI_API_KEY` / `GEMINI_API_KEY`) — at least
     one provider key is **required**; the gateway will not boot without one.
   - `EXTERNAL_URL` — optional; the public URL you will serve it on
     (e.g. `https://gateway.example.com`), used for API URL + CORS.
   - `HTTP_PORT` — host port to publish the gateway on (default `8080`).

4. **Deploy the stack.** The first boot runs database migrations and the
   publish pipeline; the healthcheck has a 5-minute grace period, so allow
   several minutes before the containers report healthy.

5. Verify: `http://<host>:<HTTP_PORT>/api/v1/health` returns
   `{"status":"healthy"}`, then open `http://<host>:<HTTP_PORT>/` for the UI.

## Option B — Manual repository stack

**Stacks → Add stack → Repository** and enter:

- Repository URL: `https://github.com/systempromptio/systemprompt-template`
- Reference: `refs/heads/main`
- Compose path: `deploy/compose/one-click.docker-compose.yml`

Add the same environment variables as above, then deploy.

## Notes

- Runs the published image `ghcr.io/systempromptio/systemprompt-template:0`
  (floating major tag — minor/patch releases are picked up on redeploy
  without a template change).
- Postgres 18 is bundled in the stack; no external database needed.
- Web assets, storage, profile state, and Postgres data persist in named
  volumes (`app_web`, `app_storage_data`, `app_profiles`, `postgres_data`).
- Docs: https://systemprompt.io/documentation/
