use std::collections::HashMap;
use std::path::PathBuf;
use std::str::FromStr;

use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::models::auth::Permission;
use systemprompt_security::policy::types::AccessScope;

pub(super) fn resolve_agent_scope(agent_id: &str) -> AccessScope {
    let map = load_all_agent_scopes();
    map.get(agent_id).copied().unwrap_or(AccessScope::Unknown)
}

pub(super) async fn scope_from_user_roles(pool: &PgPool, user_id: &str) -> AccessScope {
    match crate::repositories::get_user_roles_department(pool, user_id).await {
        Ok(Some((roles, _dept))) => {
            if roles.iter().any(|r| r == "admin") {
                AccessScope::Admin
            } else if roles.iter().any(|r| r == "user") {
                AccessScope::User
            } else {
                AccessScope::Unknown
            }
        },
        Ok(None) => AccessScope::Unknown,
        Err(e) => {
            tracing::warn!(
                error = %e,
                %user_id,
                "governance: user role lookup failed; no DB-derived scope"
            );
            AccessScope::Unknown
        },
    }
}

pub(super) fn scope_from_permissions(perms: &[Permission]) -> AccessScope {
    if perms.contains(&Permission::Admin) {
        AccessScope::Admin
    } else if perms.contains(&Permission::User) {
        AccessScope::User
    } else {
        AccessScope::Unknown
    }
}

pub(super) const fn higher_privilege(a: AccessScope, b: AccessScope) -> AccessScope {
    match (a, b) {
        (AccessScope::Admin, _) | (_, AccessScope::Admin) => AccessScope::Admin,
        (AccessScope::User, _) | (_, AccessScope::User) => AccessScope::User,
        _ => AccessScope::Unknown,
    }
}

fn load_all_agent_scopes() -> HashMap<String, AccessScope> {
    let mut scopes = HashMap::new();

    let Ok(services_path) = ProfileBootstrap::get().map(|p| PathBuf::from(&p.paths.services))
    else {
        return scopes;
    };

    let agents_dir = services_path.join("agents");
    let Ok(entries) = std::fs::read_dir(&agents_dir) else {
        return scopes;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str());
        if ext != Some("yaml") && ext != Some("yml") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&path) else {
            continue;
        };
        let Ok(config) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        extract_scopes_from_config(&config, &mut scopes);
    }

    scopes
}

fn extract_scopes_from_config(
    config: &serde_yaml::Value,
    scopes: &mut HashMap<String, AccessScope>,
) {
    let Some(agents_map) = config.get("agents").and_then(|a| a.as_mapping()) else {
        return;
    };

    for (key, agent_val) in agents_map {
        let Some(agent_id) = key.as_str() else {
            continue;
        };

        if let Some(scope) = extract_scope_for_agent(agent_val) {
            scopes.insert(agent_id.to_owned(), scope);
        }
    }
}

fn extract_scope_for_agent(agent_val: &serde_yaml::Value) -> Option<AccessScope> {
    if let Some(scope) = agent_val
        .get("oauth")
        .and_then(|o| o.get("scopes"))
        .and_then(|s| s.as_sequence())
        .and_then(|seq| seq.first())
        .and_then(|s| s.as_str())
    {
        return Some(parse_scope(scope));
    }

    let security = agent_val
        .get("card")
        .and_then(|c| c.get("security"))
        .and_then(|s| s.as_sequence())?;

    for sec in security {
        if let Some(scope) = sec
            .get("oauth2")
            .and_then(|o| o.as_sequence())
            .and_then(|seq| seq.first())
            .and_then(|s| s.as_str())
        {
            return Some(parse_scope(scope));
        }
    }

    None
}

fn parse_scope(s: &str) -> AccessScope {
    // Why: YAML values outside {admin, user, unknown} are operator typos.
    // Surface them as a warning and fall through to Unknown rather than
    // silently masquerading as Admin.
    AccessScope::from_str(s).unwrap_or_else(|err| {
        tracing::warn!(
            value = %s,
            error = %err,
            "unrecognised oauth scope in agent YAML; treating as unknown"
        );
        AccessScope::Unknown
    })
}
