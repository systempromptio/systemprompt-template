#!/bin/bash
# SVG RECORDING: Extensions & capabilities
# One Rust binary, many compiled-in extensions, all discoverable.
set -e
source "$(dirname "$0")/_colors.sh"

header "EXTENSIONS" "One binary, many capabilities — all compiled in, all discoverable"
pause 1

# ── Capabilities summary ──
subheader "Capabilities" "jobs, schemas, templates — rolled up across every extension"
pause 0.3

type_cmd "systemprompt plugins capabilities"
pause 0.3

"$CLI" plugins capabilities --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do printf "    ${CYAN}%s${R}\n" "$line"; done
echo ""
check "12 extensions, 71 schemas, 13 jobs — all in one process"
pause 1.2

divider

# ── Extension list ──
subheader "Extensions" "compiled into the binary — no sidecars, no runtime loading"
pause 0.3

type_cmd "systemprompt plugins list"
pause 0.3

"$CLI" plugins list --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | grep -E '"(id|name|priority)"' \
  | head -20 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
pause 1.2

divider

# ── Inspect one extension ──
subheader "Inspect one" "its schemas, jobs, config prefix"
pause 0.3

type_cmd "systemprompt plugins show mcp"
pause 0.3

"$CLI" plugins show mcp --profile "$PROFILE" 2>&1 \
  | grep -v "^\[profile" \
  | head -12 \
  | while IFS= read -r line; do echo "    $line"; done
echo ""
check "schemas are compile-time checked by sqlx — zero runtime surprises"
pause 1.5
