# systemprompt-astound
set dotenv-load
# Without this, `just cli ... --full-name "Test User"` word-splits the quoted
# value into two arguments before the CLI ever parses it.
set positional-arguments

# The fork-aware gates (check-fork-drift, check-dead-repository-code) compare
# against the sibling template checkout. Without this they skip silently, which
# reads as "passed" — export a default so they actually run, and let an
# already-set value win for CI or a non-standard layout.
export SIBLING_REPO := env("SIBLING_REPO", if path_exists("../systemprompt-template") == "true" { "../systemprompt-template" } else { "" })

CLI_RELEASE := "target/release/systemprompt"

# Cloud profile every deploy targets: tenant a2f658d8bc5f, Fly app
# sp-a2f658d8bc5f, served at https://astound.systemprompt.io.
# See .systemprompt/profiles/production/.
DEPLOY_PROFILE := "production"

# Use newest binary (release vs debug, whichever is most recent)
CLI := if path_exists("target/release/systemprompt") == "true" { \
    if path_exists("target/debug/systemprompt") == "true" { \
        `[ target/release/systemprompt -nt target/debug/systemprompt ] && echo target/release/systemprompt || echo target/debug/systemprompt` \
    } else { \
        "target/release/systemprompt" \
    } \
} else if path_exists("target/debug/systemprompt") == "true" { \
    "target/debug/systemprompt" \
} else { \
    "echo 'ERROR: No CLI binary found. Run: just build' && exit 1" \
}

# Default: run CLI with any arguments
default *ARGS:
    {{CLI}} "$@"

# Run CLI with full session context (profile + auth token)
cli *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    SESSION_FILE="{{justfile_directory()}}/.systemprompt/sessions/index.json"
    if [ -f "$SESSION_FILE" ]; then
        ACTIVE_KEY=$(jq -r '.active_key // "local"' "$SESSION_FILE")
        export SYSTEMPROMPT_PROFILE=$(jq -r ".sessions[\"$ACTIVE_KEY\"].profile_path // empty" "$SESSION_FILE")
        export SYSTEMPROMPT_AUTH_TOKEN=$(jq -r ".sessions[\"$ACTIVE_KEY\"].session_token // empty" "$SESSION_FILE")
    fi
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    exec {{CLI}} "$@"

