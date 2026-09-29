# Install with Docker Compose

The image `ghcr.io/systempromptio/systemprompt-template` is a prebuilt,
multi-arch (amd64 + arm64) build of this repository: the server binary, the
bundled MCP server and the `services/` tree. Installing means pulling it — no
Rust toolchain, no compile. For pulling, tags, signatures and a standalone
`docker run`, see [ghcr.md](ghcr.md); this page covers running it with the
repository's `docker-compose.yml` and operating it afterwards.

## 1. Configure

```bash
git clone https://github.com/systempromptio/systemprompt-template.git   # for docker-compose.yml + .env.example
cd systemprompt-template
cp .env.example .env
```

In `docker-compose.yml`, comment `build: .` and uncomment
`image: ghcr.io/systempromptio/systemprompt-template:latest` (or pin a version,
below). Then edit `.env`:

| variable | required | notes |
|---|---|---|
| `ANTHROPIC_API_KEY` / `OPENAI_API_KEY` / `GEMINI_API_KEY` | one of them | the first set becomes the default provider |
| `ADMIN_EMAIL` | yes, first boot | the administrator the container's first-boot `admin setup` creates; compose refuses to start without it |
| `EXTERNAL_URL` | on a server | the public URL, e.g. `https://gateway.example.com` — sets `api_external_url` and CORS |
| `POSTGRES_PASSWORD`, `HTTP_PORT`, `PG_PORT` | no | bundled Postgres and port mapping |
| `DATABASE_URL` | remote DB only | see below |

## 2. Run

```bash
docker compose pull && docker compose up -d
curl -fsS http://localhost:8080/api/v1/health
```

First boot writes the docker profile, waits for Postgres, runs migrations and
bootstraps the administrator. Open `http://localhost:8080/admin` to finish setup.

### Remote Postgres

Set `DATABASE_URL=postgres://user:pass@host:5432/systemprompt` in `.env`, drop
the compose file's `DATABASE_URL` override, and start only the app:
`docker compose up -d app`. The database needs the `uuid-ossp` and `pgcrypto`
extensions; pgvector is not required.

### More than one node

The `.env` path above is **single-node only**. On first boot it authors a
profile inside the container and generates that container's own signing key,
OAuth pepper, encryption key and manifest seed; a second node would generate
different ones and reject every token the first node minted. For two or more
nodes (and for any deployment where secrets come from a vault rather than a
`.env` file) render one profile directory — `profile.yaml` plus
`secrets.json` — make it identical on every node, and bind-mount it with
`SYSTEMPROMPT_PROFILE_DIR`. The entrypoint then uses it as-is and refuses to
start if either file is missing. Kubernetes: [helm.md](helm.md).

### Upgrade, pin, roll back

```bash
docker compose pull && docker compose up -d     # follow latest
```

To pin, set the `image:` line to a version tag, e.g.
`ghcr.io/systempromptio/systemprompt-template:0.62.0`, and `docker compose up -d`.
To roll back, set it to the previous version — versioned tags are never
rewritten. Core migrations are forward-only: roll the image back only to a
version whose migrations match the database.

### Changing configuration between releases

`services/` is baked into the image and read once at boot, so a running node
serves what it was built with. To change it without waiting for the next image,
bind-mount your own copy over `/app/services` and restart — the image already
sets `SYSTEMPROMPT_SERVICES_PATH=/app/services`, so the mount is all that is
needed. Admin-UI edits to gateway routes write inside the container and are
lost when it is replaced unless that tree is mounted, and a `:ro` mount makes
the UI editor fail on write.

## 3. The bridge (Claude Code and Cowork client)

The gateway serves bridge downloads at `<gateway>/files/downloads/` when the
operator places them there (`services/web/config/bridge.yaml`), and the admin
**Bridge Setup** page links them:

| file | platform |
|---|---|
| `systemprompt-bridge-linux-x86_64.tar.gz`, `systemprompt-bridge-linux-aarch64.tar.gz` | Linux |
| `systemprompt-bridge-macos.dmg` | macOS |
| `systemprompt-bridge-windows.exe` | Windows x86_64 |
| `install.sh` | Linux one-liner installer |

Each file has a `.sha256` beside it. Installed bridges update themselves from
the release named by `gateway.bridge_releases` in `services/ai/gateway.yaml`
(see [gateway-routes.md](../gateway-routes.md)).

### Pointing a bridge at a different gateway (local instance, staging)

The bridge verifies every synced manifest against an ed25519 public key. Each
instance signs with its own key (derived from the profile's
`manifest_signing_secret_seed`, generated at `admin setup`), so a local
`just start` instance and a staging gateway never share one. The bridge learns
the key on its first sync (trust-on-first-use) and stores it **together with
the gateway it was learned from**. A pin for one gateway is ignored for another,
and the next sync re-learns it. `systemprompt-bridge doctor` reports which
gateway the pin belongs to.

An administrator can also supply the key out of band (managed policy or the
bridge's policy environment override), and that source always wins. A sync
that fails with *"does not match the pubkey pinned from the policy"* means the
policy holds a key for a different gateway. Read the current gateway's key and
repin:

```bash
curl -s http://localhost:8080/v1/bridge/pubkey
systemprompt-bridge install --apply --pubkey <base64 from the response>
```

### Windows: which registry hive the policy lands in

The Claude Desktop / Cowork policy (`SOFTWARE\Policies\Claude`) is written to
**HKCU** when the bridge runs as an ordinary user, and to **HKLM** when it runs
elevated (or via `install --apply`, which prompts once). Cowork honours HKCU
only while no HKLM key exists, so the bridge refuses an HKCU write that a
conflicting HKLM key would shadow and says so in the sync result. Every write
is read back; a value that did not land is reported, never assumed.
`systemprompt-bridge doctor` shows which hive holds the policy.

## Building from source instead

Leave `build: .` in `docker-compose.yml` and run
`docker compose up --build`. This needs the full toolchain inside the builder
stage and takes a while cold; it exists for Dockerfile work and air-gapped
hosts, not for installs.
