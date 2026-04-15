use std::fs;
use std::path::Path;

use super::config::{DemoCategory, DemoStep, DemosConfig, QuickStartStep};

const CLI_PREFIXES: &[&str] = &[
    "run_cli_indented ",
    "run_cli_head ",
    "run_cli ",
    "\"$CLI\" ",
];

const DEFAULT_TITLE: &str = "Run the Platform";
const DEFAULT_SUBTITLE: &str =
    "Ten guided walkthroughs, each a sequential story you can run against your local instance. \
     Every step is a real shell script; every command is copy-paste ready.";

struct StepMeta {
    script: &'static str,
    label: &'static str,
    narrative: &'static str,
    outcome: &'static str,
}

struct CategoryMeta {
    id: &'static str,
    title: &'static str,
    tagline: &'static str,
    story: &'static str,
    cost: &'static str,
    steps: &'static [StepMeta],
}

const CATEGORIES: &[CategoryMeta] = &[
    CategoryMeta {
        id: "governance",
        title: "Governance",
        tagline: "Policy that runs on every tool call, not a slide in a security review.",
        story: "Staff engineers want to know what actually happens when an agent goes off-script. This walkthrough fires real PreToolUse hooks against the governance endpoint: a clean admin call passes all three rules, a user-scope agent gets denied at the scope and blocklist layers, and plaintext AWS keys, GitHub PATs, and PEM private keys are blocked before they reach a tool. Every decision lands in an audit table you can query, capped by rate limits you can inspect and hooks you can extend.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-happy-path.sh",
                label: "Walk the full hook → allow → execute path",
                narrative: "POSTs a PreToolUse hook for developer_agent calling mcp__systemprompt__systemprompt — the CLI-wrapper tool exposed by the systemprompt MCP server — sees governance return allow, then actually invokes the tool via the CLI with {command: \"admin agents list\"} so you watch the same request land in the real agent runtime. This is the Claude Code hook workflow end to end.",
                outcome: "An allow response from /hooks/govern, a live agents listing returned by the systemprompt MCP server, and a matching row in governance_decisions tied to session demo-happy-path.",
            },
            StepMeta {
                script: "02-refused-path.sh",
                label: "Deny a user-scope agent reaching for an admin MCP tool",
                narrative: "Fires the same hook as associate_agent — a user-scope identity — against the admin-only mcp__systemprompt__list_agents tool. The scope_restriction rule fires at the governance layer, which is the backup line of defense behind tool mapping.",
                outcome: "A deny response citing scope_restriction, plus the matching deny row in governance_decisions — proof that even if tool mapping is bypassed, governance still holds.",
            },
            StepMeta {
                script: "03-audit-trail.sh",
                label: "Query the decisions table and break down cost by agent",
                narrative: "Runs infra db query against governance_decisions to pull the last five decisions with agent, scope, policy, and reason, then calls analytics costs breakdown --by agent. This is the evidence you hand to a compliance reviewer.",
                outcome: "A table showing the allow and deny records from the previous steps with policy names and reasons, plus per-agent cost totals.",
            },
            StepMeta {
                script: "04-governance-happy.sh",
                label: "Allow a clean admin call and show every rule that fired",
                narrative: "POSTs a PreToolUse hook for developer_agent calling Read with a plain file path, then dumps the evaluated_rules column. You see scope_check, secret_injection, and rate_limit each return PASS — not a black-box allow.",
                outcome: "The API returns permissionDecision: allow and the audit row lists all three rules with PASS reasons.",
            },
            StepMeta {
                script: "05-governance-denied.sh",
                label: "Deny a user-scope agent and a destructive tool in one run",
                narrative: "Fires two hook calls as associate_agent: one reaches for an admin-only MCP tool, the other calls delete_agent. scope_check blocks the first; scope_check and blocklist both trigger on the second.",
                outcome: "Two deny responses with distinct policy IDs, logged to governance_decisions so you can see the reason Claude Code would surface to the agent.",
            },
            StepMeta {
                script: "06-secret-breach.sh",
                label: "Block an AWS key, a GitHub PAT, and a private key in tool input",
                narrative: "Sends four PreToolUse hooks as admin-scope developer_agent: a Bash curl carrying AKIA..., a Write dropping ghp_... into .env, a Write piping a PEM key into id_rsa, and a clean Read as control. Secret detection overrides scope.",
                outcome: "Three denies (secret_injection) and one allow, all attributed to an admin agent in the audit table — proof the safety net holds against prompt-injection exfiltration.",
            },
            StepMeta {
                script: "07-rate-limiting.sh",
                label: "Inspect the rate limit, security, and server config",
                narrative: "Calls admin config rate-limits show and compare, then admin config security show and admin config server show. These are the ceilings that cap a runaway agent before governance rules even fire.",
                outcome: "Concrete numbers for per-tier request limits, security posture, and server knobs — the config you would tune in production.",
            },
            StepMeta {
                script: "08-hooks.sh",
                label: "List every registered hook and validate the wiring",
                narrative: "Runs core hooks list to show every PreToolUse, PostToolUse, and lifecycle hook across installed plugins, then core hooks validate to confirm each one resolves. This is the extension surface for your own policies.",
                outcome: "A validated hook inventory with plugin, event, and handler — the exact place you'd drop a custom rule.",
            },
        ],
    },
    CategoryMeta {
        id: "agents",
        title: "Agents",
        tagline: "From \"which agents ship with the platform?\" to a fully traced AI call in five commands.",
        story: "Staff engineers evaluating an agent runtime want three things: to see what's configured, to watch one actually do work, and to trust that every call is auditable. This walkthrough starts at agent discovery, drills into the config and tool scopes that separate an admin agent from a user-scoped one, sends a live message that triggers real MCP tool use, and ends at the execution trace and A2A registry. By the last step you have seen events, artifacts, cost attribution, and service discovery for a single prompt.",
        cost: "API key required",
        steps: &[
            StepMeta {
                script: "01-list-agents.sh",
                label: "Discover the agents the platform ships with",
                narrative: "Lists every configured agent via admin agents list, then shows process status and — when at least one agent is defined in services/agents/ — prints the full config for the first one. The template ships with an empty agent registry so you can see the commands, the data shapes, and the exact path where new agent YAMLs would drop in.",
                outcome: "You see the agent roster (empty by default), the running-agent status table, and either a concrete agent config or a pointer to services/agents/<id>.yaml as the place to add one.",
            },
            StepMeta {
                script: "02-agent-config.sh",
                label: "Validate configs and inspect tool scopes",
                narrative: "Shows live process status, then validates the first configured agent (if any) and enumerates the MCP tools it can call. When no agents are configured, falls back to listing every MCP tool across both servers so you still see the concrete tool surface the runtime exposes.",
                outcome: "A process status table plus either a validated agent config and its tool list, or the full cross-server MCP tool inventory.",
            },
            StepMeta {
                script: "03-agent-messaging.sh",
                label: "Send a real message and let the agent work",
                narrative: "Creates a context, picks the first configured agent, messages it with \"List all agents running on this platform\" in blocking mode, then retrieves the structured artifact it produced. This is the only step that spends money and the only one that exercises the full runtime. When the template has no agents configured, the script prints a clear explanation and exits cleanly.",
                outcome: "When an agent is configured: a reply, an artifact attached to the context, and a dashboard link scoped to the exact session_id. Otherwise: a skip notice pointing at services/agents/ with instructions for adding one.",
            },
            StepMeta {
                script: "04-agent-tracing.sh",
                label: "Replay what just happened",
                narrative: "Pulls the most recent execution traces, expands the latest one to show every event, lists artifacts produced during the run, and breaks cost down by agent. Nothing is mocked — this reads the same tables the dashboard reads.",
                outcome: "The trace shows roughly eleven events including AI requests, MCP tool calls with arguments, latency per step, and a dollar figure attributed to developer_agent.",
            },
            StepMeta {
                script: "05-agent-registry.sh",
                label: "See the agents as discoverable services",
                narrative: "Queries the A2A gateway registry so you can see how other agents and external callers discover these workers, then tails the process logs for each agent. A2A is what makes this a platform rather than a single chatbot.",
                outcome: "The registry shows every agent currently registered with the A2A gateway, and the per-agent logs pull live stdout from the processes serving them (empty in the default template until you add an agent YAML).",
            },
        ],
    },
    CategoryMeta {
        id: "mcp",
        title: "MCP Servers",
        tagline: "Every MCP tool call is inventoried, governed, and costed — watch it catch a leaking secret.",
        story: "MCP is where agents meet real systems, which is exactly where enterprise reviews stall: who is connected, who called what, and what stops a misbehaving model from exfiltrating a secret. This walkthrough starts at the server inventory, then fires a governance hook with a clean payload and a payload containing an AWS key to show allow and deny decisions hitting the audit tables, and ends in the execution analytics view where every tool call is already counted.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-mcp-servers.sh",
                label: "Inventory the connected MCP servers",
                narrative: "Prints runtime status for every MCP server the platform is connected to, then lists the tools exposed by each one, filtered down to the systemprompt and skill-manager servers so you can see what an agent actually has on its tool bar.",
                outcome: "You see which MCP servers are live, their transport and health, and the exact tool names and schemas available to any agent wired into them.",
            },
            StepMeta {
                script: "02-mcp-access-tracking.sh",
                label: "Fire the governance hook, clean then dirty",
                narrative: "POSTs to /hooks/govern twice — once with a benign Read on /src/main.rs and once with a Bash command containing an AWS access key — then calls skill-manager list_plugins through the authenticated MCP path and queries the governance_decisions and user_activity tables directly.",
                outcome: "The first call returns permissionDecision allow, the second returns deny with the secret_injection rule as the reason, and the audit tables show both decisions plus the MCP access event rows the dashboard renders.",
            },
            StepMeta {
                script: "03-mcp-tool-execution.sh",
                label: "Zoom out to the execution analytics",
                narrative: "Lists the tools exposed by the systemprompt and skill-manager MCP servers, pulls the last ten MCP tool execution log entries, and prints usage stats and seven-day trends. This closes the loop from \"a call happened\" to \"here is how the fleet is being used.\"",
                outcome: "A per-server tool inventory, a chronological tool execution log with latencies, and aggregate counts and trends broken down by tool name.",
            },
        ],
    },
    CategoryMeta {
        id: "skills",
        title: "Skills & Content",
        tagline: "Skills, the content that grounds them, and the contexts that ship them to live agents.",
        story: "\"Skill\" is a loaded word, so this walkthrough defines it by showing the whole supply chain. It starts with the skill lifecycle commands, moves to the content store that skills retrieve from, then to the managed file system that backs the content, then to the plugin package that bundles skills and hooks for distribution, and ends at contexts — the conversation containers every agent session binds to. By the end you can trace a skill from on-disk markdown to a live agent reply.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-skill-lifecycle.sh",
                label: "Walk the skill lifecycle",
                narrative: "Lists every database-synced skill, prints local-vs-remote sync status, and then walks services/skills/ on disk so you see each skill YAML that ships with the template and the structured header of the first one. This is the fastest way to answer \"what does a skill look like on disk?\".",
                outcome: "You see the synced skill catalog, a sync report telling you whether cloud and local copies match, and the concrete on-disk YAMLs under services/skills/ with the first one previewed.",
            },
            StepMeta {
                script: "02-content-management.sh",
                label: "Inspect the content skills retrieve from",
                narrative: "Lists the content store, searches it for \"governance\", ranks popular documentation entries, and prints ingestion status for the documentation source. Content is what gives a skill grounded answers instead of hallucinations.",
                outcome: "You get a catalog of indexed content, a live keyword search result, a popularity ranking, and an ingestion health report for the documentation source.",
            },
            StepMeta {
                script: "03-file-management.sh",
                label: "Drop down to the files backing content",
                narrative: "Lists managed files, shows the upload configuration including size and type limits, and prints aggregate file statistics. This is the layer most content platforms hide — the script shows it is just another CLI surface.",
                outcome: "You see the managed file inventory, the concrete upload policy the platform enforces, and total counts and size by type.",
            },
            StepMeta {
                script: "04-plugin-management.sh",
                label: "Package skills into a plugin",
                narrative: "Lists core plugins, walks services/plugins/ on disk to preview each plugin YAML (including enterprise-demo and how it bundles skills, MCP servers, and hooks), lists and validates hooks, then lists loaded extensions and their capabilities. Plugins are the unit of distribution — this is how a skill reaches other installations.",
                outcome: "You see the plugin manifests on disk, the hook inventory with a green validation result, and the extension capability table that tells you exactly what the runtime exposes.",
            },
            StepMeta {
                script: "05-contexts.sh",
                label: "Bind everything to a live context",
                narrative: "Lists existing contexts, creates a new one, shows it, renames it, deletes it, and re-lists to prove cleanup worked. Contexts are the conversation containers every agent session attaches to, so this is where skills, content, and artifacts meet at runtime.",
                outcome: "You watch a full create-read-update-delete cycle against real storage, with the final list confirming the context was removed.",
            },
        ],
    },
    CategoryMeta {
        id: "infrastructure",
        title: "Infrastructure",
        tagline: "A guided tour of the platform's day-2 operational surface.",
        story: "Before you adopt anything, you want to know what it looks like at 2am. This walkthrough takes you through the operator CLI the same way an on-call engineer would: check service health, inspect the database, audit scheduled work, tail the logs, then review config. Every command is read-only, every answer is a single CLI call, and nothing here needs a dashboard.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-services.sh",
                label: "Check service health",
                narrative: "Runs infra services status and then --detailed to show the agent and MCP server processes the platform supervises, with pids, ports, and health. This is the first thing you reach for when something feels wrong.",
                outcome: "You see 2 MCP servers (systemprompt and skill-manager) running as supervised processes, plus any configured agents, with health and lifecycle hooks for start/stop/restart/cleanup.",
            },
            StepMeta {
                script: "02-database.sh",
                label: "Inspect the Postgres schema",
                narrative: "Walks the database from the outside in: connection info, table list, schema of governance_decisions, indexes, row counts, size, a live read-only SQL query, migration status, and schema validation. Nothing is hidden behind an ORM.",
                outcome: "You see the real schema, a live SELECT against governance_decisions grouped by decision, and a passing migration and schema validation check.",
            },
            StepMeta {
                script: "03-jobs.sh",
                label: "Audit scheduled background jobs",
                narrative: "Lists every registered background job, shows one job's schedule and config, then prints execution history. This is how you answer \"what runs on its own, and did the last run succeed?\"",
                outcome: "You see the registered jobs, a specific job's cron schedule and configuration, and the last N executions with status and duration.",
            },
            StepMeta {
                script: "04-logs.sh",
                label: "Tail logs, traces, and AI requests",
                narrative: "Runs infra logs view, summary, search, then trace list, request list, and tools list. Application logs, per-request traces, AI call logs, and MCP tool executions are all first-class and queryable from one CLI.",
                outcome: "You see recent log lines, a level-aggregated summary, keyword search, execution traces, AI request records, and MCP tool call history — enough to debug any failure end-to-end.",
            },
            StepMeta {
                script: "05-config.sh",
                label: "Review runtime configuration",
                narrative: "Shows the merged config, the files it came from, validation status, runtime values, filesystem paths, and the active AI provider. Config is files on disk you can diff, not a mystery service.",
                outcome: "You see the effective configuration, the source files that produced it, a green validation check, and the current AI provider binding.",
            },
        ],
    },
    CategoryMeta {
        id: "analytics",
        title: "Analytics",
        tagline: "Every agent call, every token, every dollar — queryable from the CLI.",
        story: "If you can't answer \"what did my agents do today and what did it cost?\" you can't run them in production. This walkthrough starts at the 24-hour overview and drills down: which agents are busy, what each model is costing, and the raw request stream with latency and token counts. The same telemetry powers the dashboard, so nothing here is a toy view.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-overview.sh",
                label: "Open the 24h and 7d overview",
                narrative: "Runs analytics overview for the last 24 hours and then --since 7d. One command gives you the same top-of-funnel numbers the admin dashboard shows — request volume, agent activity, cost, errors.",
                outcome: "You see a 24h and 7d snapshot of request count, active agents, total cost, and error rate in a single table.",
            },
            StepMeta {
                script: "02-agent-analytics.sh",
                label: "Drill into agent performance",
                narrative: "Aggregate stats, a ranked list of every agent with metrics, 7-day trends, and a deep-dive into developer_agent. This is how you find the one agent quietly burning budget or failing silently.",
                outcome: "You see a per-agent table with request count, success rate, and latency, a 7-day trend line, and a per-agent deep dive for developer_agent.",
            },
            StepMeta {
                script: "03-cost-analytics.sh",
                label: "Break cost down by model and agent",
                narrative: "Prints the cost summary, then breaks it down by model and by agent, then shows 7-day trends. This is your finance conversation: Haiku is 80% of calls and 12% of cost; Opus is the opposite.",
                outcome: "You see total spend, cost split by model, cost split by agent, and a 7-day cost trend — attributable to the row.",
            },
            StepMeta {
                script: "04-request-analytics.sh",
                label: "Inspect the raw request stream",
                narrative: "Shows request stats, the last 24 hours of individual AI calls, the model usage distribution, and 7-day volume trends. Every row is a real request you can click through to the full audit log.",
                outcome: "You see request-level rows with model, latency, and token counts, the model distribution, and a 7-day volume trend.",
            },
            StepMeta {
                script: "05-session-analytics.sh",
                label: "Zoom into sessions — stats, trends, and live",
                narrative: "Runs analytics sessions stats, a 7-day trend line, and the live active-session view. Sessions answer \"who's using it right now\" and \"is usage trending up\".",
                outcome: "Aggregate session counts, a 7-day trend, and a live list of sessions currently open against the platform.",
            },
            StepMeta {
                script: "06-content-traffic.sh",
                label: "Break down content and traffic",
                narrative: "analytics content stats, top pages, 7-day content trends, traffic sources, geo distribution, and device breakdown. Same binary that governs agents also tells you where your readers come from.",
                outcome: "Content engagement stats, top pages, traffic source and geo split, and a device breakdown for the public site.",
            },
            StepMeta {
                script: "07-conversations.sh",
                label: "Inspect agent conversations",
                narrative: "Prints conversation stats, 7-day trends, and the 20 most recent conversations. Every agent session is a conversation you can list and replay — no black-box chat log.",
                outcome: "Counts and a 7-day trend for conversations, plus a list of the 20 most recent with agent, status, and turn count.",
            },
            StepMeta {
                script: "08-tool-analytics.sh",
                label: "Close the loop on tool usage",
                narrative: "analytics tools stats, a ranked list of tools with per-tool metrics, and a 7-day trend. Tells you which MCP tools your agents are actually reaching for and whether usage is shifting.",
                outcome: "Per-tool call counts, latency, and error rate, plus a 7-day trend showing whether any tool is spiking or quietly dying.",
            },
        ],
    },
    CategoryMeta {
        id: "users",
        title: "Users & Access",
        tagline: "Identity, roles, sessions, and abuse response from one CLI.",
        story: "Before trusting a platform with production agents, you want to know how user identity actually works. This walkthrough lists the user directory, drills into a single user's role, confirms the current authenticated session and profile, then demonstrates the IP-ban lever end to end — add, verify, remove — so you know the abuse-response path is real and reversible.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-user-crud.sh",
                label: "List, count, stat, and search the user directory",
                narrative: "Runs admin users list, count, stats, and search admin to show the full directory, aggregate counts, and a keyword lookup. This is the baseline inventory every reviewer asks for.",
                outcome: "A paginated user list, a total count, a stats summary, and matching rows for the admin search term.",
            },
            StepMeta {
                script: "02-role-management.sh",
                label: "Drill into a single user and read their role assignment",
                narrative: "Grabs the first user ID from admin users list and pipes it into admin users show. You see the role, scope, and metadata the governance layer will key off of.",
                outcome: "A full user record with role, scope, and audit fields — the data governance rules match against.",
            },
            StepMeta {
                script: "03-session-management.sh",
                label: "Prove the current session and list available profiles",
                narrative: "Calls admin session show to display who the CLI is authenticated as, then admin session list to show every profile the operator can switch into. This is how you confirm blast radius before running anything mutating.",
                outcome: "Current session identity with auth source, plus the full list of named profiles available for switching.",
            },
            StepMeta {
                script: "04-ip-ban.sh",
                label: "Add, verify, and revoke an IP ban end to end",
                narrative: "Lists current bans, adds 192.168.99.99 with a reason, lists again to confirm, then removes it and lists a third time. A full mutation cycle in one script, leaving no residue behind.",
                outcome: "The test IP appears in the ban list after add, disappears after remove, and the before and after lists match — the abuse-response lever is real and reversible.",
            },
        ],
    },
    CategoryMeta {
        id: "web",
        title: "Web Generation",
        tagline: "The same binary that runs governance also ships your marketing site.",
        story: "Most governance platforms stop at the API. Enterprise Demo also publishes systemprompt.io from the same Rust binary, using the same CLI, against the same database. This walkthrough inventories the content model, then runs the validator so you can see the publishing pipeline is real, typed, and CI-friendly.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-web-config.sh",
                label: "Inspect the content model",
                narrative: "Lists the registered content types, templates, and static assets that drive the site. Everything the web extension renders is declared config you can diff in git — no hidden CMS.",
                outcome: "You see the full set of content types, Tera templates, and asset bundles that compose systemprompt.io.",
            },
            StepMeta {
                script: "02-sitemap-validate.sh",
                label: "Show the sitemap and validate",
                narrative: "Prints the sitemap configuration and then runs web validate, which type-checks templates, asset references, and routing before anything ships. This is the same command wired into CI.",
                outcome: "A green validation report plus the live sitemap structure — proof the site builds deterministically from config.",
            },
        ],
    },
    CategoryMeta {
        id: "cloud",
        title: "Cloud",
        tagline: "Local dev and managed cloud are the same binary, one flag away.",
        story: "Every command you just ran against localhost takes a --profile flag. Point it at a managed tenant and the exact same CLI drives production. This walkthrough shows the read-only cloud surface — identity, deployment status, and profiles — so you can see how a laptop demo promotes to a real environment without a second toolchain.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-cloud-overview.sh",
                label: "Check identity, status, and profiles",
                narrative: "Runs cloud auth whoami, cloud status, and cloud profile list against the active profile. One command surface covers who you are, what's deployed, and which environments you can target — the same binary that ran every previous demo.",
                outcome: "You see your authenticated identity, the current deployment state, and every profile available for promotion from local to managed cloud.",
            },
        ],
    },
    CategoryMeta {
        id: "performance",
        title: "Performance",
        tagline: "Trace one request end-to-end, then prove it holds under 100 concurrent workers.",
        story: "Staff engineers don't trust benchmarks they can't reproduce. This walkthrough fires a real governance request, follows it through JWT validation, scope resolution, rule evaluation, and the async audit write — showing the typed Rust structs and sqlx-checked SQL at every stage. Then it runs 2,000 requests across three load profiles and prints real throughput, p50/p90/p99, and a capacity estimate in concurrent developers.",
        cost: "",
        steps: &[
            StepMeta {
                script: "01-request-tracing.sh",
                label: "Trace one governance request end-to-end",
                narrative: "Fires a real PreToolUse governance call plus a PostToolUse track event, then queries governance_decisions and plugin_usage_events to show the typed IDs that landed, pulls the trace and request logs for the call, and prints the router-to-audit flow map annotated with the Rust struct at each stage. Every layer is compile-time checked — serde at the boundary, newtypes in the handler, sqlx::query! against the live schema.",
                outcome: "You see the request payload, the response, the matching rows in governance_decisions and plugin_usage_events, the trace and request log entries, and a six-stage flow map from Axum router through JWT, scope resolution, rule engine, async audit write, and response.",
            },
            StepMeta {
                script: "02-load-test.sh",
                label: "Run a 2,000-request production load test",
                narrative: "Uses hey to drive 500 governance requests at 50 concurrent, 500 track requests at 50 concurrent, 1,000 sustained governance requests at 100 concurrent, and 5 MCP tool calls, then reads the Postgres pool stats and counts the audit rows that landed. This is what a single box does under real concurrency — no AI calls, pure platform infrastructure.",
                outcome: "You see throughput in req/s and p50/p90/p99 latency for each of three load profiles, MCP tool call latency, live Postgres pool occupancy, a decisions-written audit query confirming zero dropped requests, and a capacity estimate in concurrent developers per instance.",
            },
        ],
    },
];

