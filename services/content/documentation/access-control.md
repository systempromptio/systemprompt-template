---
title: "Access Control: Who Reaches What, and Why"
description: "How entitlement works on this instance: the four subject bands and their precedence, the one declarative file every rule lives in, the database that enforces it, and the sync loop that keeps code and console in step."
author: "systemprompt.io"
slug: "access-control"
keywords: "access control, entitlement, rules.yaml, roles, groups, projects, marketplace, plugin, skill, mcp server, gateway route, precedence, deny overrides, sync, drift, export"
kind: "guide"
public: true
tags: ["enterprise", "admin", "access"]
published_at: "2026-09-28"
updated_at: "2026-09-28"
after_reading_this:
  - "Read the Access control page and say, for any entity, who reaches it and which band decided"
  - "Predict the resolver's answer for a person from their role, group, project and connected servers"
  - "Grant a group a tool, make an entity admin-only, or retire a rule — in the file or in the console"
  - "Tell what is enforced (the database) from what is declared (rules.yaml), and reconcile the two in either direction"
related_docs:
  - title: "Authentication"
    url: "/documentation/authentication"
  - title: "Gateway API"
    url: "/documentation/gateway-api"
  - title: "Code ↔ Instance: sources, planes and sync"
    url: "/documentation/services-sync"
---

# Access Control: Who Reaches What, and Why

**TL;DR:** Every workspace, plugin, skill, MCP server and model route a signed-in person can reach is decided by rules on that entity. The rules are **declared once**, in `services/access-control/rules.yaml`, each with a stated reason, and **enforced from the database**. A decision walks the bands from narrowest to widest — *person → project → group → connected server → role* — and the first band that names the person decides, with a deny beating an allow inside it. Code and database are compared on every boot and **never silently merged**: `/admin/sync` shows the differences and offers three ways to resolve them.

## What access control decides — and what it does not

Access control answers one question: *may this person reach this entity?* The entities are:

| Kind | Examples on this instance | What "reach" means |
|---|---|---|
| `marketplace` | `enterprise-demo` | The workspace appears in Claude Code / Cowork and its plugins can be installed |
| `plugin` | `systemprompt` | The plugin is offered |
| `skill` | `use_dangerous_secret`, `systemprompt_cli` | The skill is offered and may run |
| `agent` | `developer_agent`, `associate_agent` | The agent is listed and may be driven |
| `mcp_server` | `systemprompt` | The server is listed and its tools may be called |
| `gateway_route` | `claude-star-4203d1`, … | Requests may be routed to that model |

It does **not** decide whether a person can sign in (that is [authentication](/documentation/authentication)), whether a tool call is allowed to proceed once the person is entitled (the governance chain, `services/governance/config.yaml`), or what may be said (the gateway safety scanners, `services/gateway/policies.yaml`). Those run after entitlement, on every request.

## The model

### Subjects and bands

A rule names a **subject** at one **band**:

| Band | Precedence | Subject is… | Where a person gets it |
|---|---|---|---|
| person | 0 | one account | an override set on the user's Access tab |
| project | 140 | a project id | assigned on the Projects page or mapped from a directory group |
| group | 150 | a group id | assigned on the Groups page or mapped from the directory groups asserted at sign-in |
| connected server | 160 | a ready OAuth-backed MCP server id | recorded after its connection is verified |
| role | 200 | `user`, `admin`, … | granted at sign-in or on the Roles page |

Lower precedence is **narrower**. The ladder is walked from the top.

### How a decision is made

1. **The narrowest band in which the person matches any rule decides.** A band in which nothing names them is skipped entirely — a group rule for a group they are not in has no effect on them, good or bad.
2. **Inside that band a deny beats an allow.**
3. **An entity with any rule is closed to everyone its rules do not name**, unless it is marked `default: open`. The rules are the whole story for that entity.
4. **An entity with no rule at all inherits from its parent**: a skill from its plugin, a plugin from its marketplace. This is why most skills need no rule of their own — they follow the workspace that ships them.

Two consequences worth holding onto:

- A **role grant does not "reopen" a group-gated entity to everyone** — only to the roles it names. An entity allowed to `admin` *and* to an `engineering` group is reached by an admin in no group through the role band; a plain `user` in no group is refused by the closed default.
- A **wider band cannot rescue a narrower deny**. A person-band deny on one account beats every group and role allow that person holds.

### Worked examples

`skill/use_dangerous_secret` ships in the `enterprise-demo` marketplace, which is open to role `user`. The skill declares its own rule: closed, deny role `user`.

| Person | Walk | Result |
|---|---|---|
| role `user` | person · project · group · connected: nothing · **role: matches deny** | **refused** (decided by role) — the skill's own rule beats the inherited marketplace allow |
| role `admin` only | no band names them | **refused** — the entity declares a rule, so it is closed |

Suppose an MCP server were declared closed, allowing role `admin`, group `engineering` and project `platform-migration` (groups and projects are declared in `services/web/config/groups.yaml`):

