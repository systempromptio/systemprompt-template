# systemprompt-template
set dotenv-load
# Without this, `just cli ... --full-name "Test User"` word-splits the quoted
# value into two arguments before the CLI ever parses it.
set positional-arguments

CLI_RELEASE := "target/release/systemprompt"

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
    @if [ -n "$SYSTEMPROMPT_PROFILE" ] && [ -f "$SYSTEMPROMPT_PROFILE" ]; then \
        PROFILE_DIR="$(dirname "$SYSTEMPROMPT_PROFILE")"; \
        SECRETS_PATH="$(yq -r '.secrets.secrets_path // "./secrets.json"' "$SYSTEMPROMPT_PROFILE")"; \
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

# Build (Unix) - one build in flight at a time, always of the latest source;
# cargo's own incremental cache decides how much recompiles. Refuses when the
# target/ volume has less than BUILD_MIN_FREE_GB (default 25) GB free.
[unix]
build *FLAGS:
    @scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

# What is the build/lint/test state right now? Read this before running anything.
[unix]
build-status *RECIPE:
    @scripts/build-coordinator.sh status {{RECIPE}}

# Kept for muscle memory: `just build` always compiles the current tree (there
# is no success cache), so this is the same recipe.
[unix]
build-force *FLAGS:
    @scripts/build-coordinator.sh run build "{{FLAGS}}" -- {{just_executable()}} _build-uncoordinated {{FLAGS}}

# The real build. Call `just build` instead - this one skips coordination.
[unix]
_build-uncoordinated *FLAGS:
    #!/usr/bin/env bash
    set -euo pipefail
    export CC="${CC:-clang}"
    export CXX="${CXX:-clang++}"
    export RUSTFLAGS="${RUSTFLAGS:--D warnings}"
    SQLX_OFFLINE=true cargo build --workspace --locked {{FLAGS}}

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
    export CC="${CC:-clang}"
    export CXX="${CXX:-clang++}"
    SQLX_OFFLINE=true cargo clippy --workspace --all-targets --locked {{FLAGS}} -- -D warnings

# Unit tests: extensions/web/admin (main workspace) + the tests/ workspace.
# If sqlx offline errors appear, run `just prepare` first to refresh .sqlx.
test-unit:
    @scripts/build-coordinator.sh run test-unit "" -- {{just_executable()}} _test-unit-uncoordinated

# Why: without --no-fail-fast nextest stops at the first failing test, so a
# Gates round reports one finding and the next waits for another round. Every
# tier runs to completion and lists every failure at once.
#
# Why: the tests/ tiers pass --workspace and pick their crates with a nextest
# filter rather than -p. A -p set unifies features for that set alone, so each
# tier recompiled systemprompt-web-admin and everything above it; --workspace
# unifies once and the later tiers reuse the build.
_test-unit-uncoordinated:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    cargo nextest run --locked --no-fail-fast --no-tests=pass -p systemprompt-web-admin --tests || failed+=(web-admin)
    cargo nextest run --locked --no-fail-fast --no-tests=pass -p systemprompt-web-extension --tests || failed+=(web-extension)
    cargo nextest run --locked --no-fail-fast --manifest-path tests/Cargo.toml --workspace -E 'package(mcp-unit-tests) | package(web-unit-tests)' || failed+=(tests-workspace)
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::test-unit failed: %s\n' "${failed[@]}"; exit 1; fi

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
    # Why: falling through with no URL hands the suites a reason to skip, and a
    # skipped DB tier reports the same green as one that ran every assertion.
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ]; then
        echo "No test database. Set SYSTEMPROMPT_TEST_DATABASE_URL, or run \`just setup-local\`" >&2
        echo "so .systemprompt/profiles/local/secrets.json carries a database_url." >&2
        exit 1
    fi
    cargo nextest run --locked --no-fail-fast --manifest-path tests/Cargo.toml --workspace -E 'package(mcp-integration-tests) | package(web-integration-tests) | package(admin-db-core-tests) | package(admin-db-config-tests) | package(schema-upgrade-tests)'

# HTTP contract suite: drives every admin route under three principals and
# diffs the result against tests/contract/admin/baseline.txt. Same throwaway-
# database convention as test-integration. A status change fails the run; if
# it is deliberate, re-record with UPDATE_CONTRACT_BASELINE=1 and list it in
# the PR.
test-contract *ARGS:
    @scripts/build-coordinator.sh run test-contract "$*" -- {{just_executable()}} _test-contract-uncoordinated "$@"

_test-contract-uncoordinated *ARGS:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
        SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 -c "
    import json, urllib.parse as up
    u = up.urlsplit(json.load(open('.systemprompt/profiles/local/secrets.json'))['database_url'])
    print(up.urlunsplit((u.scheme, u.netloc, '/postgres', '', '')))")
        export SYSTEMPROMPT_TEST_DATABASE_URL
    fi
    if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ]; then
        echo "No test database. Set SYSTEMPROMPT_TEST_DATABASE_URL, or run \`just setup-local\`" >&2
        echo "so .systemprompt/profiles/local/secrets.json carries a database_url." >&2
        exit 1
    fi
    # Why: the contract suite self-skips when no database is reachable; this
    # turns that skip into a failure, so a missing database never reads as a pass.
    export SYSTEMPROMPT_REQUIRE_DB=1
    cargo nextest run --locked --no-fail-fast --manifest-path tests/Cargo.toml --workspace -E 'package(admin-contract-tests)' "$@"

# All tests. Every tier runs even after one fails: as `just` dependencies they
# stopped at the first red tier, hiding the next tier's failures for a round.
test:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    for tier in test-unit test-integration test-contract; do
        echo "==> $tier"
        {{just_executable()}} "$tier" || failed+=("$tier")
    done
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::test failed: %s\n' "${failed[@]}"; exit 1; fi

# Reject tests that return early on a missing prerequisite without saying so
lint-silent-skips:
    ./scripts/lint-silent-skips.sh tests

# Source gates ported from systemprompt-core (scripts/*.sh)
lint-gates:
    @scripts/build-coordinator.sh run lint-gates "" -- {{just_executable()}} _lint-gates-uncoordinated