pub fn scan_demos(demo_root: &Path) -> anyhow::Result<DemosConfig> {
    if !demo_root.is_dir() {
        anyhow::bail!("demo root not found: {}", demo_root.display());
    }

    let quick_start = scan_quick_start(demo_root);

    let mut categories = Vec::new();
    for meta in CATEGORIES {
        let dir = demo_root.join(meta.id);
        if !dir.is_dir() {
            tracing::warn!(
                category = meta.id,
                path = %dir.display(),
                "demo_scanner: skipping category — directory missing"
            );
            continue;
        }
        let steps = build_category_steps(&dir, meta);
        if steps.is_empty() {
            continue;
        }
        categories.push(DemoCategory {
            id: meta.id.to_string(),
            title: meta.title.to_string(),
            tagline: meta.tagline.to_string(),
            story: meta.story.to_string(),
            cost: meta.cost.to_string(),
            steps,
        });
    }

    Ok(DemosConfig {
        title: Some(DEFAULT_TITLE.to_string()),
        subtitle: Some(DEFAULT_SUBTITLE.to_string()),
        quick_start,
        categories,
    })
}

fn scan_quick_start(demo_root: &Path) -> Vec<QuickStartStep> {
    let mut steps = vec![
        QuickStartStep {
            label: "Build".to_string(),
            command: "just build".to_string(),
            description: Some("Compile the Rust workspace into a single binary.".to_string()),
        },
        QuickStartStep {
            label: "Seed local profile + Postgres".to_string(),
            command: "just setup-local <anthropic_key>".to_string(),
            description: Some(
                "Create an eval profile, start the Docker Postgres container, and run the publish pipeline. Pass whichever provider keys you have — Anthropic, OpenAI, or Gemini."
                    .to_string(),
            ),
        },
        QuickStartStep {
            label: "Start services".to_string(),
            command: "just start".to_string(),
            description: Some(
                "Launch every service on localhost:8080 — dashboard, admin panel, governance pipeline."
                    .to_string(),
            ),
        },
    ];

    if demo_root.join("governance/01-happy-path.sh").is_file() {
        steps.push(QuickStartStep {
            label: "First governance trace".to_string(),
            command: "./demo/governance/01-happy-path.sh".to_string(),
            description: Some(
                "Fire a PreToolUse hook, watch governance return allow, and land a row in governance_decisions."
                    .to_string(),
            ),
        });
    } else if demo_root.join("00-preflight.sh").is_file() {
        steps.push(QuickStartStep {
            label: "Preflight".to_string(),
            command: "./demo/00-preflight.sh".to_string(),
            description: Some(
                "Health-check services, create an admin session, and fetch a token.".to_string(),
            ),
        });
    }

    steps
}

