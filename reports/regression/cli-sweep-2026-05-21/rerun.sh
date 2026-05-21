#!/usr/bin/env bash
# Round 2: corrected syntax for previously-failed commands + deep MCP/A2A.
set -u
SP="/var/www/html/systemprompt-template/target/debug/systemprompt"
ROOT="$(cd "$(dirname "$0")" && pwd)"
LOGS="$ROOT/logs"
INDEX="$ROOT/index2.tsv"
: > "$INDEX"
N=200

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

# --- Corrected admin / cloud / analytics / web verbs ---
run "admin_session_show"   $SP admin session show
run "admin_session_list_v2" $SP admin session list
run "admin_config_paths_show" $SP admin config paths show
run "admin_config_runtime_show" $SP admin config runtime show
run "admin_config_server_show" $SP admin config server show
run "admin_config_security_show" $SP admin config security show
run "admin_config_ratelimits_show" $SP admin config rate-limits show
run "admin_keys_help"   $SP admin keys --help
run "admin_access_control_help" $SP admin access-control --help
run "cloud_auth_whoami_v2" $SP cloud auth whoami
run "cloud_sync_help"   $SP cloud sync --help
run "cloud_secrets_help" $SP cloud secrets --help
run "cloud_domain_help" $SP cloud domain --help
run "analytics_sessions_stats" $SP analytics sessions stats
run "analytics_sessions_trends" $SP analytics sessions trends
run "analytics_content_help"  $SP analytics content --help
run "analytics_traffic_help"  $SP analytics traffic --help
run "analytics_costs_help"    $SP analytics costs --help
run "infra_db_migrations_status" $SP infra db migrations status
run "web_sitemap_show"   $SP web sitemap show

# --- Corrected agent commands (positional name, not --agent) ---
run "agents_tools_dev_v2"    $SP admin agents tools developer_agent
run "agents_tools_assoc_v2"  $SP admin agents tools associate_agent
run "agents_logs_dev_v2"     $SP admin agents logs developer_agent -n 20

# --- MCP CALL with corrected syntax (--args JSON) ---
run "mcp_call_skills_v2" $SP plugins mcp call systemprompt systemprompt --args '{"command":"core skills list"}'
run "mcp_call_content_v2" $SP plugins mcp call systemprompt systemprompt --args '{"command":"core content list"}'
run "mcp_call_infra_v2"   $SP plugins mcp call systemprompt systemprompt --args '{"command":"infra services status"}'
run "mcp_call_admin_agents_v2" $SP plugins mcp call systemprompt systemprompt --args '{"command":"admin agents list"}'
run "mcp_call_analytics_v2" $SP plugins mcp call systemprompt systemprompt --args '{"command":"analytics overview"}'
run "mcp_call_web_v2"      $SP plugins mcp call systemprompt systemprompt --args '{"command":"web validate"}'
run "mcp_call_plugins_v2"  $SP plugins mcp call systemprompt systemprompt --args '{"command":"plugins list"}'
run "mcp_call_help_v2"     $SP plugins mcp call systemprompt systemprompt --args '{"command":"--help"}'
run "mcp_call_invalid_v2"  $SP plugins mcp call systemprompt systemprompt --args '{"command":"nonsense whatever"}'
run "mcp_call_malformed_v2" $SP plugins mcp call systemprompt systemprompt --args 'not-json'
run "mcp_call_empty_v2"    $SP plugins mcp call systemprompt systemprompt --args '{}'
run "mcp_call_no_args"     $SP plugins mcp call systemprompt systemprompt
run "mcp_call_no_tool"     $SP plugins mcp call systemprompt
run "mcp_call_bad_server"  $SP plugins mcp call nonexistent_server systemprompt --args '{"command":"core skills list"}'

# --- A2A messaging (positional agent, -m message) ---
run "a2a_dev_blocking_v2"  $SP admin agents message developer_agent -m "List the available skills using the systemprompt tool." --blocking --timeout 60
run "a2a_assoc_blocking_v2" $SP admin agents message associate_agent -m "What MCP tools do you have access to?" --blocking --timeout 60
run "a2a_nonexistent_v2"   $SP admin agents message nonexistent_agent -m "test" --blocking --timeout 10
run "a2a_missing_msg_v2"   $SP admin agents message developer_agent --blocking --timeout 10
run "a2a_short_timeout_v2" $SP admin agents message developer_agent -m "Take your time and think about this." --blocking --timeout 1
run "a2a_dev_json"         $SP admin agents message developer_agent -m "Just say hi." --blocking --timeout 60 --json
run "a2a_dev_nonblocking"  $SP admin agents message developer_agent -m "Background task — just acknowledge."

# --- Plugins show with correct identifier shape ---
run "plugins_show_help"    $SP plugins show --help
run "plugins_list_json"    $SP --json plugins list

# --- Final audit-spine check after A2A traffic ---
run "post2_request_list" $SP infra logs request list --limit 10
run "post2_trace_list"   $SP infra logs trace list --limit 10
run "post2_jobs_list"    $SP infra jobs list

echo "DONE round 2"
