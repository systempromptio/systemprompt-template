//! Modular policy framework.
//!
//! Each policy is a Rust struct implementing [`Policy`], registered at
//! compile-time via [`inventory::submit!`]. The pipeline driver iterates the
//! enabled, ordered list from `services/governance/config.yaml`, calling each
//! [`Policy::evaluate`] and stopping on the first deny. Adding a new policy is
//! a single new file plus one `inventory::submit!` line — no edits to the
//! pipeline driver, no recompile of the rest of the workspace.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

use serde_yaml::Value as YamlValue;
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::{SessionId, UserId};

/// Factory function that turns a YAML config block into a boxed [`Policy`].
type PolicyFactory = fn(&YamlValue) -> Box<dyn Policy>;

/// Inputs every policy receives. Borrowed; cheap to construct per request.
pub struct PolicyContext<'a> {
    pub tool_name: &'a str,
    pub agent_scope: &'a str,
    pub session_id: &'a SessionId,
    pub user_id: &'a UserId,
    pub tool_input: Option<&'a serde_json::Value>,
}

/// Outcome of one policy's evaluation.
pub enum PolicyOutcome {
    Allow {
        detail: Cow<'static, str>,
    },
    Deny {
        reason: String,
        detail: Cow<'static, str>,
    },
}

/// Contract every governance policy implements.
pub trait Policy: Send + Sync {
    /// Stable identifier (lowercase `snake_case`). Stored in `governance_decisions.policy`.
    fn id(&self) -> &'static str;
    /// Human-readable label for the dashboard.
    fn name(&self) -> &'static str;
    /// One-sentence description shown on the dashboard.
    fn description(&self) -> &'static str;
    /// Per-call evaluation. Pure for everything except `rate_limit` (in-memory window).
    fn evaluate(&self, ctx: &PolicyContext<'_>) -> PolicyOutcome;
}

/// Compile-time registration. Each built-in lives in its own file and submits
/// one of these. Adding a third-party policy means creating a new submodule
/// and one `inventory::submit!` block.
pub struct PolicyRegistration {
    pub id: &'static str,
    pub factory: PolicyFactory,
}

inventory::collect!(PolicyRegistration);

/// Per-policy config block from `services/governance/config.yaml`.
#[derive(Debug, Clone)]
pub struct PolicyConfig {
    pub id: String,
    pub enabled: bool,
    pub params: YamlValue,
}

/// Resolved, ordered, instantiated policy chain.
pub struct PolicyChain {
    entries: Vec<ChainEntry>,
}

struct ChainEntry {
    config: PolicyConfig,
    instance: Box<dyn Policy>,
}

impl PolicyChain {
    pub fn iter(&self) -> impl Iterator<Item = (&PolicyConfig, &dyn Policy)> {
        self.entries
            .iter()
            .map(|e| (&e.config, e.instance.as_ref()))
    }

}

/// Process-wide, hot-reloadable policy chain.
static CHAIN: LazyLock<RwLock<PolicyChain>> = LazyLock::new(|| RwLock::new(load_chain()));

pub fn chain() -> std::sync::RwLockReadGuard<'static, PolicyChain> {
    CHAIN.read().unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Re-read `services/governance/config.yaml` and rebuild the chain.
/// Called from the policy editor save handler so config edits take effect
/// without restarting the server.
pub fn reload() {
    let new_chain = load_chain();
    if let Ok(mut guard) = CHAIN.write() {
        *guard = new_chain;
    }
}

fn load_chain() -> PolicyChain {
    let configs = load_configs();
    let factories: HashMap<&'static str, PolicyFactory> = inventory::iter::<PolicyRegistration>()
        .map(|r| (r.id, r.factory))
        .collect();

    let mut entries = Vec::with_capacity(configs.len());
    for cfg in configs {
        let Some(factory) = factories.get(cfg.id.as_str()) else {
            tracing::warn!(
                policy = %cfg.id,
                "governance policy in config.yaml has no registered Rust impl — skipping"
            );
            continue;
        };
        let instance = factory(&cfg.params);
        entries.push(ChainEntry {
            config: cfg,
            instance,
        });
    }

    // Any registered policy that the YAML didn't mention — instantiate with
    // empty params and disabled, so the dashboard can still show it.
    let mentioned: std::collections::HashSet<String> =
        entries.iter().map(|e| e.config.id.clone()).collect();
    for r in inventory::iter::<PolicyRegistration>() {
        if mentioned.contains(r.id) {
            continue;
        }
        let cfg = PolicyConfig {
            id: r.id.to_string(),
            enabled: false,
            params: YamlValue::Null,
        };
        let instance = (r.factory)(&cfg.params);
        entries.push(ChainEntry {
            config: cfg,
            instance,
        });
    }

    PolicyChain { entries }
}

fn config_path() -> Option<PathBuf> {
    let bootstrap = ProfileBootstrap::get().ok()?;
    Some(PathBuf::from(&bootstrap.paths.services).join("governance/config.yaml"))
}

fn load_configs() -> Vec<PolicyConfig> {
    let Some(path) = config_path() else {
        tracing::warn!("ProfileBootstrap unavailable; governance running with defaults");
        return default_configs();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        tracing::info!(
            path = %path.display(),
            "governance config.yaml not found; using built-in defaults"
        );
        return default_configs();
    };
    let Ok(root) = serde_yaml::from_str::<YamlValue>(&text) else {
        tracing::warn!(
            path = %path.display(),
            "governance config.yaml is not valid YAML; using built-in defaults"
        );
        return default_configs();
    };
    parse_policies(&root).unwrap_or_else(default_configs)
}

fn parse_policies(root: &YamlValue) -> Option<Vec<PolicyConfig>> {
    let policies = root
        .get("governance")
        .and_then(|g| g.get("policies"))
        .and_then(|p| p.as_sequence())?;

    let mut out = Vec::with_capacity(policies.len());
    for entry in policies {
        let id = entry.get("id").and_then(|v| v.as_str())?.to_string();
        let enabled = entry
            .get("enabled")
            .and_then(YamlValue::as_bool)
            .unwrap_or(true);
        out.push(PolicyConfig {
            id,
            enabled,
            params: entry.clone(),
        });
    }
    Some(out)
}

/// Defaults match the historical pre-modular pipeline order/behaviour so the
/// system stays safe even with no YAML on disk.
fn default_configs() -> Vec<PolicyConfig> {
    [
        ("secret_scan", YamlValue::Null),
        ("scope_check", YamlValue::Null),
        ("tool_blocklist", YamlValue::Null),
        ("rate_limit", YamlValue::Null),
    ]
    .into_iter()
    .map(|(id, params)| PolicyConfig {
        id: id.to_string(),
        enabled: true,
        params,
    })
    .collect()
}

