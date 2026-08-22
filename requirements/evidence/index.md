# Requirements evidence pack

Screenshot evidence for [compliance-register.md](../compliance-register.md).
Regenerate with `just e2e-screens` against a running `just start`; this file
is written by the same script, so it cannot drift from the images beside it.

Captured 2026-08-22T10-54-50 against http://localhost:8080.
Principals come from the deterministic e2e seed (`just e2e-seed --reset`).

| File | Path | Principal |
|------|------|-----------|
| req-002-login-invite-only.png | `/admin/login` | anonymous |
| req-001-users-roster.png | `/admin/access/users` | admin |
| req-001-user-detail.png | `/admin/access/user?id=e2e-member-1` | platform admin |
| req-003-overview.png | `/admin/analytics?tab=overview&preset=30d&org=e2e-corp` | platform admin |
| req-003-usage-trends.png | `/admin/analytics?tab=overview&preset=30d&bucket=week&org=e2e-corp` | platform admin |
| req-004-spend.png | `/admin/analytics?tab=spend&org=e2e-corp` | platform admin |
| req-004-model-mix.png | `/admin/entities/requests?tab=models` | platform admin |
| req-005-seats.png | `/admin/analytics?tab=seats&org=e2e-corp&inactive_days=30` | platform admin |
| req-005-inactive-seats.png | `/admin/analytics?tab=seats&org=e2e-corp&inactive_days=90` | platform admin |
| req-006-drilldown.png | `/admin/analytics?tab=usage&preset=30d&org=e2e-corp&department=Engineering` | platform admin |
| req-006-user-drilldown.png | `/admin/analytics/users/e2e-member-1` | platform admin |
| req-007-008-code-tab.png | `/admin/analytics?tab=code&preset=30d&org=e2e-corp` | platform admin |
| req-009-spend-meters.png | `/admin/analytics?tab=spend&org=e2e-corp` | platform admin |
| requests-log.png | `/admin/entities/requests` | platform admin |
