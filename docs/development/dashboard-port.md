# Dashboard port coverage

Source: `systemprompt-astound` at `083df9e1` (changes reviewed August 31–September 8, 2026).
Destinations: template and internal. This document records the feature boundary;
build and runtime results are recorded below once validation completes.

| Feature | Source anchors | Integrated surface |
| --- | --- | --- |
| Dense shared design, charts, navigation | `9fd0af41`, `8e8a04ec`, `259c3374` | Shared partials, CSS, JS components, Overview landing, compatibility routes |
| Overview and analytics | `521f1ec6`, `7aa6d0c0`, `1eb4519a` | Six analytics tabs, request/p50/model charts, hook metrics, rollup/anomaly jobs |
| People and access | `625424ac`, `89099784`, `9250433b`, `2a724de2` | Groups, projects, per-person roles/devices, user access tab, custom role strings |
| Scope attribution | `970c4eef`, `53cd1a16` | Primary scope defaults, exclusive/member accounting, filters and recomputation |
| Requests and governance | `ecfd9ef6`, `6f832df9`, `e074847b`, `f8853281` | Rejected/unattributed requests, warning and secrets exports, approvals, time-window preservation |
| Account and connected clients | `74b745ed`, `38ea6926`, `4283cc0d`, `4306d94a` | Settings persistence/account closure, connection tabs, device rows, connector lifecycle |
| Conversations | `900c1168`, `7f23329d`, `4306d94a` | Session-derived identity, readable transcripts, folded tool results, side-call summaries |
| Query scaling | `e23331e1` | Bounded listings, batched membership/usage, page-before-title enrichment, retained totals |
| Generic platform catalog | `5ef33dc1`, `20987651`, `58fc9396` | MCP, marketplace, plugin, skill and gateway pages; authenticated connector APIs |

## Compatibility decisions

- Departments, organizations, plans, memberships and their ACLs remain intact.
  Groups/projects are independent additions; no department conversion is inferred.
- New migration numbers start at 051, above either destination's historical chain.
  Astound migration 046 and tenant seed/backfill migrations are not imported.
- Core facade dependencies use 0.48.0 in both Rust workspaces. Local path patches
  are development-only and must not be committed.
- The template retains passkeys, registration, magic links, evaluations and demos.
  Internal additionally retains Odoo/operator login, enterprise reporting and bridge consent.
- Known console roles control route permissions; entitlement roles remain free text.
  Existing `admin` operators can manage directory mappings without requiring a new bootstrap role.
- Astound's tenant identities, private marketplace content, SSO-only restrictions,
  knowledge ingestion and requirements workflows are excluded. Governance policy is unchanged.

## Acceptance checks

Asset references, imports and JavaScript syntax; coordinated builds; fresh/additive
schema installation; populated upgrade data preservation; owner/admin history;
read-only mutation denial; custom roles; settings persistence; connection tabs;
legacy routes and destination-specific pages. Full release gates remain part of
the existing promotion flow. No dashboard latency target is claimed without measurements.
