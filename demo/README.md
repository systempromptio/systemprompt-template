# Live Demo Script

Nine scripts. Run them in order.

```
./demo/00-preflight.sh
./demo/01-happy-path.sh
./demo/02-refused-path.sh
./demo/03-audit-trail.sh
./demo/04-governance-happy.sh
./demo/05-governance-denied.sh
./demo/06-governance-secret-breach.sh
./demo/07-mcp-access-tracking.sh
./demo/08-request-tracing.sh
```

---

## Overview

| Demo | What it proves | Cost |
|------|---------------|------|
| 00 | Services are running | Free |
| 01 | Admin agent can call MCP tools and return structured artifacts | ~$0.01 |
| 02 | User agent is denied tools at the mapping level (no tools available) | ~$0.01 |
| 03 | Every AI request is traced with typed IDs and cost tracking | Free |
| 04 | Governance hook evaluates rules and ALLOWS admin-scope tool calls | ~$0.01 |
| 05 | Governance hook evaluates rules and DENIES user-scope tool calls | ~$0.01 |
| 06 | Secret detection blocks leaked credentials in tool inputs | Free |
| 07 | MCP tool calls are OAuth-authenticated and audit-logged | Free |
| 08 | End-to-end request tracing, typed IDs, flow maps, 100-request benchmark | Free |

---

## STEP 0: PREFLIGHT

```
./demo/00-preflight.sh
```

You should see services running: 3 agents + 2 MCP servers.

If anything is wrong:

```
./demo/00-preflight.sh
```

If services are down, it tells you the fix commands.

---

## STEP 1: HAPPY PATH

```
./demo/01-happy-path.sh
```

SAY: "This is the platform agent. Admin scope. It has access to the systemprompt MCP server — the CLI executor. It calls the tool, gets the real agent list, returns it. Fully governed, fully traced."

EXPECT: A real list of agents. Takes ~5 seconds.

---

## STEP 2: REFUSED PATH

```
./demo/02-refused-path.sh
```

SAY: "Same question. Revenue agent. User scope. This agent has no MCP server access. The CLI tool does not exist for this agent. Governance enforces it at the mapping level."

EXPECT: "I do not have access to that tool. This operation requires elevated permissions that have not been granted to this agent."

---

## STEP 3: AUDIT TRAIL

```
./demo/03-audit-trail.sh
```

This shows the two traces. Then copy-paste the trace IDs to dig deeper:

```
systemprompt infra logs trace show PASTE_TRACE_ID --all
```

SAY for happy path trace: "11 events. 3 AI requests, 1 MCP tool call, 7 execution steps. Skills loaded, tool executed in 300ms, cost tracked."

SAY for refused path trace: "4 events. 1 AI request, zero MCP calls, 3 steps. No tools available. That is governance enforcement."

Optional — cost breakdown:

```
systemprompt analytics costs breakdown --by agent
```

---

## STEP 4: GOVERNANCE HAPPY PATH

```
./demo/04-governance-happy.sh
```

SAY: "When the admin agent calls an MCP tool, the PreToolUse hook fires. It POSTs to the governance endpoint. Backend validates the JWT, resolves the agent scope to admin, evaluates three rules — scope check, secret detection, rate limit. All pass. Tool executes."

EXPECT: Agent successfully lists agents, governance log shows ALLOW.

---

## STEP 5: GOVERNANCE DENIED PATH

```
./demo/05-governance-denied.sh
```

SAY: "Same flow. User-scope agent tries an admin-only MCP tool. The scope check fails. Tool is blocked before it executes. The agent gets the denial and tells the user."

EXPECT: Part 1 shows raw deny JSON. Part 2 shows agent refusing.

---

## STEP 6: SECRET INJECTION BREACH

```
./demo/06-governance-secret-breach.sh
```

SAY: "Even admin-scope agents are blocked if tool input contains plaintext secrets. AWS keys, GitHub PATs, private keys — all caught by pattern matching before the tool executes. This is the safety net against prompt injection."

EXPECT: Tests 1-3 DENIED, Test 4 ALLOWED.

---

## STEP 7: MCP ACCESS TRACKING

```
./demo/07-mcp-access-tracking.sh
```

SAY: "Every MCP tool call is tracked. We make two real calls — one to the skill-manager, one to the systemprompt CLI server. Both authenticate via OAuth, both execute a tool. Every event — authentication, tool execution — is recorded in user_activity."

EXPECT: Two successful MCP tool calls, then a database query showing 4 events (2 authenticated + 2 used). Script prints the dashboard URL where you can see them.

SAY: "On the dashboard you can see the MCP Server Access section — per-server stats, Granted/Rejected badges, and the live activity feed picks them up in real time. When a rejection happens, the metric ribbon shows an MCP Rejections counter."

Dashboard: http://localhost:8080/admin/

---

## STEP 8: REQUEST TRACING

```
./demo/08-request-tracing.sh
```

SAY: "This traces a single request end-to-end. JSON arrives at the HTTP boundary — that's the only untyped moment. Serde immediately deserializes it into Rust structs. From there, every field is typed. UserId, SessionId, AgentName — all newtype wrappers. The compiler prevents mixing them up. Every database query uses sqlx macros — checked against the live schema at compile time."

SAY for benchmark: "100 concurrent governance requests. Each one: JWT validation, scope resolution, three rule evaluations, async database write. All typed. Zero garbage collector pauses. That's what Rust's zero-cost abstractions deliver."

EXPECT: Typed payloads, ID table, 4 CLI log commands, ASCII flow map, benchmark results.

---

## WHY 3 AI REQUESTS FOR THE PLATFORM AGENT?

Lee may ask. The answer:

1. AI receives the message, sees MCP tools, decides to call `systemprompt`
2. MCP tool returns result, AI processes the tool output
3. AI formats the final response

Normal multi-turn tool use. Each step traced and costed separately.

---

## IF SOMETHING GOES WRONG

```
systemprompt infra services cleanup --yes
systemprompt infra services start --kill-port-process
```

Wait for "All services started successfully" then retry from step 1.

---

## COST

Steps 1, 2, 4, 5 each cost one AI call (~$0.01 on Gemini Flash). Steps 0, 3, 6, 7, 8 are free (read-only or direct API calls, no AI).
