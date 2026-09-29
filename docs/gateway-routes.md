# Gateway routes: providers, CLI configuration, and access control

`POST /v1/messages` at the Anthropic wire format. Every inference request flows through the same governance pipeline as every tool call, on infrastructure you operate.

- **SDK- and Claude-Desktop-compatible.** Authenticated with a systemprompt JWT in `x-api-key` (falls back to `Authorization: Bearer`). No new credential type — existing user JWTs serve as the gateway credential.
- **Routes by `model_pattern`.** Built-in tags: `anthropic`, `openai`, `moonshot` (Kimi), `qwen`, `gemini`, `minimax`. Anthropic is a transparent byte proxy (extended thinking, cache-control headers, SSE events preserved verbatim). OpenAI-compatible providers get full Anthropic↔OpenAI request/response/SSE conversion. Upstream API keys resolve from the secrets file by name.
- **Zero overhead when disabled.** The `/v1` router mounts only if `gateway.enabled: true` in `services/ai/gateway.yaml`.

## Services YAML

The catalog and the routes are services configuration, shipped in the image beside the agents and MCP servers — `services/ai/providers.yaml` and `services/ai/gateway.yaml`, both listed in `services/config/config.yaml` `includes:`. They are not profile sections: a profile that still carries `providers:` or `gateway:` fails to load with a message naming these files. Nothing in either file varies per environment; the credentials they name by `api_key_secret` live in each environment's secret store.

```yaml
providers:
  - name: anthropic
    protocol: anthropic
    endpoint: https://api.anthropic.com/v1
    api_key_secret: anthropic
    models:
      - id: claude-sonnet-5
  - name: minimax
    protocol: anthropic
    endpoint: https://api.minimax.io/anthropic/v1
    api_key_secret: minimax
    models:
      - id: MiniMax-M2
gateway:
  enabled: true
  default_provider: anthropic
  routes:
    - model_pattern: "claude-*"
      provider: anthropic
    - model_pattern: "MiniMax-*"
      provider: minimax
```

Each provider is declared once under `providers:` — its wire `protocol`, `endpoint`, `api_key_secret`, and the `models` it serves (each with optional `aliases` and `upstream_model`). Gateway `routes` carry no connectivity; they only map a requested `model_pattern` to a provider by name, and `default_provider` forwards any model no route matches.

Routes evaluate in order; first `model_pattern` match wins. On a model entry, `upstream_model` aliases a client-requested model to a different upstream name without the client knowing.

**Hosting is read from the endpoint.** A provider on `aiplatform.googleapis.com` is Vertex AI. For those providers the secret is a Google service account, which is exchanged for an OAuth token and fills `{project}` in the endpoint. The request envelope follows the platform: Claude on Vertex (`wire: anthropic`, `…/publishers/anthropic`) is posted to `/models/<id>:rawPredict` or `:streamRawPredict`, with `anthropic_version` in the body. The gateway and the in-process AI service share this resolution, so any provider one of them can reach, the other can too. This template ships the Anthropic, OpenAI and Gemini public APIs (`services/ai/providers.yaml`); a Vertex provider is an addition, not a default.

## Configuring routes from the CLI

Worked example: proxy every Anthropic model to Gemini Flash. Instead of hand-editing the services files, use `admin config`, which edits them, validates against the merged catalog, and reconciles access control. To make a client that asks for `claude-*` actually serve Google Gemini Flash:

```bash
# 1. Store the upstream key and register the provider + model in services/ai/providers.yaml
systemprompt admin config secret set gemini <GEMINI_API_KEY>
systemprompt admin config catalog provider add --name gemini --protocol gemini \
  --endpoint https://generativelanguage.googleapis.com/v1beta --api-key-secret gemini
systemprompt admin config catalog model add --provider gemini --id gemini-2.5-flash

# 2. Point the claude-* route at gemini and rewrite the upstream model name
systemprompt admin config gateway route add --model-pattern 'claude-*' \
  --provider gemini --upstream-model gemini-2.5-flash
```

A client `POST /v1/messages` with `model: claude-haiku-4-5` then returns `model: gemini-2.5-flash`.

## Client attribution

