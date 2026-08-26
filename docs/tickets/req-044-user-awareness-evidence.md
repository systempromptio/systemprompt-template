# REQ-044 — User Awareness: evidence that this is largely Supported, not a Gap

Status: evidence response for the register (row 45, marked "Gap — stakeholder
identified"). Recommendation: re-status to "Supported with caveat (project
dimension pending DEC-001)" after a demo of the matrix below.

## The requirement

> Enforce dynamic access control based on the user's active session context —
> project, role, and permissions — determining which MCP servers, models, and
> knowledge sources the user can access.

## What already exists

Every session is authorized through a single rule table
(`access_control_rules`, core `crates/infra/security/schema/`) evaluated by a
deny-overrides, narrowest-band-wins resolver
(`crates/infra/security/src/authz/resolver.rs`). It gates exactly the entity
classes the requirement names:

| Requirement asks | Enforced entity type | Where checked |
|---|---|---|
| MCP servers | `mcp_server` | runtime authz webhook + signed bridge manifest |
| Models | `gateway_route` | gateway dispatch (`gateway_entities.rs`) |
| Knowledge sources | `plugin` / `skill` / `mcp_server` | same resolver |

Subject dimensions on this instance, by precedence band (lower = wins):

| Band | Dimension | Source |
|---|---|---|
| 0 | user | per-user override rows |
| 100 | department | `services/access-control/departments.yaml` |
| 150 | salesforce (`linked`, derived) | `salesforce_user_identities` row, 60s cache |
| 200 | role | `users.roles` + `roles.yaml` |
| 300 | organization | `organization_members` + `plans.yaml` |

Properties matching the acceptance criteria:

- **Different contexts → different authorized sets.** Two users with different
  role/department/org/linked state resolve different entity sets. The
  admin "effective access" matrix
  (`extensions/web/admin/src/repositories/users/access_control/matrix.rs`,
  `repositories/governance/effective.rs`) renders this per user.
- **Unauthorized resources are not discoverable.** Denied entities are removed
  from the signed bridge manifest (`marketplace_filter.rs`) — the client never
  sees them — and are refused again at call time by the runtime resolver, so
  discovery and callability are both gated.
- **Decisions are auditable.** Every decision lands in the governance spine
  with policy + trace id (`systemprompt infra logs trace list`); denials are
  recorded like any other call.

Live example on this instance: the `salesforce` derived dimension hides the
Salesforce MCP server, six per-object plugins, and their skills from any user
without a linked Salesforce identity, in both the manifest and at runtime.

## The genuine gap: the project axis

There is no `project`/`team` subject dimension — only user, department,
salesforce, role, organization. Adding one is a clean seam (one
`register_subject_attribute_provider!` + a membership table + a loader,
mirroring `extensions/web/admin/src/authz/{department,organization}.rs`), but
it is **blocked on DEC-001**: Astound has not yet defined what a project/team
is, whether a user belongs to one or many, and whether project is an
entitlement boundary or a reporting label. Estimate once DEC-001 lands:
~1–2 weeks (consistent with the DEC-001 note).

## Verification to add now (register's own method)

Role/project/session matrix + negative access-control test: an e2e/contract
matrix that, for a set of principals differing in role, department, org, and
salesforce-linked state, asserts (a) the manifest/entity lists differ as
configured, (b) a denied MCP server / gateway route returns a deny that is
audited, (c) no denied entity appears in discovery surfaces.
