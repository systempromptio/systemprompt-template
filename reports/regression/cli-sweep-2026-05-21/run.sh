#!/usr/bin/env bash
# CLI regression sweep driver. Captures exit/stdout/stderr per command.
set -u
SP="/var/www/html/systemprompt-template/target/debug/systemprompt"
ROOT="$(cd "$(dirname "$0")" && pwd)"
LOGS="$ROOT/logs"
INDEX="$ROOT/index.tsv"
: > "$INDEX"
N=0

run() {
  local label="$1"; shift
  N=$((N+1))
  local slug
  slug=$(printf "%03d-%s" "$N" "$(echo "$label" | tr ' /:' '___' | tr -cd 'A-Za-z0-9_.-')")
  local log="$LOGS/$slug.log"
  {
    printf '== cmd: %s\n' "$*"
    timeout 90 "$@" 2>&1
    local ec=$?
    printf '\n== exit: %s\n' "$ec"
    printf "%d\t%s\t%s\t%s\n" "$N" "$ec" "$label" "$*" >> "$INDEX"
  } > "$log" 2>&1
  printf '[%03d] exit=%s %s\n' "$N" "$(awk -F$'\t' -v n="$N" '$1==n{print $2}' "$INDEX")" "$label"
}

# --- Smoke: top-level + domain help ---
run "help_root" $SP --help
run "version" $SP --version
for d in core infra admin cloud analytics web plugins build; do
  run "help_${d}" $SP "$d" --help
done

# --- CORE domain ---
for sub in skills content files contexts plugins hooks artifacts; do
  run "help_core_${sub}" $SP core "$sub" --help
done
run "core_skills_list"    $SP core skills list
run "core_skills_list_json" $SP core skills list --json
run "core_content_list"   $SP core content list
run "core_files_list"     $SP core files list
run "core_contexts_list"  $SP core contexts list
run "core_plugins_list"   $SP core plugins list
run "core_plugins_validate" $SP core plugins validate
run "core_hooks_list"     $SP core hooks list
run "core_hooks_validate" $SP core hooks validate

# --- INFRA domain ---
for sub in services db jobs logs; do
  run "help_infra_${sub}" $SP infra "$sub" --help
done
run "infra_services_status" $SP infra services status
run "infra_jobs_list"       $SP infra jobs list
run "infra_logs_help"       $SP infra logs --help
run "infra_logs_request_list" $SP infra logs request list --limit 5
run "infra_logs_trace_list"   $SP infra logs trace list --limit 5
run "infra_db_help"           $SP infra db --help
run "infra_db_migrations_list" $SP infra db migrations list

# --- ADMIN domain ---
for sub in users agents config session bridge access-control keys setup bootstrap; do
  run "help_admin_${sub}" $SP admin "$sub" --help
done
run "admin_session_current" $SP admin session current
run "admin_session_list"    $SP admin session list
run "admin_users_list"      $SP admin users list --limit 5
run "admin_users_count"     $SP admin users count
run "admin_users_stats"     $SP admin users stats
run "admin_agents_list"     $SP admin agents list
run "admin_agents_registry" $SP admin agents registry
run "admin_agents_validate" $SP admin agents validate
run "admin_agents_status_dev" $SP admin agents status developer_agent
run "admin_agents_status_assoc" $SP admin agents status associate_agent
run "admin_agents_tools_dev"   $SP admin agents tools --agent developer_agent
run "admin_agents_tools_assoc" $SP admin agents tools --agent associate_agent
run "admin_agents_logs_dev"    $SP admin agents logs developer_agent --tail 20
run "admin_config_paths_list"  $SP admin config paths list
run "admin_config_runtime_list" $SP admin config runtime list
run "admin_config_server_list"  $SP admin config server list
run "admin_config_security_list" $SP admin config security list
run "admin_config_provider_list" $SP admin config provider list
run "admin_config_ratelimits_list" $SP admin config rate-limits list
run "admin_keys_list"  $SP admin keys list
run "admin_bridge_list" $SP admin bridge list
run "admin_access_control_export" $SP admin access-control export

# --- CLOUD (read-only) ---
for sub in auth init tenant profile deploy status restart sync secrets dockerfile db domain; do
  run "help_cloud_${sub}" $SP cloud "$sub" --help
done
run "cloud_auth_whoami" $SP cloud auth whoami
run "cloud_auth_status" $SP cloud auth status
run "cloud_status"      $SP cloud status
run "cloud_profile_list" $SP cloud profile list
run "cloud_tenant_list"  $SP cloud tenant list
run "cloud_sync_status"  $SP cloud sync status
run "cloud_secrets_list" $SP cloud secrets list
run "cloud_domain_list"  $SP cloud domain list

