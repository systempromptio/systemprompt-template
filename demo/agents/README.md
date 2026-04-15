<div align="center">
  <a href="https://systemprompt.io">
    <img src="https://systemprompt.io/logo.svg" alt="systemprompt.io" width="150" />
  </a>
  <p><strong>Production infrastructure for AI agents</strong></p>
  <p><a href="https://systemprompt.io">systemprompt.io</a> • <a href="https://systemprompt.io/documentation">Documentation</a> • <a href="https://github.com/systempromptio/systemprompt-core">Core</a> • <a href="https://github.com/systempromptio/systemprompt-template">Template</a></p>
</div>

---

# Agent Demos

Agent discovery, configuration, messaging, tracing, and A2A registry.

## Prerequisites

Run `../00-preflight.sh` first.

## Scripts

| # | Script | What it proves | Cost |
|---|--------|---------------|------|
| 01 | list-agents.sh | Agent discovery — admin and core views | Free |
| 02 | agent-config.sh | Validation, MCP tool access, process status | Free |
| 03 | agent-messaging.sh | Full agent pipeline — AI reasoning, MCP tools, artifacts | ~$0.01 |
| 04 | agent-tracing.sh | Execution traces, artifacts, cost attribution | Free |
| 05 | agent-registry.sh | A2A gateway discovery, agent logs | Free |

## Notes

- Scripts 01, 02, 04, 05 are read-only and free
- Script 03 sends a real message to the developer_agent (one AI call ~$0.01)
- Script 04 shows traces from prior agent runs — run 03 first for data
