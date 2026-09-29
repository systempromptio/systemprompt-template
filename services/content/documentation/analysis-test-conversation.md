---
title: "Create a Conversation and See It Land"
description: "Connect a client, invoke one skill on purpose, then find that conversation under Analysis and open its record. Includes how long to wait and what to check when it does not appear."
author: "systemprompt.io"
slug: "analysis-test-conversation"
keywords: "conversation, skill invocation, bridge, connect code, attribution, hooks, session id, conversation rollup, transcript"
kind: "guide"
public: true
tags: ["enterprise", "admin", "analytics"]
published_at: "2026-09-16"
updated_at: "2026-09-28"
after_reading_this:
  - "Connect a client to this gateway and invoke a named skill deliberately"
  - "Find the resulting conversation under Analysis, from the skill's page or filtered by skill"
  - "Open the conversation's record and transcript"
  - "Diagnose a missing conversation: no hooks, no session id, or rollup not yet run"
related_docs:
  - title: "Analysis: The Record of Every Conversation and Skill"
    url: "/documentation/analysis"
  - title: "Connect Claude Code"
    url: "/documentation/connect-claude-code"
  - title: "Install the Bridge"
    url: "/documentation/bridge-install"
---

# Create a conversation and see it land

**TL;DR:** A skill conversation exists because a signed-in person invoked a skill through a connected client whose inference went through this gateway. Connect Claude Code through the bridge, invoke one named skill, wait a minute or two, then open `/admin/analysis/skills` and click that skill.

## Prerequisites

- A user account on this instance, and its email or user id.
- The gateway's HTTPS address, or `http://localhost:8080` for a local instance.
- At least one skill you can name, from a marketplace your account receives — for example `who_am_i` in the `systemprompt` plugin, which Claude Code shows as `/systemprompt:who-am-i`.

## Step 1: Connect a client

Follow [Install the Bridge](/documentation/bridge-install), then [Connect Claude Code](/documentation/connect-claude-code) or [Connect OpenCode](/documentation/connect-opencode). Cowork is a separate client; connecting one does not configure the other.

For a local gateway, sign the bridge in against the loopback address (the bridge accepts `http://localhost` and `http://127.0.0.1` as well as HTTPS). To connect without a browser, an operator can issue a single-use connect code from the CLI:

```bash
systemprompt admin bridge issue-code --user-id you@example.com
systemprompt-bridge login --gateway http://localhost:8080 --code <code>
```

Codes are single-use with a ten-minute time to live, so issue one immediately before connecting.

**Expected result:** `systemprompt-bridge sync` completes and the plugin's skills are listed in the client.

## Step 2: Confirm the client is really routed through the gateway

In the connected client, ask any short question. Then open `/admin/history` in the console and confirm a request appears for your user.

A successful sign-in does not prove routing. This step does.

## Step 3: Invoke one skill on purpose

Model access and skill access are separate. Invoke the skill explicitly rather than hoping the model reaches for it: use its slash command, or ask for it by name so the client calls its `Skill` tool. Let the run finish.

The client's hooks report the invocation to this gateway as it happens — the plugin, the skill and the Claude Code session id — and the gateway stamps the same session id on every inference request of that session. That shared id is what joins the skill to the conversation.

**Expected result:** the skill's instructions are visible in the client's output, not a generic model answer.

## Step 4: Wait for the rollup

The `conversation_rollup` job re-derives a conversation's row, and a row per skill it invoked, every minute for any conversation touched on any plane. Give it a minute or two before concluding anything is wrong. The Analysis pages read those rows, not raw events, so a conversation is invisible until the rollup has seen it.

## Step 5: Find the conversation

1. Open `/admin/analysis/skills`, choose the **Skills** tab, and pick a window that covers today.
2. Click the skill's name. Its page, `/admin/analysis/skills/<plugin:skill>`, lists the conversations behind it with person, tool calls, tokens, cost and judge score.

Alternatively, open `/admin/analysis/conversations` and filter by the skill in the filter ribbon.

## Step 6: Open the record

Click the conversation. `/admin/analysis/conversations/<context_id>` shows it on every plane: the turn ledger, tool calls, governance decisions, safety findings, and the skills invoked with the marketplace version served at the time. The raw transcript for the same session is also reachable from `/admin/history`.

## When it does not appear

Work down this list in order. The causes are distinct and the fix differs for each.

| Check | Symptom | What it means | Fix |
|---|---|---|---|
| 1. Routed? | Nothing for your user on `/admin/history`. | The client's inference is not going through this gateway. | Re-run [Connect Claude Code](/documentation/connect-claude-code) and confirm routing (Step 2). |
| 2. Invoked? | The request is on History but the skill shows no invocations on Skills. | No hook reported the invocation — the plugin carrying the governance hooks is not installed in the client, or the skill was never actually invoked. | Run `systemprompt-bridge sync`, confirm the plugin is installed, and invoke the skill by its slash command. |
| 3. Joined? | The skill counts invocations and people, but **Conv.**, tokens and cost are zero. | The session that invoked it produced no gateway conversation with the same session id: the inference went elsewhere, or the client did not send its session id. | Use a client that routes both hooks and inference through this gateway, from the same session. |
| 4. Rolled up? | Everything above is fine, but the conversation is missing. | The rollup has not run since the change. | Wait a minute and reload, or run `systemprompt infra jobs run conversation_rollup`. |

**Attributed is not install-verified.** Attributed means a hook reported the skill. The **Installs** column is stricter: consumers holding a verified installation receipt for the published skill. A conversation can be attributed without the device having reported a receipt; receipts are listed under **Distribution** on the marketplace's [Versions](/documentation/analysis-versions) page.

## Troubleshooting

### The client asks for a code on every run

**Symptom:** A repeat run prompts for a connect code.
**Cause:** The stored credential did not validate against this gateway, usually because a different gateway issued it.
**Solution:** Sign the bridge in again against this gateway with a fresh code.

### The conversation is in History but has no skills

**Symptom:** `/admin/history` shows the request and Analysis › Conversations lists it, but its skills column is empty.
**Cause:** The request's session carries no hook-reported skill invocation, so there is nothing to attribute it to.
**Solution:** Re-run invoking the skill explicitly, and confirm the skill is served from a marketplace this user receives.

## Related pages

- [Analysis overview](/documentation/analysis)
- [Measure Which Skills Are Used](/documentation/analysis-measure-skills)
- [Connect Claude Code](/documentation/connect-claude-code)
- [Install the Bridge](/documentation/bridge-install)