# Gates are independent read-only checks; they run concurrently and every
# failure is reported, so one red gate cannot hide the rest.
_lint-gates-uncoordinated:
    #!/usr/bin/env bash
    set -uo pipefail
    gates=(
        check-discarded-results.sh
        check-fail-open.sh
        lint-schema.sh
        lint-extensions.sh
        check-migration-numbers.sh
        lint-layers.sh
        lint-repo-construction.sh
        check-json-value.sh
        check-sqlx.sh
        check-http-errors.sh
        check-test-value.sh
        lint-silent-skips.sh
        lint-raw-ids.sh
        check-glob-reexports.sh
        check-comments.sh
        lint-inline-comments.sh
        check-duplicate-types.sh
        check-field-copy-from.sh
        check-repository-naming.sh
        check-admin-template-links.sh
        check-admin-template-assets.sh
        # admin-css-classes + frontend-standards now run as cargo tests in
        # tests/unit/web/src/ (admin_css_classes.rs, frontend_standards.rs).
        check-spec-shape.sh
        check-template-fields.sh
        check-dead-repository-code.sh
        check-file-headers.sh
        check-file-size.sh
        check-asset-reachability.sh
        check-workspace-deps.sh
        check-dockerfile-paths.sh
        check-dropped-schema.sh
        validate-services.sh
        check-release-tag.sh
        check-core-ref.sh
        coverage-badge.sh
        check-docs-version.sh
        # check-schema-baseline.sh joins this list with the first ladder rung
        # (tests/fixtures/schema/release-baseline-0.61.0.sql); until then it
        # has nothing to check and would only be red. release.sh runs it
        # regardless, so no release can go out without the rung.
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

# The whole gate, in one command — exactly what .github/workflows/gates.yml
# runs on every push to next and on ordinary PRs (the static, lint and test
# tiers; the browser tier is `just e2e-gate`). Run it before you push so CI is
# confirmation, not discovery. `preflight` adds the coverage floor/ratchet.
verify: preflight-static preflight-lint test
    @echo "verify: format, sqlx cache, lint gates, clippy, docs, msrv, and tests all pass"

# ══════════════════════════════════════════════════════════════════════════════
# PREFLIGHT (local stand-in for CI — tiered, cheapest first)
# ══════════════════════════════════════════════════════════════════════════════

# Everything: static gates → lint/doc/msrv → tests → coverage floor+ratchet.
preflight: preflight-static preflight-lint test coverage-check

# Tier 0 — seconds, no compile. Formatting, sqlx cache freshness, pins, the
# release-script self-tests, deploy config, and the source gates. The gates
# run UNCOORDINATED here: they are read-only, so queueing them on the build
# lock only made this hang behind whoever was mid test run.
#
# Every check runs even after one fails, and the recipe fails if any did: a
# formatting slip must not hide thirty source gates until the next round.
preflight-static:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    check() { echo "==> $1"; shift; "$@" || failed+=("$*"); }
    version="$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -1)"
    check "fmt (root)" cargo fmt --all -- --check
    check "fmt (tests)" cargo fmt --manifest-path tests/Cargo.toml --all -- --check
    check "sqlx cache" bash scripts/check-sqlx-cache.sh
    check "release version pins" bash scripts/sync-release-version.sh "$version" --check
    check "core version pins" bash scripts/sync-core-version.sh --check
    check "core crate versions" bash scripts/check-core-crate-versions.sh
    check "release proof self-test" bash tests/scripts/release-proof.sh
    check "release merge self-test" bash tests/scripts/release-merge.sh
    check "release helper self-test" bash tests/scripts/release-helper.sh
    check "deploy config" python3 scripts/check-deploy-config.py
    check "docker migrate-profile" python3 docker/test_migrate_profile.py
    check "docker container-state" python3 docker/test_container_state.py
    check "source gates" {{just_executable()}} _lint-gates-uncoordinated
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::preflight-static failed: %s\n' "${failed[@]}"; exit 1; fi

# Tier 1 — compilers. Clippy, rustdoc as errors, MSRV. Each runs even after
# one fails, as in preflight-static.
preflight-lint:
    #!/usr/bin/env bash
    set -uo pipefail
    failed=()
    for recipe in clippy doc-check msrv-check; do
        echo "==> $recipe"
        {{just_executable()}} "$recipe" || failed+=("$recipe")
    done
    if [ "${#failed[@]}" -gt 0 ]; then printf '::error::preflight-lint failed: %s\n' "${failed[@]}"; exit 1; fi

# Weekly deep pass: preflight plus the network-touching supply-chain gates.
preflight-full: preflight deny audit machete hack

# Rustdoc with warnings denied (root workspace, as quality.yml ran it).
# Single-flight coordinated.
doc-check:
    @scripts/build-coordinator.sh run doc-check "" -- {{just_executable()}} _doc-check-uncoordinated

_doc-check-uncoordinated:
    SQLX_OFFLINE=true RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked

# Both workspaces must build on the declared minimum supported Rust version,
# and must declare the same one. The number is read from the manifests, never
# hardcoded — see scripts/check-msrv.sh for why that matters.
msrv-check:
    @scripts/build-coordinator.sh run msrv-check "" -- bash scripts/check-msrv.sh

# ══════════════════════════════════════════════════════════════════════════════
# COVERAGE (raw llvm-cov; floor + ratchet vs tracked coverage/baseline.json)
# ══════════════════════════════════════════════════════════════════════════════

# Instrumented test run over both workspaces; writes coverage-report/.
# See scripts/coverage.sh for the sccache/mold neutralisation notes.
coverage:
    @scripts/build-coordinator.sh run coverage "" -- bash scripts/coverage.sh

# Enforce the floor and ratchet recorded in coverage/baseline.json.
coverage-check: coverage
    bash scripts/coverage-check.sh

# Re-record coverage/baseline.json at the measured value (deliberate act —
# commit the result, then `just coverage-badge`). Raise "floor" by hand.
coverage-baseline: coverage
    UPDATE_BASELINE=1 bash scripts/coverage-check.sh

# Rewrite the README's coverage badge from coverage/baseline.json. The
# `coverage-badge.sh --check` gate fails the build if the two disagree.
coverage-badge:
    bash scripts/coverage-badge.sh --write

# Browsable HTML tree from the last `just coverage` run (GNU find required).
coverage-html:
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="$(pwd)"
    if [ ! -f "$ROOT/coverage-report/tests.profdata" ]; then
        echo "Run 'just coverage' first" >&2
        exit 1
    fi
    FIND=find; command -v gfind >/dev/null 2>&1 && FIND=gfind
    TOOLDIR="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
    TBASE="${COVERAGE_TARGET_DIR:-$ROOT/coverage-report/target}"
    BINS=$(for t in "$TBASE-tests" "$TBASE-root"; do \
        "$FIND" "$t/debug/deps" -maxdepth 1 -executable -type f ! -name '*.d' ! -name '*.so' -printf '%T@ %p\n' 2>/dev/null; \
    done | sort -rn | awk '{ base=$2; sub(".*/", "", base); sub(/-[0-9a-f]+$/, "", base); if (!seen[base]++) print $2 }')
    OBJ_ARGS=""
    for b in $BINS; do OBJ_ARGS="$OBJ_ARGS --object $b"; done
    ROOT_RE="$(printf '%s' "$ROOT" | sed 's/[][\.*^$()+?{}|]/\\&/g')"
    mkdir -p "$ROOT/coverage-report/html"
    "$TOOLDIR/llvm-cov" show \
        --instr-profile="$ROOT/coverage-report/tests.profdata" \
        $OBJ_ARGS \
        --ignore-filename-regex="(\.cargo|/rustc/|/registry/|/debug/build/|/tests/|/target/|systemprompt-core/|\.vendor/|${ROOT_RE}/src/(main|lib)\.rs|extensions/(cli|mcp)/[^/]+/src/main\.rs|extensions/cli/[^/]+/src/commands/|extensions/.*/extension\.rs|build\.rs)" \
        --format=html \
        --output-dir="$ROOT/coverage-report/html"
    echo "Coverage report: coverage-report/html/index.html"

# Remove all coverage artifacts (instrumented target dirs included).
# Refuses while a coordinated run holds the lock: coverage-report/ carries the
# instrumented test binaries, so deleting it mid-run makes every remaining test
# fail to exec and the report come out at 0.00% — a failure that looks like a
# code regression and is not.
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

# Record tests/fixtures/schema/release-baseline-<version>.sql: the schema a
# fresh install of a release produces, plus its extension_migrations rows,
# dumped by the local Postgres container's own pg_dump (always the server's
# major — the host client may be older and refuse). With no argument the
# current tree is installed under the workspace version — run it after every
# version bump (scripts/check-schema-baseline.sh enforces that). With a
# version, that release's gateway tarball is fetched and ITS binary does the
# install (linux-amd64, linux-arm64 or darwin-arm64), so a rung can be added
# for a release that shipped before the ladder existed. The upgrade test
# restores every rung and runs the current installer over it; the ladder is
# append-only.
schema-baseline VERSION="":
    #!/usr/bin/env bash
    set -euo pipefail
    version="{{VERSION}}"
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    if [ -n "$version" ]; then
        case "$(uname -s)-$(uname -m)" in
            Linux-x86_64) target=linux-amd64 ;;
            Linux-aarch64|Linux-arm64) target=linux-arm64 ;;
            Darwin-arm64) target=darwin-arm64 ;;
            *) echo "schema-baseline: no gateway tarball for $(uname -s)-$(uname -m)" >&2; exit 1 ;;
        esac
        name="systemprompt-gateway-$version-$target.tar.gz"
        echo "schema-baseline: downloading $name"
        gh release download "v$version" -R systempromptio/systemprompt-template -p "$name" -p SHA256SUMS.gateway -D "$tmp" \
            || { echo "schema-baseline: release v$version has no $name" >&2; exit 1; }
        (cd "$tmp" && grep " $name\$" SHA256SUMS.gateway | { sha256sum -c - 2>/dev/null || shasum -a 256 -c -; } >/dev/null)
        tar xzf "$tmp/$name" -C "$tmp"
        cli="$tmp/${name%.tar.gz}/systemprompt"
        core_ref="v$(sed -n 's/^systemprompt = { version = "\([0-9.]*\)".*/\1/p' Cargo.toml | head -1)"
    else
        just build
        version="$(awk '/^\[workspace\.package\]/{p=1;next}/^\[/{p=0}p&&/^version[[:space:]]*=/{gsub(/[[:space:]"]/,""); sub(/^version=/,""); print; exit}' Cargo.toml)"
        core_ref="$(tr -d '[:space:]' < bridge/CORE_REF)"
        cli="{{CLI}}"
    fi
    container="$(docker compose -p "$(just _project_name local)" -f .systemprompt/docker/local.yaml ps -q postgres)"
    [ -n "$container" ] || { echo "schema-baseline: local Postgres is not running (just db-up)" >&2; exit 1; }
    base_url="$(jq -r '.database_url' .systemprompt/profiles/local/secrets.json)"
    scratch="sp_schema_baseline_$$"
    scratch_url="${base_url%/*}/$scratch"
    fixture="tests/fixtures/schema/release-baseline-$version.sql"
    mkdir -p "$(dirname "$fixture")"
    cleanup() { docker exec "$container" psql -U systemprompt -d postgres -qc "DROP DATABASE IF EXISTS \"$scratch\" WITH (FORCE)" >/dev/null 2>&1 || true; rm -rf "$tmp"; }
    trap cleanup EXIT
    docker exec "$container" psql -U systemprompt -d postgres -qc "CREATE DATABASE \"$scratch\""
    echo "schema-baseline: fresh install of $version (core $core_ref) into $scratch"
    if ! log="$(SYSTEMPROMPT_DATABASE_URL="$scratch_url" "$cli" infra db migrate --profile local 2>&1)"; then
        echo "$log" | tail -30; echo "schema-baseline: fresh install failed" >&2; exit 1
    fi
    {
        echo "-- systemprompt-template release-baseline: $version (core $core_ref)"
        echo "-- Recorded by 'just schema-baseline' from a fresh install; the upgrade test"
        echo "-- restores it and migrates forward. Re-record after every version bump."
        docker exec "$container" pg_dump -U systemprompt --schema-only --no-owner --no-privileges --no-comments "$scratch"
        docker exec "$container" pg_dump -U systemprompt --data-only --inserts --no-owner --table=extension_migrations "$scratch"
    } | grep -v -e '^\\' -e "set_config('search_path'" > "$fixture"
    echo "schema-baseline: wrote $fixture ($(wc -l < "$fixture") lines)"
    git diff --stat -- "$fixture" | tail -1
    bash scripts/check-schema-baseline.sh

