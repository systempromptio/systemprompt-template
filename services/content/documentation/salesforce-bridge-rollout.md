---
title: "Step 5 — Rolling Out the Bridge"
description: "Install the Astound bridge, point it at your gateway, link a device via Salesforce sign-in, and verify the first live Salesforce query end to end."
author: "Astound Digital"
slug: "salesforce-bridge-rollout"
keywords: "astound bridge, device link, bridge-auth, gateway url, marketplace sync, rollout, salesforce sign in, loopback redirect"
kind: "guide"
public: true
tags: ["salesforce", "bridge", "rollout", "setup"]
published_at: "2026-07-29"
updated_at: "2026-07-31"
after_reading_this:
  - "Install the bridge and point it at your deployed gateway"
  - "Complete the device-link flow with Salesforce SSO or a passkey"
  - "Verify the first live Salesforce query and its audit record"
  - "Support a rollout without needing an admin step per user"
related_playbooks:
  - title: "Start Here — Standing Up the Gateway"
    url: "/documentation/use-case-admin"
  - title: "Overview"
    url: "/documentation/salesforce"
  - title: "Step 3 — Hosted MCP Access"
    url: "/documentation/salesforce-hosted-mcp"
  - title: "Step 4 — Users, Seats and Roles"
    url: "/documentation/salesforce-provisioning"
---

# Step 5 — Rolling Out the Bridge

**TL;DR:** Install `astound-bridge` pointed at your gateway, sign in with
Salesforce, approve the device link, and the Salesforce tools appear. This is
the only step your end users perform, and it takes about two minutes.

## What the Bridge Is

The bridge is a desktop application that connects a local AI client to the
governed gateway. It holds a **platform** credential and nothing else.

This bears emphasis because it is the security property that makes the whole
design work: **the bridge never talks to Salesforce.** It has no Salesforce
token, no client secret, no private key. When a Salesforce tool is called, the
bridge asks the gateway, and the gateway mints and injects the Salesforce
credential server-side. A stolen laptop yields a revocable platform credential,
never Salesforce access.

Its defaults on a user's machine:

| | |
|---|---|
| Binary | `astound-bridge` |
| Config | `astound-bridge.toml` under the `astound` config directory |
| Workspace | `~/Astound` |
| Gateway override | `ASTOUND_BRIDGE_GATEWAY_URL` |

## 1. Set the Gateway URL

The bridge ships with a default gateway of `http://localhost:8080`. **Change
this before distributing it.** Two ways:

```bash
# At install time
astound-bridge install --gateway https://your-host

# Or via environment
export ASTOUND_BRIDGE_GATEWAY_URL=https://your-host
```

For a fleet rollout, bake the gateway URL in at install time so users never see
the setting. This must match the host in step 1's `redirect_uri` — if the two
disagree, the user completes the Salesforce login and then fails at device link,
which is a confusing failure to debug in the field.

## 2. Install and Sign In

The user installs the bridge and opens it. The setup screen offers **Sign in
with Salesforce**, which opens their browser to the gateway.

From there, the flow is step 1's login: the gateway redirects to your Salesforce
org, the user authenticates with their normal Salesforce credentials and MFA,
and the gateway gates the claims, provisions or links the account, and records
the Salesforce Username.

Because it is a browser flow against Salesforce directly, the bridge never sees
the user's credentials, and your org's existing login policies — MFA, IP ranges,
login hours — all apply unchanged.

## 3. Approve the Device Link

After login the user lands on the device-link page, which shows the local
address the bridge is listening on and asks for approval.

Approving issues a one-shot exchange code and redirects it back to the bridge,
which trades it for its lasting gateway credential. Denying returns an error to
the bridge and nothing is issued.

**Only loopback redirects are accepted** — `http://127.0.0.1:PORT` or
`http://localhost:PORT`. The bridge runs on the user's own machine, so any
other redirect target would be handing the link code to a third party. A
non-loopback redirect is rejected with an explicit error naming the accepted
forms. If a user sees that page, something is intercepting the flow; treat it as
a security event rather than a configuration bug.

Both the device-link page and its approve and deny actions require an
authenticated session, so an unauthenticated request can never mint a code.

## 4. Sync and Verify

Once linked, the bridge pulls the signed marketplace manifest. It contains only
the plugins, skills, agents, and MCP servers the user's role, organization, and
Salesforce link state entitle them to — filtered server-side, so a user cannot
see or reach anything step 4 did not grant them. The Salesforce MCP server and
the Salesforce plugins appear only for accounts with a linked Salesforce
identity; a passkey-only account gets the rest of the marketplace without them
(see `services/access-control/salesforce.yaml`).

The `salesforce` server is advertised to the client as a gateway URL, of the
form `https://your-host/api/v1/mcp/salesforce/mcp` — **never** the Salesforce
endpoint. This is what keeps the credential injection server-side and the audit
trail complete.

The user's first real query proves every step at once:

> "Show me my five most recent open opportunities."

This exercises the full chain: bridge credential, gateway authorization,
per-user token mint, Salesforce Hosted MCP, and the audit write.

## Verify

```bash
# The tool call, with identity, decision, and result
systemprompt infra logs trace list --limit 5
systemprompt infra logs trace show <trace-id>

# Errors during the rollout window
systemprompt infra logs view --level error --since 1h
```

A trace naming the signed-in user, the `salesforce` server, and a successful
result confirms the integration end to end.

Then run the test that actually proves per-user identity: have **two** users
with deliberately different Salesforce visibility run the same query. Their
results must differ. If two users with different record access get identical
results, per-user identity is not working — revisit step 2, since the likely
cause is a fallback that is not acting as the individual.

## Rolling Out to a Team

The per-user cost is zero admin steps, provided steps 1–4 are done. A user is
ready when three things are true:

1. Their email domain is in `allowed_email_domains` and claimed by an
   organization.
2. They are pre-authorized on the Salesforce app.
3. A seat is free.

A practical sequence: pilot with two or three users across different Salesforce
profiles to confirm permissions differ as expected; check seat headroom against
your rollout size; confirm the Salesforce pre-authorized profiles cover everyone
in scope; then distribute the bridge with the gateway URL baked in.

The pre-authorization check is the one worth doing before rollout rather than
after. A user missing from it logs in perfectly and then fails on every tool
call, which reads to them as a broken product rather than a missing permission.

## Troubleshooting

| Symptom | Cause |
|---------|-------|
| Bridge cannot reach the gateway | Wrong gateway URL, or the host is unreachable. Check `ASTOUND_BRIDGE_GATEWAY_URL` |
| "Invalid redirect" on device link | A non-loopback redirect was attempted. Only `127.0.0.1` and `localhost` are accepted |
| Login works, no tools appear | The user landed unattached or their role has no grant — see step 4 |
| Tools appear but every call fails | The user is not pre-authorized on the Salesforce app, or never captured a Salesforce Username — see step 2 |
| `?sso=seat_limit` during rollout | The organization is out of seats — see step 4 |
| Some users get all tools, others none | Role or organization differences. Compare their memberships and roles |
| Marketplace shows nothing after linking | The sync has not completed, or no marketplace is granted to the user's role |

## You Are Done

With all five steps complete, any provisioned user in your Salesforce
organization can install the bridge, sign in with their existing Salesforce
credentials, and work against their own Salesforce data — with every call
executed under their own Salesforce permissions and recorded in the governance
audit trail.

Back to the **[Overview](/documentation/salesforce)**.
