# Recording Guide: systemprompt.io Product Demo Video

**Target length:** 7:00 — 8:30  
**Target audience:** CTOs and engineering leaders evaluating AI governance  
**Tone:** Technical but accessible. Show, don't sell.

---

## Technical Setup

### Screen Recording

- **Resolution:** 1920x1080 native, export at 1080p
- **FPS:** 30fps (terminal content, no fast motion)
- **Recording tool:** OBS Studio (screen capture + mic audio)
- **Export:** H.264, high quality, .mp4

### Terminal

- **Font:** JetBrains Mono, 16pt minimum (must be legible at 720p)
- **Theme:** Dark background — Catppuccin Mocha, Dracula, or One Dark
- **Prompt:** Minimal. Just `$` or short hostname. Remove git status, clock, etc.
- **Window:** 120 columns x 30 rows minimum
- **Scrollback:** Cleared before each demo segment

### Layout

- **Demos 01-06:** Full-screen terminal
- **Demos 07-09:** Split screen — terminal left (60%), browser dashboard right (40%)
- **Dashboard URL:** `http://localhost:8080/admin/`

### Audio

- **Microphone:** External USB mic or headset (not laptop built-in)
- **Environment:** Quiet room, no keyboard clicks during narration
- **Levels:** Test with OBS audio meter — peak at -6dB, average at -18dB

---

## Pre-Recording Checklist

- [ ] Build release binary: `cargo build --release`
- [ ] Run `./demo/00-preflight.sh` — verify 3 agents + 2 MCP servers running
- [ ] Run each demo once to warm caches and confirm output
- [ ] Clear terminal scrollback: `clear && printf '\033[3J'`
- [ ] Close all desktop notifications (Do Not Disturb mode)
- [ ] Close browser tabs except dashboard
- [ ] Test audio levels — record 10 seconds and play back
- [ ] Verify screen recording captures both terminal and browser
- [ ] Set `RUST_LOG=warn` to suppress debug output

---

## Video Segments

### Segment 1: Intro (0:00 — 0:30) | 30 seconds

**Visual:** Title card or terminal with systemprompt.io banner

**NARRATOR:**

> This is systemprompt.io — AI governance infrastructure you deploy on your own servers. One Rust binary. Zero cloud dependencies. In the next eight minutes, I'll show you every enforcement layer running live: tool governance, secret detection, MCP access tracking, request tracing, a live AI agent, and a load test pushing two thousand requests through the governance engine. Everything you're about to see runs on a single machine. Total AI cost for the entire demo: under one cent.

**Action:** None. Static visual or slow type of `systemprompt --help` output.

---

### Segment 2: Governance Allow (0:30 — 1:15) | 45 seconds

**Demo script:** `./demo/01-happy-path.sh`

**NARRATOR (before running):**

> When Claude Code calls a tool, a PreToolUse hook fires. That hook POSTs to systemprompt.io's governance endpoint with the tool name, agent identity, and scope. Let's see what happens when an admin-scope agent calls an MCP tool.

**Action:** Run the script. Let it execute fully.

**NARRATOR (during output):**

> Governance evaluates three rules: scope check, secret detection, rate limit. All pass. The response is ALLOW. Now the MCP tool executes and returns real data — this is a live tool call, not a mock.

**Highlight:** Point to `"permissionDecision": "allow"` in the JSON response, then the MCP tool output showing actual agent data.

---

### Segment 3: Governance Deny (1:15 — 1:45) | 30 seconds

**Demo script:** `./demo/02-refused-path.sh`

**NARRATOR (before running):**

> Same tool. Different agent. This time it's a user-scope agent trying to access an admin tool.

**Action:** Run the script.

**NARRATOR (during output):**

> Denied. The scope_restriction rule fails immediately. In production, this agent would never even see the tool because MCP mappings filter it out. But governance provides a second enforcement layer. Defense-in-depth — two independent gates, both must pass.

**Highlight:** Point to `"permissionDecision": "deny"` and `"policy": "scope_restriction"`.

---

### Segment 4: Audit Trail (1:45 — 2:15) | 30 seconds

**Demo script:** `./demo/03-audit-trail.sh`

**NARRATOR (before running):**

> Both decisions — the allow and the deny — are in the database. Let's query them.

**Action:** Run the script.

**NARRATOR (during output):**

