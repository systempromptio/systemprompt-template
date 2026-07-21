#!/usr/bin/env bash
# Rust standards audit — appends violations from new commits + working tree
# to ISSUE.md. Observational only; exits 0 always.
#
# Ported from systemprompt-core, retargeted at extensions/ + src/. The
# deterministic gates (sqlx allowlist, schema, HTTP errors, raw ids, file size,
# module heads) live in their own blocking scripts and are not duplicated here;
# this script covers the fuzzy rules that need a human to adjudicate.
#
# Run: ./scripts/audit-rust-standards.sh
# State: .audit-state holds last audited SHA. ISSUE.md is the append-only ledger.

set -uo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$REPO_ROOT"

STATE_FILE=".audit-state"
LEDGER="ISSUE.md"
NOW="$(date -u +'%Y-%m-%d %H:%M UTC')"

if ! command -v rg >/dev/null 2>&1; then
    echo "ripgrep (rg) required" >&2
    exit 0
fi

LAST_SHA=""
if [ -f "$STATE_FILE" ]; then
    LAST_SHA="$(cat "$STATE_FILE" 2>/dev/null || true)"
fi
CURRENT_SHA="$(git rev-parse HEAD)"

if [ -z "$LAST_SHA" ] || ! git cat-file -e "$LAST_SHA" 2>/dev/null; then
    RANGE="HEAD~10..HEAD"
    SCOPE_LABEL="bootstrap (last 10 commits)"
else
    RANGE="${LAST_SHA}..HEAD"
    SCOPE_LABEL="commits ${LAST_SHA:0:8}..${CURRENT_SHA:0:8}"
fi

if ! git rev-parse "$RANGE" >/dev/null 2>&1; then
    RANGE=""
fi

declare -a BLOCKERS=()
declare -a MAJORS=()
declare -a MINORS=()

