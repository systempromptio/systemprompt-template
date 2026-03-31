use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use systemprompt::models::ProfileBootstrap;

static SCOPE_CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();

pub(super) fn resolve_agent_scope(agent_id: &str) -> String {
    let map = SCOPE_CACHE.get_or_init(load_all_agent_scopes);
    map.get(agent_id)
        .cloned()
        .unwrap_or_else(|| "unknown".to_string())
}

fn load_all_agent_scopes() -> HashMap<String, String> {
    let mut scopes = HashMap::new();

    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .ok();

    let Some(services_path) = services_path else {
        return scopes;
    };

    let agents_dir = services_path.join("agents");
    if !agents_dir.exists() {
        return scopes;
    }

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

fn extract_scopes_from_config(config: &serde_yaml::Value, scopes: &mut HashMap<String, String>) {
    let Some(agents_map) = config.get("agents").and_then(|a| a.as_mapping()) else {
        return;
    };

    for (key, agent_val) in agents_map {
        let Some(agent_id) = key.as_str() else {
            continue;
        };

        let scope = extract_scope_for_agent(agent_val);
        if let Some(s) = scope {
            scopes.insert(agent_id.to_string(), s);
        }
    }
}

fn extract_scope_for_agent(agent_val: &serde_yaml::Value) -> Option<String> {
    if let Some(scope) = agent_val
        .get("oauth")
        .and_then(|o| o.get("scopes"))
        .and_then(|s| s.as_sequence())
        .and_then(|seq| seq.first())
        .and_then(|s| s.as_str())
    {
        return Some(scope.to_string());
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
            return Some(scope.to_string());
        }
    }

    None
}
