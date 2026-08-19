#!/usr/bin/env bash
# `serde_json::Value` erases type information; the standard treats every
# occurrence as a code smell requiring justification. This gate ties each
# production occurrence to a `// JSON:` comment naming the sanctioned reason
# (protocol boundary, trait contract, outgoing fixed-shape construction).
#
# A justification covers the occurrences below it: each `serde_json::Value`
# (or bare `Value` from a `use serde_json::Value` import) must have a
# `// JSON:` line within the 25 lines above it in the same file. One comment
# per struct/impl/function region is the intended granularity, not one per
# line.
#
# Test workspaces and build.rs are out of scope.
set -uo pipefail

cd "$(dirname "$0")/.."

MATCHES=""
while IFS= read -r file; do
    case "$file" in
        tests/*|*/tests/*) continue ;;
        */build.rs) continue ;;
    esac
    FOUND=$(awk '
        { line[FNR] = $0; n = FNR }
        /use serde_json::.*\bValue\b/ { imported = 1 }
        END {
            for (i = 1; i <= n; i++) {
                if (line[i] ~ /^[[:space:]]*\/\//) continue
                hit = 0
                if (line[i] ~ /serde_json::Value/) hit = 1
                else if (imported && line[i] ~ /[^A-Za-z0-9_:]Value[^A-Za-z0-9_]/ \
                         && line[i] !~ /use serde_json/) hit = 1
                if (!hit) continue
                lo = i - 25; if (lo < 1) lo = 1
                ok = 0
                for (j = lo; j <= i; j++) {
                    if (line[j] ~ /\/\/ JSON:/) ok = 1
                }
                if (!ok) printf "%s:%d:%s\n", FILENAME, i, line[i]
            }
        }
    ' "$file")
    [ -n "$FOUND" ] && MATCHES+="${FOUND}"$'\n'
done < <(git ls-files -co --exclude-standard 'extensions/**/*.rs' 'src/**/*.rs' 'bridge/src/**/*.rs' | sort -u)

if [ -z "$MATCHES" ]; then
    echo "check-json-value: OK (every serde_json::Value carries a // JSON: justification)"
    exit 0
fi

echo "check-json-value: serde_json::Value without a '// JSON:' justification within"
echo "25 lines above. Type the data with a #[derive(Deserialize)] struct, or state"
echo "the protocol-boundary reason:"
echo ""
printf '%s' "$MATCHES"
exit 1