> Every governance decision is queryable. Decision, tool name, agent identity, scope, policy, reason, timestamp. This is what your compliance team needs — a complete audit trail of every AI tool interaction, exportable to your SIEM.

**Highlight:** The table showing ALLOW for developer_agent and DENY for associate_agent side by side.

---

### Segment 5: Secret Detection (2:15 — 3:15) | 60 seconds

**Demo script:** `./demo/06-governance-secret-breach.sh`

**NARRATOR (before running):**

> Prompt injection can trick an AI agent into passing credentials to an external tool. systemprompt.io inspects every tool input for plaintext secrets before execution. Let's test it.

**Action:** Run the script. It runs four tests sequentially.

**NARRATOR (during each test — pace with output):**

> **Test one:** An AWS access key in the tool input. Blocked.
>
> **Test two:** A GitHub personal access token. Blocked.
>
> **Test three:** An RSA private key. Blocked.
>
> **Test four:** A clean file read. This one passes through.

**NARRATOR (after all four):**

> Even admin-scope agents cannot bypass secret detection. Three denials, one allow. All four decisions are in the audit trail.

**Highlight:** The visual contrast of three DENY responses followed by one ALLOW. Pause briefly on each.

---

### Segment 6: MCP Access Tracking (3:15 — 4:15) | 60 seconds

**Demo script:** `./demo/07-mcp-access-tracking.sh`  
**Layout:** Switch to split screen — terminal left, dashboard right

**NARRATOR (before running):**

> Every MCP tool call is tracked. Authentication events, tool executions, successes, and rejections — all recorded. Let's make two real MCP calls and see them appear.

**Action:** Run the script.

**NARRATOR (during output):**

> Two MCP servers: skill-manager and systemprompt CLI. Both authenticate via OAuth. Both execute a tool. The dashboard shows MCP Server Access with per-server stats and granted/rejected badges.

**NARRATOR (pointing to dashboard):**

> On the right, the admin dashboard picks up these events in real time. The MCP Server Access section shows connection counts, tool calls, and a live activity feed. If a rejection happens, the metric ribbon shows an MCP Rejections counter.

**Highlight:** Terminal showing successful tool calls, then the dashboard's MCP Server Access section updating.

---

### Segment 7: Request Tracing & Benchmark (4:15 — 5:30) | 75 seconds

**Demo script:** `./demo/08-request-tracing.sh`

**NARRATOR (before running):**

> Let's trace a single request end to end. JSON arrives at the HTTP boundary — that's the only untyped moment. From there, everything is Rust structs, newtype wrappers, and compile-time checked queries.

**Action:** Run the script. It will take 30-45 seconds due to the benchmark.

**NARRATOR (during typed data section, ~15s):**

> Every field is typed. UserId, SessionId, AgentName — all newtype wrappers. The compiler prevents mixing them up. Every database query is checked against the live schema at compile time by sqlx.

**NARRATOR (during flow map, ~10s):**

> Six stages. HTTP entry, JWT validation, scope resolution, rule engine, async audit write, response. The audit write is fire-and-forget — the response returns before the database write completes.

**NARRATOR (during benchmark, wait for results then narrate ~20s):**

> Two hundred concurrent governance requests. Each one: JWT validation, scope resolution, three rule evaluations, async database write. [Wait for results.] Sub-5ms median latency. Sub-10ms p99. That's what zero garbage collector pauses and Rust's ownership model deliver. A single instance supports hundreds of concurrent developers.

**Highlight:** The benchmark results — throughput number, p50/p90/p99 latencies, and the capacity estimate.

---

### Segment 8: Live AI Agent (5:30 — 6:30) | 60 seconds

**Demo script:** `./demo/09-agent-tracing.sh`  
**Layout:** Split screen — terminal left, dashboard traces right

**NARRATOR (before running):**

> Everything so far has been deterministic API calls — no AI involved. Now let's run a live agent. The developer_agent will receive a message, reason about which tools to call, execute an MCP tool, and produce a structured artifact. Every step traced.

**Action:** Run the script. The agent takes ~15-20 seconds to respond.

**NARRATOR (while agent is thinking):**

> The agent is reasoning now. It sees available MCP tools, decides which to call, and processes the result. This is multi-turn tool use — three AI requests in a single interaction.

**NARRATOR (after agent responds):**

> The agent called the MCP tool, processed the result, and created an artifact. The full execution trace shows 11 events, three AI requests, one MCP tool call. Total cost: under one cent. The dashboard shows the trace in real time — every event, every timing, every token count.

