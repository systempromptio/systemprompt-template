# Install the gateway via GitHub Container Registry

`systemprompt` is published to GHCR as [`ghcr.io/systempromptio/systemprompt-template`](https://github.com/systempromptio/systemprompt-template/pkgs/container/systemprompt-template). The image is a single compiled Rust binary plus the `services/` config tree — same binary that ships via Helm and Render.

Pick GHCR when:
- You want a public, rate-limit-free OCI pull source backed by GitHub auth.
- You prefer pulling from the same host as the source repo.
- You already authenticate to `ghcr.io` for other private packages.

## Quickstart — Compose

The image expects a reachable Postgres and at least one AI key. The simplest working setup is the bundled compose file:

```bash
git clone https://github.com/systempromptio/systemprompt-template
cd systemprompt-template
cp .env.example .env   # set ANTHROPIC_API_KEY / OPENAI_API_KEY / GEMINI_API_KEY
# Edit docker-compose.yml: comment `build: .` and uncomment `image: ghcr.io/...:latest`
docker compose up
```

Then open http://localhost:8080.

## Quickstart — standalone `docker run`

You must provide `DATABASE_URL` pointing at a Postgres the container can reach, and at least one AI key:

```bash
docker run --rm -p 8080:8080 \
  -e DATABASE_URL=postgres://user:pw@host.docker.internal:5432/systemprompt \
  -e ANTHROPIC_API_KEY=sk-ant-... \
  ghcr.io/systempromptio/systemprompt-template:latest
```

On first boot the entrypoint writes `/app/services/profiles/docker/{profile.yaml,secrets.json}`, waits for Postgres, runs migrations, and starts the API on port 8080.

## Tags

- `latest` — most recent release.
- `<major>.<minor>.<patch>` (e.g. `0.4.0`), `<major>.<minor>` (e.g. `0.3`), `<major>` (e.g. `0`) — published when a `v*` tag ships through the release pipeline.

If a version tag is missing from GHCR, the release workflow hasn't completed for it yet — pin to `latest` or a tag you can see on the [package page](https://github.com/systempromptio/systemprompt-template/pkgs/container/systemprompt-template).

## Authenticated pulls

Anonymous pulls work (the package is public). If you hit rate limits or need to pull a private fork:

```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u <your-github-username> --password-stdin
```

The token needs `read:packages`.

## Verify signature (versioned releases only)

Versioned tags are signed with cosign (keyless, GitHub OIDC). `latest` is not re-signed on every push.

```bash
cosign verify \
  --certificate-identity-regexp='https://github.com/systempromptio/systemprompt-deploy/' \
  --certificate-oidc-issuer='https://token.actions.githubusercontent.com' \
  ghcr.io/systempromptio/systemprompt-template:<version>
```

The SBOM is attached as a cosign attestation:

```bash
cosign download attestation ghcr.io/systempromptio/systemprompt-template:<version> \
  | jq -r '.payload | @base64d | fromjson | .predicate' > sbom.json
```

## Platforms

Multi-arch (`linux/amd64`, `linux/arm64`) for versioned releases. The `latest` tag tracks whatever arch the most recent successful build produced.

## Environment variables

| Variable | Required | Notes |
|---|---|---|
| `DATABASE_URL` | yes | `postgres://user:pw@host:port/db` |
| `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` | at least one | AI provider keys |
| `JWT_SECRET` | no | Auto-generated on first boot if unset |
| `HTTP_PORT` | no | Exposed host port (default 8080) |
| `HOST` / `PORT` | no | Bind inside container (default `0.0.0.0:8080`) |
| `RUST_LOG` | no | Log filter (default `info`) |

## What's inside

- `/app/bin/systemprompt` — the CLI + server binary.
- `/app/services/` — YAML config (agents, skills, MCP servers, plugins).
- `/app/web/` — prerendered web assets.
- `/app/migrations/` — embedded SQL migrations (run by entrypoint).

To poke around:

```bash
docker run --rm -it --entrypoint /bin/bash ghcr.io/systempromptio/systemprompt-template:latest
```

Docs: https://systemprompt.io/documentation/?utm_source=ghcr&utm_medium=install_doc