# Point git at the tracked hooks (pre-commit patch-marker guard + fast gates).
# There is deliberately NO pre-push hook: pushes to next are gated by CI
# (gates.yml). Run once per clone.
init-hooks:
    git config core.hooksPath .githooks
    @echo "git hooks now sourced from .githooks/"

# Cross-file referential integrity for services/ (ACL entity ids, MCP ports)
validate:
    bash scripts/validate-services.sh

# Verify every production extension source has a `//!` module head
check-headers:
    bash scripts/check-file-headers.sh

# Observational Rust-standards audit — appends to ISSUE.md, never blocks
audit-standards:
    bash scripts/audit-rust-standards.sh

# 300-line ceiling on extension sources (same script CI runs)
file-size:
    bash scripts/check-file-size.sh

# Every Cargo workspace in the repo. `tests/` is excluded from the root
# workspace, so a bare root-level scan silently skips its lockfile. Keep in
# sync with `git ls-files '*Cargo.lock'`.
workspaces := ". tests"

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
    hits=$(grep -rn 'UserId::admin()' extensions/ src/ --include='*.rs' 2>/dev/null \
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
    # Per-crate prepare for binary/extension crates that cargo sqlx skips.
    #
    # These caches are SCRATCH, not artefacts: each crate's queries are copied
    # into the root .sqlx below and the per-crate directory is then removed.
    # Every crate here is a root workspace member, so the root cache is the one
    # the macros resolve against — `extensions/mcp/systemprompt` is a member
    # that has never had its own cache and builds fine, and a workspace-wide
    # `SQLX_OFFLINE=true cargo check --all-targets` passes with them deleted.
    # Leaving them on disk committed a duplicate copy of the root cache, which
    # is why a routine prepare showed up as a large unrelated diff.
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
            # Scratch, not an artefact — the queries now live in the root cache.
            rm -rf "{{justfile_directory()}}/$dir/.sqlx"
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

# Stop this clone's services (clean shutdown; avoids orphaned MCP children)
stop:
    {{CLI}} infra services stop --all

# Start server with release binary
start-release:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ -f .systemprompt/docker/local.yaml ]; then
        just db-up local
    fi
    exec {{CLI_RELEASE}} infra services start --profile local

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
# profile/compose files), so it never reverts a non-default install to 8080/5432.
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
                --admin-email "$ADMIN_EMAIL" \
                --default-provider "$DEFAULT_PROVIDER" \
                "${KEY_ARGS[@]}"
        else
            # No key arg: let the CLI prompt for which provider to use. DB,
            # environment, and migrations stay non-interactive (flags + env);
            # only the provider selection is interactive, and the chosen
            # provider becomes the default.
            SYSTEMPROMPT_NON_INTERACTIVE=1 "$BIN" admin setup --no-migrate --environment local \
                --db-host localhost --db-port "$PG_PORT" \
                --db-user systemprompt --db-password 123 --db-name systemprompt \
                --admin-email "$ADMIN_EMAIL"
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
            # Tokens carry the issuer, and a validator resolves (issuer, kid)
            # by fetching that issuer's JWKS. Left on the default port it
            # points at whatever else owns 8080, so every MCP tool call fails
            # with "kid does not match any known signing key".
            "$BIN" admin config security set --jwt-issuer "http://localhost:${HTTP_PORT}"
            # CORS is seeded on the default port too, so the admin UI served
            # from the chosen port is refused by its own API.
            "$BIN" admin config server cors add "http://localhost:${HTTP_PORT}" || true
            "$BIN" admin config server cors add "http://127.0.0.1:${HTTP_PORT}" || true
            "$BIN" admin config server cors remove "http://localhost:8080" || true
            "$BIN" admin config server cors remove "http://127.0.0.1:8080" || true
        fi
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
    # Core 61 seals gateway accounting records and requires a durable 32-byte
    # at-rest key. Keep an existing key so a local re-run can still open its
    # journal; generate one only for profiles created by older Core releases.
    python3 - "$PROFILE_DIR/secrets.json" <<'PYTHON'
    import json
    import secrets
    import sys
    from pathlib import Path

    path = Path(sys.argv[1])
    data = json.loads(path.read_text())
    key = data.get("encryption_master_key", "")
    if len(key) != 64 or any(char not in "0123456789abcdefABCDEF" for char in key):
        data["encryption_master_key"] = secrets.token_hex(32)
        path.write_text(json.dumps(data, indent=2) + "\n")
    PYTHON
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