| Person | Walk | Result |
|---|---|---|
| role `user`, group `engineering` | person: nothing · project: nothing · **group: matches allow** | **allowed** (decided by group) |
| role `admin`, no group | person · project · group · connected: nothing · **role: matches allow** | **allowed** (decided by role) |
| role `user`, no group, no project | no band names them | **refused** — the entity is closed |

The **Audience grid** tab of `/admin/access-control` runs exactly these walks, for every role, group and project, against every entity; the word in each cell is the band that decided. To check one real person, use the **Check a person** tab (or open their user page and choose **Access** — the same view): the same resolver, with their actual roles, groups, projects and connections. The **Rules** tab is the ledger by entity, grouped by kind, and model routes there carry the name and "pattern → provider" line their gateway declaration gives them rather than the generated id alone.

## The file: `services/access-control/rules.yaml`

Every rule on the instance is declared in this one file, entity by entity. Marketplaces are entities like any other — there is no second place where a workspace's audience is written.

```yaml
entities:
  - entity: marketplace/enterprise-demo   # <kind>/<id>
    default: open                          # open | closed — what an unmatched person gets
    why: Public enterprise governance evaluation demo.   # REQUIRED; the justification on every rule row
    allow:                                 # band → subjects
      role: [user]

  - entity: skill/use_dangerous_secret
    default: closed
    why: >-
      Access-control demonstration: a dangerous capability that exists in the
      catalog but is denied by policy.
    deny:                                  # same shape as allow
      role: [user]

  - entity: gateway_route/*                # glob: route ids are generated, never written
    default: open
    why: Every model route is reachable by every signed-in role.
    allow:
      role: [user, admin]
```

Rules of the file, all checked by `scripts/validate-services.sh` in CI and again when the server reads it:

- `why` is required and non-empty. A rule with no reason is a rule nobody can review.
- Every `entity` must exist: plugins, skills, agents and MCP servers in `services/`, marketplaces under `services/marketplaces/`.
- `gateway_route` and `hook` take only the glob `*`, expanded against the live catalog at boot. A hand-written route id names a route that cannot exist.
- Band keys are `role`, `group`, `project`, `connector`. Every `group:`/`project:` value must be declared in `services/web/config/groups.yaml`.
- A subject may not be both allowed and denied on the same band of one entity.
- `valid_until` is optional, an RFC 3339 instant (`2026-12-31T00:00:00Z`), and applies to every rule of the entity. A declaration already past it is treated as not declared; the hourly `access_expiry` sweep deletes the rows once the instant passes. The console's date picker writes the same window on a rule it saves, and the export writes it back.
- **No `services/marketplaces/*/config.yaml` may carry an `access:` block.** The gate refuses it, so the second truth cannot return.

Per-person overrides (the `user` band) have no place in the file. They belong to one account, are set on that person's Access tab, and are never synced or exported.

## Database and console

The database — `access_control_rules` and `access_control_entities` — is what the resolver reads. It is also what the console edits:

- **A group's Access tab** (`/admin/groups/<id>` → Access) sets that group's band on any entity. Allow or deny asks for a **reason** and refuses to save without one.
- **A person's Access tab** (`/admin/users/<id>` → Access) sets a person-band override. The reason is optional there.
- Every console write is stamped `source = dashboard`, so the Sync page can tell a console decision from one the file wrote.

The Access control page shows this database, one row per entity: the bands as chips, the reason in full, and a **Code ↔ DB** column saying whether the file agrees. The column has three words. **In sync** — the file and the database agree on this entity. **Drift** — they differ; the link opens the Sync page filtered to it. **Unknown** — the declaration could not be compared at all, and the notice at the top of the page says why: either `rules.yaml` did not parse, or the services tree it validates against did not compose (a plugin including an MCP server no file declares, for instance). *Unknown* never means "no rules" or "not yet synced" — a readable file against an empty database is *Drift*. Expand a row for the individual rules and the resolver's outcome in a sentence.

## The sync loop

Code and database are two copies of the same intent. Either may be edited; neither is silently overwritten.

**On boot** the server reads `rules.yaml` and, if the database holds **no** band rules at all (a fresh install), seeds it from the file in one transaction. Otherwise it computes the differences, logs them (`sync_drift` in the server log), and **writes nothing**. A deploy that adds a rule to the file therefore does not land that rule until an administrator chooses to. The same contract governs the groups and gateway-policy planes: no plane rewrites its tables on a restart. People are outside the loop entirely — membership, manual roles and person-band overrides are never declared in code, never reported as drift and never exported.

**The Sync tab** (`/admin/access-control?tab=sync`) says what happened, in the words a reader acts on. The tiles count differences by **cause** — *added in code*, *changed in code*, *removed from code*, *written in console*, *awaiting a bundle* — and a **Next:** line names the one action that settles the plane. Every difference is a line: the cause, the entity and band, what code and the database each hold, who wrote the database row (`code`, `console` or `bundle`), and a **Resolves with** badge naming the button. Three directions:

| Action | Writes | Leaves alone |
|---|---|---|
| **Insert only** | Rules and entity defaults the file declares that the database lacks | Everything that already exists, even if it differs |
| **Overwrite from code** | Adds what code added, corrects what code changed, and **deletes** what code removed: every undeclared row on an entity the file names, and every row the file itself wrote on an entity it has since dropped (with that entity's default, once nothing is left on it) | Rows the console or a bundle wrote on entities the file does not mention; every person-band override |
| **Export to code** | Nothing in the database. Renders it as a `rules.yaml` to copy or download | — |

Both writes run in one transaction that re-reads the tables first, are recorded in the activity log with your name and the counts, and are confirmed in words that name what will be deleted.

The asymmetry is the point. Code is the source for everything code wrote, so deleting an entity from the file and pressing Overwrite removes its rows even though the file no longer says a word about it. The console is the source for what it wrote, so a console row on an entity the file has never named survives every Overwrite and reaches the file only through **Export** — the instance never writes into the repository itself.

**To make the console the source and the code catch up:** edit in the console, press **Export**, replace `services/access-control/rules.yaml` with the result, commit, deploy. The next boot reports *in sync*. Lines beginning `# NOTE:` in the export name what could not be expressed — a band whose rows carry different reasons, or gateway routes whose rule sets differ from one another — and must be resolved by hand.

**To make the code the source and the database catch up:** edit the file, commit, deploy, open the Sync page, choose **Insert only** (additive) or **Overwrite from code** (authoritative).

The Sync page is instance-wide — access control is one **plane** of it, beside the **sources** every declaration comes from. Each plane records the declared hash it last applied, when, by whom and in which mode, so the card can say *declared `ab12…` · last applied `ab12…` by an administrator (overwrite)* or *declaration changed since last apply*. How sources, planes and hashes fit together: [Code ↔ Instance](/documentation/services-sync).

## External kits

A marketplace can arrive as a signed **services bundle** published by another GitHub repository — a *kit* in Anthropic marketplace format — rather than from this repository's `services/`. **Kits carry no access.** A kit's sidecar has no `access:` block, and one it carries anyway is ignored as a declaration and flagged on the Sync page. Who reaches a kit's marketplace is declared **here**, like every internal one, with one extra key naming the source it arrives from:

```yaml
  - entity: marketplace/team-dev
    owner: bundle:team-kit
    default: closed
    why: Development workspace published by the team kit repository.
    allow:
      group: [engineering]
```

`owner: bundle:<name>` lets the entity be declared before the bundle is pinned: until that source is active the Sync page lists it as **awaiting its bundle** rather than failing boot, and the CI gate accepts the id without a local `services/marketplaces/<id>`. Once the bundle is active the entity validates and is governed like any other. A kit release therefore can never widen who sees it — ownership of *content* is the kit's; ownership of *access* is this repository's.

## How-tos

**Grant a group a tool.** Either add the group to the entity's `allow.group` list in `rules.yaml` and sync, or open the group → Access, set the entity to *Allow*, write the reason, Save. Then **Export** if you want the file to carry it.

**Make an entity admin-only.** Declare it with `default: closed` and `allow: { role: [admin] }`. If your installation defines a role strictly stronger than `admin`, name it beside `admin` always — an entity open to `admin` but closed to the stronger role is a hole. Declaring any rule opts the entity out of the marketplace cascade, which is what makes the list restrictive rather than additive.

**Explain one person's access.** Access control → Check a person, or User page → Access. Each row says *allow*/*deny* and the band that decided. Compare with the Audience grid tab to see what would change if they joined a group or gained a role.

**Retire a rule — or a whole entity — cleanly.** Remove it from `rules.yaml`, deploy, Sync → Overwrite. The row is deleted whether or not the entity is still in the file, because the file wrote it. Insert only would leave it in place, listed as *removed from code* until an Overwrite.

**Add a subject band.** A new dimension (cost centre, clearance, …) is a provider registered in `extensions/web/admin/src/authz/` and a new key in `BandMap`; no core change.

## Glossary

- **Entity** — the thing reached: marketplace, plugin, skill, agent, MCP server, gateway route.
- **Subject** — who a rule names: a person, project, group, connected server, or role.
- **Band** — the kind of subject, with a precedence number; lower is narrower.
- **Default open / closed** — what an entity gives to a person no rule names. Closed unless declared otherwise.
- **Cascade** — a ruleless entity's inheritance from plugin then marketplace.
- **Declared** — what `rules.yaml` says. **Enforced** — what the database holds.
- **Drift** — any difference between the two: missing in DB, only in DB, differs, default differs.
- **Source** — `yaml` for rows the file placed, `dashboard` for rows the console wrote.

## Not declared here

Inbound Slack apps are not declared here: `authz.allowed_roles` in `services/slack/*.yaml` projects the `slack_workspace:<workspace_id>` entity at startup and stays with the app it gates.

## Caveat: remote services bundles

This instance ships its services tree in the repository. If a profile ever pins a remote services bundle (`services.sources`), core's own boot reconcile also projects that bundle's marketplaces into the same tables from their manifests. Keep `rules.yaml` authoritative: the Sync page will show any such rows as drift, and **Overwrite from code** restores the declared state.
