use super::meta::CategoryMeta;

pub(super) const PLATFORM_CATEGORIES: &[CategoryMeta] = &[
    CategoryMeta {
        id: "infrastructure",
        title: "Infrastructure",
        tagline: "A guided tour of the platform's day-2 operational surface.",
        story: "Before you adopt anything, you want to know what it looks like at 2am. This walkthrough takes you through the operator CLI the same way an on-call engineer would: check service health, inspect the database, audit scheduled work, tail the logs, then review config. Every command is read-only, every answer is a single CLI call, and nothing here needs a dashboard.",
        cost: "",
        feature_url: "https://systemprompt.io/features/self-hosted-ai-platform",
    },
    CategoryMeta {
        id: "skills",
        title: "Skills & Content",
        tagline: "Skills, the content that grounds them, and the contexts that ship them to live agents.",
        story: "\"Skill\" is a loaded word, so this walkthrough defines it by showing the whole supply chain. It starts with the skill lifecycle commands, moves to the content store that skills retrieve from, then to the managed file system that backs the content, then to the plugin package that bundles skills and hooks for distribution, and ends at contexts — the conversation containers every agent session binds to. By the end you can trace a skill from on-disk markdown to a live agent reply.",
        cost: "",
        feature_url: "https://systemprompt.io/features/bridge",
    },
    CategoryMeta {
        id: "web",
        title: "Web Generation",
        tagline: "The same binary that runs governance also ships your marketing site.",
        story: "Most governance platforms stop at the API. Enterprise Demo also publishes systemprompt.io from the same Rust binary, using the same CLI, against the same database. This walkthrough inventories the content model, then runs the validator so you can see the publishing pipeline is real, typed, and CI-friendly.",
        cost: "",
        feature_url: "https://systemprompt.io/features/web-publisher",
    },
    CategoryMeta {
        id: "performance",
        title: "Performance",
        tagline: "Trace one request end-to-end, then prove it holds under 100 concurrent workers.",
        story: "Staff engineers don't trust benchmarks they can't reproduce. This walkthrough fires a real governance request, follows it through JWT validation, scope resolution, rule evaluation, and the async audit write — showing the typed Rust structs and sqlx-checked SQL at every stage. Then it runs 2,000 requests across three load profiles and prints real throughput, p50/p90/p99, and a capacity estimate in concurrent developers.",
        cost: "",
        feature_url: "",
    },
];