fn build_category_steps(dir: &Path, meta: &CategoryMeta) -> Vec<DemoStep> {
    let mut out = Vec::new();
    for step in meta.steps {
        let path = dir.join(step.script);
        let Ok(content) = fs::read_to_string(&path) else {
            tracing::warn!(
                category = meta.id,
                script = step.script,
                "demo_scanner: missing script — skipping step"
            );
            continue;
        };
        let commands = extract_commands(&content);
        out.push(DemoStep {
            path: format!("demo/{}/{}", meta.id, step.script),
            name: step.script.to_string(),
            label: step.label.to_string(),
            narrative: step.narrative.to_string(),
            outcome: step.outcome.to_string(),
            commands,
        });
    }
    out
}

fn extract_commands(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        let cmd = CLI_PREFIXES
            .iter()
            .find_map(|prefix| {
                trimmed
                    .strip_prefix(prefix)
                    .map(|args| format!("systemprompt {}", args.trim()))
            })
            .or_else(|| trimmed.starts_with("systemprompt ").then(|| trimmed.to_string()));
        if let Some(c) = cmd {
            let cleaned = c.trim_end_matches(['\\']).trim().to_string();
            if !cleaned.is_empty() && !out.contains(&cleaned) {
                out.push(cleaned);
            }
        }
        if out.len() >= 6 {
            break;
        }
    }
    out
}
