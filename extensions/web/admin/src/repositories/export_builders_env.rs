use super::export::PluginFile;
use super::export_builders::PluginBuildContext;

pub(super) fn build_env_files(ctx: &PluginBuildContext<'_>, files: &mut Vec<PluginFile>) -> bool {
    let mut missing = Vec::new();
    for var_def in &ctx.plugin.variables {
        if var_def.required && !var_def.secret {
            let val = ctx.env_vars.get(&var_def.name).map_or("", String::as_str);
            if val.is_empty() {
                missing.push(var_def.name.clone());
            }
        }
    }
    if !missing.is_empty() {
        tracing::warn!(
            plugin_id = ctx.plugin_id,
            missing = ?missing,
            "Plugin export: missing required environment variables, skipping .env.plugin"
        );
        return false;
    }

    let has_secrets = ctx.plugin.variables.iter().any(|v| v.secret);

    let mut env_lines = Vec::new();
    env_lines.push("# Plugin environment variables".to_string());
    env_lines.push(format!("# Generated for plugin: {}", ctx.plugin_id));
    for var_def in &ctx.plugin.variables {
        let is_secret = var_def.secret;
        if is_secret {
            continue;
        }
        let val = ctx.env_vars.get(&var_def.name).map_or("", String::as_str);
        if !var_def.description.is_empty() {
            env_lines.push(format!("# {}", var_def.description));
        }
        env_lines.push(format!("{}={val}", var_def.name));
    }
    env_lines.push(String::new());
    files.push(PluginFile {
        path: ".env.plugin".to_string(),
        content: env_lines.join("\n"),
        executable: false,
    });

    let claude_md = if has_secrets {
        format!(
            "# {} Plugin\n\n\
             ## Environment Setup\n\n\
             This plugin requires environment variables to be loaded before running scripts.\n\n\
             **Non-secret** variables are stored in `.env.plugin` and loaded automatically \
             by the SessionStart hook.\n\n\
             **Secret** variables (API keys, credentials) are managed via the MCP server. \
             Use the `get_secrets` or `manage_secrets` tools to access them securely.\n",
            ctx.plugin.base.name
        )
    } else {
        format!(
            "# {} Plugin\n\n\
             ## Environment Setup\n\n\
             This plugin requires environment variables to be loaded before running scripts.\n\n\
             **Important:** The `.env.plugin` file contains your credentials. \
             They are loaded automatically by the SessionStart hook via `$CLAUDE_ENV_FILE`.\n",
            ctx.plugin.base.name
        )
    };
    files.push(PluginFile {
        path: "CLAUDE.md".to_string(),
        content: claude_md,
        executable: false,
    });

    true
}
