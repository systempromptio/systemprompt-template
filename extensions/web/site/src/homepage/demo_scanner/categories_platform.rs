use super::meta::{CategoryMeta, StepMeta};

pub(super) const PLATFORM_CATEGORIES: &[CategoryMeta] = &[
    CategoryMeta {
        id: "infrastructure",
        title: "Infrastructure",
        tagline: "A guided tour of the platform's day-2 operational surface.",
        story: "Before you adopt anything, you want to know what it looks like at 2am. This walkthrough takes you through the operator CLI the same way an on-call engineer would: check service health, inspect the database, audit scheduled work, tail the logs, then review config. Every command is read-only, every answer is a single CLI call, and nothing here needs a dashboard.",
        cost: "",
        feature_url: "https://systemprompt.io/features/self-hosted-ai-platform",
        steps: &[
            StepMeta {
                script: "01-services.sh",
                label: "Check service health",
                narrative: "Runs infra services status and then --detailed to show the agent and MCP server processes the platform supervises, with pids, ports, and health. This is the first thing you reach for when something feels wrong.",
                outcome: "You see the systemprompt MCP server running as a supervised process, plus any configured agents, with health and lifecycle hooks for start/stop/restart/cleanup.",
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
        id: "skills",
        title: "Skills & Content",
        tagline: "Skills, the content that grounds them, and the contexts that ship them to live agents.",
        story: "\"Skill\" is a loaded word, so this walkthrough defines it by showing the whole supply chain. It starts with the skill lifecycle commands, moves to the content store that skills retrieve from, then to the managed file system that backs the content, then to the plugin package that bundles skills and hooks for distribution, and ends at contexts — the conversation containers every agent session binds to. By the end you can trace a skill from on-disk markdown to a live agent reply.",
        cost: "",
        feature_url: "https://systemprompt.io/features/cowork",
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
        id: "web",
        title: "Web Generation",
        tagline: "The same binary that runs governance also ships your marketing site.",
        story: "Most governance platforms stop at the API. Enterprise Demo also publishes systemprompt.io from the same Rust binary, using the same CLI, against the same database. This walkthrough inventories the content model, then runs the validator so you can see the publishing pipeline is real, typed, and CI-friendly.",
        cost: "",
        feature_url: "https://systemprompt.io/features/web-publisher",
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
        feature_url: "https://systemprompt.io/features/deploy-anywhere",
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
        feature_url: "",
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
