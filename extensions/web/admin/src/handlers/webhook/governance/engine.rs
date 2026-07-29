//! The process-wide, hot-reloadable [`GovernanceEngine`] instance.
//!
//! Core's engine is caller-owned; this module supplies the template's three
//! deployment decisions — where the config lives
//! (`<services>/governance/config.yaml` per the profile), that every
//! enforcement point shares one engine so the rate limiter sees every call,
//! and [`reload`] so the policy editor's save handler rebuilds the chain
//! without a restart.

use std::path::PathBuf;
use std::sync::{LazyLock, RwLock, RwLockReadGuard};

use systemprompt::config::ProfileBootstrap;
use systemprompt_security::policy::{GovernanceConfig, GovernanceEngine};

static ENGINE: LazyLock<RwLock<GovernanceEngine>> = LazyLock::new(|| RwLock::new(build()));

pub(crate) fn engine() -> RwLockReadGuard<'static, GovernanceEngine> {
    ENGINE
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

pub(crate) fn reload() {
    let rebuilt = build();
    *ENGINE
        .write()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = rebuilt;
}

// Why: the dashboard's "as code" link. The registry carries no source
// location, and the four builtins live in core — so the map is declared here,
// where the display concern is.
pub(crate) fn source_path_for(id: &str) -> String {
    match id {
        "secret_scan" | "scope_check" | "tool_blocklist" | "rate_limit" => {
            format!("systemprompt-security/src/policy/builtin/{id}.rs")
        },
        _ => String::new(),
    }
}

fn build() -> GovernanceEngine {
    let config =
        config_path().map_or_else(GovernanceConfig::defaults, |p| GovernanceConfig::load(&p));
    GovernanceEngine::from_config(&config)
}

fn config_path() -> Option<PathBuf> {
    let bootstrap = ProfileBootstrap::get()
        .inspect_err(|e| {
            tracing::error!(
                error = %e,
                "governance profile bootstrap failed; policies fall back to built-in defaults"
            );
        })
        .ok()?;
    Some(PathBuf::from(&bootstrap.paths.services).join("governance/config.yaml"))
}
