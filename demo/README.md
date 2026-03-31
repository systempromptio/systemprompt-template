# Live Demo Script

Ten scripts. Run them in order.

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
./demo/09-agent-tracing.sh
```

---

## Overview

| Demo | What it proves | Cost |
|------|---------------|------|
| 00 | Services are running | Free |
| 01 | Governance ALLOWS admin-scope tool call, then MCP tool executes | Free |
| 02 | Governance DENIES user-scope agent calling admin tool (scope restriction) | Free |
| 03 | Governance audit trail — both decisions queryable | Free |
| 04 | Detailed rule evaluation — all 3 rules pass for admin agent | Free |
| 05 | Detailed rule evaluation — scope check + blocklist deny for user agent | Free |
| 06 | Secret detection blocks leaked credentials in tool inputs | Free |
| 07 | MCP tool calls are OAuth-authenticated and audit-logged | Free |
| 08 | End-to-end request tracing, typed IDs, flow maps, 100-request benchmark | Free |
| 09 | Full agent pipeline — AI reasoning, MCP tool calls, artifacts, tracing | ~$0.01 |

Demos 01-08 call the governance API directly with curl — simulating Claude Code's PreToolUse hook workflow. No AI calls, deterministic, instant.

Demo 09 is the only demo that runs a live agent. It shows the platform agent runtime with full tracing, artifacts, and MCP tool calls.

---

## STEP 0: PREFLIGHT

```
./demo/00-preflight.sh
```

You should see services running: 3 agents + 2 MCP servers.

If services are down, it tells you the fix commands.

---

## STEP 1: HAPPY PATH

```
./demo/01-happy-path.sh
```

SAY: "This simulates what Claude Code does under the hood. When the admin agent calls a tool, the PreToolUse hook fires and POSTs to the governance endpoint. Admin scope — all rules pass — ALLOW. Then the MCP tool executes and returns real data."

EXPECT: Part 1 shows ALLOW JSON. Part 2 shows the actual agent list from MCP.

---

## STEP 2: REFUSED PATH

```
./demo/02-refused-path.sh
```

SAY: "Same tool, different agent. User scope. Governance denies it — scope_restriction rule fails. In production, this agent would never even see the tool because MCP mappings filter it. But governance provides a second enforcement layer."

EXPECT: DENY JSON with scope_restriction policy.

---

## STEP 3: AUDIT TRAIL

```
./demo/03-audit-trail.sh
```

SAY: "Both decisions are in the database. The ALLOW from demo 01, the DENY from demo 02. Every governance decision is queryable — decision, tool, agent, scope, policy, reason."

EXPECT: Table showing ALLOW for developer_agent and DENY for associate_agent.

Optional — cost breakdown:

```
systemprompt analytics costs breakdown --by agent
```

---

## STEP 4: GOVERNANCE HAPPY PATH

```
./demo/04-governance-happy.sh
```

SAY: "Now let's see the rule evaluation in detail. Admin agent, clean tool input. Three rules evaluated: scope check passes for admin, secret detection finds nothing, rate limit is fine. All pass. ALLOW."

EXPECT: ALLOW JSON. Audit shows evaluated_rules with all 3 rules passed.

---

## STEP 5: GOVERNANCE DENIED PATH

```
./demo/05-governance-denied.sh
```

SAY: "Two denial scenarios. First: user-scope agent tries an admin tool — scope check fails. Second: user-scope agent tries a destructive tool — both scope check and blocklist trigger. When Claude Code receives DENY, it blocks the tool and shows the governance reason to the user."

EXPECT: Two DENY responses with different policies.

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

## STEP 9: AGENT TRACING

```
./demo/09-agent-tracing.sh
```

SAY: "This is the live agent. Admin scope, MCP tools available. It calls the systemprompt MCP server, gets real data, produces a structured artifact. Every step is traced — AI requests, MCP tool calls, execution timing, cost attribution."

EXPECT: Agent response with real agent list. Artifact retrieval. Full trace showing ~11 events, 3 AI requests, 1 MCP tool call. Takes ~5 seconds.

WHY 3 AI REQUESTS?

1. AI receives the message, sees MCP tools, decides to call `systemprompt`
2. MCP tool returns result, AI processes the tool output
3. AI formats the final response

Normal multi-turn tool use. Each step traced and costed separately.

Dashboard: http://localhost:8080/admin/traces

---

## IF SOMETHING GOES WRONG

```
systemprompt infra services cleanup --yes
systemprompt infra services start --kill-port-process
```

Wait for "All services started successfully" then retry from step 1.

---

## COST

Steps 0-8 are free (direct API calls, CLI commands, no AI). Step 9 costs one AI call (~$0.01 on Gemini Flash).
