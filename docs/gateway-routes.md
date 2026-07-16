# Gateway routes: providers, CLI configuration, and access control

`POST /v1/messages` at the Anthropic wire format. Every inference request flows through the same governance pipeline as every tool call, on infrastructure you operate.

- **SDK- and Claude-Desktop-compatible.** Authenticated with a systemprompt JWT in `x-api-key` (falls back to `Authorization: Bearer`). No new credential type — existing user JWTs serve as the gateway credential.
- **Routes by `model_pattern`.** Built-in tags: `anthropic`, `openai`, `moonshot` (Kimi), `qwen`, `gemini`, `minimax`. Anthropic is a transparent byte proxy (extended thinking, cache-control headers, SSE events preserved verbatim). OpenAI-compatible providers get full Anthropic↔OpenAI request/response/SSE conversion. Upstream API keys resolve from the secrets file by name.
- **Zero overhead when disabled.** The `/v1` router mounts only if `gateway.enabled: true` in the active profile.

## Profile YAML

```yaml
providers:
  - name: anthropic
    protocol: anthropic
    endpoint: https://api.anthropic.com/v1
    api_key_secret: anthropic
    models:
      - id: claude-sonnet-4-20250514
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

## Configuring routes from the CLI

Worked example: proxy every Anthropic model to Gemini Flash. Instead of hand-editing the profile, use `admin config`. To make a client that asks for `claude-*` actually serve Google Gemini Flash:

```bash
# 1. Store the upstream key and register the provider + model in the profile registry
systemprompt admin config secret set gemini <GEMINI_API_KEY>
systemprompt admin config catalog provider add --name gemini --protocol gemini \
  --endpoint https://generativelanguage.googleapis.com/v1beta --api-key-secret gemini
systemprompt admin config catalog model add --provider gemini --id gemini-2.5-flash

# 2. Point the claude-* route at gemini and rewrite the upstream model name
systemprompt admin config gateway route add --model-pattern 'claude-*' \
  --provider gemini --upstream-model gemini-2.5-flash
```

A client `POST /v1/messages` with `model: claude-haiku-4-5` then returns `model: gemini-2.5-flash`.

## Routes are access-controlled

Each route is gated by an `access_control_entities` row keyed on its id, which is content-addressed (`hash(model_pattern, provider)`). Changing a route's provider mints a *new* id, so a freshly-edited route is denied (`unknown to access control`) until the entity is materialised. The `admin config` CLI edits the profile only; it does **not** reconcile the access-control catalog. Materialise it one of two ways:

- **Re-run the publish pipeline** — `systemprompt infra jobs run publish_pipeline` (also runs via `just publish`). It registers every route in the live profile and the `gateway_route: "*"` wildcard in `services/access-control/roles.yaml` grants them. Dynamic, but must be re-run after every route change.
- **Pin it in `services/access-control/roles.yaml`** (committed, survives a clean install) — add an explicit grant so the ACL loader self-materialises the row at publish time:

  ```yaml
  - entity_type: gateway_route
    entity_id: claude-star-39ccd3   # synthesize_route_id("claude-*", "gemini")
    access: allow
    default_included: true
    roles: [user]
  ```

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
