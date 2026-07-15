#!/usr/bin/env bash
set -uo pipefail

BANNED='(user_id|agent_id|task_id|tenant_id|context_id|session_id|file_id|skill_id|client_id|artifact_id|message_id|role_id|hook_id|execution_step_id|content_id|source_id)'

PATTERN="(\bpub\s+)?\b${BANNED}\s*:\s*(Option<)?&?(\s)?(mut\s+)?(String|str)\b"

SEARCH_DIRS=(extensions src)

if command -v rg >/dev/null 2>&1; then
    RAW=$(rg -n --no-heading --color=never \
        -g '*.rs' \
        -g '!tests/**' \
        -g '!**/target/**' \
        -g '!**/.sqlx/**' \
        -g '!**/oauth/**' \
        -e "$PATTERN" \
        "${SEARCH_DIRS[@]}" 2>/dev/null || true)
else
    RAW=$(grep -rnE \
        --include='*.rs' \
        --exclude-dir=target \
        --exclude-dir=.sqlx \
        --exclude-dir=tests \
        --exclude-dir=oauth \
        "$PATTERN" \
        "${SEARCH_DIRS[@]}" 2>/dev/null || true)
fi

MATCHES=""
while IFS= read -r line; do
    [ -z "$line" ] && continue
    file="${line%%:*}"
    rest="${line#*:}"
    lineno="${rest%%:*}"
    content="${rest#*:}"

    case "$file" in
        crates/entry/api/src/routes/oauth/*) continue ;;
        crates/tests/*) continue ;;
        */target/*|*/.sqlx/*) continue ;;
    esac

    if [[ "$file" == */types.rs ]]; then
        if awk -v ln="$lineno" '
            NR <= ln {
                if (match($0, /struct[[:space:]]+[A-Za-z_][A-Za-z0-9_]*(Row|Output)\b/)) {
                    in_struct = 1
                } else if ($0 ~ /^\}/) {
                    in_struct = 0
                }
            }
            END { exit (in_struct ? 0 : 1) }
        ' "$file" 2>/dev/null; then
            continue
        fi
    fi

    MATCHES+="${file}:${lineno}:${content}"$'\n'
done <<< "$RAW"

if [ -z "$MATCHES" ]; then
    echo "lint-raw-ids: OK (no raw ID fields found)"
    exit 0
fi

echo "lint-raw-ids: raw String/&str used for typed-ID field names:"
echo ""
printf '%s' "$MATCHES"
exit 1