# Deploy to cloud
# Note: publish_pipeline runs automatically on server startup with correct profile URLs
deploy *FLAGS: core-guard deploy-check
    just build --release
    {{CLI_RELEASE}} cloud deploy {{FLAGS}}

# Pre-deploy preflight — no build, no push. `deploy` depends on it, so a
# cloud profile the binary would refuse to boot (no server.instance_id,
# missing identity secrets) is caught here, not after the image is live.
deploy-check:
    {{CLI}} cloud doctor --distributed

# Check deployment status
status:
    {{CLI}} cloud status

# ══════════════════════════════════════════════════════════════════════════════
# MCP & BUILD ALL
# ══════════════════════════════════════════════════════════════════════════════

# Build all MCP servers (reads from manifest.yaml files)
# Single-flight and fingerprint-skipped: a tree whose MCP servers already
# built returns immediately instead of re-paying the per-package rebuild.
build-mcp:
    @scripts/build-coordinator.sh run build-mcp "" -- {{just_executable()}} _build-mcp-uncoordinated

_build-mcp-uncoordinated:
    DATABASE_URL="$(just _db-url)" {{CLI}} build mcp --release

# Build everything for deployment (Rust binary + MCP servers + web assets)
build-all:
    just build --release
    just build-mcp
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

# Test build without pushing
docker-test:
    just build-all
    just docker-build test
    @echo "Docker build successful! Image: systemprompt-template:test"

# ══════════════════════════════════════════════════════════════════════════════
# AIR-GAPPED SCENARIO
# ══════════════════════════════════════════════════════════════════════════════

# Bring up the network-isolated air-gap stack (postgres + mock-inference + app + monitor + ingress)
airgap-up:
    #!/usr/bin/env bash
    set -euo pipefail
    # Dockerfile.airgap-prebuilt COPYs the host-built binaries from
    # deploy/scenarios/airgap/.bin/ — `target` is a symlink to a shared cargo
    # cache that buildkit can't follow, so we dereference-copy them in first
    # (mirrors scaled-up).
    if [[ ! -x target/release/systemprompt || ! -x target/release/systemprompt-mcp-agent ]]; then
        echo "ERROR: release binaries missing. Run: just build --release" >&2
        exit 1
    fi
    mkdir -p deploy/scenarios/airgap/.bin
    cp -L target/release/systemprompt           deploy/scenarios/airgap/.bin/systemprompt
    cp -L target/release/systemprompt-mcp-agent deploy/scenarios/airgap/.bin/systemprompt-mcp-agent
    docker compose -f deploy/scenarios/airgap/docker-compose.airgap.yml up -d --build