**Highlight:** The artifact output, then the execution trace showing event count and cost breakdown.

---

### Segment 9: Load Test (6:30 — 8:00) | 90 seconds

**Demo script:** `./demo/10-load-test.sh`

**NARRATOR (before running):**

> Let's see how this scales. I'm going to fire two thousand requests at the governance endpoint — five hundred at a time, then a sustained burst of a thousand at a hundred concurrent. Every request does JWT validation, scope resolution, three rule evaluations, and an async database write.

**Action:** Run the script. It takes ~45-60 seconds total. Narrate during the warmup and between tests.

**NARRATOR (after Test 1 results appear):**

> [Read the throughput number] requests per second. Sub-[read p50] millisecond median latency. That's five hundred governance decisions with full audit trails.

**NARRATOR (after summary table):**

> Look at the capacity estimate. A single instance on this machine supports [read number] concurrent developers. Three instances behind a load balancer — [read number]. And this is a dev laptop, not production hardware. On dedicated infrastructure with PgBouncer and NVMe storage, these numbers multiply.

**NARRATOR (after audit count):**

> Every single request audited. Zero dropped. Zero failed. Zero garbage collector pauses. That's Rust.

**Highlight:** The summary comparison table and the enterprise capacity estimate. These are the money shots for CTOs.

---

### Segment 10: Outro (8:00 — 8:30) | 30 seconds

**Visual:** Dashboard overview or title card

**NARRATOR:**

> Nine capabilities. One binary. Every tool call governed, every secret scanned, every action traced, and it scales to hundreds of developers on a single instance. Self-hosted, air-gap capable, and the entire demo costs under a penny in AI spend. Deploy it on your infrastructure. Own it completely. Get started at systemprompt.io.

---

## Post-Production

### YouTube Chapter Markers

```
0:00 — Intro
0:30 — Governance: Allow Path
1:15 — Governance: Deny Path
1:45 — Audit Trail
2:15 — Secret Detection
3:15 — MCP Access Tracking
4:15 — Request Tracing & Benchmark
5:30 — Live AI Agent
6:30 — Load Test
8:00 — Outro
```

### YouTube Description

```
systemprompt.io is AI governance infrastructure — a single Rust binary you deploy on your own servers. This demo shows every enforcement layer running live:

- Tool governance with allow/deny decisions
- Secret detection blocking leaked credentials
- MCP access tracking with OAuth authentication
- End-to-end request tracing with typed IDs
- 200-request benchmark showing sub-5ms latency
- Live AI agent with MCP tools, artifacts, and full tracing

Total AI cost: under $0.01.

Get started: https://systemprompt.io
Documentation: https://systemprompt.io/documentation/
GitHub: https://github.com/systempromptio/systemprompt-template
```

### YouTube Tags

```
AI governance, Claude Code, MCP governance, AI agent security, AI audit trail, self-hosted AI, AI compliance, tool use governance, secret detection, request tracing, Rust, systemprompt
```

### Thumbnail

- Dark background matching site theme
- Terminal screenshot showing ALLOW/DENY JSON
- Text overlay: "AI Governance — Live Demo"
- Brand orange accent (#f79938)
- 1280x720 minimum

### Editing Notes

- Add subtle zoom-ins on key JSON responses (the ALLOW/DENY moments)
- Brief lower-third labels for each segment transition (e.g., "Secret Detection")
- No background music during terminal segments (distraction)
- Optional ambient music during intro/outro at -20dB
- Cut dead time during benchmark execution if it runs long — keep to 10s max of waiting

---

## Alternate Cuts

### Short Cut (2:00)

For social media and landing page autoplay:

1. **Intro** — 10s (shortened)
2. **Demo 01** — 20s (ALLOW only, skip audit)
3. **Demo 06** — 30s (all 4 tests, fast)
4. **Demo 08** — 30s (benchmark results only)
5. **Outro** — 10s (shortened)

Skip demos 02, 03, 07, 09.

### Medium Cut (4:00)

For product page and email campaigns:

1. **Intro** — 20s
2. **Demos 01+02** — 45s (allow then deny, no audit)
3. **Demo 06** — 45s
4. **Demo 08** — 60s (tracing + benchmark)
5. **Demo 09** — 30s (agent response + trace only)
6. **Outro** — 20s

Skip demos 03, 07.
