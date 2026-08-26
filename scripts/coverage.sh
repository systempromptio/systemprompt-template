#!/usr/bin/env bash
# Line-coverage run for all three workspaces (root, tests/, bridge/), ported
# from systemprompt-core's `just coverage` recipe. Raw llvm-cov rather than
# cargo-llvm-cov, for the same host-toolchain reasons core documents:
#
#   1. sccache via [build] rustc-wrapper in ~/.cargo/config.toml returns
#      cached uninstrumented rlibs — neutralised by CARGO_BUILD_RUSTC_WRAPPER="".
#   2. A mold linker pinned by target.<triple>.rustflags strips the
#      profile-runtime constructors, silently producing zero profraw files.
#      Setting the RUSTFLAGS env replaces target rustflags entirely (cargo's
#      flag-resolution order), so the default linker links. cargo-llvm-cov
#      MERGES target rustflags back in, which is why it cannot be used.
#
# Builds only in dedicated target dirs under coverage-report/ so concurrent
# agents sharing this checkout are unaffected. %m%c profraw naming (continuous
# mode, no %p) because PID reuse across many test processes silently
# overwrites per-PID files.
#
# DB-backed suites (mcp-integration, admin-contract) manage their own
# throwaway *_test databases via SYSTEMPROMPT_TEST_DATABASE_URL, same
# derivation as `just test-integration` — no dedicated coverage database is
# needed here, unlike core's shared-dev-DB situation.
#
# Outputs (all under coverage-report/, gitignored):
#   summary.json   llvm-cov export --summary-only (totals + per-file)
#   report.txt     human-readable llvm-cov report
#   lcov.info      lcov export for external tooling
#   tests.profdata merged profile (input to `just coverage-html`)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"
PROFDIR="$ROOT/coverage-report/profraw"
TBASE="${COVERAGE_TARGET_DIR:-$ROOT/coverage-report/target}"

rm -rf "$PROFDIR" "$ROOT/coverage-report/tests.profdata"
mkdir -p "$PROFDIR"