# Tear down the air-gap stack and remove its volumes
airgap-down:
    docker compose -f deploy/scenarios/airgap/docker-compose.airgap.yml down -v

# ONE-COMMAND air-gap proof. Ensures the sealed stack is up (builds the image
# only if it is missing), warm-builds the loadtest crate so the run emits no
# compiler spew, runs all three assertion scripts (01 egress, 02 load,
# 03 governance) WITHOUT dying on the first failure, then prints a single
# PASS/FAIL summary. Leaves the stack up for inspection by default — pass
# TEARDOWN=true to remove it (and its volumes) at the end.
#
#   just airgap                # run, leave stack up
#   just airgap TEARDOWN=true  # run, then tear down
airgap TEARDOWN="false":
    #!/usr/bin/env bash
    set -uo pipefail
    COMPOSE_FILE="deploy/scenarios/airgap/docker-compose.airgap.yml"
    PORT="${AIRGAP_HTTP_PORT:-8090}"
    LOADTEST_MANIFEST="../systemprompt-core/crates/tests/loadtest/Cargo.toml"

    # 1. Ensure the stack is up. Build the image only if it is not present yet
    #    (a first-time build pulls in ../systemprompt-core and takes ~10 min).
    if curl -fsS -o /dev/null --max-time 3 "http://localhost:${PORT}/api/v1/health" 2>/dev/null; then
      echo "  air-gap stack already healthy on :${PORT}"
    else
      if docker compose -f "$COMPOSE_FILE" config --images 2>/dev/null \
         | xargs -r -I{} docker image inspect {} >/dev/null 2>&1; then
        echo "  air-gap image present — starting stack (no rebuild)"
        docker compose -f "$COMPOSE_FILE" up -d
      else
        echo "  air-gap image missing — building stack (first run, ~10 min)"
        docker compose -f "$COMPOSE_FILE" up -d --build
      fi
      echo "  waiting for app healthcheck on :${PORT} ..."
      for i in $(seq 1 120); do
        if curl -fsS -o /dev/null "http://localhost:${PORT}/api/v1/health" 2>/dev/null; then
          echo "  app healthy after ${i}s"
          break
        fi
        sleep 1
      done
    fi

    # 2. Warm-build the loadtest crate quietly so STEP 02's `cargo run` emits no
    #    build output mid-demo. Non-fatal: 02-load.sh re-checks the manifest.
    if [[ -f "$LOADTEST_MANIFEST" ]]; then
      echo "  warm-building the loadtest crate ..."
      cargo build --quiet --manifest-path "$LOADTEST_MANIFEST" 2>/dev/null || true
    else
      echo "  loadtest crate not found at ${LOADTEST_MANIFEST} — skipping warm-build" >&2
      echo "  (it is unpublished systemprompt-core dev tooling; 02-load.sh will build it on demand if present)" >&2
    fi

    # 3. Run all three scripts, capturing each exit code (do NOT stop on first
    #    failure — the operator must see the full picture).
    declare -A RESULT
    for s in 01-egress-assert 02-load 03-governance; do
      echo ""
      if "./demo/scenarios/airgap/${s}.sh"; then
        RESULT[$s]="PASS"
      else
        RESULT[$s]="FAIL"
      fi
    done

    # 4. Single PASS/FAIL summary.
    echo ""
    echo "══════════════════════════════════════════════════════════"
    echo "  AIR-GAP PROOF SUMMARY"
    echo "══════════════════════════════════════════════════════════"
    OVERALL=0
    for s in 01-egress-assert 02-load 03-governance; do
      printf "  %-22s %s\n" "$s" "${RESULT[$s]}"
      [[ "${RESULT[$s]}" == "PASS" ]] || OVERALL=1
    done
    echo "══════════════════════════════════════════════════════════"
    [[ "$OVERALL" -eq 0 ]] && echo "  RESULT: PASS" || echo "  RESULT: FAIL"

    # 5. Optional teardown.
    if [[ "{{TEARDOWN}}" == "true" ]]; then
      echo ""
      echo "  TEARDOWN=true — removing the air-gap stack and volumes"
      just airgap-down
    fi

    exit "$OVERALL"

# Run the air-gap demo scripts in sequence, stopping on first failure.
# Policies (quotas/safety) ship as services/gateway/policies.yaml and are
# ingested by the publish_pipeline job at server boot. Model exposure lives
# in the profile provider registry (profile.providers in
# .systemprompt/profiles/airgap/profile.yaml).
airgap-test:
    #!/usr/bin/env bash
    set -euo pipefail
    ./demo/scenarios/airgap/01-egress-assert.sh
    ./demo/scenarios/airgap/02-load.sh
    ./demo/scenarios/airgap/03-governance.sh

# Reproducibility proof: tear down (incl. volumes), bring back up reusing the
# already-built image, run the full assertion suite from zero state. Prints
# wall-clock time. Use this in front of a reviewer who wants to see the demo
# work from a clean container + clean database, without a 10-minute image
# rebuild. Image-level reproducibility is a separate concern — see
# demo/scenarios/airgap/architecture.md §9 (the [patch.crates-io] block
# requires systemprompt-core >= 0.10.4 to be published before the image can
# be rebuilt from this repo in isolation).
airgap-fresh-test:
    #!/usr/bin/env bash
    set -euo pipefail
    COMPOSE_FILE="deploy/scenarios/airgap/docker-compose.airgap.yml"
    # Refuse to run if the image isn't already built — the rebuild path needs
    # the sibling systemprompt-core repo and a 10-minute window, and silently
    # falling into that on a demo machine is a bad surprise.
    if ! docker image inspect airgap-app >/dev/null 2>&1 \
       && ! docker compose -f "$COMPOSE_FILE" config --images 2>/dev/null | head -1 | xargs -I{} docker image inspect {} >/dev/null 2>&1; then
      echo "ERROR: app image not present. First-time build needed:" >&2
      echo "  just airgap-up   # builds the image (~10 min, needs ../systemprompt-core)" >&2
      exit 1
    fi
    START=$(date +%s)
    just airgap-down
    # No --build: reuse the existing image. This is the from-zero DATA reset,
    # not the from-zero BUILD reset.
    docker compose -f "$COMPOSE_FILE" up -d
    echo "Waiting for app healthcheck..."
    for i in $(seq 1 120); do
      if curl -fsS -o /dev/null "http://localhost:${AIRGAP_HTTP_PORT:-8090}/api/v1/health" 2>/dev/null; then
        echo "App healthy after ${i}s"
        break
      fi
      sleep 1
    done
    just airgap-test
    END=$(date +%s)
    echo ""
    echo "═══════════════════════════════════════════════════════"
    echo "  FRESH AIR-GAP RUN COMPLETE in $((END - START))s"
    echo "═══════════════════════════════════════════════════════"

