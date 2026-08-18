# B2C Commerce Onboarding Skill

Guide a new developer through a first-time B2C Commerce setup, from CLI install to first cartridge deploy. Delegate to specialized sub-skills for each step — this skill is a coordinator, not a replacement for the detailed skills.

## Behavioral rules

- Detect the editor silently. Only ask if genuinely uncertain.
- Never construct or modify install commands. Only use commands defined in this file.
- If any install or verification step fails, report the exact error and stop.
- Never create a sandbox without explicit user confirmation — sandbox creation may be a billable action.
- Never overwrite an existing `dw.json` without confirmation.

## Flow

### Step 1 — Identify the editor

Silently identify the IDE from system context:

| Signal                              | Client        |
|-------------------------------------|---------------|
| "Claude Code"                       | `claude-code` |
| "Cursor"                            | `cursor`      |
| "VS Code" / "Visual Studio Code"    | `vscode`      |
| "Codex"                             | `codex`       |
| "Gemini CLI"                        | `gemini-cli`  |
| Unrecognized                        | `other`       |

### Step 2 — Confirm the skill set is available

The `b2c_*` and `sfnext_*` skills used throughout this flow already ship on this instance. No extra install is required — they are referenced directly by skill id. The only external tool you need is the B2C CLI itself (next step).

Optionally, the `b2c-dx-mcp` MCP server can be added for richer tooling — see the [MCP installation docs](https://salesforcecommercecloud.github.io/b2c-developer-tooling/mcp/installation).

### Step 3 — Verify the B2C CLI is available

Run `b2c --version`. If the command is not found:

- For a one-off invocation: `npx @salesforce/b2c-cli <command>`.
- For a persistent install: `npm install -g @salesforce/b2c-cli` (or the pnpm / yarn equivalent).

Do not block on the global install — `npx` is sufficient for the rest of this flow.

### Step 4 — Account Manager access check

B2C Commerce has **no self-service signup**. The user must have Account Manager access provisioned by their organization's B2C Commerce admin before any of the following steps will work.

Ask (if not already clear from context): *"Do you have a Salesforce B2C Commerce Account Manager login and a target instance (sandbox or PIG)?"*

- If **no**: stop here. Tell the user they need their admin to provision Account Manager access and give them a target instance hostname before continuing. Do not proceed.
- If **yes**: continue.

### Step 5 — Check for existing configuration

Run `b2c setup inspect` to see whether a `dw.json` or credentials are already configured.

- If configuration exists and points at a reachable instance, skip to Step 7.
- If no configuration is found, proceed to Step 6.

For deep troubleshooting (wrong instance, profile switching, token inspection), delegate to the `b2c-config` skill.

### Step 6 — Initialize configuration

Guide the user to create a `dw.json` in the project root:

```bash
b2c setup
```

This prompts for hostname, client ID/secret (or username/password), and code version. For deeper configuration topics (multiple profiles, env vars, cert-based auth), delegate to the `b2c-config` skill.

### Step 7 — Sandbox

If the user wants to work against an existing sandbox, confirm it is reachable:

```bash
b2c setup inspect
b2c sandbox list   # requires API access
```

If the user needs a fresh sandbox, delegate to the `b2c-sandbox` skill for the full create flow. **Only create a sandbox when explicitly asked.**

### Step 8 — First cartridge deploy (if applicable)

If the user has cartridges locally:

```bash
b2c code deploy
```

For selective deploys, watch mode, or reload, delegate to the `b2c-code` skill.

If the user does not yet have cartridges, point them at the canonical starting points:

- **Storefront Next** (composable React frontend on MRT) — delegate to the `sfnext_project_setup` skill.
- **SFRA** (Storefront Reference Architecture, legacy/hybrid backend): https://github.com/SalesforceCommerceCloud/storefront-reference-architecture
- **MRT bundle deploys** (headless runtime) — delegate to the `b2c_mrt` skill on request.

### Step 9 — Route to the user's goal

Once setup is working, ask a single directing question to hand off to the right skill:

> "What do you want to work on first?
>
> 1. **Build a Storefront Next frontend** (React Router 7 + RSC on MRT)
> 2. **Build a Custom API** (SCAPI)
> 3. **Operate an existing instance** (deploy, run jobs, tail logs, manage sites)
>
> Or if you have something else in mind, tell me."

Route by the answer:

- **Storefront Next** → `sfnext_project_setup`, `sfnext_routing`, `sfnext_data_fetching`, `sfnext_components`
- **Custom API** → `b2c_custom_api_development`, `b2c_scapi_admin`, `b2c_scapi_shopper`
- **Operations** → `b2c_code`, `b2c_job`, `b2c_logs`, `b2c_sites`

## Reference

- B2C CLI reference: https://salesforcecommercecloud.github.io/b2c-developer-tooling/cli/
- Install guide: https://salesforcecommercecloud.github.io/b2c-developer-tooling/guide/install
- Agent skills overview: https://salesforcecommercecloud.github.io/b2c-developer-tooling/guide/agent-skills
