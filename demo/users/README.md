<div align="center">
  <a href="https://systemprompt.io">
    <img src="https://systemprompt.io/logo.svg" alt="systemprompt.io" width="150" />
  </a>
  <p><strong>Production infrastructure for AI agents</strong></p>
  <p><a href="https://systemprompt.io">systemprompt.io</a> • <a href="https://systemprompt.io/documentation">Documentation</a> • <a href="https://github.com/systempromptio/systemprompt-core">Core</a> • <a href="https://github.com/systempromptio/systemprompt-template">Template</a></p>
</div>

---

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
