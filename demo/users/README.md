# User & Auth Demos

User management, roles, sessions, and IP ban management.

## Prerequisites

Run `../00-preflight.sh` first.

## Scripts

| # | Script | What it proves | Cost |
|---|--------|---------------|------|
| 01 | user-crud.sh | User listing, counts, statistics, search | Free |
| 02 | role-management.sh | User details and role inspection | Free |
| 03 | session-management.sh | Current session, available profiles | Free |
| 04 | ip-ban.sh | Add/remove IP bans with verification | Free |

## Notes

- Scripts 01-03 are read-only
- Script 04 adds a test ban (192.168.99.99) and removes it — safe cleanup
