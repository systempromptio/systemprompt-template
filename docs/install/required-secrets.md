# Required secrets

Everything an operator must provide for Claude Code, OpenCode and the other
gateway clients to work end to end. Each provider entry in
[`services/ai/providers.yaml`](../../services/ai/providers.yaml) names the
secret it reads its credential from; the profile secret store supplies the
value per environment. The catalog is shipped in the image and is identical in
every environment — only the secrets differ.

## The secrets

| secret | provider entry | credential | unlocks |
|---|---|---|---|
| `anthropic` | `anthropic` | Anthropic API key | the `claude-*` models, including the default `claude-sonnet-5` |
| `openai` | `openai` | OpenAI API key | the `gpt-*` and `o*` models |
| `gemini` | `gemini` | Gemini API key (public endpoint) | the `gemini-*` models |

One secret is the minimum: `anthropic` alone gives Claude Code and OpenCode a
working default, since `services/ai/gateway.yaml` sets
`default_provider: anthropic` and `default_model: "claude-sonnet-5[1m]"`
(Claude Code's 1M-context form of `claude-sonnet-5`; other hosts get the bare
id). Every other secret adds models to the picker. The container entrypoint
fills these from `ANTHROPIC_API_KEY`, `OPENAI_API_KEY` and `GEMINI_API_KEY` on
first boot.

A model whose provider has no secret is still advertised on `/v1/models` — that
endpoint filters by API surface, not by credential — and fails at dispatch with
`Gateway API key secret '<name>' not configured`. Advertise only what you have
credentialed, or expect that error in the audit trail.

### Gemini: API key, not Vertex

The shipped `gemini` entry targets the **public endpoint**
(`generativelanguage.googleapis.com`), which takes an API key on
`x-goog-api-key`. **Vertex AI** rejects API-key authentication as a class, with
`401 CREDENTIALS_MISSING`; it wants a Google service account. To add Vertex,
declare a separate provider on an `aiplatform.googleapis.com` endpoint whose
secret is the whole service-account JSON document. The gateway recognises it by
its `"type": "service_account"` field, exchanges it for an access token, and
fills the endpoint's `{project}` segment from the key's `project_id` — a
catalog that names a project literally is refused at boot. Model ids must be
unique across the whole catalog, so a Vertex twin of a public model needs its
own id (for example a `vertex-` prefix); two entries claiming the same id fail
at boot with `DuplicateModel`.

## Setting a secret

```bash
systemprompt admin config secret set <name> <value>
```

`set` is the only subcommand. It writes into the secrets file of the **active
profile**, so switch profiles first when targeting a deployed environment
(`systemprompt admin session switch production`). Infrastructure secrets —
database URLs, the at-rest pepper, the signing seed, the encryption master
key — are refused by this command and are provisioned out of band.

The secret store is loaded once at process start, so a newly set secret takes
effect on the next server restart, not immediately.

## Verifying

**The catalog, unauthenticated.** `/v1/models` carries no auth layer, so this
needs no credential and proves only that the catalog loaded and the surface
filter works:

```bash
curl -sS localhost:8080/v1/models | jq -r '.data[].id'
```

**A real dispatch, which is what actually proves a secret.** `/v1/messages`
needs a PAT *and* a minted session; an arbitrary `x-session-id` is rejected
rather than created on demand.

```bash
PAT=$(systemprompt admin users api-key issue --user <user-id> --name scratch \
        | grep -o 'sp-live-[^ ]*')
SID=$(curl -sS -X POST localhost:8080/api/public/gateway/sessions \
        -H "Authorization: Bearer $PAT" -H 'content-type: application/json' \
        -d '{}' | jq -r .session_id)

curl -sS -X POST localhost:8080/v1/messages \
  -H "Authorization: Bearer $PAT" -H "x-session-id: $SID" \
  -H 'content-type: application/json' -d '{
    "model":"gemini-2.5-flash","max_tokens":64,
    "messages":[{"role":"user","content":"ping"}]}' | jq '{stop:.stop_reason, usage}'
```

Substitute the model id for whichever secret you are checking. A credential
problem surfaces as a dispatch error naming the secret.

Every call above lands a row with the user, the model, the tokens and the cost:

```bash
systemprompt infra logs request list --limit 5
```

## How a developer reaches these models

Nothing changes on the developer's machine. The bridge install points Claude
Code at the gateway and enables gateway model discovery, so every credentialed
model appears in the `/model` picker beside the Claude models. OpenCode reads
the same catalog. A request names the catalog id, the gateway routes it by the
patterns in [`services/ai/gateway.yaml`](../../services/ai/gateway.yaml),
rewrites it to the upstream name where `upstream_model` differs, and audits the
call.
