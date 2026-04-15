<div align="center">
  <a href="https://systemprompt.io">
    <img src="https://systemprompt.io/logo.svg" alt="systemprompt.io" width="150" />
  </a>
  <p><strong>Production infrastructure for AI agents</strong></p>
  <p><a href="https://systemprompt.io">systemprompt.io</a> • <a href="https://systemprompt.io/documentation">Documentation</a> • <a href="https://github.com/systempromptio/systemprompt-core">Core</a> • <a href="https://github.com/systempromptio/systemprompt-template">Template</a></p>
</div>

---

# Governance Demos

Tool access control, scope enforcement, secret detection, and audit trails.

## Prerequisites

Run `../00-preflight.sh` first to start services and acquire a token.

## Scripts

| # | Script | What it proves | Cost |
|---|--------|---------------|------|
| 01 | happy-path.sh | Governance ALLOWS admin-scope tool call, MCP tool executes | Free |
| 02 | refused-path.sh | Governance DENIES user-scope agent calling admin tool | Free |
| 03 | audit-trail.sh | Both decisions queryable in governance_decisions table | Free |
| 04 | governance-happy.sh | Detailed rule evaluation — all 3 rules pass for admin agent | Free |
| 05 | governance-denied.sh | Scope check + blocklist deny for user agent | Free |
| 06 | secret-breach.sh | Secret detection blocks leaked credentials in tool inputs | Free |
| 07 | rate-limiting.sh | Rate limit, security, and server configuration | Free |
| 08 | hooks.sh | Hook listing and validation across all plugins | Free |

## How it works

Demos 01-06 call the governance API directly with `curl`, simulating Claude Code's PreToolUse hook workflow. No AI calls, deterministic, instant.

The governance pipeline:
1. JWT validation (token authentication)
2. Scope resolution (admin vs user agent)
3. Rule engine (scope_check, secret_detection, rate_limit)
4. Audit write (async database INSERT)
5. Response (ALLOW or DENY)
