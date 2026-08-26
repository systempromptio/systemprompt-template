# REQ-053 — ADFS SSO: design + questions for Astound

Status: prep. Implementation starts once Astound answers the questions below.
Owner ref: Sergey Kubrakov (stakeholder), register row 54.

## What exists today (evidence)

- Salesforce SSO is implemented entirely in this repo, on core primitives:
  `extensions/web/admin/src/handlers/salesforce_auth/{start,callback,identity,tokens,unlink}.rs`,
  config in `services/web/config/salesforce.yaml`. The callback exchanges the
  code (PKCE S256), gates on verified email + allow-listed domain, resolves a
  federated identity `{issuer, external_sub} -> users.id` (core
  `federated_identity.rs`), and mints the session JWT via core
  `SessionCreationService` / `generate_jwt`.
- Core has a generic trusted-issuer seam: `TrustedIssuer { issuer, jwks_uri,
  audience, typ_allowlist, allowed_client_ids, can_issue_id_jag }` on the
  profile security config, consumed by the RFC 8693 token-exchange plane. No
  `trusted_issuers` are declared on this instance yet.
- There is **no SAML or WS-Fed code anywhere** in core or this repo.
- Roles live in `users.roles`; nothing maps IdP claims to roles today. The
  closest pattern is the derived `salesforce: linked` subject dimension
  (`services/access-control/salesforce.yaml` projected by
  `repositories/config/salesforce_yaml_loader.rs`, dimension registered in
  `extensions/web/admin/src/authz/salesforce.rs`).

## Proposed shape

1. `extensions/web/admin/src/handlers/adfs_auth/` — sibling of
   `salesforce_auth`: OIDC authorization-code + PKCE against the ADFS
   `/adfs/oauth2/authorize` + `/adfs/oauth2/token` endpoints; id_token
   validated against the ADFS JWKS; identity resolved through the same
   federated-identity table keyed on the ADFS issuer.
2. `services/web/config/adfs.yaml` — enabled flag, authority URL, client id,
   redirect_uri, allowed email domains, auto-provision policy.
3. **Group → permission mapping** (the genuinely new build):
   `services/access-control/adfs.yaml` mapping AD group names/SIDs from the
   token's group claim to systemprompt roles (`admin`, `user`), departments,
   and organization membership. Projected at login into `users.roles` /
   membership rows (and optionally a derived `adfs` subject dimension for
   entity grants, mirroring the Salesforce pattern). Changes to AD groups take
   effect at next login/token refresh — see Q5.
4. Deny by default: a user whose token carries no mapped group gets no session
   (register acceptance criterion).

## Questions for Astound (blocking implementation)

1. **ADFS version / protocol.** Is the farm ADFS 2016+ with OIDC enabled
   (`/adfs/.well-known/openid-configuration` reachable)? If it is SAML/WS-Fed
   only, scope changes materially — no SAML stack exists in the product.
2. **Federation metadata + test tenant.** Discovery URL for a
   test/staging ADFS, and a client registration (client id, redirect URI
   `https://astound.systemprompt.io/admin/auth/adfs/callback`) we can develop
   against. Confidential client with secret, or public + PKCE?
3. **Group claims.** A sample id_token/access_token payload showing how groups
   are emitted (claim name, group names vs SIDs, nesting flattened or not,
   token-size behaviour for users in many groups).
4. **The mapping itself.** Which AD groups correspond to: systemprompt admin,
   regular user, and each project/team grouping you want reflected? (Project
   entitlements also need DEC-001 answered.)
5. **Synchronization policy.** Register acceptance says group changes are
   reflected "according to the agreed synchronization/session policy" — is
   next-login sufficient, or is mid-session revocation required (that means a
   periodic re-check or short session TTL; state the acceptable TTL)?
6. **Deprovisioning.** Is disabling the AD account (SSO stops working) enough,
   or must the systemprompt account be closed/offboarded automatically? (The
   Salesforce reconciliation job is the existing pattern.)
7. **DEC-002 closure.** This requirement implies AD/ADFS is the identity
   source of truth. Confirm, so Salesforce-as-IdP stays a linking mechanism
   for the Salesforce MCP only and DEC-002 can be marked decided.

## Estimate (once answered, OIDC path)

~2 weeks: handler + config + JWKS validation ~1 wk; group→role/department/org
mapping loader + projection + deprovision policy + e2e (login, claim mapping,
deny-on-no-group) ~1 wk.
