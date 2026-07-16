# Deploy the gateway on DigitalOcean (1-Click droplet)

Runs the `systemprompt-gateway` server on a single DigitalOcean droplet with a bundled Postgres — one self-contained VM.

## Install

1. Deploy the **systemprompt** 1-Click App from the DigitalOcean Marketplace (link will land here once the listing is live). Recommended size: 2 vCPU / 4 GB.
2. SSH in as root. Setup starts automatically and asks for your AI provider API key(s) — at least one of Anthropic / OpenAI / Gemini is required. The gateway will not start until a key is provided.
3. Setup starts the stack, runs migrations and the publish pipeline (several minutes), then prints your gateway URL (`http://<droplet-ip>:8080`).

Re-run setup any time: `sudo /opt/systemprompt/setup.sh`. Configuration lives in `/opt/systemprompt/.env`; apply changes with `systemctl restart systemprompt`.

## Custom domain + TLS

Point a domain at the droplet, set `EXTERNAL_URL=https://your.domain` in `/opt/systemprompt/.env`, put your preferred TLS terminator in front of port 8080 (e.g. Caddy/nginx), and restart the service.

Docs: https://systemprompt.io/documentation/?utm_source=digitalocean&utm_medium=install_doc