# ══════════════════════════════════════════════════════════════════════════════
# SCALED / DISTRIBUTED SCENARIO
# ══════════════════════════════════════════════════════════════════════════════

# Bring up the multi-replica scaled stack (postgres primary/replica + N app replicas + 1 scheduler + nginx LB)
scaled-up REPLICAS="3":
    #!/usr/bin/env bash
    set -euo pipefail
    # Stage the host-built binaries into a real dir inside the build context —
    # `target` is a symlink to a shared cargo cache that buildkit can't follow.
    if [[ ! -x target/release/systemprompt || ! -x target/release/systemprompt-mcp-agent ]]; then
        echo "ERROR: release binaries missing. Run: just build --release" >&2
        exit 1
    fi
    mkdir -p deploy/scenarios/scaled/.bin
    cp -L target/release/systemprompt           deploy/scenarios/scaled/.bin/systemprompt
    cp -L target/release/systemprompt-mcp-agent deploy/scenarios/scaled/.bin/systemprompt-mcp-agent
    docker compose -f deploy/scenarios/scaled/docker-compose.scaled.yml up -d --build --scale app={{REPLICAS}}

# Tear down the scaled stack and remove its volumes
scaled-down:
    docker compose -f deploy/scenarios/scaled/docker-compose.scaled.yml down -v

# ONE COMMAND: reset → build → up → wait-for-health → mint token → run all fast
# proofs → capture logs → single verdict. Leaves the stack up by default.
#   just scaled-demo                # 3 replicas, stack left up
#   REPLICAS=5 just scaled-demo     # scale wider
#   KEEP=0 just scaled-demo         # tear down at the end
#   SOAK=1 just scaled-demo         # also run the ~1h soak (long!)
scaled-demo:
    #!/usr/bin/env bash
    set -uo pipefail
    chmod +x demo/scenarios/scaled/run.sh
    ./demo/scenarios/scaled/run.sh

# Run the scaled demo scripts in sequence against an ALREADY-RUNNING stack.
# Prefer `just scaled-demo` (full lifecycle). Use this only when the stack is
# already up and healthy. Skips 02-soak.sh — the long (~1h) sustained soak; run
# it on its own when needed: ./demo/scenarios/scaled/02-soak.sh
scaled-test:
    #!/usr/bin/env bash
    set -euo pipefail
    chmod +x demo/scenarios/scaled/01-load.sh \
             demo/scenarios/scaled/03-replica-distribution.sh \
             demo/scenarios/scaled/04-scheduler-exactly-once.sh
    ./demo/scenarios/scaled/01-load.sh
    ./demo/scenarios/scaled/03-replica-distribution.sh
    ./demo/scenarios/scaled/04-scheduler-exactly-once.sh

# ══════════════════════════════════════════════════════════════════════════════
# ADMIN & PLUGINS
# ══════════════════════════════════════════════════════════════════════════════

# Generate WebAuthn setup token for admin user
webauthn-admin EMAIL:
    {{CLI}} admin users webauthn generate-setup-token --email "{{EMAIL}}"

# Generate plugin output
marketplace:
    {{CLI}} core plugins generate

# Update Anthropic official plugins from vendor submodule and reimport
update-anthropic-plugins:
    git submodule update --remote vendor/knowledge-work-plugins
    {{CLI}} infra jobs run import_anthropic_plugins

# ══════════════════════════════════════════════════════════════════════════════
# TERMINAL RECORDINGS (README SVGs)
# ══════════════════════════════════════════════════════════════════════════════

# Regenerate terminal SVG recordings. Pass numbers to limit scope, e.g. `just record-svgs 3 7`.
record-svgs *NUMBERS:
    ./demo/recording/svg/record.sh {{NUMBERS}}

# ══════════════════════════════════════════════════════════════════════════════
# BENCHMARKS
# ══════════════════════════════════════════════════════════════════════════════