# Get DATABASE_URL from profile secrets (for sqlx compile-time checks)
_db-url:
    @PROFILE="${SYSTEMPROMPT_PROFILE:-}"; \
    [ -n "$PROFILE" ] || PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"; \
    if [ -f "$PROFILE" ]; then \
        PROFILE_DIR="$(dirname "$PROFILE")"; \
        SECRETS_PATH="$(yq -r '.secrets.secrets_path // "./secrets.json"' "$PROFILE")"; \
        if [ "${SECRETS_PATH#/}" = "$SECRETS_PATH" ]; then \
            SECRETS_FILE="$PROFILE_DIR/$SECRETS_PATH"; \
        else \
            SECRETS_FILE="$SECRETS_PATH"; \
        fi; \
        if [ -f "$SECRETS_FILE" ]; then \
            jq -r '.database_url' "$SECRETS_FILE"; \
        else \
            echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
        fi; \
    else \
        cat .systemprompt/tenants.json 2>/dev/null | jq -r '.tenants[] | select(.tenant_type == "local") | .database_url' | head -1 || echo "postgres://systemprompt:systemprompt@localhost:5432/systemprompt"; \
    fi

# ══════════════════════════════════════════════════════════════════════════════
# BUILD & CHECK
# ══════════════════════════════════════════════════════════════════════════════

# Build (Windows) - always uses offline mode
[windows]
build *FLAGS:
    $env:SQLX_OFFLINE="true"; cargo build --workspace {{FLAGS}}

# Build (Unix) - single-flight: dedupes concurrent identical builds across agents
[unix]
build *FLAGS:
    @scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

# What is the build/lint/test state right now? Read this before running anything.
[unix]
build-status *RECIPE:
    @scripts/build-coordinator.sh status {{RECIPE}}

# Re-run even if the coordinator considers this source tree already green
[unix]
build-force *FLAGS:
    @BUILD_FORCE=1 scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

# The real build. Call `just build` instead - this one skips coordination.
[unix]
_build-uncoordinated *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    # Default to the `local` profile when one is set up but no SYSTEMPROMPT_PROFILE
    # is explicitly exported — keeps the in-build migrate step from failing
    # with "Profile '' not found" on a fresh clone where setup-local writes
    # secrets.json before invoking `just build`.
    SECRETS_FILE_DEFAULT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ] && [ -f "$SECRETS_FILE_DEFAULT_PROFILE" ]; then
        export SYSTEMPROMPT_PROFILE="local"
    else
        export SYSTEMPROMPT_PROFILE="${SYSTEMPROMPT_PROFILE:-}"
    fi
    # aws-lc-sys refuses to build with GCC <10 due to bug #95189.
    # Force clang if available so release (LTO) builds succeed.
    if command -v clang >/dev/null 2>&1; then
        export CC="${CC:-clang}"
        export CXX="${CXX:-clang++}"
    fi
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    db_reachable() {
        local url="$1"
        local pgcmd=""
        if command -v pg_isready >/dev/null 2>&1; then pgcmd="pg_isready"
        elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then pgcmd="/opt/homebrew/opt/libpq/bin/pg_isready"
        elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then pgcmd="/usr/local/opt/libpq/bin/pg_isready"
        fi
        if [ -n "$pgcmd" ]; then
            "$pgcmd" -d "$url" -t 2 >/dev/null 2>&1 && return 0 || return 1
        fi
        local hostport="${url#*@}"; hostport="${hostport%%/*}"
        local host="${hostport%:*}"; local port="${hostport##*:}"
        [ "$port" = "$host" ] && port=5432
        (exec 3<>/dev/tcp/"$host"/"$port") >/dev/null 2>&1 && { exec 3<&-; exec 3>&-; return 0; } || return 1
    }
    if [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            if db_reachable "$DB_URL"; then
                export DATABASE_URL="$DB_URL"
                echo "Using database: $DB_URL"
            else
                echo "Database not reachable, using offline mode"
                USE_OFFLINE=true
            fi
        else
            echo "No database_url in secrets, using offline mode"
            USE_OFFLINE=true
        fi
    else
        echo "No local profile secrets found, using offline mode"
        USE_OFFLINE=true
    fi
    # Sync DATABASE_URL to MCP extension directories for sqlx compile-time checks
    if [ "$USE_OFFLINE" = "false" ]; then
        for dir in extensions/mcp/*/; do
            if [ -f "$dir/Cargo.toml" ]; then
                echo "DATABASE_URL=$DATABASE_URL" > "$dir/.env"
            fi
        done
    fi
    cargo update systemprompt --quiet 2>/dev/null || true
    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo build --workspace {{FLAGS}}
    else
        # Apply pending schema migrations before the online sqlx compile-time
        # check sees the live DB. Build the CLI in offline mode first so
        # drift between checked-in `.sqlx/` and the unmigrated live schema
        # can't deadlock the bootstrap.
        echo "Applying pending migrations before online build..."
        SQLX_OFFLINE=true cargo build --bin systemprompt --quiet
        target/debug/systemprompt infra db migrate
        SQLX_OFFLINE=false cargo build --workspace {{FLAGS}}
    fi

# Clippy (Windows) - always uses offline mode
[windows]
clippy *FLAGS: lint-no-synthesis lint-no-untyped-admin lint-gates
    $env:SQLX_OFFLINE="true"; cargo clippy --workspace {{FLAGS}} -- -D warnings

# Clippy (Unix) - single-flight, same coordinator as `just build`
[unix]
clippy *FLAGS:
    @scripts/build-coordinator.sh run clippy "{{FLAGS}}" -- {{just_executable()}} _clippy-uncoordinated {{FLAGS}}

# The real clippy. Call `just clippy` instead - this one skips coordination.
[unix]
_clippy-uncoordinated *FLAGS: lint-no-synthesis lint-no-untyped-admin lint-gates
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    USE_OFFLINE=false
    db_reachable() {
        local url="$1"
        local pgcmd=""
        if command -v pg_isready >/dev/null 2>&1; then pgcmd="pg_isready"
        elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then pgcmd="/opt/homebrew/opt/libpq/bin/pg_isready"
        elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then pgcmd="/usr/local/opt/libpq/bin/pg_isready"
        fi
        if [ -n "$pgcmd" ]; then
            "$pgcmd" -d "$url" -t 2 >/dev/null 2>&1 && return 0 || return 1
        fi
        local hostport="${url#*@}"; hostport="${hostport%%/*}"
        local host="${hostport%:*}"; local port="${hostport##*:}"
        [ "$port" = "$host" ] && port=5432
        (exec 3<>/dev/tcp/"$host"/"$port") >/dev/null 2>&1 && { exec 3<&-; exec 3>&-; return 0; } || return 1
    }
    if [ -f "$SECRETS_FILE" ]; then
        DB_URL=$(sed -n 's/.*"database_url"[[:space:]]*:[[:space:]]*"\([^"]*\)".*/\1/p' "$SECRETS_FILE" 2>/dev/null | head -1)
        if [ -n "$DB_URL" ] && [ "$DB_URL" != "null" ]; then
            if db_reachable "$DB_URL"; then
                export DATABASE_URL="$DB_URL"
            else
                USE_OFFLINE=true
            fi
        else
            USE_OFFLINE=true
        fi
    else
        USE_OFFLINE=true
    fi
    if [ "$USE_OFFLINE" = "true" ]; then
        SQLX_OFFLINE=true cargo clippy --workspace {{FLAGS}} -- -D warnings
    else
        SQLX_OFFLINE=false cargo clippy --workspace {{FLAGS}} -- -D warnings
    fi
    # bridge/ is a standalone workspace and is not covered by --workspace
    cargo clippy --manifest-path bridge/Cargo.toml --all-targets {{FLAGS}} -- -D warnings

# Unit tests: extensions/web/admin (main workspace) + the tests/ workspace.
# If sqlx offline errors appear, run `just prepare` first to refresh .sqlx.
test-unit:
    @scripts/build-coordinator.sh run test-unit "" -- {{just_executable()}} _test-unit-uncoordinated

_test-unit-uncoordinated:
    cargo nextest run -p systemprompt-web-admin --tests
    cargo nextest run -p systemprompt-web-extension --tests
    cargo nextest run --manifest-path tests/Cargo.toml -p mcp-unit-tests -p web-unit-tests

# DB-backed integration tests. Creates/drops throwaway mcp_ext_test_*
# databases on the maintenance DB; the harness guard refuses any database
# name that is not 'test', 'postgres', or '*_test'. Falls back to the local
# profile's server with the database swapped to 'postgres'.
test-integration:
    @scripts/build-coordinator.sh run test-integration "" -- {{just_executable()}} _test-integration-uncoordinated

_test-integration-uncoordinated:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
        SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 -c "
    import json, urllib.parse as up
    u = up.urlsplit(json.load(open('.systemprompt/profiles/local/secrets.json'))['database_url'])
    print(up.urlunsplit((u.scheme, u.netloc, '/postgres', '', '')))")
        export SYSTEMPROMPT_TEST_DATABASE_URL
    fi
    cargo nextest run --manifest-path tests/Cargo.toml -p mcp-integration-tests -p web-integration-tests -p admin-db-core-tests -p admin-db-config-tests -p gateway-integration-tests

# HTTP contract suite: drives every admin route under three principals and
# diffs the result against tests/contract/admin/baseline.txt. Same throwaway-
# database convention as test-integration. A status change fails the run; if
# it is deliberate, re-record with UPDATE_CONTRACT_BASELINE=1 and list it in
# the PR.
test-contract:
    @scripts/build-coordinator.sh run test-contract "" -- {{just_executable()}} _test-contract-uncoordinated

_test-contract-uncoordinated:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
        SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 -c "
    import json, urllib.parse as up
    u = up.urlsplit(json.load(open('.systemprompt/profiles/local/secrets.json'))['database_url'])
    print(up.urlunsplit((u.scheme, u.netloc, '/postgres', '', '')))")
        export SYSTEMPROMPT_TEST_DATABASE_URL
    fi
    cargo nextest run --manifest-path tests/Cargo.toml -p admin-contract-tests

# All tests
test: test-unit test-integration test-contract

# Source gates ported from systemprompt-core (scripts/*.sh)
lint-gates:
    @scripts/build-coordinator.sh run lint-gates "" -- {{just_executable()}} _lint-gates-uncoordinated

# Gates are independent read-only checks; they run concurrently and every
# failure is reported, so one red gate cannot hide the rest.
_lint-gates-uncoordinated:
    #!/usr/bin/env bash
    set -uo pipefail
    gates=(
        lint-schema.sh
        lint-extensions.sh
        check-migration-numbers.sh
        lint-layers.sh
        lint-repo-construction.sh
        check-json-value.sh
        check-sqlx.sh
        check-http-errors.sh
        check-test-value.sh
        lint-raw-ids.sh
        check-glob-reexports.sh
        check-comments.sh
        lint-inline-comments.sh
        check-duplicate-types.sh
        check-repository-naming.sh
        check-admin-template-links.sh
        check-admin-template-assets.sh
        # admin-css-classes + frontend-standards now run as cargo tests in
        # extensions/web/tests/ (admin_css_classes.rs, frontend_standards.rs).
        check-fork-drift.sh
        check-dead-repository-code.sh
        check-file-headers.sh
        check-file-size.sh
        check-asset-reachability.sh
        check-workspace-deps.sh
        validate-services.sh
    )
    logdir=$(mktemp -d)
    trap 'rm -rf "$logdir"' EXIT
    pids=()
    for gate in "${gates[@]}"; do
        bash "scripts/$gate" >"$logdir/$gate.log" 2>&1 &
        pids+=("$!:$gate")
    done
    failed=()
    for entry in "${pids[@]}"; do
        pid=${entry%%:*}
        gate=${entry#*:}
        if ! wait "$pid"; then
            failed+=("$gate")
        fi
    done
    if [ ${#failed[@]} -gt 0 ]; then
        for gate in "${failed[@]}"; do
            echo "==== FAILED: $gate ===="
            cat "$logdir/$gate.log"
        done
        echo "lint gates failed: ${failed[*]}"
        exit 1
    fi
    echo "all ${#gates[@]} lint gates passed"

# The whole gate, in one command. This repo runs no hosted CI, so nothing
# else will catch what this misses: run it before you push. `preflight` adds
# the coverage floor/ratchet on top; use it before merging.
verify: preflight-static preflight-lint test
    @echo "verify: format, sqlx cache, lint gates, clippy, docs, msrv, and tests all pass"

# ══════════════════════════════════════════════════════════════════════════════
# PREFLIGHT (local stand-in for CI — tiered, cheapest first)
# ══════════════════════════════════════════════════════════════════════════════

# Everything: static gates → lint/doc/msrv → tests → coverage floor+ratchet.
# This is the mandatory pre-merge gate; there is no CI behind it.
preflight: preflight-static preflight-lint test coverage-check

# Tier 0 — seconds. Formatting, sqlx cache freshness, and the source gates.
preflight-static:
    cargo fmt --all -- --check
    bash scripts/check-sqlx-cache.sh
    {{just_executable()}} lint-gates

# Tier 1 — compilers. Clippy (both workspaces), rustdoc as errors, MSRV.
preflight-lint:
    {{just_executable()}} clippy
    {{just_executable()}} doc-check
    {{just_executable()}} msrv-check

# Weekly deep pass: preflight plus the network-touching supply-chain gates.
preflight-full: preflight deny audit machete hack

# Rustdoc with warnings denied, across all three workspaces (root, tests/,
# bridge/) — mirrors core's quality.yml docs job. Single-flight coordinated.
doc-check:
    @scripts/build-coordinator.sh run doc-check "" -- {{just_executable()}} _doc-check-uncoordinated

_doc-check-uncoordinated:
    SQLX_OFFLINE=true RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps
    SQLX_OFFLINE=true RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path tests/Cargo.toml --workspace --no-deps
    RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path bridge/Cargo.toml --no-deps

# The workspace must build on the declared minimum supported Rust version
# (rust-version in Cargo.toml). Requires: rustup toolchain install 1.94.0
msrv-check:
    @scripts/build-coordinator.sh run msrv-check "" -- {{just_executable()}} _msrv-check-uncoordinated

_msrv-check-uncoordinated:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! rustup toolchain list | grep -q '^1\.94\.0'; then
        echo "msrv-check: toolchain 1.94.0 missing — run: rustup toolchain install 1.94.0" >&2
        exit 1
    fi
    SQLX_OFFLINE=true cargo +1.94.0 check --workspace

# ══════════════════════════════════════════════════════════════════════════════
# COVERAGE (raw llvm-cov; floor + ratchet vs tracked coverage/baseline.json)
# ══════════════════════════════════════════════════════════════════════════════

# Instrumented test run over all three workspaces; writes coverage-report/.
# See scripts/coverage.sh for the sccache/mold neutralisation notes.
coverage:
    @scripts/build-coordinator.sh run coverage "" -- bash scripts/coverage.sh

# Enforce the floor and ratchet recorded in coverage/baseline.json.
coverage-check: coverage
    bash scripts/coverage-check.sh

# Re-record coverage/baseline.json at the measured value (deliberate act —
# commit the result). Raise the "floor" field by hand as milestones land.
coverage-baseline: coverage
    UPDATE_BASELINE=1 bash scripts/coverage-check.sh

# Browsable HTML tree from the last `just coverage` run.
coverage-html:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="$(pwd)"
    if [ ! -f "$ROOT/coverage-report/tests.profdata" ]; then
        echo "Run 'just coverage' first" >&2
        exit 1
    fi
    TOOLDIR="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
    TBASE="${COVERAGE_TARGET_DIR:-$ROOT/coverage-report/target}"
    BINS=$(for t in "$TBASE-root" "$TBASE-tests" "$TBASE-bridge"; do \
        find "$t/debug/deps" -maxdepth 1 -executable -type f ! -name '*.d' ! -name '*.so' -printf '%T@ %p\n' 2>/dev/null; \
    done | sort -rn | awk '{ base=$2; sub(".*/", "", base); sub(/-[0-9a-f]+$/, "", base); if (!seen[base]++) print $2 }')
    OBJ_ARGS=""
    for b in $BINS; do OBJ_ARGS="$OBJ_ARGS --object $b"; done
    mkdir -p "$ROOT/coverage-report/html"
    "$TOOLDIR/llvm-cov" show \
        --instr-profile="$ROOT/coverage-report/tests.profdata" \
        $OBJ_ARGS \
        --ignore-filename-regex="(\.cargo|/rustc/|/registry/|/debug/build/|/tests/|/target/|systemprompt-core/|systemprompt-astound/src/(main|lib)\.rs|bridge/src/main\.rs|extensions/cli/salesforce/src/(main\.rs|commands/)|extensions/.*/extension\.rs|build\.rs)" \
        --format=html \
        --output-dir="$ROOT/coverage-report/html"
    echo "Coverage report: coverage-report/html/index.html"

# Remove all coverage artifacts (instrumented target dirs included).
# Refuses while a coordinated run holds the lock: coverage-report/ carries the
# instrumented test binaries, so deleting it mid-run makes every remaining test
# fail to exec ("No such file or directory", nextest exit 70) and the report
# come out at 0.00% — a failure that looks like a code regression and is not.
coverage-clean:
    #!/usr/bin/env bash
    set -euo pipefail
    LOCK="${COORD_STATE_DIR:-{{ justfile_directory() }}/.build}/lock"
    if [ -d "$LOCK" ]; then
        PID="$(cat "$LOCK/pid" 2>/dev/null || echo)"
        if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
            echo "refusing: '$(cat "$LOCK/recipe" 2>/dev/null || echo run)' is running (pid $PID)." >&2
            echo "  Deleting coverage-report/ now would pull the instrumented binaries" >&2
            echo "  out from under it. Wait for it, or override with COVERAGE_CLEAN_FORCE=1." >&2
            [ "${COVERAGE_CLEAN_FORCE:-0}" = "1" ] || exit 1
        fi
    fi
    rm -rf coverage-report/

# Point git at the tracked hooks (pre-commit patch-marker guard + fast gates,
# pre-push static+lint tiers). Run once per clone.
init-hooks:
    git config core.hooksPath .githooks
    @echo "git hooks now sourced from .githooks/"

# Cross-file referential integrity for services/ (ACL entity ids, MCP ports)
validate:
    bash scripts/validate-services.sh

# Shared sources that differ from the sibling fork must be recorded in
# .fork-divergence. Needs SIBLING_REPO; skips cleanly without it.
check-fork-drift:
    bash scripts/check-fork-drift.sh

# Verify every production extension source has a `//!` module head
check-headers:
    bash scripts/check-file-headers.sh

# Observational Rust-standards audit — appends to ISSUE.md, never blocks
audit-standards:
    bash scripts/audit-rust-standards.sh

# 300-line ceiling on extension sources (same script CI runs)
file-size:
    bash scripts/check-file-size.sh

# Every Cargo workspace in the repo. `tests/` and `bridge/` are excluded from
# the root workspace, so a bare root-level scan silently skips their lockfiles.
# Keep in sync with `git ls-files '*Cargo.lock'`.
workspaces := ". tests bridge"

# Detect unused dependencies across every workspace
machete:
    #!/usr/bin/env bash
    set -euo pipefail
    for w in {{ workspaces }}; do
        echo "==> cargo machete: $w"
        (cd "$w" && cargo machete)
    done

# Supply-chain gates across every workspace: cargo-deny (licenses/bans/
# advisories, root deny.toml discovered via --manifest-path) and cargo-audit
deny:
    #!/usr/bin/env bash
    set -euo pipefail
    for w in {{ workspaces }}; do
        echo "==> cargo deny: $w"
        cargo deny --manifest-path "${w%/}/Cargo.toml" check
    done

check-bans:
    cargo deny check bans

audit:
    #!/usr/bin/env bash
    set -euo pipefail
    for w in {{ workspaces }}; do
        echo "==> cargo audit: $w"
        cargo audit --file "${w%/}/Cargo.lock"
    done

# Build every feature powerset (catches feature-flag drift); weekly tier only
hack:
    cargo hack --workspace --feature-powerset --depth 2 check

# Structural guard: `UserId::admin()` is banned outside sanctioned call sites.
# The allowlist is empty by design — this repo has no sanctioned site; adding
# one requires justification in review.
lint-no-untyped-admin:
    #!/usr/bin/env bash
    set -euo pipefail
    hits=$(grep -rn 'UserId::admin()' extensions/ src/ bridge/src/ --include='*.rs' 2>/dev/null \
        | grep -v '/tests/' \
        || true)
    if [ -n "$hits" ]; then
        echo "lint-no-untyped-admin: untyped UserId::admin() outside the sanctioned call sites:"
        echo "$hits"
        exit 1
    fi

# Structural guard: no string-literal `UserId::new("...")` in extension code.
# String literals are how principal synthesis sneaks in — every legitimate
# UserId::new call takes a validated identifier as a variable, never a literal.
# Allowlisted: test code (regression tests intentionally construct ids) and
# any future bootstrap/provisioning module.
lint-no-synthesis:
    #!/usr/bin/env bash
    set -euo pipefail
    hits=$(grep -rEn 'UserId::new\("' extensions/ \
        --include='*.rs' \
        --exclude-dir=tests \
        --exclude-dir=bootstrap \
        || true)
    if [ -n "$hits" ]; then
        echo "error: forbidden synthesized principal — UserId::new with string literal"
        echo "$hits"
        echo
        echo "UserId::new must take a validated identifier (from cookie, query,"
        echo "JWT claim, or DB row), never a hard-coded literal. If this is"
        echo "legitimate bootstrap code, move it to extensions/**/bootstrap/."
        exit 1
    fi

# Prepare SQLx offline query cache (requires running database)
prepare:
    #!/usr/bin/env bash
    set -euo pipefail
    SECRETS_FILE="{{justfile_directory()}}/.systemprompt/profiles/local/secrets.json"
    if [ ! -f "$SECRETS_FILE" ]; then
        echo "Error: No local profile secrets found at $SECRETS_FILE"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    DB_URL=$(jq -r '.database_url // empty' "$SECRETS_FILE" 2>/dev/null)
    if [ -z "$DB_URL" ] || [ "$DB_URL" = "null" ]; then
        echo "Error: No database_url in secrets"
        exit 1
    fi
    PG_ISREADY=""
    if command -v pg_isready >/dev/null 2>&1; then PG_ISREADY="pg_isready"
    elif [ -x /opt/homebrew/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/opt/homebrew/opt/libpq/bin/pg_isready"
    elif [ -x /usr/local/opt/libpq/bin/pg_isready ]; then PG_ISREADY="/usr/local/opt/libpq/bin/pg_isready"
    fi
    if [ -z "$PG_ISREADY" ] || ! "$PG_ISREADY" -d "$DB_URL" -t 2 >/dev/null 2>&1; then
        echo "Error: Database not reachable at $DB_URL"
        echo "Run 'just db-up' first to start the database"
        exit 1
    fi
    # Apply pending migrations before sqlx prepare — otherwise the macros
    # see a schema older than the code references and fail with
    # "relation ... does not exist". Skipped if no CLI binary exists yet
    # (first-time bootstrap before any build).
    if [ -x "{{CLI}}" ]; then
        echo "Applying pending migrations..."
        {{CLI}} infra db migrate --profile local
    else
        echo "Warning: no systemprompt binary yet; skipping migrate step."
        echo "  If sqlx prepare fails with 'relation does not exist',"
        echo "  build first ('just build') then re-run 'just prepare'."
    fi
    echo "Preparing SQLx offline cache..."
    export DATABASE_URL="$DB_URL"
    # Drop the incremental artifacts of every crate that uses sqlx, so each
    # query macro re-expands against the freshly-migrated schema.
    #
    # This has to be all of them, not just the crate whose schema changed.
    # `cargo sqlx prepare` collects query data emitted by macro expansion, so
    # a crate cargo considers fresh contributes nothing to the run and its
    # queries are pruned from .sqlx as though they no longer existed. That is
    # what made prepare non-deterministic: a cold cache re-expanded everything
    # and kept the full set, while a warm one silently dropped whatever it did
    # not rebuild (the event_outbox queries from systemprompt-events being the
    # usual casualty). The emitted set must not depend on target/ state.
    #
    # Dependencies count too, not just workspace members — their queries land
    # in the workspace cache the same way.
    SQLX_PKGS=$(cargo metadata --format-version 1 2>/dev/null \
        | jq -r '.packages[] | select(.dependencies[]?.name == "sqlx") | .name' \
        | sort -u)
    if [ -z "$SQLX_PKGS" ]; then
        echo "Error: could not resolve the sqlx-dependent package list."
        echo "Without it, prepare would prune queries it simply did not rebuild."
        exit 1
    fi
    for pkg in $SQLX_PKGS; do
        cargo clean -p "$pkg" 2>/dev/null || true
    done
    # Workspace-level prepare (catches lib crates)
    cargo sqlx prepare --workspace
    # Second pass: the tests workspace builds systemprompt-web-admin with
    # `governance-ssr`, whose queries a default-feature prepare never expands
    # (and therefore prunes). prepare regenerates .sqlx wholesale, so the two
    # passes are unioned by hand — query filenames are content hashes, so a
    # plain copy merge cannot collide.
    cp -a .sqlx .sqlx.default-pass
    cargo sqlx prepare --workspace -- --package systemprompt-web-admin --features systemprompt-web-admin/governance-ssr
    cp -n .sqlx.default-pass/query-*.json .sqlx/ 2>/dev/null || true
    rm -rf .sqlx.default-pass
    # Per-crate prepare for binary/extension crates that cargo sqlx skips
    EXTENSION_DIRS="extensions/web extensions/mcp/shared extensions/mcp/systemprompt"
    for dir in $EXTENSION_DIRS; do
        if [ -f "{{justfile_directory()}}/$dir/Cargo.toml" ]; then
            # Skip crates with no sqlx dependency — prepare would only
            # resurrect an orphaned .sqlx cache.
            if ! grep -qE '^sqlx' "{{justfile_directory()}}/$dir/Cargo.toml"; then
                continue
            fi
            echo "  Preparing $dir..."
            (cd "{{justfile_directory()}}/$dir" && cargo sqlx prepare 2>&1 | tail -1) || true
            if ls "{{justfile_directory()}}/$dir/.sqlx/"*.json >/dev/null 2>&1; then
                cp "{{justfile_directory()}}/$dir/.sqlx/"*.json "{{justfile_directory()}}/.sqlx/"
            fi
        fi
    done
    echo "SQLx cache prepared successfully ($(ls {{justfile_directory()}}/.sqlx/ | wc -l) queries cached)"

# ══════════════════════════════════════════════════════════════════════════════
# SERVICES & DATABASE
# ══════════════════════════════════════════════════════════════════════════════

# Start server (always uses local profile)
start:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f .systemprompt/docker/local.yaml ]; then
        just db-up local
    fi
    exec {{CLI}} infra services start --profile local

# Optional: running server + binary provenance + recent build/lint/test results
[unix]
server-status:
    @scripts/server-state.sh report

# Start server with release binary
start-release:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f .systemprompt/docker/local.yaml ]; then
        just db-up local
    fi
    exec {{CLI_RELEASE}} infra services start --profile local

# Stop this clone's services
stop:
    {{CLI}} infra services stop --all

# Run migrations
migrate:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    {{CLI}} infra db migrate

# When an already-applied migration file is edited (e.g. a seed fix), its
# stored checksum stops matching the file and `migrate` / `start` refuse to
# proceed. `infra db migrate-repair` re-aligns the tracking table by dropping
# the drifted rows and re-applying those migrations — every migration is
# idempotent (guarded seeds or CREATE ... IF NOT EXISTS), so re-running them
# re-records the current checksum without touching your data.
# Repair migration checksum drift in place — no data loss, no destructive reset.
repair-migrations:
    {{CLI}} infra db migrate-repair --apply

# Per-clone docker compose project name. Derived from the absolute justfile directory
# so a second clone on the same host gets its own containers and volumes.
_project_name TENANT:
    #!/usr/bin/env bash
    set -euo pipefail
    HASH=$(printf '%s' "{{justfile_directory()}}" | { sha256sum 2>/dev/null || shasum -a 256; } | cut -c1-8)
    LEAF=$(basename "{{justfile_directory()}}" | tr '_' '-' | tr '[:upper:]' '[:lower:]' | sed 's/[^a-z0-9-]/-/g')
    printf 'sp-%s-%s-%s\n' "$LEAF" "$HASH" "{{TENANT}}"

# Start PostgreSQL for a specific tenant (default: local)
db-up TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml up -d

# Stop PostgreSQL for a specific tenant
db-down TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml down

# Show PostgreSQL logs for a specific tenant
db-logs TENANT="local":
    docker compose -p "$(just _project_name {{TENANT}})" -f .systemprompt/docker/{{TENANT}}.yaml logs -f

# List all tenant databases
db-list:
    @ls -1 .systemprompt/docker/*.yaml 2>/dev/null | xargs -I {} basename {} .yaml || echo "No tenant databases found"

# ══════════════════════════════════════════════════════════════════════════════
# AUTH & TENANT & PROFILE
# ══════════════════════════════════════════════════════════════════════════════

# Authenticate with SystemPrompt Cloud
login ENV="production":
    {{CLI}} cloud auth login {{ENV}}

# Clear saved credentials
logout:
    {{CLI}} cloud auth logout

# Show current user and tenant
whoami:
    {{CLI}} cloud auth whoami

# Tenant operations (interactive menu)
tenant:
    {{CLI}} cloud tenant

# Set up a local-only profile + Docker Postgres (no cloud, no login required).
# Pass keys as positional args, or leave blank to be prompted interactively:
#   just setup-local sk-ant-... sk-... AIza...
# Port and Postgres port can be overridden for running multiple clones on one host:
#   just setup-local sk-ant-... "" "" 8081 5433
# A bare re-run preserves the ports chosen at first setup (read back from the
# profile/compose files), so it never reverts an 8081/5436 install to 8080/5432.
# ADMIN_EMAIL defaults to `git config user.email`.
setup-local ANTHROPIC_KEY="" OPENAI_KEY="" GEMINI_KEY="" HTTP_PORT="8080" PG_PORT="5432" ADMIN_EMAIL="":
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{justfile_directory()}}"
    PROFILE_DIR="$ROOT/.systemprompt/profiles/local"
    DOCKER_DIR="$ROOT/.systemprompt/docker"
    ANTHROPIC_KEY="{{ANTHROPIC_KEY}}"
    OPENAI_KEY="{{OPENAI_KEY}}"
    GEMINI_KEY="{{GEMINI_KEY}}"
    HTTP_PORT="{{HTTP_PORT}}"
    PG_PORT="{{PG_PORT}}"
    ADMIN_EMAIL="{{ADMIN_EMAIL}}"
    if [ -z "$ADMIN_EMAIL" ]; then
        ADMIN_EMAIL="$(git config user.email 2>/dev/null || true)"
    fi
    if [ -z "$ADMIN_EMAIL" ]; then
        ADMIN_EMAIL="admin@localhost.localdomain"
    fi
    export SYSTEMPROMPT_PROFILE="$PROFILE_DIR/profile.yaml"
    # Default ports on a re-run mean "keep what I had", not "move me back to
    # 8080/5432": read the original choice back from the files setup wrote.
    if [ "$HTTP_PORT" = "8080" ] && [ -f "$PROFILE_DIR/profile.yaml" ]; then
        SAVED_HTTP="$(sed -n 's/^ *port: //p' "$PROFILE_DIR/profile.yaml" | head -1)"
        if [ -n "$SAVED_HTTP" ]; then
            HTTP_PORT="$SAVED_HTTP"
        fi
    fi
    if [ "$PG_PORT" = "5432" ] && [ -f "$DOCKER_DIR/local.yaml" ]; then
        SAVED_PG="$(sed -n 's/.*"\([0-9][0-9]*\):5432".*/\1/p' "$DOCKER_DIR/local.yaml" | head -1)"
        if [ -n "$SAVED_PG" ]; then
            PG_PORT="$SAVED_PG"
        fi
    fi
    # Whether a key was passed as a positional arg. When none is and there is
    # nothing to preserve, generation still needs a provider: on a TTY we let
    # `admin setup` drive its own "Select your AI provider" menu (the CLI owns
    # the prompt); off a TTY we cannot prompt, so keys must come as args. A
    # developer who keeps .systemprompt/ across reclones re-runs with no args
    # and is never asked again (the profile.yaml guard below skips generation).
    HAS_KEY=false
    if [ -n "$ANTHROPIC_KEY" ] || [ -n "$OPENAI_KEY" ] || [ -n "$GEMINI_KEY" ]; then
        HAS_KEY=true
    fi
    if [ "$HAS_KEY" = false ] && [ ! -f "$PROFILE_DIR/secrets.json" ] && [ ! -t 0 ]; then
        echo ""
        echo "================================================================"
        echo "  setup-local needs an AI provider API key"
        echo "================================================================"
        echo ""
        echo "  Not running on a TTY, so the provider menu can't be shown."
        echo "  Pass a key as an argument (one of Anthropic, OpenAI, Gemini):"
        echo "    just setup-local <anthropic_key> [openai_key] [gemini_key]"
        echo ""
        exit 1
    fi
    if [ ! -x target/debug/systemprompt ] && [ ! -x target/release/systemprompt ]; then
        echo "Building debug binary..."
        just build
    fi
    # Resolve the binary at runtime: the {{CLI}} variable is evaluated by `just`
    # at parse time, so on a cold clone (no binary yet) it expands to an error
    # stub — useless for the bootstrap/keygen calls below, which run only after
    # the build above has produced the binary.
    if [ -x target/release/systemprompt ]; then
        BIN="$ROOT/target/release/systemprompt"
    else
        BIN="$ROOT/target/debug/systemprompt"
    fi
    mkdir -p "$PROFILE_DIR" "$DOCKER_DIR"
    # Rewrite the compose file when it exists but pins a different host port.
    # Guarding only on existence meant a re-run with a new PG_PORT kept the old
    # mapping, brought Postgres up on the old port, and then waited 60s for the
    # new one before dying on "Postgres did not become ready" — which names the
    # symptom and hides the cause.
    if [ -f "$DOCKER_DIR/local.yaml" ] \
        && ! grep -q "\"${PG_PORT}:5432\"" "$DOCKER_DIR/local.yaml"; then
        echo "Docker compose pins a different host port; rewriting for $PG_PORT."
        echo "Recreating the container so the new mapping takes effect..."
        docker compose -p "$(just _project_name local)" -f "$DOCKER_DIR/local.yaml" down 2>/dev/null || true
        rm -f "$DOCKER_DIR/local.yaml"
    fi
    if [ ! -f "$DOCKER_DIR/local.yaml" ]; then
        echo "Writing Docker compose for local Postgres (host port $PG_PORT)..."
        cat > "$DOCKER_DIR/local.yaml" <<YAML
    services:
      postgres:
        image: postgres:18-alpine
        restart: unless-stopped
        environment:
          POSTGRES_USER: systemprompt
          POSTGRES_PASSWORD: 123
          POSTGRES_DB: systemprompt
        ports:
          - "${PG_PORT}:5432"
        volumes:
          - postgres_data:/var/lib/postgresql
        healthcheck:
          test: ["CMD-SHELL", "pg_isready -U systemprompt -d systemprompt"]
          interval: 5s
          timeout: 5s
          retries: 5
    volumes:
      postgres_data: {}
    YAML
    fi
    echo "Starting local Postgres via Docker..."
    just db-up local
    echo "Waiting for Postgres to accept connections on localhost:${PG_PORT}..."
    for i in $(seq 1 60); do
        if (exec 3<>/dev/tcp/127.0.0.1/${PG_PORT}) 2>/dev/null; then
            exec 3<&- 3>&-
            # Also confirm the server actually answers pg_isready, not just a half-open socket.
            CONTAINER=$(docker compose -p "$(just _project_name local)" -f .systemprompt/docker/local.yaml ps -q postgres)
            if [ -n "$CONTAINER" ] && docker exec "$CONTAINER" pg_isready -U systemprompt -d systemprompt >/dev/null 2>&1; then
                echo "Postgres is ready."
                break
            fi
        fi
        if [ "$i" = "60" ]; then
            echo "ERROR: Postgres did not become ready within 60s." >&2
            exit 1
        fi
        sleep 1
    done
    if [ ! -f "$PROFILE_DIR/profile.yaml" ]; then
        echo "Generating profile + provider registry + secrets via 'admin setup'..."
        if [ "$HAS_KEY" = true ]; then
            # Keys supplied as args: fully non-interactive. The default provider
            # is the first key given, so the generated config (the providers
            # registry, gateway default, ai/config.yaml) is consistent with the
            # single key.
            KEY_ARGS=()
            DEFAULT_PROVIDER=""
            if [ -n "$ANTHROPIC_KEY" ]; then KEY_ARGS+=(--anthropic-key "$ANTHROPIC_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=anthropic; fi
            if [ -n "$OPENAI_KEY" ]; then KEY_ARGS+=(--openai-key "$OPENAI_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=openai; fi
            if [ -n "$GEMINI_KEY" ]; then KEY_ARGS+=(--gemini-key "$GEMINI_KEY"); [ -z "$DEFAULT_PROVIDER" ] && DEFAULT_PROVIDER=gemini; fi
            "$BIN" admin setup --yes --no-migrate --environment local \
                --db-host localhost --db-port "$PG_PORT" \
                --db-user systemprompt --db-password 123 --db-name systemprompt \
                --default-provider "$DEFAULT_PROVIDER" \
                "${KEY_ARGS[@]}"
        else
            # No key arg: let the CLI prompt for which provider to use. DB,
            # environment, and migrations stay non-interactive (flags + env);
            # only the provider selection is interactive, and the chosen
            # provider becomes the default.
            SYSTEMPROMPT_NON_INTERACTIVE=1 "$BIN" admin setup --no-migrate --environment local \
                --db-host localhost --db-port "$PG_PORT" \
                --db-user systemprompt --db-password 123 --db-name systemprompt
        fi
        if [ "$HTTP_PORT" != "8080" ]; then
            "$BIN" admin config server set --port "$HTTP_PORT" \
                --api-server-url "http://localhost:${HTTP_PORT}" \
                --api-internal-url "http://localhost:${HTTP_PORT}" \
                --api-external-url "http://localhost:${HTTP_PORT}"
            # The authz hook URL is an absolute webhook target baked at
            # `admin setup` time on the default port; re-point it at the
            # chosen port so the gateway's govern callback reaches this server.
            "$BIN" admin config governance set --mode webhook \
                --url "http://localhost:${HTTP_PORT}/api/public/govern/authz"
            # jwt_issuer is baked on the default port too, and it is not
            # cosmetic: it is the base a client resolves
            # `{iss}/.well-known/jwks.json` against. Left at :8080 on a
            # non-default port, every external verifier (the bridge, Claude
            # Code) fetches the signing keys of whatever else is on :8080 —
            # or nothing — and rejects this server's tokens as minted under an
            # unknown authority.
            "$BIN" admin config security set \
                --jwt-issuer "http://localhost:${HTTP_PORT}"
            # Same for CORS: the seeded origins name :8080, so the admin UI
            # served from the chosen port is refused by its own API.
            "$BIN" admin config server cors add "http://localhost:${HTTP_PORT}" || true
            "$BIN" admin config server cors add "http://127.0.0.1:${HTTP_PORT}" || true
            "$BIN" admin config server cors remove "http://localhost:8080" || true
            "$BIN" admin config server cors remove "http://127.0.0.1:8080" || true
        fi
        # The generator binds 127.0.0.1, which no container can route to.
        # Containerized clients (`just claude`, clean-client) need the gateway
        # on 0.0.0.0 from the first start, not after a start-time remediation.
        "$BIN" admin config server set --host 0.0.0.0
    elif [ "$HAS_KEY" = true ]; then
        # Profile generation is one-shot, guarded on profile.yaml. `just db-down`
        # drops the database but leaves the profile, so a re-run with different
        # keys would silently keep the old provider registry. Say so loudly and
        # point at the one command that actually re-provisions.
        echo ""
        echo "================================================================"
        echo "  Existing profile reused — supplied keys were NOT applied"
        echo "================================================================"
        echo ""
        echo "  $PROFILE_DIR/profile.yaml already exists, so 'admin setup' was"
        echo "  skipped and the provider registry/keys are unchanged."
        echo "  To re-provision from the keys you just passed:"
        echo ""
        echo "    rm -rf \"$PROFILE_DIR\" && just setup-local <keys...> $HTTP_PORT $PG_PORT"
        echo ""
    fi
    mkdir -p "$ROOT/web/dist"
    echo "Building binaries (release, full workspace)..."
    just build --release
    echo "Running database migrations..."
    just migrate
    echo "Ensuring bootstrap admin user ($ADMIN_EMAIL)..."
    "$BIN" admin bootstrap --email "$ADMIN_EMAIL"
    if [ ! -f "$ROOT/signing_key.pem" ]; then
        echo "Generating JWT signing key..."
        "$BIN" admin keys generate --output "$ROOT/signing_key.pem"
    fi
    echo "Publishing assets..."
    just publish
    echo ""
    echo "Local setup complete. Run: just start"

# List all tenants
tenants:
    {{CLI}} cloud tenant list

# Profile operations (interactive menu)
profile:
    {{CLI}} cloud profile

# List all profiles
profiles:
    {{CLI}} cloud profile list

# ══════════════════════════════════════════════════════════════════════════════
# SYNC
# ══════════════════════════════════════════════════════════════════════════════

# Content and skills are ingested from services/ at server startup and by
# `just publish` (publish_pipeline job); there is no separate local sync command.

# Core 0.29.0 removed `cloud sync`. Pushing is `just deploy` (cloud deploy),
# and pulling is `cloud backup`, which downloads the tenant's runtime services/
# tree. The old sync-push / sync-pull recipes called a command that no longer
# exists, so they are gone rather than aliased to something they never were.

# Download the tenant's runtime services/ tree (--list to inspect first)
backup *ARGS:
    {{CLI}} cloud backup "$@"

# ══════════════════════════════════════════════════════════════════════════════
# DEPLOY
# ══════════════════════════════════════════════════════════════════════════════

# Build everything and deploy to cloud — one command, no preceding build step.
# Note: publish_pipeline runs automatically on server startup with correct profile URLs
# Pinned to the `production` profile so a deploy never follows whichever profile
# the CLI session happens to be switched to.
deploy *FLAGS: build-all
    {{CLI_RELEASE}} cloud deploy --profile {{DEPLOY_PROFILE}} {{FLAGS}}

# Pre-deploy preflight only — no build, no push
deploy-check:
    {{CLI}} cloud doctor --profile {{DEPLOY_PROFILE}}

# Check deployment status
status:
    {{CLI}} cloud status --profile {{DEPLOY_PROFILE}}

# ── Profile-aware deploys (any target, not just our pinned production) ───────
# `deploy` stays pinned to production on purpose. Operators deploying the same
# repo to their own infra (e.g. an Oracle VM with a remote Postgres) work with
# their own profile dir under .systemprompt/profiles/<name>/ and use these.

# Validate any profile without a running server: files, secrets keys, URLs,
# database reachability from this machine, and (with a built CLI) that the
# profile deserializes — the loader rejects unknown YAML keys. Add --live to
# also probe <api_external_url>/api/v1/health.
profile-check PROFILE="local" *FLAGS:
    @bash scripts/profile-check.sh {{PROFILE}} {{FLAGS}}

# Deploy to a cloud tenant under a non-default profile. Self-host targets do
# not cloud-deploy — use `just bundle` instead.
deploy-to PROFILE *FLAGS: build-all
    {{CLI_RELEASE}} cloud deploy --profile {{PROFILE}} {{FLAGS}}

# Package everything a self-host deployment needs (the same manifest the
# production Dockerfile ships) into dist/systemprompt-<profile>.tar.gz,
# including an UNPACK.md with the first-run steps (migrate, serve, probe).
bundle PROFILE:
    @bash scripts/bundle-profile.sh {{PROFILE}}

# Run the server in the foreground under any profile (self-host style).
serve PROFILE="local":
    SYSTEMPROMPT_PROFILE=.systemprompt/profiles/{{PROFILE}}/profile.yaml {{CLI}} infra services serve --foreground

# ══════════════════════════════════════════════════════════════════════════════
# MCP & BUILD ALL
# ══════════════════════════════════════════════════════════════════════════════

# Build all MCP servers (reads from manifest.yaml files) — single-flight and
# fingerprint-skipped, so a tree whose MCP servers already built returns
# immediately instead of re-paying the per-package rebuild.
build-mcp:
    @scripts/build-coordinator.sh run build-mcp "" -- {{just_executable()}} _build-mcp-uncoordinated

_build-mcp-uncoordinated:
    DATABASE_URL="$(just _db-url)" {{CLI}} build mcp --release

# Build CLI extensions. `systemprompt build mcp` only walks type: mcp manifests,
# so these need their own recipe or `plugins run <name>` finds no binary.
build-cli:
    @scripts/build-coordinator.sh run build-cli "" -- cargo build --release -p systemprompt-cli-salesforce

# Build everything for deployment (Rust binary + MCP servers + web assets)
build-all:
    just build --release
    just build-mcp
    just build-cli
    just bridge-package-host
    just downloads-fetch
    just web-build
    {{CLI_RELEASE}} infra jobs run publish_pipeline
    @echo "All components built"

# ══════════════════════════════════════════════════════════════════════════════
# WEB ASSETS & PUBLISHING
# ══════════════════════════════════════════════════════════════════════════════

# Copy web assets to dist (CSS, JS, images)
web-assets:
    {{CLI}} infra jobs run copy_extension_assets

# Publish: compile templates, bundle CSS/JS, copy assets, prerender content
publish:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_PROFILE:-}" ]; then
        export SYSTEMPROMPT_PROFILE="{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml"
    fi
    {{CLI}} infra jobs run publish_pipeline

# Stage setup-skill dashboard assets (skill assets ride into the Cowork VM)
cowork-setup-assets *ARGS:
    scripts/generate-cowork-setup-assets.py {{ARGS}}

# Build web assets only (templates + CSS + JS + copy to dist)
web-build:
    {{CLI}} infra jobs run bundle_admin_css
    {{CLI}} infra jobs run copy_extension_assets

# ══════════════════════════════════════════════════════════════════════════════
# DOCKER
# ══════════════════════════════════════════════════════════════════════════════

# Build Docker image for local testing
docker-build TAG="local":
    docker build -f Dockerfile -t systemprompt-template:{{TAG}} .

# Run image locally for testing
docker-run TAG="local":
    docker run -p 8080:8080 --env-file .env systemprompt-template:{{TAG}}

# Build the branded bridge. Its own standalone workspace, NOT the server
# workspace — `just build` does not touch it, and a bare `cargo build` from the
# repo root silently builds the server instead.
bridge-build *ARGS: core-checkout
    cd {{justfile_directory()}}/bridge && cargo build --release {{ARGS}}

# The client depends on systemprompt-bridge by relative path, and that crate is
# not published — so unlike the server, building the client needs the core
# repository checked out beside this one. Clones it when absent, fast-forwards
# it when present, and leaves local work alone.
core-checkout:
    #!/usr/bin/env bash
    set -euo pipefail
    CORE="{{justfile_directory()}}/../systemprompt-core"
    if [ -d "$CORE/.git" ]; then
        if [ -n "$(git -C "$CORE" status --porcelain)" ]; then
            echo "core checkout has local changes — leaving it as it is."
        else
            echo "Updating $CORE"
            git -C "$CORE" pull --ff-only --quiet || echo "warn: could not fast-forward core; using it as it is." >&2
        fi
    else
        echo "Cloning systemprompt-core beside this repo (the client needs it)."
        git clone --quiet https://github.com/systempromptio/systemprompt-core "$CORE"
    fi

# Package the branded bridge as a Linux release tarball into dist/
# Coordinated: bridge/ and the core sibling are both in the fingerprint, so a
# failed deploy retried on the same tree skips straight past this step.
bridge-package-linux:
    @scripts/build-coordinator.sh run bridge-package "" -- scripts/package-bridge-linux.sh

# Cross-compile the Windows bridge exe (x86_64-pc-windows-msvc via cargo-xwin —
# msvc is required: it statically links WebView2Loader, a -gnu build ships a
# bare exe that dies at start on "WebView2Loader.dll was not found") and stage
# it into storage/files/downloads/. Follow with `just publish`.
bridge-package-windows: core-checkout
    @scripts/build-coordinator.sh run bridge-package-windows "" -- scripts/package-bridge-windows.sh

# Build, sign, notarize, and stage the macOS bridge DMG (universal arm64 +
# x86_64) into storage/files/downloads/. macOS only — needs a "Developer ID
# Application" cert in the keychain plus a notarytool credential profile; the
# script header has the one-time store-credentials command. Follow with
# `just publish`.
[macos]
bridge-package-macos: core-checkout
    @scripts/build-coordinator.sh run bridge-package-macos "" -- scripts/package-bridge-macos.sh

# Backfill storage/files/downloads/ from the deployed host. The dir is
# gitignored and each platform's bridge binary can only be built on that
# platform's toolchain, so before a deploy every asset another machine produced
# must be fetched back — otherwise the freshly baked image 404s download links
# the site already serves. Local files always win over the remote copy.
downloads-fetch:
    @scripts/fetch-remote-downloads.sh

# Package whatever bridge this host's toolchain can build (build-all calls
# this; downloads-fetch then backfills the platforms this host cannot build).
[linux]
bridge-package-host: bridge-package-linux

[macos]
bridge-package-host: bridge-package-macos

[windows]
bridge-package-host:
    @echo "warn: no bridge packaging runs on Windows hosts - relying on downloads-fetch."

# Installs the client if it is not there yet; re-running it with a fresh code
# re-binds the machine to whoever that code belongs to.
# Point Claude Code on THIS host at the gateway (CODE comes from /admin/profile)
connect CODE GATEWAY="http://localhost:8080":
    curl -fsSL {{GATEWAY}}/files/downloads/install.sh | sh -s -- \
        --download-base {{GATEWAY}}/files/downloads --code {{CODE}}

# A code is needed the first time only: the credential it is exchanged for
# persists in a per-gateway volume, so later runs are just `just
# claude`. `just claude-reset` signs out and makes a code necessary again.
# Claude Code, connected, in a container (CODE from /admin/profile, first run only)
claude CODE="" GATEWAY="http://localhost:8080":
    #!/usr/bin/env bash
    set -euo pipefail

    # The page prints the gateway as a browser sees it. Inside a container
    # localhost is the container, so rewrite it to the host alias. Keep the
    # original too: reaching it from the host is what separates "the gateway is
    # down" from "the gateway is up but the container cannot route to it".
    HOST_GATEWAY="{{GATEWAY}}"
    GATEWAY="{{GATEWAY}}"
    GATEWAY="${GATEWAY//localhost/host.docker.internal}"
    GATEWAY="${GATEWAY//127.0.0.1/host.docker.internal}"

    # Scope the session to this clone AND its gateway, the same way the Docker
    # Postgres project name is scoped, so several checkouts coexist.
    #
    # Two things force it. A credential is only valid for the gateway that
    # issued it, so sharing one home makes a second gateway look like a broken
    # sign-in: the PAT is found, whoami fails against the wrong host, and
    # bootstrap drops to asking for a code. And a fixed container name would
    # make a second clone attach to the first clone's session instead of
    # starting its own.
    REPO_SLUG="$(basename "{{justfile_directory()}}" | sed 's|[^A-Za-z0-9]|-|g')"
    REPO_HASH="$(printf '%s' "{{justfile_directory()}}" | sha256sum | cut -c1-8)"
    GW_SLUG="$(printf '%s' "$GATEWAY" | sed -e 's|^https\?://||' -e 's|[^A-Za-z0-9]|-|g')"
    SCOPE="${REPO_SLUG}-${REPO_HASH}-${GW_SLUG}"
    VOL="astound-claude-${SCOPE}"
    NAME="astound-claude-${SCOPE}"

    if [ "$(docker inspect -f '{{{{.State.Running}}}}' "$NAME" 2>/dev/null)" = "true" ]; then
        echo "Already running for this repo — opening another session in it."
        exec docker exec -it "$NAME" bash -lc claude
    fi
    docker rm -f "$NAME" >/dev/null 2>&1 || true

    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    # The client is a separate workspace and depends on systemprompt-bridge by
    # relative path into a sibling core checkout, so building it needs that
    # checkout present. `bridge-build` fetches it. Do this before the code is
    # spent: a first-run compile can outlive the code's 10-minute TTL, so
    # `just bridge-build` belongs in setup rather than here.
    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    if [ ! -f "$BRIDGE" ] || [ ! -x "$BRIDGE" ]; then
        echo "Client not built yet — fetching core and building it."
        echo "warn: a first build takes minutes and the code expires in 10." >&2
        echo "      If it is rejected as expired, issue a fresh one and re-run." >&2
        just bridge-build
    fi

    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi

    # Prove the container can reach the gateway BEFORE the code is spent —
    # otherwise an unroutable gateway surfaces as a connect error with the code
    # already consumed. --entrypoint is required: the image's entrypoint would
    # otherwise swallow this as its own arguments, print its banner, and exit 0,
    # so the check would pass without making a request.
    gateway_reachable() {
        docker run --rm --entrypoint curl \
            --add-host host.docker.internal:host-gateway \
            astound-clean-client:local \
            -sf --max-time 5 "$GATEWAY/health" >/dev/null 2>&1
    }

    if ! gateway_reachable; then
        # Separate the two causes before touching anything. If the gateway does
        # not answer on the host either, it is down or on another port, and
        # rebinding it would be treating the wrong illness — Docker Desktop and
        # WSL2 proxy host.docker.internal to host loopback, so a 127.0.0.1 bind
        # is genuinely reachable there and the bind is often not the problem.
        if ! curl -sf --max-time 5 "$HOST_GATEWAY/health" >/dev/null 2>&1; then
            echo "ERROR: the gateway is not answering at $HOST_GATEWAY" >&2
            echo "" >&2
            echo "  It is not reachable from this host either, so it is down or" >&2
            echo "  listening on another port — not a container routing problem." >&2
            echo "  Start it, then re-run:" >&2
            echo "" >&2
            echo "      just start" >&2
            echo "      just server-status" >&2
            echo "" >&2
            echo "  Your code has not been used." >&2
            exit 1
        fi

        # The gateway is up on the host but the container cannot route to it.
        # On native Linux Docker that is exactly what a loopback bind causes,
        # and widening it is the documented remedy. Any other bind is left
        # alone rather than guessed at.
        if [ -x target/release/systemprompt ]; then
            SP="{{ justfile_directory() }}/target/release/systemprompt"
        else
            SP="{{ justfile_directory() }}/target/debug/systemprompt"
        fi
        # `show` prints its labels on stderr and its values on stdout, so the
        # human-readable form cannot be parsed by name. The JSON artifact can.
        # A parse failure yields an empty host, which matches nothing below and
        # so declines to remediate rather than guessing.
        BOUND_HOST="$("$SP" --json admin config server show --profile local 2>/dev/null \
            | python3 -c "import json,sys; d=json.load(sys.stdin); print(next((s.get('content','') for s in d.get('sections',[]) if s.get('heading')=='host'),''))" \
            2>/dev/null || true)"

        if [ "$BOUND_HOST" = "127.0.0.1" ] || [ "$BOUND_HOST" = "localhost" ]; then
            echo "notice: the gateway is bound to $BOUND_HOST, which no container can" >&2
            echo "        route to. Rebinding it to 0.0.0.0 and restarting." >&2
            echo "        This widens the gateway to your LAN — revert with:" >&2
            echo "            systemprompt admin config server set --host 127.0.0.1" >&2
            # Two things about this restart. `restart` needs an explicit target
            # — a bare `restart` exits 1 with "Must specify target (api, agent,
            # mcp)" — and the bind only affects the API listener, so restarting
            # `api` leaves the MCP servers up. And it SERVES in the foreground
            # rather than returning, so it has to be detached or it would hang
            # here forever; readiness is established by polling below, not by
            # the command exiting.
            "$SP" admin config server set --host 0.0.0.0 --profile local >/dev/null
            nohup "$SP" infra services restart api --profile local \
                >/dev/null 2>&1 &

            READY=0
            for _ in $(seq 1 45); do
                if gateway_reachable; then READY=1; break; fi
                sleep 1
            done
            if [ "$READY" -eq 0 ]; then
                echo "warn: the API server did not come back within 45s." >&2
                echo "      Check it with: just server-status" >&2
            fi
        fi
    fi

    if ! gateway_reachable; then
        echo "ERROR: the container cannot reach the gateway at $GATEWAY" >&2
        echo "" >&2
        echo "  The gateway answers on this host, so it is running — but the" >&2
        echo "  container cannot route to it, and it is bound to" >&2
        echo "  ${BOUND_HOST:-an unspecified host}, which rebinding did not fix." >&2
        echo "  Something between the two is in the way: a firewall on the" >&2
        echo "  docker bridge, or a docker network without host-gateway." >&2
        echo "" >&2
        echo "      just server-status" >&2
        echo "      curl -sf $HOST_GATEWAY/health      # from this host" >&2
        echo "" >&2
        echo "  Your code has not been used. Re-run this once the container can" >&2
        echo "  reach the gateway." >&2
        exit 1
    fi

    # No code on a repeat run: the stored credential is reused, and bootstrap
    # only falls back to asking when there is neither. Passing an empty value
    # would look like "a code was supplied" to that check.
    # Look for the credential itself, not merely the volume: a volume left over
    # from a run that never completed sign-in holds none, and bootstrap would
    # then drop to an interactive code prompt — which is what a non-interactive
    # caller sees as a hang.
    CODE_ENV=()
    if [ -n "{{CODE}}" ]; then
        CODE_ENV=(-e ASTOUND_BRIDGE_CODE="{{CODE}}")
    elif ! docker run --rm --entrypoint test \
            -v "$VOL":/home/tester \
            astound-clean-client:local \
            -f /home/tester/.config/astound/astound-bridge.pat >/dev/null 2>&1; then
        echo "ERROR: not signed in yet, and no code was given." >&2
        echo "" >&2
        echo "  A code is needed the first time. Take one from /admin/profile:" >&2
        echo "" >&2
        echo "      just claude <code>" >&2
        echo "" >&2
        echo "  Later runs need no code — the credential is kept." >&2
        exit 1
    fi

    exec docker run -it --rm \
        --name "$NAME" \
        --hostname "${REPO_SLUG:0:24}" \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="$GATEWAY" \
        "${CODE_ENV[@]}" \
        -e CLEAN_CLIENT_EXEC_CLAUDE=1 \
        -e CLEAN_CLIENT_ALLOW_STATE=1 \
        -v "$VOL":/home/tester \
        -v "$BRIDGE:/usr/local/bin/astound-bridge:ro" \
        -v "{{justfile_directory()}}/deploy/clean-client/bootstrap.sh:/usr/local/bin/bootstrap.sh:ro" \
        "${PORTS[@]}" \
        astound-clean-client:local /usr/local/bin/bootstrap.sh

# ──────────────────────────────────────────────────────────────────────────────
# CLEAN CLIENT — a config-free Linux box for testing the Claude Code + bridge
# integration. See deploy/clean-client/README.md.
# ──────────────────────────────────────────────────────────────────────────────

# Build the clean-client image (context is deploy/clean-client only — no repo state)
clean-client-build *ARGS:
    docker build {{ARGS}} -f deploy/clean-client/Dockerfile -t astound-clean-client:local deploy/clean-client

# Shell into a clean client. PERSIST=1 keeps the login across runs.
# GATEWAY overrides the gateway URL (default: this WSL host on :8080).
clean-client PERSIST="0" GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail
    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    # Mount the bridge you just built, read-only. Absent is not fatal: you can
    # still test Claude Code against the gateway without the bridge installed.
    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    MOUNTS=()
    # -f as well as -x: a bare `docker run -v` against a missing path makes
    # docker create a root-owned DIRECTORY there, and `[ -x dir ]` is true, so
    # an -x-only test would happily mount the directory and report a bridge that
    # is not there.
    if [ -d "$BRIDGE" ]; then
        echo "ERROR: $BRIDGE is a directory (docker created it from a stale -v mount)." >&2
        echo "       Remove it with: sudo rmdir '$BRIDGE'" >&2
        exit 1
    fi
    if [ -f "$BRIDGE" ] && [ -x "$BRIDGE" ]; then
        MOUNTS+=(-v "$BRIDGE:/usr/local/bin/astound-bridge:ro")
    else
        echo "warn: $BRIDGE not found — run 'cd bridge && cargo build --release' for the full flow." >&2
    fi

    # PERSIST keeps ~/.config/astound and ~/.claude in a named volume so a PAT
    # survives a restart. Off by default: a throwaway HOME is the point.
    if [ "{{PERSIST}}" = "1" ]; then
        MOUNTS+=(-v astound-clean-home:/home/tester -e CLEAN_CLIENT_ALLOW_STATE=1)
        echo "State persists in volume 'astound-clean-home' — 'just clean-client-reset' wipes it."
    fi

    # 8767 is the bridge's plugin-OAuth loopback port; it must be reachable from
    # a Windows browser. It is NOT published if your primary distro already
    # holds it, since that bind would just fail.
    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it. Plugin OAuth loopback will not work." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi

    # Note what is deliberately NOT here: no --env-file, no $HOME mounts, no
    # repo mount. The container must start from nothing.
    exec docker run -it --rm \
        --name astound-clean-client \
        --hostname clean-client \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        "${MOUNTS[@]}" "${PORTS[@]}" \
        astound-clean-client:local

alias cc := clean-client-ready

# Clean client, signed in and ready: paste one code, then type `claude`.
clean-client-ready GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail

    # Already running (another terminal has it): open a second shell in it
    # rather than failing on the name conflict.
    if [ "$(docker inspect -f '{{{{.State.Running}}}}' astound-clean-client 2>/dev/null)" = "true" ]; then
        echo "Container 'astound-clean-client' is already running — opening a shell in it."
        exec docker exec -it astound-clean-client bash -l
    fi

    # Stopped leftover (a crashed run, or one started without --rm): restart it
    # and shell in, so anything written outside the /home/tester volume survives.
    # Only if it refuses to start do we reclaim the name.
    if docker inspect astound-clean-client >/dev/null 2>&1; then
        echo "Container 'astound-clean-client' is stopped — restarting it."
        if docker start astound-clean-client >/dev/null 2>&1; then
            exec docker exec -it astound-clean-client bash -l
        fi
        echo "warn: it would not restart — removing it and starting fresh." >&2
        docker rm -f astound-clean-client >/dev/null
    fi

    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi

    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    if [ ! -f "$BRIDGE" ] || [ ! -x "$BRIDGE" ]; then
        echo "ERROR: $BRIDGE not found — run 'just bridge-build' first." >&2
        exit 1
    fi

    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi

    # State persists so a second run reuses the PAT instead of burning a code;
    # CLEAN_CLIENT_ALLOW_STATE tells the entrypoint that reuse is deliberate
    # here rather than host config leaking in.
    exec docker run -it --rm \
        --name astound-clean-client \
        --hostname clean-client \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        -e CLEAN_CLIENT_ALLOW_STATE=1 \
        -v astound-clean-home:/home/tester \
        -v "$BRIDGE:/usr/local/bin/astound-bridge:ro" \
        -v "{{justfile_directory()}}/deploy/clean-client/bootstrap.sh:/usr/local/bin/bootstrap.sh:ro" \
        "${PORTS[@]}" \
        astound-clean-client:local /usr/local/bin/bootstrap.sh

# End-to-end: run the published installer with a PAT and assert managed MCP (see script header for how to mint the PAT)
clean-client-install PAT GATEWAY="http://host.docker.internal:8080":
    GATEWAY="{{GATEWAY}}" scripts/clean-client-install.sh "{{PAT}}"

# Drops this clone's sessions and the credentials they stored, so the next run
# redeems a fresh code. ALL=1 signs out every clone on this host.
# Sign out of `just claude` (this repo; ALL=1 for every repo)
claude-reset ALL="0":
    #!/usr/bin/env bash
    set -euo pipefail
    # Scoped to this clone by default: signing every other checkout out because
    # one of them wanted a clean slate is not what anyone means by "reset".
    if [ "{{ALL}}" = "1" ]; then
        FILTER='name=^astound-claude-'
        echo "Signing out every repo on this host."
    else
        REPO_SLUG="$(basename "{{justfile_directory()}}" | sed 's|[^A-Za-z0-9]|-|g')"
        REPO_HASH="$(printf '%s' "{{justfile_directory()}}" | sha256sum | cut -c1-8)"
        FILTER="name=^astound-claude-${REPO_SLUG}-${REPO_HASH}-"
    fi
    docker ps -aq --filter "$FILTER" | while read -r c; do
        docker rm -f "$c" >/dev/null 2>&1 && echo "removed container $c"
    done
    FOUND=0
    for v in $(docker volume ls -q --filter "$FILTER"); do
        docker volume rm "$v" >/dev/null 2>&1 && { echo "removed $v"; FOUND=1; }
    done
    [ "$FOUND" = "1" ] || echo "Nothing to sign out of."
    echo "'just claude <code>' will start from nothing."

# Wipe the persisted clean-client state volume
clean-client-reset:
    -docker rm -f astound-clean-client 2>/dev/null
    -docker volume rm astound-clean-home
    @echo "Clean-client state wiped."

# Isolated dev sandbox on a real project: clean client + the repo mounted at
# /workspace/project. HOME stays virgin (device-link auth as usual); only the
# project directory crosses into the container. The image ships Playwright +
# Chromium so the dev_test skill works against the mounted project.
dev-sandbox REPO PERSIST="0" GATEWAY="http://host.docker.internal:8080":
    #!/usr/bin/env bash
    set -euo pipefail
    REPO_ABS="$(readlink -f "{{REPO}}")"
    if [ ! -d "$REPO_ABS" ]; then
        echo "ERROR: {{REPO}} is not a directory" >&2
        exit 1
    fi
    if ! docker image inspect astound-clean-client:local >/dev/null 2>&1; then
        echo "Image missing — building it first." >&2
        just clean-client-build
    fi
    BRIDGE="{{justfile_directory()}}/bridge/target/release/astound-bridge"
    MOUNTS=(-v "$REPO_ABS:/workspace/project")
    if [ -d "$BRIDGE" ]; then
        echo "ERROR: $BRIDGE is a directory (docker created it from a stale -v mount)." >&2
        echo "       Remove it with: sudo rmdir '$BRIDGE'" >&2
        exit 1
    fi
    if [ -f "$BRIDGE" ] && [ -x "$BRIDGE" ]; then
        MOUNTS+=(-v "$BRIDGE:/usr/local/bin/astound-bridge:ro")
    else
        echo "warn: $BRIDGE not found — run 'cd bridge && cargo build --release' for the full flow." >&2
    fi
    if [ "{{PERSIST}}" = "1" ]; then
        MOUNTS+=(-v astound-clean-home:/home/tester -e CLEAN_CLIENT_ALLOW_STATE=1)
        echo "State persists in volume 'astound-clean-home' — 'just clean-client-reset' wipes it."
    fi
    PORTS=()
    if ss -ltn 2>/dev/null | grep -q ':8767 '; then
        echo "warn: host port 8767 already in use — not publishing it. Plugin OAuth loopback will not work." >&2
    else
        PORTS+=(-p 127.0.0.1:8767:8767)
    fi
    exec docker run -it --rm \
        --name astound-dev-sandbox \
        --hostname dev-sandbox \
        --add-host host.docker.internal:host-gateway \
        -e ASTOUND_BRIDGE_GATEWAY_URL="{{GATEWAY}}" \
        "${MOUNTS[@]}" "${PORTS[@]}" \
        astound-clean-client:local

# Install the Playwright e2e suite's dependencies (playwright/ directory)
e2e-install:
    cd playwright && npm install && npx playwright install chromium

# Run the Playwright e2e suite against a running gateway (GATEWAY_URL env
# overrides the default http://localhost:8080). Not part of `just validate` —
# it needs a live stack: `just start` first.
e2e *ARGS:
    cd playwright && npx playwright test {{ARGS}}

# Seed deterministic e2e principals + analytics trail (idempotent; touches only
# e2e-*/@e2e.local rows). `--reset` deletes and re-creates exactly those rows.
e2e-seed *ARGS:
    cd playwright && npx tsx setup/seed.ts {{ARGS}}

# Capture the requirements evidence pack -- one named screenshot per REQ row,
# plus an index.md recording the URL and principal each came from -- into the
# gitignored playwright/screenshots/<stamp>/ directory (needs `just start`).
e2e-screens:
    cd playwright && npx tsx scripts/capture.ts

# Prove one requirements-register row end to end (see
# requirements/compliance-register.md). Needs `just start` and `just e2e-seed`.
e2e-req REQ:
    cd playwright && npx playwright test --grep "@{{REQ}}"

# Test build without pushing
docker-test:
    just build-all
    just docker-build test
    @echo "Docker build successful! Image: systemprompt-template:test"

# ══════════════════════════════════════════════════════════════════════════════
# ADMIN & PLUGINS
# ══════════════════════════════════════════════════════════════════════════════

# Generate WebAuthn setup token for admin user
webauthn-admin EMAIL:
    {{CLI}} admin users webauthn generate-setup-token --email "{{EMAIL}}"

# Update Anthropic official plugins from vendor submodule and reimport
update-anthropic-plugins:
    git submodule update --remote vendor/knowledge-work-plugins
    {{CLI}} infra jobs run import_anthropic_plugins

# ══════════════════════════════════════════════════════════════════════════════
# BENCHMARKS
# ══════════════════════════════════════════════════════════════════════════════

# Benchmark governance endpoint. Downloads `hey` for the host OS/arch on first run.
benchmark REQUESTS="200" CONCURRENCY="100":
    #!/usr/bin/env bash
    set -e
    # Use system hey if available, else /tmp/hey
    if command -v hey >/dev/null 2>&1; then
        HEY="$(command -v hey)"
    else
        HEY="/tmp/hey"
    fi
    # Re-download if the cached binary can't execute here (e.g. Linux hey on a Mac).
    if ! { [[ -x "$HEY" ]] && "$HEY" --help >/dev/null 2>&1; }; then
        rm -f "$HEY"
        HEY="/tmp/hey"
        OS_ARCH="$(uname -s)/$(uname -m)"
        case "$OS_ARCH" in
            Darwin/*)
                HEY_URL="https://hey-release.s3.us-east-2.amazonaws.com/hey_darwin_amd64"
                echo "Installing hey from $HEY_URL..."
                curl -fsSL "$HEY_URL" -o "$HEY" && chmod +x "$HEY"
                ;;
            Linux/x86_64|Linux/amd64)
                HEY_URL="https://hey-release.s3.us-east-2.amazonaws.com/hey_linux_amd64"
                echo "Installing hey from $HEY_URL..."
                if ! curl -fsSL "$HEY_URL" -o "$HEY"; then
                    echo "ERROR: failed to download hey. Run: sudo apt-get install hey" >&2; exit 1
                fi
                chmod +x "$HEY"
                ;;
            *) echo "ERROR: no prebuilt hey for $OS_ARCH. Install with 'brew install hey' or 'go install github.com/rakyll/hey@latest'." >&2; exit 1 ;;
        esac
        if ! "$HEY" --help >/dev/null 2>&1; then
            echo "ERROR: hey won't run on $OS_ARCH." >&2
            if [[ "$OS_ARCH" == "Darwin/arm64" ]]; then
                echo "       Apple Silicon: 'softwareupdate --install-rosetta' or 'brew install hey'." >&2
            else
                echo "       Install manually: 'sudo apt-get install hey' or 'go install github.com/rakyll/hey@latest'." >&2
            fi
            rm -f "$HEY"; exit 1
        fi
    fi
    TOKEN_FILE="demo/.token"
    if [[ ! -f "$TOKEN_FILE" ]]; then
        echo "ERROR: No token. Run: ./demo/00-preflight.sh" >&2
        exit 1
    fi
    TOKEN=$(cat "$TOKEN_FILE")
    echo "Governance endpoint: {{REQUESTS}} requests, {{CONCURRENCY}} concurrent"
    echo ""
    "$HEY" -n {{REQUESTS}} -c {{CONCURRENCY}} -m POST \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"bench","tool_input":{"file_path":"/src/main.rs"}}' \
        "http://localhost:8080/api/public/hooks/govern?plugin_id=enterprise-demo"


# --- Release ------------------------------------------------------------

# Step A of a release: bump every version pin to the new core release and
# gate locally (migrate + build + clippy). Review + commit + push, then
# `just release <version>`. See docs/RELEASING.md.
core-bump version:
    @! grep -q '^\[patch\.crates-io\]' Cargo.toml || (echo "ERROR: [patch.crates-io] is active — publish core and re-comment it first" && exit 1)
    scripts/sync-release-version.sh {{version}}
    cargo update -w
    just db-up
    cargo run --bin systemprompt -- infra db migrate || true
    just build
    just clippy
    @echo "core-bump {{version}} complete — review the diff, run tests, commit to main, push, then: just release {{version}}"

# Step B: tag the release. Nothing downstream is automatic — this repo runs no
# hosted CI, so the tag is a marker and the artifacts are built here by hand
# (just build-all, just docker-build) when a release actually needs shipping.
release version:
    @test -z "$(git status --porcelain)" || (echo "ERROR: working tree not clean" && exit 1)
    git fetch origin main
    @test "$(git rev-parse HEAD)" = "$(git rev-parse origin/main)" || (echo "ERROR: HEAD != origin/main — push first" && exit 1)
    scripts/sync-release-version.sh {{version}} --check
    just verify
    git tag "v{{version}}"
    git push origin "v{{version}}"
    @echo "v{{version}} tagged and pushed. No CI will pick it up — build artifacts locally with 'just build-all'."
