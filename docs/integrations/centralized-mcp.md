# Centralized MCP connections

A marketplace can expose third-party MCP servers (an issue tracker, a wiki, a
CRM) through this instance's MCP gateway. Each person links their **own**
account on the server; the bridge displays the same server-owned connection
model in its Profile view, and **Manage on server** opens that page. Provider
credentials are never placed in a marketplace bundle or desktop config.

Until the Connectors console page lands, a person links accounts from the
Connections section of `/admin/profile`; the routes below are the same either
way.

## Provisioning

A connector is an `mcp_servers.<id>` entry under `services/mcp/`, listed in
`services/config/config.yaml`'s `includes:`, with a `connector:` block for the
outbound OAuth (shape and adapters: [configured connectors](../CONFIGURED-CONNECTORS.md)).
`enabled: true` is the only enablement switch. The profile page, the bridge
manifest and the credential accessor read the same loaded services
configuration. Restart after changes. There is no separate enablement flag in
secrets or environment variables.

Store the application settings below in the active server profile's secret
store or equivalent uppercase environment variables. Never put credentials in
service YAML. Reuse the instance's existing `encryption_master_key`; do not
rotate it for this integration. `oauth_at_rest_pepper` hashes OAuth identifiers
and cannot replace the encryption key used to recover provider tokens.

| Setting | Value |
| --- | --- |
| `encryption_master_key` | Persistent 32-byte encryption key encoded as 64 hexadecimal characters; required to store OAuth grants, preserve it across restarts |
| `mcp_credential_broker_secret` | Random server-only secret of at least 32 bytes; shared by core and the account accessor in the same instance |
| `<provider>_mcp_client_id`, `<provider>_mcp_client_secret` | Only for a provider without dynamic client registration: the OAuth client the provider issued, named by `client_id_secret` / `client_secret` in the connector block |

Set `server.api_external_url` to the HTTPS origin of the deployment and
provision its credentials independently. Local user grants are not copied into
production. Register one callback per provider under that origin:

- `https://YOUR-SERVER/api/public/connectors/<provider>/callback`

A provider that publishes MCP authorization-server metadata and supports
dynamic client registration (Atlassian's hosted MCP server is one) needs no
client secret at all. Its organization must allow the server callback domain
and, where IP allowlisting is enabled, the server's outbound IP.

After a provider-side reset (a sandbox refresh, a rotated app), `POST
/api/public/admin/connectors/<id>/reprovision` resets every user's grant for
that server.

The registry definitions stay enabled because enabled marketplace plugins must
resolve their server references. The manifest filter independently requires
provider configuration, the person's entitlement (`services/access-control/rules.yaml`,
`mcp_server/<id>`) and a verified connection before publishing any connector to
them. A registered server is not proof of a working provider connection.

## Local development

For the local profile, link accounts at `http://localhost:8080/admin/profile`;
the callback is `http://localhost:8080/api/public/connectors/<provider>/callback`.
Use `localhost` consistently for login and consent so the session cookie is
available when the provider redirects back. If the server runs on a remote
development machine, forward port 8080 to the browser's machine before starting
consent.

A denied OAuth redirect suggests the application's domain is not allowed.
Consent followed by denied tool access can indicate an IP restriction or missing
product permissions on the provider side.

## Acceptance checks

1. Use two entitled users with different provider permissions and one user
   without the entitlement.
2. Link accounts on the server. Check the same identities and states in the bridge.
3. Sync a clean Claude Code installation. Verify the connectors with `/mcp` and
   make a read-only request against each provider, without provider CLI login.
4. Enroll another machine as the same person. Sync and repeat without consent.
5. Disconnect an account. Requests must fail immediately; the account views
   refresh while visible, and manifest sync follows availability changes.
6. Exercise expired grants, provider outages, callback replay/account switches,
   concurrent refresh, and denial for the unentitled user. An outage preserves
   the grant.
7. Check manifests and local config for absence of provider tokens. Check audit
   identity and trace IDs for tool calls and governance denials.

## Verification and deployment

Failed account verification preserves encrypted grants for retry while blocking
connector use until verification succeeds. Tenant identity must match the
provider's authenticated resource list.

Validate bridge synchronization, ordinary-user access, multiple devices,
refresh, disconnect and provider outages for the deployment being released.
Verify MCP session continuity across replicas before using multiple application
instances. A single successful account check does not establish these properties.
