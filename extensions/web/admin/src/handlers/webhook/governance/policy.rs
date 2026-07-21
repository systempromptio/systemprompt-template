//! Modular governance policy framework — template side.
//!
//! Policies implement [`systemprompt_security::policy::GovernancePolicy`]
//! from core and return the shared [`Decision`] / [`DenyReason`] types, so
//! the audit row shape and CLI view are identical to the user→entity authz
//! plane. The template owns three concerns that core's plain
//! [`systemprompt_security::policy::GovernanceChain`] doesn't:
//!
//! 1. **Compile-time registration** via the `inventory` crate so adding a
//!    policy is one new file + one `inventory::submit!`.
//! 2. **Per-policy YAML config** from `services/governance/config.yaml`
//!    (enabled flag, per-policy params).
//! 3. **Hot reload** so the policy editor save handler can rebuild the chain
//!    without restarting the server.
//!
//! Each entry carries the resolved [`PolicyConfig`] alongside the boxed
//! `GovernancePolicy` impl. The pipeline driver in [`super::rules_runner`]
//! iterates entries, honours `enabled`, and emits a per-entry trace for the
//! audit row.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};

use serde_yaml::Value as YamlValue;
use systemprompt::config::ProfileBootstrap;

pub(crate) use systemprompt_security::policy::GovernancePolicy;

type PolicyFactory = fn(&YamlValue) -> Box<dyn GovernancePolicy>;

/// Compile-time registration. Each built-in lives in its own file and submits
/// one of these.
pub(crate) struct PolicyRegistration {
    pub id: &'static str,
    pub factory: PolicyFactory,
    /// Source file the policy is defined in (set with `file!()`). Surfaced on
    /// the dashboard as the "as code" link.
    pub source_path: &'static str,
}

inventory::collect!(PolicyRegistration);

pub(crate) fn source_path_for(id: &str) -> &'static str {
    inventory::iter::<PolicyRegistration>()
        .find(|r| r.id == id)
        .map_or("", |r| r.source_path)
}

#[derive(Debug, Clone)]
pub(crate) struct PolicyConfig {
    pub id: String,
    pub enabled: bool,
    pub params: YamlValue,
}

pub(crate) struct PolicyChain {
    entries: Vec<ChainEntry>,
}

struct ChainEntry {
    config: PolicyConfig,
    instance: Box<dyn GovernancePolicy>,
}

impl PolicyChain {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&PolicyConfig, &dyn GovernancePolicy)> {
        self.entries
            .iter()
            .map(|e| (&e.config, e.instance.as_ref()))
    }
}

/// Process-wide, hot-reloadable policy chain.
static CHAIN: LazyLock<RwLock<PolicyChain>> = LazyLock::new(|| RwLock::new(load_chain()));

pub(crate) fn chain() -> std::sync::RwLockReadGuard<'static, PolicyChain> {
    CHAIN
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn reload() {
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

    let mentioned: std::collections::HashSet<String> =
        entries.iter().map(|e| e.config.id.clone()).collect();
    for r in inventory::iter::<PolicyRegistration>() {
        if mentioned.contains(r.id) {
            continue;
        }
        let cfg = PolicyConfig {
            id: r.id.to_owned(),
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
        let id = entry.get("id").and_then(|v| v.as_str())?.to_owned();
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

fn default_configs() -> Vec<PolicyConfig> {
    [
        ("secret_scan", YamlValue::Null),
        ("scope_check", YamlValue::Null),
        ("tool_blocklist", YamlValue::Null),
        ("rate_limit", YamlValue::Null),
    ]
    .into_iter()
    .map(|(id, params)| PolicyConfig {
        id: id.to_owned(),
        enabled: true,
        params,
    })
    .collect()
}
