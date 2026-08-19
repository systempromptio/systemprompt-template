---
title: "Use Case — Standing Up the Gateway for Your Organization"
description: "The admin journey end to end: create the Salesforce app, point the repository at your org, apply the spec, sign in, and hand the bridge to your team. What you own, in what order, and where each hazard lies."
author: "Astound Digital"
slug: "use-case-admin"
keywords: "admin, operator, setup, salesforce connector, external client app, apply, seats, roles, admin console, provisioning"
kind: "guide"
public: true
tags: ["admin", "setup", "salesforce", "getting-started"]
published_at: "2026-07-31"
updated_at: "2026-07-31"
after_reading_this:
  - "Know the five things you own as the operator, and the order they happen in"
  - "Know which single step cannot be automated, and why"
  - "Run diff, dry-run and apply against your org with the hazards understood"
  - "Read the admin console and know which roles see which parts of it"
related_playbooks:
  - title: "Use Case — Your Salesforce Day"
    url: "/documentation/use-case-salesforce-user"
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Salesforce Integration Overview"
    url: "/documentation/salesforce"
  - title: "Step 1 — Salesforce App Setup"
    url: "/documentation/salesforce-app-setup"
  - title: "Step 5 — Rolling Out the Bridge"
    url: "/documentation/salesforce-bridge-rollout"
---

# Use Case — Standing Up the Gateway for Your Organization

**Who this is for:** the person who owns the Salesforce org's setup and the
platform deployment. Usually one or two people, once per organization. This is
the advanced operator path — if you just want to use the gateway, start at
[the documentation home](/documentation/) instead.

**What you need before you start:** admin access to a Salesforce org, shell
access to the deployment, and an hour. Your team needs none of this — see
[Your Salesforce Day](/documentation/use-case-salesforce-user) for what they do.

This page is the journey. Each step links to the reference page that covers it
in full; nothing here is restated there in more detail.

## What You Are Actually Building

One sentence: you are giving every person in your Salesforce org the ability to
ask their own Salesforce questions in plain English, from their own laptop,
under their own Salesforce permissions, with every call recorded.

The chain that makes that true has five links, and you build them in order:

| # | Link | Where it lives |
|---|---|---|
| 1 | A Salesforce app that trusts your platform | Salesforce Setup — manual, once |
| 2 | A key pair that lets the platform mint per-user tokens | `openssl`, then your secrets store |
| 3 | The org configured to match a spec you keep in the repo | `services/salesforce/org.yaml` |
| 4 | Users, organizations and seats | `services/access-control/plans.yaml` |
| 5 | The bridge on your team's machines | `astound-bridge install` |

Nothing is stored on the user's side. The bridge holds a platform credential and
never a Salesforce one, so a lost laptop costs you a revocation, not a breach.

## Step 1 — The One Manual Step

Create an **External Client App** in Salesforce Setup, upload a signing
certificate to it, and copy the consumer key and secret.

This is the only step that cannot be automated, and it will stay that way: the
consumer secret is not readable through any Salesforce API. Everything after it
is a command.

Two details decide whether the rest of the setup works at all:

- The scopes must include **`mcp_api`**. Without it the login succeeds and every
  subsequent tool call returns 401 — which reads to a user as a broken product
  rather than a missing checkbox.
- **Use digital signatures** must be on, with your certificate uploaded. Without
  it the platform cannot mint a token, so the automation in step 3 cannot run.

Set Permitted Users to *All users may self-authorize* for now. Step 3 tightens it.

Full detail: **[Step 1 — Salesforce App Setup](/documentation/salesforce-app-setup)**
and **[Step 2 — JWT-Bearer and the Signing Certificate](/documentation/salesforce-jwt-bearer)**.

## Step 2 — Point the Repository at Your Org

Four files, and one distinction that catches people.

`services/web/config/salesforce.yaml` carries the SSO client: your My Domain
URL, the consumer key, the callback URL, and `allowed_email_domains`.

`services/salesforce/org.yaml` is the desired state of your Salesforce org. Its
`callback_url` must be **character-identical** to the `redirect_uri` above —
Salesforce compares them exactly, and a trailing slash fails the login with no
useful error.

`services/access-control/plans.yaml` claims your email domain for an
organization.

Secrets — the consumer secret, the private key, and the certificate — go in the
profile's `secrets.json` or the matching environment variables. Never in
`services/`.

**The distinction:** there are two independent domain lists, and they do
different jobs.

| List | Decides |
|---|---|
| `allowed_email_domains` in `salesforce.yaml` | Who may sign in at all |
| `email_domains` in `plans.yaml` | Which organization they join |

Setting the first and forgetting the second does not fail loudly. The user signs
in successfully, lands unattached to any organization, gets no plan grants and
no seat check, and sees nothing at all. If a user reports "I logged in and it's
empty", this is almost always why.

## Step 3 — Apply the Spec

```bash
systemprompt plugins run salesforce diff              # what differs
systemprompt plugins run salesforce apply --dry-run   # full validation, writes nothing
systemprompt plugins run salesforce apply             # apply it
```

`apply` sets the OAuth scopes, policies, callback URL and PKCE requirement;
creates the `Salesforce_MCP_Access` permission set and the grant that
pre-authorizes the app; assigns that permission set to every user who has a
recorded Salesforce Username; and activates the hosted MCP servers.

Always run `--dry-run` first. It submits a real metadata package with
`checkOnly`, so Salesforce runs its full validation and writes nothing.

