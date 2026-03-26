use std::path::PathBuf;

use systemprompt::models::ProfileBootstrap;

pub(super) fn resolve_agent_scope(agent_id: &str) -> String {
    let services_path = ProfileBootstrap::get()
        .map(|p| PathBuf::from(&p.paths.services))
        .ok();

    let Some(services_path) = services_path else {
        return "unknown".to_string();
    };

    let agents_dir = services_path.join("agents");
    if !agents_dir.exists() {
        return "unknown".to_string();
    }

    let Ok(entries) = std::fs::read_dir(&agents_dir) else {
        return "unknown".to_string();
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
        if let Some(scope) = extract_scope_from_config(&config, agent_id) {
            return scope;
        }
    }

    "unknown".to_string()
}

fn extract_scope_from_config(config: &serde_yaml::Value, agent_id: &str) -> Option<String> {
    let agents_map = config.get("agents")?.as_mapping()?;
    let agent_val = agents_map.get(&serde_yaml::Value::String(agent_id.to_string()))?;

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