# --- ANALYTICS ---
for sub in overview conversations agents tools requests sessions content traffic costs; do
  run "help_analytics_${sub}" $SP analytics "$sub" --help
done
run "analytics_overview" $SP analytics overview
run "analytics_conversations_list" $SP analytics conversations list --limit 5
run "analytics_agents_list" $SP analytics agents list
run "analytics_tools_list"  $SP analytics tools list
run "analytics_requests_list" $SP analytics requests list --limit 5
run "analytics_sessions_list" $SP analytics sessions list --limit 5
run "analytics_content_list" $SP analytics content list
run "analytics_traffic_list" $SP analytics traffic list
run "analytics_costs_list"   $SP analytics costs list

# --- WEB ---
for sub in content-types templates assets sitemap validate; do
  run "help_web_${sub}" $SP web "$sub" --help
done
run "web_content_types_list" $SP web content-types list
run "web_templates_list"     $SP web templates list
run "web_assets_list"        $SP web assets list
run "web_sitemap_list"       $SP web sitemap list
run "web_validate"           $SP web validate

# --- PLUGINS / MCP ---
run "plugins_list"      $SP plugins list
run "plugins_show"      $SP plugins show systemprompt
run "plugins_validate"  $SP plugins validate
run "plugins_config"    $SP plugins config
run "plugins_capabilities" $SP plugins capabilities
run "plugins_mcp_help"  $SP plugins mcp --help
run "plugins_mcp_list"  $SP plugins mcp list
run "plugins_mcp_status" $SP plugins mcp status
run "plugins_mcp_validate" $SP plugins mcp validate
run "plugins_mcp_tools"  $SP plugins mcp tools
run "plugins_mcp_list_packages" $SP plugins mcp list-packages
run "plugins_mcp_logs_sp" $SP plugins mcp logs systemprompt --tail 20

# --- MCP CALL deep test: every domain through the MCP tool ---
run "mcp_call_core_skills_list" $SP plugins mcp call systemprompt systemprompt '{"command":"core skills list"}'
run "mcp_call_core_content_list" $SP plugins mcp call systemprompt systemprompt '{"command":"core content list"}'
run "mcp_call_infra_services_status" $SP plugins mcp call systemprompt systemprompt '{"command":"infra services status"}'
run "mcp_call_admin_agents_list" $SP plugins mcp call systemprompt systemprompt '{"command":"admin agents list"}'
run "mcp_call_analytics_overview" $SP plugins mcp call systemprompt systemprompt '{"command":"analytics overview"}'
run "mcp_call_web_validate" $SP plugins mcp call systemprompt systemprompt '{"command":"web validate"}'
run "mcp_call_plugins_list" $SP plugins mcp call systemprompt systemprompt '{"command":"plugins list"}'
run "mcp_call_help" $SP plugins mcp call systemprompt systemprompt '{"command":"--help"}'
# Negative
run "mcp_call_invalid" $SP plugins mcp call systemprompt systemprompt '{"command":"nonsense-domain whatever"}'
run "mcp_call_malformed_json" $SP plugins mcp call systemprompt systemprompt 'not-json'
run "mcp_call_missing_command" $SP plugins mcp call systemprompt systemprompt '{}'

# --- A2A messaging ---
run "a2a_message_blocking_dev" $SP admin agents message --agent developer_agent --message "List the available skills using the systemprompt tool." --blocking --timeout 60
run "a2a_message_blocking_assoc" $SP admin agents message --agent associate_agent --message "What MCP tools do you have access to?" --blocking --timeout 60
run "a2a_message_nonexistent" $SP admin agents message --agent nonexistent_agent --message "test" --blocking --timeout 10
run "a2a_message_missing" $SP admin agents message --agent developer_agent --blocking --timeout 10
run "a2a_message_short_timeout" $SP admin agents message --agent developer_agent --message "Take your time." --blocking --timeout 1

# --- Global flag matrix on a stable command ---
run "flag_json"  $SP --json admin agents list
run "flag_yaml"  $SP --yaml admin agents list
run "flag_quiet" $SP --quiet admin agents list
run "flag_verbose" $SP --verbose admin agents list
run "flag_nocolor" $SP --no-color admin agents list
run "flag_noninter" $SP --non-interactive admin agents list

# --- Post-sweep audit-spine check ---
run "post_request_list" $SP infra logs request list --limit 10
run "post_trace_list"   $SP infra logs trace list --limit 10

echo "DONE: $N commands"