Three things worth knowing before you run it:

**The certificate is mandatory.** `apply` refuses to run without one, on
purpose. A metadata deploy is declarative, so a package that omits the
certificate *clears* the app's digital signature — and since that signature is
how the tool authenticates, it cannot repair the damage it just caused.
Recovery costs a manual upload in Setup.

**The subject is the Username, not the email.** Salesforce matches the assertion
on the Username, and the two routinely differ
(`you@company.com.dev`, `ed.aa5967144c6c@agentforce.com`). Find it under
**Setup → Users**.

**A brand-new org has nobody to assign.** Assignees come from the platform
database — the record of who has completed an SSO login — not from `org.yaml`.
On a fresh org, name yourself: `apply --user "you@yourcompany.com.dev"`.

Full detail: **[Step 3 — Hosted MCP Access](/documentation/salesforce-hosted-mcp)**
and **[Step 4 — Users, Seats and Roles](/documentation/salesforce-provisioning)**.

## Step 4 — Restart and Sign In

```bash
just build && just start
```

A restart is required, not just `just publish`: `salesforce.yaml` is cached at
first read, and `plans.yaml` is projected into access-control rules at startup.

Then go to `/admin/login` and click **Sign in with Salesforce**. Your first
login creates your account automatically, provided the domain is allow-listed, a
seat is free, and you are pre-authorized on the app.

If it does not work, the redirect tells you why: `?sso=not_provisioned` means
auto-provisioning is off and no account exists; `?sso=seat_limit` means the
organization is full.

Note that signing in with Salesforce gives you the `user` role, not `admin`.
That is the next step.

## Step 5 — Roles and the Console

Roles are stored on the user record and re-read from the database on **every
request**, so a change takes effect immediately — no sign-out, no waiting for a
token to expire. That is deliberate: revocation has to be instant to be worth
anything.

Promote yourself from the CLI:

```bash
systemprompt admin users role promote you@yourcompany.com admin
```

What each role sees:

| Role | Sees |
|---|---|
| `user` | Profile, settings, and device setup only. Everything else redirects to their profile |
| `admin` | The full console: access, catalog, entities, and the customer report |
| Platform admin | The above plus enterprise administration and internal reports |

The console divides into four areas. **Access** is people — users, departments,
personal access tokens, device certificates. **Catalog** is what they can use —
plugins, skills, MCP servers. **Entities** is what happened — AI requests,
sessions, tool traces, contexts. **Reports** is the rollup.

When you need to answer "what did this cost, who ran it, and was it allowed",
start at **Entities → Traces** and open the trace.

## Step 6 — Hand It to the Team

Your team's onboarding is two minutes and involves no admin step per person,
provided steps 1–5 are done: install the bridge, sign in with Salesforce,
approve the device link.

Bake the gateway URL in at install time so nobody has to configure anything:

```bash
astound-bridge install --gateway https://your-host
```

Before you distribute, check three things. Every user's email domain is
allow-listed and claimed by an organization. You have seat headroom for the
rollout size. And the Salesforce pre-authorized profiles cover everyone in
scope — that last one is worth verifying *before* rollout, because a missing
user logs in perfectly and then fails on every single tool call.

Pilot with two or three people on deliberately different Salesforce profiles and
have them run the same query. **Their results must differ.** Identical results
mean per-user identity is not working, and you should revisit step 2.

Full detail: **[Step 5 — Rolling Out the Bridge](/documentation/salesforce-bridge-rollout)**.

## A Second Operator, Before You Finish

Platform operators — as distinct from Salesforce users — are created from the
CLI and sign in with a passkey rather than through Salesforce. There is no
self-service recovery for a lost passkey and no email service configured to
deliver one, so **create a second operator account now**, while you still have
one that works.

```bash
systemprompt admin users create --name "Sam" --email sam@yourcompany.com
systemprompt admin users role promote sam@yourcompany.com admin
systemprompt admin users webauthn generate-setup-token --email sam@yourcompany.com
```

The third command prints a one-shot setup link, valid for 15 minutes. Send it
through a channel you already trust.

Full detail: **[Authentication](/documentation/authentication)**.

## Verify the Whole Chain

```bash
# The org matches the spec on every readable field
systemprompt plugins run salesforce diff --exit-code   # 0 = clean, 1 = drift

# A real user's tool call, with identity, decision, cost and result
systemprompt infra logs trace list --limit 5
systemprompt infra logs trace show <trace-id>

# Anything that went wrong during the rollout window
systemprompt infra logs view --level error --since 1h
```

A trace naming a real signed-in user, the `salesforce` server, and a successful
result is the end-to-end proof.

## When Something Breaks

The failure tables are kept with the steps that cause them:
[app and login failures](/documentation/salesforce-app-setup),
[token minting](/documentation/salesforce-jwt-bearer),
[seats and provisioning](/documentation/salesforce-provisioning),
[bridge and rollout](/documentation/salesforce-bridge-rollout).

The four you will meet most often:

| Symptom | Cause |
|---|---|
| Login works, user sees nothing | Their domain is not claimed by any organization in `plans.yaml` |
| Login works, every tool call 401s | The `mcp_api` scope is missing, or the user is not pre-authorized on the app |
| `invalid_grant: invalid assertion` after an apply | The deploy cleared the certificate. Re-upload it and re-tick "Enable JWT Bearer Flow" |
| Callback fails with no useful error | `callback_url` and `redirect_uri` differ — Salesforce compares them character for character |