if [ -z "${SYSTEMPROMPT_TEST_DATABASE_URL:-}" ] && [ -f .systemprompt/profiles/local/secrets.json ]; then
    SYSTEMPROMPT_TEST_DATABASE_URL=$(python3 -c "
import json, urllib.parse as up
u = up.urlsplit(json.load(open('.systemprompt/profiles/local/secrets.json'))['database_url'])
print(up.urlunsplit((u.scheme, u.netloc, '/postgres', '', '')))")
    export SYSTEMPROMPT_TEST_DATABASE_URL
fi

# One invocation per workspace; each gets its own target dir so a plain build
# never poisons an instrumented one (or vice versa). nextest, not cargo test:
# process-per-test isolates the OnceLock-global fixtures (Config::install)
# that collide inside one test process, honours the serial DB test-groups,
# and --no-fail-fast keeps one red package from silently dropping every
# later package out of the denominator.
run_instrumented() {
    local tdir="$1"; shift
    CARGO_BUILD_RUSTC_WRAPPER="" \
    RUSTC_WRAPPER="" \
    CARGO_TARGET_DIR="$tdir" \
    LLVM_PROFILE_FILE="$PROFDIR/%m%c.profraw" \
    RUSTFLAGS="-C instrument-coverage -C llvm-args=--runtime-counter-relocation" \
    SQLX_OFFLINE=true \
    cargo nextest run --no-fail-fast --build-jobs 4 "$@" \
        || echo "warning: test failures above — continuing to coverage report"
}

echo "==> [1/3] Instrumented tests: root workspace"
run_instrumented "$TBASE-root" --workspace --tests

echo "==> [2/3] Instrumented tests: tests/ workspace"
run_instrumented "$TBASE-tests" --manifest-path tests/Cargo.toml --workspace

echo "==> [3/3] Instrumented tests: bridge/ workspace"
run_instrumented "$TBASE-bridge" --manifest-path bridge/Cargo.toml --workspace

PROFRAW_COUNT=$(find "$PROFDIR" -name '*.profraw' | wc -l)
echo "==> Generated $PROFRAW_COUNT profraw files"
if [ "$PROFRAW_COUNT" -eq 0 ]; then
    echo "error: zero profraw files — the sccache/mold overrides are not taking effect" >&2
    exit 1
fi

TOOLDIR="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"
LLVM_PROFDATA="$TOOLDIR/llvm-profdata"
LLVM_COV="$TOOLDIR/llvm-cov"

echo "==> Merging profile data"
find "$PROFDIR" -name '*.profraw' > "$ROOT/coverage-report/profraw-list.txt"
"$LLVM_PROFDATA" merge -sparse -f "$ROOT/coverage-report/profraw-list.txt" \
    -o "$ROOT/coverage-report/tests.profdata"

# Test binaries land in <target>/debug/deps; dedupe by crate basename keeping
# the newest build of each (same awk as core).
collect_bins() {
    find "$1/debug/deps" -maxdepth 1 -executable -type f ! -name '*.d' ! -name '*.so' \
        -printf '%T@ %p\n' 2>/dev/null \
        | sort -rn \
        | awk '{ base=$2; sub(".*/", "", base); sub(/-[0-9a-f]+$/, "", base); if (!seen[base]++) print $2 }'
}
BINS="$(collect_bins "$TBASE-root"; collect_bins "$TBASE-tests"; collect_bins "$TBASE-bridge")"
OBJ_ARGS=()
for b in $BINS; do OBJ_ARGS+=(--object "$b"); done

# Denominator exclusions, kept explicit per-file (per-directory only for test
# code) so ordinary testable code added alongside them still counts:
#   src/main.rs, src/lib.rs        — process entry / pure re-export shims
#   extensions/**/extension.rs     — inventory registration glue, no logic
#   */build.rs                     — build scripts
# Keep this regex in sync between coverage.sh, coverage-check.sh docs, and
# `just coverage-html`.
# systemprompt-core/ is the sibling checkout pulled in via [patch.crates-io];
# it has its own coverage CI and must not dilute this repo's denominator.
# bridge/src/main.rs is a process supervisor (brand consts + run_with_brand
# delegation); on Linux it is the crate's only compiled file, mirroring core's
# explicit process-entry exclusions.
IGNORE_RE="(\.cargo|/rustc/|/registry/|/debug/build/|/tests/|/target/|systemprompt-core/|systemprompt-astound/src/(main|lib)\.rs|bridge/src/main\.rs|extensions/cli/[^/]+/src/(main\.rs|commands/)|extensions/.*/extension\.rs|build\.rs)"

echo "==> Coverage report"
"$LLVM_COV" report \
    --instr-profile="$ROOT/coverage-report/tests.profdata" \
    "${OBJ_ARGS[@]}" \
    --ignore-filename-regex="$IGNORE_RE" \
    | tee "$ROOT/coverage-report/report.txt"

"$LLVM_COV" export \
    --instr-profile="$ROOT/coverage-report/tests.profdata" \
    "${OBJ_ARGS[@]}" \
    --ignore-filename-regex="$IGNORE_RE" \
    --summary-only \
    > "$ROOT/coverage-report/summary.json"

"$LLVM_COV" export \
    --instr-profile="$ROOT/coverage-report/tests.profdata" \
    "${OBJ_ARGS[@]}" \
    --ignore-filename-regex="$IGNORE_RE" \
    --format=lcov \
    > "$ROOT/coverage-report/lcov.info"

TOTAL=$(jq -r '.data[0].totals.lines.percent' "$ROOT/coverage-report/summary.json")
printf '==> Total line coverage: %.2f%%\n' "$TOTAL"
echo "Reports: coverage-report/{report.txt,summary.json,lcov.info}"
echo "For HTML: just coverage-html"
