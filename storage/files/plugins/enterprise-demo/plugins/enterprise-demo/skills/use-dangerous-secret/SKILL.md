---
name: use-dangerous-secret
description: "Demonstrates governance blocking when plaintext secrets are detected in tool inputs"
---

# Use Dangerous Secret

A demonstration skill that attempts to use a plaintext secret in a tool call. This skill is designed to be **blocked by the governance hook**.

## What This Demonstrates

The enterprise governance system includes a PreToolUse hook that inspects all tool inputs for plaintext secrets (API keys, tokens, passwords). When this skill is invoked, the governance hook detects the secret pattern and **denies** the tool call before it executes.

## Usage

When asked to demonstrate secret detection, attempt to write a file containing the following test API key:

```
sk-ant-demo-FAKE12345678901234567890
```

The governance PreToolUse hook will intercept this tool call and block it because it detects the `sk-ant-` prefix pattern, which matches the Anthropic API Key secret pattern.

## Expected Behavior

1. Agent attempts to use a tool (e.g., Write) with the secret value in the input
2. The PreToolUse governance hook fires
3. The hook sends the tool input to the governance endpoint for evaluation
4. The secret detection rule identifies the `sk-ant-` pattern
5. The governance endpoint returns a **deny** decision
6. Claude Code blocks the tool call and shows the denial reason

This demonstrates that enterprise governance policies can prevent sensitive data from being passed through agent tool calls, even when the agent is explicitly instructed to do so.
