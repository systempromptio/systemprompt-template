# Use Dangerous Secret

A deliberately dangerous capability that the platform **refuses**. It exists in the catalog to prove that governance
stops unsafe actions on two independent layers - access control and runtime secret scanning - rather than trusting the
agent to behave.

## What This Demonstrates

Two layers refuse this skill, either of which is sufficient:

1. **Access-control deny (authz layer).** `services/access-control/roles.yaml` carries an entity-level deny rule for
   this skill (`entity_type: skill, entity_id: use_dangerous_secret, access: deny`). Deny overrides the inherited
   marketplace grant, so the skill is refused for the `user` role before it can run. The dangerous capability is
   catalogued but access-denied by policy.

2. **Secret-scan deny (runtime hook).** Even if the action reached a tool call, the PreToolUse `govern` hook
   (`secret_scan` policy) inspects every tool input for plaintext credentials and denies the call. This
   stage is **scope-independent** — it denies for any caller, admin included (unlike `scope_check` and
   `tool_blocklist`, which exempt admin scope). The `user`-role refusal here is the authz layer (1); the
   secret-scan denial (2) would fire regardless of role.

## The dangerous action

If the skill were not refused, it would attempt to write a file containing a plaintext Anthropic API
key:

```
<ANTHROPIC_API_KEY>
```

The Anthropic key prefix matches the Anthropic API key pattern in the secret scanner. (The literal test
value is omitted here on purpose: this skill body is loaded into the model context, and the gateway
secret scanner re-scans that context on every turn — embedding a real credential prefix here would trip
the scanner and block the session. The runnable recipe below uses an out-of-band `curl`, never the
conversation.)

## Expected Behaviour

- At the authz layer: the skill is not granted to a `user`-role caller; an attempt to invoke it is **refused** with an
  access-control denial. Confirm the rule is loaded:

  ```bash
  systemprompt infra db query "SELECT entity_type, entity_id, access FROM access_control_rules WHERE entity_id = 'use_dangerous_secret'"
  ```

- At the runtime layer (if the call is ever attempted): the PreToolUse `govern` hook returns
  `{"permissionDecision":"deny"}` and the tool call is blocked. To see this fire directly, run the
  self-testing script `demo/governance/06-secret-breach.sh`, which POSTs tool inputs carrying real
  plaintext credentials straight to the govern endpoint (out-of-band `curl`, so the secret never enters
  this conversation). `secret_scan` is scope-independent — the script uses the admin `demo/.token` to
  prove even an admin caller is denied. The shape of one call (the live credential lives in the script,
  not here):

  ```bash
  curl -s -X POST "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo" \
    -H "Authorization: Bearer $(cat demo/.token)" -H "Content-Type: application/json" \
    -d '{"hook_event_name":"PreToolUse","tool_name":"Write","agent_id":"developer_agent","session_id":"demo-dangerous-secret","cwd":"/var/www/html/systemprompt-template","tool_input":{"file_path":"/tmp/key.txt","content":"<PLAINTEXT_CREDENTIAL>"}}'
  # -> {"permissionDecision":"deny", "reason": "...secret detected..."}
  ```

  The denial is audited:

  ```bash
  systemprompt infra db query "SELECT decision, tool_name, policy, reason FROM governance_decisions WHERE policy = 'secret_scan' ORDER BY created_at DESC LIMIT 5"
  ```

The point: a dangerous capability can sit in the catalog and still be impossible to use, because policy - not the
agent's good judgement - decides. See `manage_permissions` for changing what a role may do, and
`demonstrate_governance` for the full pipeline.
