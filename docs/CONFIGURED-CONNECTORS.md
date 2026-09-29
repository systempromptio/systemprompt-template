# Configured personal MCP accounts

Profile authentication is independent of marketplace membership. Active users
can authorize configured personal connectors; resource rules still govern use.
Enabled servers are listed even when `display_in_web` is false. Servers without
personal authorization show “No authorization required”. Saved accounts remain
disconnectable after a server is removed or disabled.

## Generic OAuth

Add the server YAML to the service aggregator's includes:

```yaml
mcp_servers:
  example-tools:
    type: external
    binary: ''
    package: null
    port: 5050
    endpoint: https://mcp.example.com/mcp
    enabled: true
    display_in_web: false
    oauth:
      required: false
      scopes: [user]
      audience: mcp
      client_id: null
    connector:
      adapter: generic
      scopes: [tools:read]
      authorization_origins:
        - https://login.example.com
      # Omit both for dynamic client registration:
      # client_id_secret: example_mcp_client_id
      # client_secret: example_mcp_client_secret
```

The `oauth` block governs inbound access; `connector` configures outbound OAuth.
Core supplies the credential-broker endpoint automatically when `connector` is
set and `external_auth` is absent. Preserve the existing broker secret.

The resource must publish protected-resource metadata identifying its OAuth
authorization server, whose metadata must advertise PKCE S256. The resource
origin is trusted. Explicitly list any additional discovery, registration,
authorization or token origin. HTTPS is required, including for internal
services with certificates trusted by the host. HTTP redirects are not followed.

Without dynamic registration, configure secret references for a registered
client. Its callback is
`https://YOUR-SERVER/api/public/connectors/example-tools/callback`.
Client secrets never belong in YAML or browser responses.

Generic verification initializes MCP and lists accessible tools. It does not
invent an account identity when no provider identity endpoint exists. Providers
with a dedicated adapter (Atlassian, GitHub) keep their specific verification.

Changing a generic resource or OAuth settings requires reconnection. Refresh
also checks the original issuer and token endpoint. Disconnect removes local
credential access for every enrolled device.

Three optional `connector` keys shape the flow for issuers that need them:

| Key | Effect |
| --- | --- |
| `display_name` | Label on the Connectors row instead of the server id |
| `authorization_params` | Extra query parameters on the authorization request. Only `access_type`, `prompt`, `login_hint` and `hd` are accepted; every parameter the flow itself sets is refused at load |
| `identity: userinfo` | After consent, read `sub` and `email` from the issuer's OIDC `userinfo_endpoint` and show the address on the row. Requires the `openid` scope |

Issuer identifiers are compared as URLs, with a single trailing slash ignored,
so an issuer advertised as `https://idp.example/` in protected-resource metadata
matches `https://idp.example` in its own metadata.

A Google-hosted MCP server is the common case for the two optional keys:
Google issues a refresh token only when offline access is requested, so such a
row carries `authorization_params: {access_type: offline, prompt: consent}` —
without it the connection expires with the hour-long access token — and
`identity: userinfo` so the row shows the Google address.

## The Connectors page

`/admin/connectors` (Account group) is where a person manages their own
connections. A health strip counts configured, connected and broken
connectors and names the next step; the cards below are grouped by what to do
next — *Needs your attention* (reconnect required, verification required,
temporarily unavailable), *Ready to connect*, *Connected*, and *Nothing to do*
(built in, or not open to this account). Each card shows the provider kind,
what it unlocks, the plugins that carry the server (`mcp_servers.include` of
every enabled plugin), the verified account and resource, and one primary
action: **Connect** when nothing is saved, **Test connection** when something
is, **Reconnect** only after a failure. The page is server-rendered from the
same snapshot the browser then polls (`GET /api/public/account/connections`,
`storage/files/js/pages/connectors.js`), and the card model lives in one place
on each side (`handlers/ssr/ssr_connectors_cards.rs`,
`storage/files/js/services/connector-labels.js`).