# Benchmark the governance endpoint against the running profile.
#
# Why the URL is derived and not hardcoded: this recipe pointed at
# http://localhost:8080 regardless of the profile. On an instance brought up by
# `just setup-local <keys> <http_port> <pg_port>` that benchmarks nothing, or
# worse, whichever unrelated instance happens to hold 8080 -- and it reports a
# number either way. The demos already solve this in demo/_common.sh; this reads
# the same api_server_url.
#
# `hey` must be installed. The old auto-download from
# hey-release.s3.us-east-2.amazonaws.com now answers 403 for every asset, so the
# fallback could only ever produce a confusing failure part-way through a run.
benchmark REQUESTS="200" CONCURRENCY="100":
    #!/usr/bin/env bash
    set -euo pipefail
    ROOT="{{justfile_directory()}}"
    if ! command -v hey >/dev/null 2>&1; then
        echo "benchmark: 'hey' is not installed." >&2
        echo "  Debian/Ubuntu: sudo apt-get install hey" >&2
        echo "  macOS:         brew install hey" >&2
        echo "  Any platform:  go install github.com/rakyll/hey@latest" >&2
        exit 1
    fi
    # SYSTEMPROMPT_PROFILE is not always a profile NAME. `set dotenv-load` pulls
    # in .env, where it is conventionally an absolute path to a profile.yaml --
    # and in a copied .env, a path into a different checkout entirely. Accept
    # both forms and never build a directory out of a path.
    PROFILE_YAML=""
    case "${SYSTEMPROMPT_PROFILE:-}" in
        */*.yaml|*/*.yml) PROFILE_YAML="$SYSTEMPROMPT_PROFILE" ;;
        "")               PROFILE_YAML="$ROOT/.systemprompt/profiles/local/profile.yaml" ;;
        *)                PROFILE_YAML="$ROOT/.systemprompt/profiles/${SYSTEMPROMPT_PROFILE}/profile.yaml" ;;
    esac
    # A profile belonging to another checkout describes another server; prefer
    # this one's if the pointed-at file is outside the tree.
    case "$PROFILE_YAML" in
        "$ROOT"/*) ;;
        *) [ -f "$ROOT/.systemprompt/profiles/local/profile.yaml" ] \
             && PROFILE_YAML="$ROOT/.systemprompt/profiles/local/profile.yaml" ;;
    esac
    BASE_URL="${BASE_URL:-}"
    if [ -z "$BASE_URL" ] && [ -f "$PROFILE_YAML" ]; then
        BASE_URL=$(grep -E '^[[:space:]]*api_server_url:' "$PROFILE_YAML" | head -1 \
            | sed -E 's/.*api_server_url:[[:space:]]*//; s/[[:space:]]*$//; s/^"//; s/"$//')
    fi
    [ -n "$BASE_URL" ] && [ "$BASE_URL" != "null" ] || BASE_URL="http://localhost:8080"
    TOKEN_FILE="$ROOT/demo/.token"
    if [ ! -s "$TOKEN_FILE" ]; then
        echo "benchmark: no token at $TOKEN_FILE — run ./demo/00-preflight.sh first." >&2
        exit 1
    fi
    TOKEN=$(cat "$TOKEN_FILE")
    echo "Governance endpoint at $BASE_URL: {{REQUESTS}} requests, {{CONCURRENCY}} concurrent"
    echo ""
    hey -n {{REQUESTS}} -c {{CONCURRENCY}} -m POST \
        -H "Authorization: Bearer $TOKEN" \
        -H "Content-Type: application/json" \
        -d '{"hook_event_name":"PreToolUse","tool_name":"Read","agent_id":"developer_agent","session_id":"bench","tool_input":{"file_path":"/src/main.rs"}}' \
        "$BASE_URL/api/public/hooks/govern?plugin_id=enterprise-demo"

# Syntax-check install.sh (install.sh is the user-facing installer)
install-sh-test:
    bash -n scripts/install.sh
    shellcheck scripts/install.sh 2>/dev/null || echo "(install shellcheck to lint: apt install shellcheck)"

# Check the Nix flake builds + runs
flake-check:
    nix flake check
    nix run .# -- --version

# --- Release ------------------------------------------------------------

# Adopt a published core release: move every pin (lockstep — the workspace
# version, helm appVersion and deploy image pins follow core), point
# bridge/CORE_REF at its tag, refresh BOTH lockfiles so the core crates cannot
# drift between them (scripts/check-core-crate-versions.sh), migrate the LOCAL
# database with the new binary, and gate (build + clippy). Then
# `just schema-baseline` for the new rung, a CHANGELOG entry, `just verify`,
# push next, and `just release X.Y.Z`. See docs/RELEASING.md.
#
# LOCAL-ONLY. The migrate is pinned to `--profile local`: a bare `infra db
# migrate` follows the active CLI session, which can be a cloud profile. A
# failed migrate stops the bump (no `|| true`).
core-bump version:
    @! grep -q '^\[patch\.crates-io\]' Cargo.toml || (echo "ERROR: [patch.crates-io] is active — publish core and re-comment it first" && exit 1)
    scripts/sync-release-version.sh {{version}}
    scripts/sync-core-version.sh {{version}}
    cargo update -w
    cargo update -w --manifest-path tests/Cargo.toml
    just db-up
    cargo run --bin systemprompt -- infra db migrate --profile local
    just build
    just clippy
    @echo "core-bump {{version}} complete — then: just schema-baseline, CHANGELOG, just verify, push next, just release {{version}}"

# Pin bridge/CORE_REF to a core `next` commit while [patch.crates-io] is
# active (the sibling checkout's HEAD by default). CI checks that ref out.
core-pin REF="":
    #!/usr/bin/env bash
    set -euo pipefail
    ref="{{REF}}"
    core="${CORE_REPO:-../systemprompt-core}"
    [ -n "$ref" ] || ref="$(git -C "$core" rev-parse HEAD)"
    # check-core-ref.sh accepts only a vX.Y.Z tag or a full 40-char SHA, so expand abbreviations.
    case "$ref" in v[0-9]*) ;; *) ref="$(git -C "$core" rev-parse --verify "${ref}^{commit}")" ;; esac
    printf '%s\n' "$ref" > bridge/CORE_REF
    echo "bridge/CORE_REF = $ref"

# What "build next and next together" means in practice: while
# [patch.crates-io] is active the server compiles against ../systemprompt-core
# in place, so the only way a deploy can ship exactly the core CI gates is if
# that checkout is clean and sits at bridge/CORE_REF. Refuses otherwise; with
# the patch inactive there is nothing to check.
core-guard:
    #!/usr/bin/env bash
    set -euo pipefail
    grep -qE '^\[patch\.crates-io\]' Cargo.toml || { echo "core-guard: patch inactive; building against the published core"; exit 0; }
    core="${CORE_REPO:-../systemprompt-core}"
    # -e, not -d: a worktree's .git is a file, and a detached worktree is the sanctioned clean checkout.
    [ -e "$core/.git" ] || { echo "core-guard: no core checkout at $core" >&2; exit 1; }
    expected="$(tr -d '[:space:]' < bridge/CORE_REF)"
    head="$(git -C "$core" rev-parse HEAD)"
    [ "$head" = "$expected" ] || { echo "core-guard: $core is at ${head:0:12}, bridge/CORE_REF pins ${expected:0:12}; commit and 'just core-pin' first" >&2; exit 1; }
    dirty="$(git -C "$core" status --porcelain --untracked-files=all)"
    [ -z "$dirty" ] || { echo "core-guard: $core has uncommitted changes; commit them on core next (or set them aside) before deploying:" >&2; echo "$dirty" >&2; exit 1; }
    echo "core-guard: $core clean at ${head:0:12} == bridge/CORE_REF"

