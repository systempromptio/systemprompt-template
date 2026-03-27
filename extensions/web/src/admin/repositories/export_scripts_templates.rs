use std::path::Path;

use super::export_scripts::{format_script_command, HookEntry};
use super::hook_catalog;

pub(super) fn build_transcript_script_from_template(
    services_path: &Path,
    token: &str,
    platform_url: &str,
    plugin_id: &str,
) -> String {
    if let Ok(Some(template)) =
        hook_catalog::read_hook_template(services_path, "tracking_stop", "transcript.sh.tmpl")
    {
        return hook_catalog::render_tracking_script(&template, plugin_id, token, platform_url);
    }
    build_transcript_script(token, platform_url, plugin_id)
}

pub(super) fn build_transcript_script_ps1_from_template(
    services_path: &Path,
    token: &str,
    platform_url: &str,
    plugin_id: &str,
) -> String {
    if let Ok(Some(template)) =
        hook_catalog::read_hook_template(services_path, "tracking_stop", "transcript.ps1.tmpl")
    {
        return hook_catalog::render_tracking_script(&template, plugin_id, token, platform_url);
    }
    build_transcript_script_ps1(token, platform_url, plugin_id)
}

fn build_transcript_script(token: &str, platform_url: &str, plugin_id: &str) -> String {
    let url = format!("{platform_url}/api/public/hooks/transcript?plugin_id={plugin_id}");
    format!(
        r#"#!/usr/bin/env bash
LOG="/tmp/sp-transcript-{plugin_id}.log"
INPUT=$(cat)
TRANSCRIPT_PATH=$(echo "$INPUT" | grep -o '"transcript_path"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"transcript_path"[[:space:]]*:[[:space:]]*"//' | sed 's/"$//')
SESSION_ID=$(echo "$INPUT" | grep -o '"session_id"[[:space:]]*:[[:space:]]*"[^"]*"' | head -1 | sed 's/.*"session_id"[[:space:]]*:[[:space:]]*"//' | sed 's/"$//')
if [ -z "$TRANSCRIPT_PATH" ] || [ ! -f "$TRANSCRIPT_PATH" ]; then
  exit 0
fi
TRANSCRIPT_CONTENT=$(python3 -c "
import json, sys
lines = []
with open(sys.argv[1], 'r') as f:
    for line in f:
        line = line.strip()
        if line:
            try:
                lines.append(json.loads(line))
            except json.JSONDecodeError:
                pass
json.dump(lines, sys.stdout)
" "$TRANSCRIPT_PATH" 2>/dev/null)
if [ -z "$TRANSCRIPT_CONTENT" ]; then
  TRANSCRIPT_CONTENT=$(cat "$TRANSCRIPT_PATH" | jq -s '.' 2>/dev/null || echo '[]')
fi
PAYLOAD=$(jq -n --arg sid "$SESSION_ID" --argjson transcript "$TRANSCRIPT_CONTENT" \
  '{{"session_id": $sid, "transcript": $transcript}}')
curl -s --max-time 30 -X POST "{url}" \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d "$PAYLOAD" \
  >> "$LOG" 2>&1
exit 0
"#
    )
}

fn build_transcript_script_ps1(token: &str, platform_url: &str, plugin_id: &str) -> String {
    let url = format!("{platform_url}/api/public/hooks/transcript?plugin_id={plugin_id}");
    format!(
        r#"$logFile = Join-Path $env:TEMP "sp-transcript-{plugin_id}.log"
$reader = [System.IO.StreamReader]::new([System.Console]::OpenStandardInput())
$body = $reader.ReadToEnd()
$reader.Close()
$inputObj = $body | ConvertFrom-Json -ErrorAction SilentlyContinue
$transcriptPath = $inputObj.transcript_path
$sessionId = $inputObj.session_id
if (-not $transcriptPath -or -not (Test-Path $transcriptPath)) {{ exit 0 }}
$lines = @()
Get-Content $transcriptPath | ForEach-Object {{
    $trimmed = $_.Trim()
    if ($trimmed) {{ try {{ $lines += ($trimmed | ConvertFrom-Json) }} catch {{}} }}
}}
$payload = @{{ session_id = $sessionId; transcript = $lines }} | ConvertTo-Json -Depth 50 -Compress
$headers = @{{ "Authorization" = "Bearer {token}"; "Content-Type" = "application/json" }}
try {{
    Invoke-RestMethod -Uri "{url}" -Method Post -Headers $headers -Body $payload -TimeoutSec 30 -ErrorAction SilentlyContinue | Out-Null
}} catch {{
    Add-Content -Path $logFile -Value "[$(Get-Date -Format o)] Error: $_" -ErrorAction SilentlyContinue
}}
exit 0
"#
    )
}

pub(super) fn transcript_hook_entry(script_name: &str, is_windows: bool) -> HookEntry {
    let command = format_script_command(script_name, is_windows);
    HookEntry {
        event: "Stop".to_string(),
        matcher: None,
        hook_type: super::export_scripts::HookType::Command { command },
        is_async: true,
    }
}
