# Live Demo Script

Four scripts. Run them in order.

```
./demo/00-preflight.sh
./demo/01-happy-path.sh
./demo/02-refused-path.sh
./demo/03-audit-trail.sh
```

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

## STEP 7: MCP ACCESS TRACKING

```
./demo/07-mcp-access-tracking.sh
```

SAY: "Every MCP tool call is tracked. We make two real calls — one to the skill-manager, one to the systemprompt CLI server. Both authenticate via OAuth, both execute a tool. Every event — authentication, tool execution — is recorded in user_activity."

EXPECT: Two successful MCP tool calls, then a database query showing 4 events (2 authenticated + 2 used). Script prints the dashboard URL where you can see them.

SAY: "On the dashboard you can see the MCP Server Access section — per-server stats, Granted/Rejected badges, and the live activity feed picks them up in real time. When a rejection happens, the metric ribbon shows an MCP Rejections counter."

Dashboard: http://localhost:8080/admin/

---

## COST

Steps 1 and 2 each cost one AI call (~$0.01 on Gemini Flash). Step 3 is free (read-only). Step 7 is free (direct MCP calls, no AI).
