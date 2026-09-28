//! Every kind of configuration the instance loads from `services/`, as one
//! static list.
//!
//! The configuration page shows what this instance is running, and "what"
//! has to be enumerated somewhere: core discovers most of these by path
//! rather than by a registry, so nothing in the running process can list
//! them. This is that list. A kind is either *projected* — a sync plane
//! owns it and the database is what is enforced — or *served from code*:
//! the composed tree is read at boot or per request and the database never
//! holds a copy. The distinction is the honest one and the page prints it.
//!
//! An import archive is classified against the same list, so an entry the
//! instance cannot project is named for what it is rather than ignored.

/// One kind of configuration under `services/`.
#[derive(Debug, Clone, Copy)]
pub struct ConfigKind {
    pub id: &'static str,
    pub label: &'static str,
    pub purpose: &'static str,
    // Why: `path` is relative to `services/` and names a directory when
    // `is_dir`; `plane` is set only for a projected kind; `page_url` only
    // where the console has a page for the kind.
    pub path: &'static str,
    pub is_dir: bool,
    pub plane: Option<&'static str>,
    pub page_url: Option<&'static str>,
}

const fn file(
    id: &'static str,
    label: &'static str,
    purpose: &'static str,
    path: &'static str,
) -> ConfigKind {
    ConfigKind {
        id,
        label,
        purpose,
        path,
        is_dir: false,
        plane: None,
        page_url: None,
    }
}

const fn dir(
    id: &'static str,
    label: &'static str,
    purpose: &'static str,
    path: &'static str,
) -> ConfigKind {
    ConfigKind {
        id,
        label,
        purpose,
        path,
        is_dir: true,
        plane: None,
        page_url: None,
    }
}

const fn paged(kind: ConfigKind, page_url: &'static str) -> ConfigKind {
    ConfigKind {
        page_url: Some(page_url),
        ..kind
    }
}

const fn projected(kind: ConfigKind, plane: &'static str, page_url: &'static str) -> ConfigKind {
    ConfigKind {
        plane: Some(plane),
        ..paged(kind, page_url)
    }
}

// Why: projected kinds first, in plane order; then everything served from
// code, grouped the way the tree is — the page prints them in this order.
pub const CONFIG_KINDS: &[ConfigKind] = &[
    projected(
        file(
            "access_control",
            "Access control",
            "who reaches which marketplace, plugin, skill, server and route",
            "access-control/rules.yaml",
        ),
        "access_control",
        "/admin/access-control",
    ),
    projected(
        file(
            "groups",
            "Groups and projects",
            "the containers people land in and the directory groups that place them",
            "web/config/groups.yaml",
        ),
        "groups",
        "/admin/groups",
    ),
    projected(
        file(
            "gateway_policies",
            "Gateway policies",
            "quota windows, safety scanners and block lists on every inference request",
            "gateway/policies.yaml",
        ),
        "gateway_policies",
        "/admin/gateway/policies",
    ),
    projected(
        file(
            "gateway_routes",
            "Gateway routes",
            "which model names dispatch to which provider",
            "ai/gateway.yaml",
        ),
        "gateway_routes",
        "/admin/gateway",
    ),
    projected(
        file(
            "governance",
            "Governance chain",
            "the four request-time stages every tool call passes through",
            "governance/config.yaml",
        ),
        "governance",
        "/admin/governance",
    ),
    file(
        "config",
        "Root aggregator",
        "the includes list and instance settings every flat file hangs off",
        "config/config.yaml",
    ),
    paged(
        dir(
            "marketplaces",
            "Marketplaces",
            "what a signed-in user is offered; a content hash is its version",
            "marketplaces",
        ),
        "/admin/marketplaces",
    ),
    paged(
        dir(
            "plugins",
            "Plugins",
            "binding descriptors that group skills, servers and hooks by reference",
            "plugins",
        ),
        "/admin/plugins",
    ),
    paged(
        dir(
            "skills",
            "Skills",
            "the instruction bodies the bridge ships to Claude Code",
            "skills",
        ),
        "/admin/skills",
    ),
    paged(
        dir(
            "mcp",
            "MCP servers",
            "server definitions the instance supervises and proxies",
            "mcp",
        ),
        "/admin/mcp",
    ),
    dir(
        "agents",
        "Agents",
        "the agent definitions plugins bind by reference",
        "agents",
    ),
    dir(
        "hooks",
        "Hooks",
        "hook definitions plugins bind by reference",
        "hooks",
    ),
    dir(
        "slack",
        "Slack apps",
        "inbound Slack apps and the roles each one admits",
        "slack",
    ),
    dir(
        "artifacts",
        "Artifacts",
        "dashboards rendered from a skill's output",
        "artifacts",
    ),
    file(
        "providers",
        "Provider catalog",
        "every provider and model the gateway can reach, with pricing",
        "ai/providers.yaml",
    ),
    file(
        "ai_config",
        "AI defaults",
        "the default provider and the inference settings behind it",
        "ai/config.yaml",
    ),
    file(
        "scheduler",
        "Scheduler",
        "the jobs the instance runs on a timer",
        "scheduler/config.yaml",
    ),
    dir(
        "external_agents",
        "External agents",
        "the clients the bridge recognises",
        "external_agents",
    ),
    dir(
        "content",
        "Content",
        "documentation and other sources the publish pipeline ingests",
        "content",
    ),
    file(
        "blog",
        "Blog ingestion",
        "where posts come from and how they are published",
        "config/blog.yaml",
    ),
    file(
        "web_config",
        "Web config",
        "the public site's settings",
        "web/config.yaml",
    ),
    file(
        "web_metadata",
        "Site metadata",
        "titles, descriptions and social cards",
        "web/metadata.yaml",
    ),
    file(
        "homepage",
        "Homepage",
        "the public landing page's sections",
        "web/config/homepage.yaml",
    ),
    file(
        "navigation",
        "Navigation",
        "the public site's menus",
        "web/config/navigation.yaml",
    ),
    file(
        "theme",
        "Theme",
        "brand colours and type",
        "web/config/theme.yaml",
    ),
    dir(
        "web_templates",
        "Web templates",
        "the public site's page shells",
        "web/templates",
    ),
];

#[must_use]
pub fn kind_by_id(id: &str) -> Option<&'static ConfigKind> {
    CONFIG_KINDS.iter().find(|k| k.id == id)
}

// Why: an exact file first, then the nearest directory that contains the
// path, so `web/config/groups.yaml` is the groups plane and not "web".
#[must_use]
pub fn kind_for_path(rel: &str) -> Option<&'static ConfigKind> {
    let rel = rel.trim_start_matches("services/").trim_start_matches('/');
    if let Some(k) = CONFIG_KINDS.iter().find(|k| !k.is_dir && k.path == rel) {
        return Some(k);
    }
    CONFIG_KINDS
        .iter()
        .filter(|k| k.is_dir)
        .filter(|k| rel.starts_with(k.path) && rel[k.path.len()..].starts_with('/'))
        .max_by_key(|k| k.path.len())
}