**Test connection** (`POST /api/public/account/connections/{server}/test`)
returns the snapshot plus a step-by-step report — credential, MCP session,
tools, identity — each with its outcome and duration, so a failed probe says
which stage failed instead of a blank error
(`services/connector_oauth/report.rs`). A probe always reaches the provider;
an ordinary broker call made within 30 seconds of a recorded outage is held
without one, so a provider outage does not turn into one refresh per MCP
request. Every 4xx from a token endpoint (RFC 6749 §5.2 — `invalid_grant`,
`unauthorized_client`, `invalid_client`, …) retires the grant and asks for a
reconnect; only a 5xx, a 429 or a transport failure is treated as an outage.

After consent the browser returns to `/admin/connectors#connector-<server>`.

### Session-attested servers

A server that requires the platform's own OAuth (`oauth.required: true` with
non-empty `oauth.scopes`) and declares no `connector:` block is authenticated
by the caller's signed-in session, not by a grant. Its card reads *Connected*
for anyone whose roles carry those scopes (`admin` means a manage role, `user`
any active account) and *Not open to your account* otherwise; its **Test
connection** checks the session's scope and whether the server answers
(`services/connector_readiness.rs`). There is nothing to connect or
disconnect.

### Readiness

`Connection::readiness` is the one answer to "will this person's calls to the
server work": configured; needs no sign-in, or is entitled, `connected` (or
`temporarily_unavailable`) and verified. The bridge manifest filter and the
`connector:` authorization band both ask it, and the manifest filter records
why it dropped a server a rule admitted in the manifest's diagnostics.

## Connecting a client

`/admin/connect` is the three-step wizard that used to sit on the profile:
choose Claude Code, Claude Desktop or OpenCode, mint a single-use connect code
(ten minutes), then copy the install or sign-in command
(`storage/files/js/pages/connect-code.js`). It also lists the bridge
downloads under `/files/downloads/` and the client guides. The profile page
links to both pages and no longer carries either.

## Failures

Connector accounts are validated by server id, not by a fixed provider list,
so a new generic server needs no migration. Apply migrations with the new
binary before adding a connector; an older binary cannot manage generic
accounts, so do not roll back after creating them without first
disconnecting/removing those accounts.

Authorization database failures stop inference with HTTP 503 and a retry hint.
Governance hooks return an explicit deny envelope. Look for
`authorization_unavailable` in logs; the next request evaluates fresh policy
after database access is restored.

## Gating skills on a connection

A live connection is an authorization subject. `services/access-control/rules.yaml`
accepts a `connector:` band (precedence 160, between `group` and `role`) whose
values are server ids; a person matches while `mcp_connector_accounts` holds a
`connected` row for that server. The band is available to any entity: a
marketplace whose skills only work against a connected account can be opened
to exactly the people who connected it. The subject provider lives in
`extensions/web/admin/src/authz/` beside the group and project providers.

## Tool permissions in the clients

Every tool on a managed server is allowed by default in Claude Code and
Claude Desktop/Cowork — the governance chain already judges each call, so the
client's own "Claude wants to use …" prompt adds no control. The bridge writes
`permissions.allow` rules (`mcp__<server>`, plus `mcp__plugin_<plugin>_<server>`
for each plugin that mirrors the server) into Claude Code's managed settings
file when it is writable and `~/.claude/settings.json` otherwise, and a per-tool
`toolPolicy` map into the desktop `managedMcpServers` policy from the tool list
the server reported to the bridge's auth probe (`metadata/mcp-tools.json`).

Every enabled server must declare `tool_policy` on its deployment — `allow`
(the value every server here carries), `prompt` or `deny`. A server without it
is withheld from the signed manifest and rejected at boot validation:

```yaml
mcp_servers:
  example-tools:
    tool_policy: prompt   # allow | prompt | deny; required
```

The decision travels in the signed manifest (`ManagedMcpServer.tool_policy`
under the `*` key), so a change here reaches every bridge on its next sync.
