pub(super) fn read_agent_from_fs(
    agents_path: &std::path::Path,
    agent_id: &str,
) -> (String, String, String) {
    if !agents_path.exists() {
        return (String::new(), String::new(), String::new());
    }

    let md_path = agents_path.join(format!("{agent_id}.md"));
    let system_prompt = if md_path.exists() {
        std::fs::read_to_string(&md_path).unwrap_or_default()
    } else {
        String::new()
    };

    let Ok(entries) = std::fs::read_dir(agents_path) else {
        return (agent_id.to_string(), String::new(), system_prompt);
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
        if let Some(agent) = config.get("agents").and_then(|a| a.get(agent_id)) {
            let name = agent
                .get("card")
                .and_then(|c| c.get("name"))
                .and_then(|v| v.as_str())
                .unwrap_or(agent_id)
                .to_string();
            let description = agent
                .get("card")
                .and_then(|c| c.get("description"))
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            return (name, description, system_prompt);
        }
    }

    (agent_id.to_string(), String::new(), system_prompt)
}

pub(super) struct McpServerData {
    pub name: String,
    pub description: String,
    pub binary: String,
    pub package_name: String,
    pub port: i32,
    pub endpoint: String,
    pub oauth_required: bool,
    pub oauth_scopes: Vec<String>,
    pub oauth_audience: String,
}

#[allow(clippy::too_many_lines)]
pub(super) fn read_mcp_server_from_fs(
    services_path: &std::path::Path,
    mcp_server_id: &str,
) -> McpServerData {
    let mcp_config_path = services_path.join("mcp-servers").join(mcp_server_id);
    if mcp_config_path.exists() {
        let config_yaml = mcp_config_path.join("config.yaml");
        if config_yaml.exists() {
            if let Ok(content) = std::fs::read_to_string(&config_yaml) {
                if let Ok(cfg) = serde_yaml::from_str::<serde_yaml::Value>(&content) {
                    return McpServerData {
                        name: cfg
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(mcp_server_id)
                            .to_string(),
                        description: cfg
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        binary: cfg
                            .get("binary")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        package_name: cfg
                            .get("package_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        #[allow(clippy::cast_possible_truncation)]
                        port: cfg
                            .get("port")
                            .and_then(serde_yaml::Value::as_i64)
                            .unwrap_or(0) as i32,
                        endpoint: cfg
                            .get("endpoint")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        oauth_required: cfg
                            .get("oauth_required")
                            .and_then(serde_yaml::Value::as_bool)
                            .unwrap_or(false),
                        oauth_scopes: cfg
                            .get("oauth_scopes")
                            .and_then(|v| v.as_sequence())
                            .map(|seq| {
                                seq.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect()
                            })
                            .unwrap_or_default(),
                        oauth_audience: cfg
                            .get("oauth_audience")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };
                }
            }
        }
    }

    McpServerData {
        name: mcp_server_id.to_string(),
        description: String::new(),
        binary: String::new(),
        package_name: String::new(),
        port: 0,
        endpoint: String::new(),
        oauth_required: false,
        oauth_scopes: vec![],
        oauth_audience: String::new(),
    }
}

pub(super) struct HookData {
    pub name: String,
    pub description: String,
    pub event: String,
    pub matcher: String,
    pub command: String,
    pub is_async: bool,
}

pub(super) fn read_hook_from_fs(
    services_path: &std::path::Path,
    hook_id: &str,
) -> Option<HookData> {
    let plugins_path = services_path.join("plugins");
    if !plugins_path.exists() {
        return None;
    }

    let Ok(entries) = std::fs::read_dir(&plugins_path) else {
        return None;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let config_path = path.join("config.yaml");
        if !config_path.exists() {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(&config_path) else {
            continue;
        };
        let Ok(cfg) = serde_yaml::from_str::<serde_yaml::Value>(&content) else {
            continue;
        };
        if let Some(hooks) = cfg
            .get("plugin")
            .and_then(|p| p.get("hooks"))
            .and_then(|h| h.as_sequence())
        {
            for hook in hooks {
                let id = hook.get("id").and_then(|v| v.as_str()).unwrap_or("");
                if id == hook_id {
                    return Some(HookData {
                        name: hook
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or(hook_id)
                            .to_string(),
                        description: hook
                            .get("description")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        event: hook
                            .get("event")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        matcher: hook
                            .get("matcher")
                            .and_then(|v| v.as_str())
                            .unwrap_or(".*")
                            .to_string(),
                        command: hook
                            .get("command")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        is_async: hook
                            .get("is_async")
                            .and_then(serde_yaml::Value::as_bool)
                            .unwrap_or(false),
                    });
                }
            }
        }
    }

    None
}