# Test code lives in two places: the standalone tests/ workspace and per-crate
# extensions/*/tests/ directories.
is_test_path() {
    case "$1" in
        tests/*|*/tests/*) return 0 ;;
        *) return 1 ;;
    esac
}

# ---- pattern grouping over a fileset ----
# args: $1 = label scope (e.g. "commit abc123" or "working tree")
#       $2..N = files to scan
scan_files() {
    local label="$1"; shift
    [ "$#" -eq 0 ] && return

    # 1. paraphrase rustdoc — a `///` whose first word merely restates the item
    #    name or signature beneath it. `Returns Ok/Err/Some/None ...` is exempt:
    #    documenting a specific non-obvious outcome is a sanctioned use.
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        case "$f" in */build.rs|*.md) continue ;; esac
        MAJORS+=("\`${f}:${l}\` — paraphrase rustdoc (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/tests/**' -g '!**/target/**' \
        '^\s*///\s*(Returns (?!`?(Ok|Err|Some|None))|Fetches |Inserts |Deletes |Updates |Creates |Builds |Gets |Errors if |Constructs |Computes? |Resolves |Lists |Parses |Wraps |Converts |Sums )' \
        "$@" 2>/dev/null || true)

    # 2. rustdoc that narrates the body's own control flow — a `///` numbered or
    #    bulleted step list is a second, un-compiled copy of the match below it.
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        case "$f" in */build.rs|*.md) continue ;; esac
        MAJORS+=("\`${f}:${l}\` — rustdoc narrating body control flow (step list) (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/tests/**' -g '!**/target/**' \
        '^\s*///\s+[0-9]+\.\s' \
        "$@" 2>/dev/null || true)

    # 3. `///` in test code (banned — tests document themselves by name)
    while IFS=: read -r f l _; do
        is_test_path "$f" || continue
        BLOCKERS+=("\`${f}:${l}\` — \`///\` banned in test code (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/target/**' \
        '^\s*///' \
        "$@" 2>/dev/null || true)

    # 4. unwrap / expect in library code (rough — false-positives on validated newtypes)
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        case "$f" in */build.rs) continue ;; esac
        window=$(sed -n "$((l-2)),$((l+1))p" "$f" 2>/dev/null || true)
        echo "$window" | grep -qE 'Regex::new|LazyLock::new|once_cell|OnceLock::new|OnceCell::new' && continue
        MAJORS+=("\`${f}:${l}\` — \`.unwrap()\`/\`.expect()\` in library code (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/tests/**' -g '!**/target/**' \
        -e '\.unwrap\(\)|\.expect\(' \
        "$@" 2>/dev/null || true)

    # 5. println / eprintln / dbg outside carve-outs
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        case "$f" in */build.rs) continue ;; esac
        BLOCKERS+=("\`${f}:${l}\` — \`println!\`/\`eprintln!\`/\`dbg!\` outside carve-out (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/tests/**' -g '!**/target/**' \
        -e '^\s*(println!|eprintln!|dbg!)' \
        "$@" 2>/dev/null || true)

    # 6. anyhow in a library pub signature (extensions are libraries; only the
    #    root src/ binary is an application boundary)
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        case "$f" in */build.rs) continue ;; esac
        BLOCKERS+=("\`${f}:${l}\` — \`anyhow\` in library pub signature (${label})")
    done < <(rg -n --color=never \
        -g 'extensions/**/*.rs' -g '!**/tests/**' -g '!**/target/**' \
        -e '^\s*pub (async )?fn .*anyhow' \
        "$@" 2>/dev/null || true)

    # 7. *Manager type names
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        case "$f" in *.md) continue ;; esac
        MAJORS+=("\`${f}:${l}\` — \`*Manager\` naming (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/tests/**' -g '!**/target/**' \
        -e '^\s*(pub )?(struct|enum|trait) \w+Manager\b' \
        "$@" 2>/dev/null || true)

    # 8. inline #[cfg(test)] mod tests
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        BLOCKERS+=("\`${f}:${l}\` — inline \`#[cfg(test)] mod tests\` (use tests/) (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/tests/**' -g '!**/target/**' \
        -e '#\[cfg\(test\)\]\s*mod tests' \
        "$@" 2>/dev/null || true)

    # 9. let _ = <fallible> without Why comment on same/prev line
    while IFS=: read -r f l _; do
        is_test_path "$f" && continue
        case "$f" in */build.rs) continue ;; esac
        prev_line=$(sed -n "$((l-1))p" "$f" 2>/dev/null || true)
        echo "$prev_line" | grep -qE '// Why:|// JSON:' && continue
        MINORS+=("\`${f}:${l}\` — \`let _ = …\` without WHY justification (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/tests/**' -g '!**/target/**' \
        -e '^\s*let _ = ' \
        "$@" 2>/dev/null || true)

    # 10. zero-assertion test fn — a test fn body with no assertion token at all.
    #     Fuzzy: scans each fn from its attribute to the next fn/attribute boundary.
    while IFS=: read -r f l _; do
        is_test_path "$f" || continue
        prev_line=$(sed -n "$((l-1))p" "$f" 2>/dev/null || true)
        echo "$prev_line" | grep -qE '#\[.*test' || continue
        body=$(awk -v start="$l" 'NR>=start {
            print
            if (NR>start && ($0 ~ /^\s*(#\[|pub |async fn |fn )/)) exit
        }' "$f" 2>/dev/null | head -n 80)
        echo "$body" | grep -qE 'assert|expect\(|unwrap_err|panic!|should_panic' && continue
        MINORS+=("\`${f}:${l}\` — test fn with no assertion token (${label})")
    done < <(rg -n --color=never \
        -g '*.rs' -g '!**/target/**' \
        -e '^\s*(async )?fn .*\(' \
        "$@" 2>/dev/null || true)
}

# ---- build the fileset: working tree ∪ commit range ----
#
# Both sources are scanned from the current on-disk content, so a file in both
# would yield byte-identical findings twice under two different labels. Take
# the union once and report a single honest scope.
mapfile -t FILES < <(
    {
        git diff --name-only --diff-filter=AM 2>/dev/null
        git diff --name-only --cached --diff-filter=AM 2>/dev/null
        git ls-files --others --exclude-standard 2>/dev/null
        [ -n "$RANGE" ] && git diff --name-only "$RANGE" 2>/dev/null
    } | grep -E '\.rs$' | sort -u
)

EXISTING=()
for f in "${FILES[@]}"; do
    [ -f "$f" ] && EXISTING+=("$f")
done

if [ "${#EXISTING[@]}" -gt 0 ]; then
    scan_files "$SCOPE_LABEL + working tree" "${EXISTING[@]}"
fi

# ---- write to ledger ----
if [ ! -f "$LEDGER" ]; then
    printf '# Rust Standards Violations Ledger\n\n' > "$LEDGER"
fi

total=$((${#BLOCKERS[@]} + ${#MAJORS[@]} + ${#MINORS[@]}))

{
    printf '## %s — audit run\n\n' "$NOW"
    printf 'Scope: %s + working tree (%d files)\n\n' "$SCOPE_LABEL" "${#EXISTING[@]}"
    if [ "$total" -eq 0 ]; then
        printf '(no violations)\n\n'
    else
        if [ "${#BLOCKERS[@]}" -gt 0 ]; then
            printf '### BLOCKER\n\n'
            printf -- '- %s\n' "${BLOCKERS[@]}" | sort -u
            printf '\n'
        fi
        if [ "${#MAJORS[@]}" -gt 0 ]; then
            printf '### MAJOR\n\n'
            printf -- '- %s\n' "${MAJORS[@]}" | sort -u
            printf '\n'
        fi
        if [ "${#MINORS[@]}" -gt 0 ]; then
            printf '### MINOR\n\n'
            printf -- '- %s\n' "${MINORS[@]}" | sort -u
            printf '\n'
        fi
    fi
} >> "$LEDGER"

echo "$CURRENT_SHA" > "$STATE_FILE"

echo "audit-rust-standards: appended to $LEDGER (BLOCKER=${#BLOCKERS[@]} MAJOR=${#MAJORS[@]} MINOR=${#MINORS[@]})"
exit 0