Every `/v1/messages` row names the client that sent it (`ai_requests.client_kind`) and how
strongly that is evidenced (`ai_requests.client_attestation`). Hosts the bridge provisions —
Claude Code, Claude Desktop, Codex, OpenCode, Hermes — are attested by the bridge's per-host
token with nothing to configure; a direct client (Pi, a raw SDK, curl) declares itself with
`x-systemprompt-client: <kind>` from the closed vocabulary, or is recorded as `other` with its
User-Agent and SDK headers kept as evidence. The full tier table, the header names, the 400
behaviour for a malformed declaration, and copy-paste SDK examples are on the hosted page:
[Gateway → Client attribution](https://systemprompt.io/documentation/services/gateway#client-attribution).
On this instance the tier is the tone of the client chip on every conversation page, and
`systemprompt infra logs request list` shows it as the `attestation` column.

## Routes are access-controlled

Each route is gated by an `access_control_entities` row keyed on its id, which is content-addressed (`hash(model_pattern, provider)`). Changing a route's provider mints a *new* id, so a freshly-edited route is denied (`unknown to access control`) until the catalog is reconciled. Reconciliation makes the catalog equal to the live gateway.yaml's routes — new ids are registered, and rows no route produces any more are deleted along with their grants — and it happens in two places:

- **At boot** — the `governance_bootstrap` job reconciles the catalog from the loaded services config, then seeds an empty `access_control_rules` from `services/access-control/rules.yaml`, expanding its `gateway_route/*` glob against that catalog (a non-empty table is only compared and its drift reported on `/admin/sync`).
- **After a CLI edit** — `systemprompt admin config gateway route …` reconciles immediately, so the edit takes effect without a restart.

Routes are granted by a glob, never by a literal id:

```yaml
- entity: gateway_route/*
  default: open
  why: Every model route is reachable by every signed-in role.
  allow:
    role: [user]
```

Route ids are generated, so there is no id to write out — a literal `gateway_route/<id>` fails the boot by name, and `scripts/validate-services.sh` rejects it in CI. The only form written is `gateway_route/*`, expanded at boot against the live catalog.

## Extensible provider registry

`GatewayRoute.provider` is a free-form string resolved at dispatch time against a startup-built registry. Extension crates register new upstreams with:

```rust
inventory::submit! {
    systemprompt_api::services::gateway::GatewayUpstreamRegistration {
        tag: "my-provider",
        factory: || std::sync::Arc::new(MyUpstream),
    }
}
```

The `GatewayUpstream` trait (`async fn proxy(&self, ctx: UpstreamCtx<'_>)`) is the single integration seam. Built-in tags seeded automatically; extension tags may shadow built-ins (logged as a warning). Full detail: [`core/CHANGELOG.md`](https://github.com/systempromptio/systemprompt-core/blob/main/CHANGELOG.md#030---2026-04-22).

## Bridge self-update feed

The desktop bridge updates itself through this gateway. Release assets live in a
private GitHub repository that the bridge holds no credential for, so the
gateway resolves the newest `bridge-v*` release and proxies the bytes:

| Route | Purpose |
|-------|---------|
| `GET /v1/bridge/latest?platform=<slug>` | Version, SHA-256, size, and release-notes URL for the newest published build |
| `GET /v1/bridge/download/{platform}` | Streams that platform's asset |

Both are authenticated exactly like the other `/v1/bridge/*` routes. Platform
slugs are `macos`, `windows`, `linux-x86_64`, and `linux-aarch64`.

```yaml
gateway:
  enabled: true
  bridge_releases:
    repo: systempromptio/systemprompt-core
    # Only for a private repo: the secret's name, not its value, so the token
    # never lands in a config file. Needs `contents: read` on the repo.
    # token_secret: SYSTEMPROMPT_BRIDGE_RELEASES_TOKEN
    tag_prefix: bridge-v
    assets:
      windows: systemprompt-bridge-x86_64-pc-windows-msvc.exe
      linux-x86_64: systemprompt-bridge-x86_64-unknown-linux-gnu
```

This is what `services/ai/gateway.yaml` ships. There is no `macos` slug: the
self-updater installs a zipped `.app` (unpacked with `ditto`, its signature
verified before it is swapped into `/Applications`), and core publishes a bare
darwin binary that only the `update` CLI command can install. A fleet that
publishes its own signed bundles adds `macos: <name>.zip`. Asset names must
match the release's file names exactly.

The advertised `sha256` is read from the release's `SHA256SUMS` — generated and
cosign-signed by the release workflow — rather than computed here, so the digest
the updater enforces is the one that was signed at publish time. A download that
does not match it is discarded and never executed.

**Omitting `bridge_releases` disables updates rather than breaking them.** The
endpoints answer `404`, and the bridge treats a failed check as a debug log:
the button stays as it is and no error reaches the user.

**Staged rollouts and pinning** are config, not a client release. Set
`pinned_version: 0.1.6` to hold a fleet on one build; remove it to resume
tracking the newest release.