# Clone systemprompt-core beside this repo when absent (the patch-active build
# and core's contract gates need it), leave local work alone, and compare a
# clean checkout with bridge/CORE_REF: a mismatch is fatal by default and a
# warning with MISMATCH=warn. It never moves the checkout — updating here
# could mix a server built from one core with gates run against another.
core-checkout MISMATCH="fail":
    #!/usr/bin/env bash
    set -euo pipefail
    CORE="{{justfile_directory()}}/../systemprompt-core"
    if [ -e "$CORE/.git" ]; then
        if [ -n "$(git -C "$CORE" status --porcelain)" ]; then
            echo "core checkout has local changes — leaving it as it is."
        else
            echo "Using existing core checkout at $(git -C "$CORE" rev-parse --short HEAD)"
            expected="$(tr -d '\r\n' < "{{justfile_directory()}}/bridge/CORE_REF")"
            head="$(git -C "$CORE" rev-parse HEAD)"
            pinned="$(git -C "$CORE" rev-parse "$expected^{commit}")"
            if [ "$head" != "$pinned" ]; then
                if [ "{{MISMATCH}}" = "warn" ]; then
                    echo "warn: core checkout ${head:0:9} differs from bridge/CORE_REF ${pinned:0:9};" >&2
                    echo "      using the checkout as it is. Pin with: just core-pin" >&2
                else
                    echo "core checkout differs from bridge/CORE_REF; pin core (just core-pin) or check out $expected." >&2
                    exit 1
                fi
            fi
        fi
    else
        echo "Cloning systemprompt-core beside this repo at bridge/CORE_REF."
        git clone --quiet https://github.com/systempromptio/systemprompt-core "$CORE"
        git -C "$CORE" checkout --quiet "$(tr -d '\r\n' < "{{justfile_directory()}}/bridge/CORE_REF")"
    fi

# Install the compiled gateway binaries from a GitHub Release instead of
# building them: `systemprompt-gateway-<version>-<target>.tar.gz` (linux-amd64,
# linux-arm64, darwin-arm64) holds systemprompt and systemprompt-mcp-agent,
# built by release-gateway.yml. They land in target/release/, where
# `just start` and setup-local already look, so a clone needs no Rust
# toolchain. Verified against the release's SHA256SUMS.gateway. Default
# version = the workspace version in Cargo.toml.
fetch-release VERSION="":
    #!/usr/bin/env bash
    set -euo pipefail
    v="{{VERSION}}"
    [ -n "$v" ] || v=$(sed -n 's/^version = "\([0-9.]*\)"/\1/p' Cargo.toml | head -1)
    case "$(uname -s)-$(uname -m)" in
        Linux-x86_64) target=linux-amd64 ;;
        Linux-aarch64|Linux-arm64) target=linux-arm64 ;;
        Darwin-arm64) target=darwin-arm64 ;;
        *) echo "fetch-release: no gateway tarball for $(uname -s)-$(uname -m); use the image (docs/install/ghcr.md) or 'just build --release'." >&2; exit 1 ;;
    esac
    name="systemprompt-gateway-$v-$target.tar.gz"
    tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
    echo "==> downloading $name from release v$v"
    gh release download "v$v" -R systempromptio/systemprompt-template -p "$name" -p SHA256SUMS.gateway -D "$tmp" \
        || { echo "fetch-release: release v$v has no $name (gh auth, or the release does not exist yet)." >&2; exit 1; }
    (cd "$tmp" && grep " $name\$" SHA256SUMS.gateway | { sha256sum -c - 2>/dev/null || shasum -a 256 -c -; })
    tar xzf "$tmp/$name" -C "$tmp"
    mkdir -p target/release
    install -m 0755 "$tmp/${name%.tar.gz}"/systemprompt "$tmp/${name%.tar.gz}"/systemprompt-mcp-* target/release/
    echo "==> installed into target/release/:"; ls -1 target/release/systemprompt target/release/systemprompt-mcp-* | sed 's/^/    /'
    target/release/systemprompt --version

# Promote the exact green next-push candidate through a frozen PR onto main,
# then tag the merge (release-gateway.yml runs on the tag). Run it once to open
# the PR, and again after the PR's "Verify frozen promotion" proof is green to
# merge and tag. Replaces the old `gate` / `promote` pair and the mutable
# `promote` ref. See docs/RELEASING.md and docs/BRANCHING.md.
release version:
    bash scripts/release.sh {{version}}

# ---------------------------------------------------------------------------
# Browser e2e (Playwright, playwright/). Runs against an ALREADY-RUNNING stack:
# `just start` first. Global setup pings /health and fails fast rather than
# booting a server, because this clone's server may be shared with other agents.
# ---------------------------------------------------------------------------

# Install the Playwright e2e suite's dependencies (playwright/ directory)
e2e-install:
    cd playwright && npm ci && npx playwright install chromium

# Run the Playwright e2e suite against a running gateway (GATEWAY_URL env
# overrides the default http://localhost:8080). Not part of `just verify` —
# it needs a live stack: `just start` first.
e2e *ARGS:
    bash scripts/check-spec-shape.sh
    cd playwright && npx playwright test {{ARGS}}

# Seed deterministic e2e principals + traffic (idempotent; touches only
# e2e-*/@e2e.local rows). `--reset` deletes and re-creates exactly those rows.
e2e-seed *ARGS:
    cd playwright && npx tsx setup/seed.ts {{ARGS}}

# The browser tier as a gate. It needs a live stack, which nothing here can
# start for you (the server on this clone may be shared), so this fails
# loudly with the command to run rather than skipping quietly — a gate that
# silently omits the e2e suite is worse than one that stops.
e2e-gate:
    #!/usr/bin/env bash
    set -euo pipefail
    URL="${GATEWAY_URL:-http://localhost:8080}"
    if ! curl -fsS -o /dev/null "$URL/health"; then
        echo "e2e-gate: the e2e tier needs a running stack at $URL." >&2
        echo "          Run 'just start' (or set GATEWAY_URL) and re-run." >&2
        exit 1
    fi
    {{just_executable()}} e2e --project chromium

# Print a short-lived login link for an active user on a local development profile.
dev-login USER:
    #!/usr/bin/env bash
    set -euo pipefail
    export SYSTEMPROMPT_PROFILE="${SYSTEMPROMPT_PROFILE:-{{justfile_directory()}}/.systemprompt/profiles/local/profile.yaml}"
    exec {{CLI}} plugins run dev-login "{{USER}}"

# Focused functional regression checks for shared dashboard changes.
test-dashboard stage="all":
    @scripts/build-coordinator.sh run test-dashboard "{{stage}}" -- bash scripts/test-dashboard.sh "{{stage}}"
