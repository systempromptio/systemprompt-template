#!/usr/bin/env bash
# Line-coverage run for both workspaces (root, tests/), ported
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

# Why: the object collection below relies on GNU find (-executable, -printf).
# BSD find on macOS has neither and would silently collect nothing.
FIND=find
if ! find /dev/null -maxdepth 0 -printf '' >/dev/null 2>&1; then
    if command -v gfind >/dev/null 2>&1; then FIND=gfind
    else echo "error: coverage.sh needs GNU find (macOS: brew install findutils)" >&2; exit 1; fi
fi

# Peak memory is several instrumented binaries linking at once, and a GitHub
# runner dies there — the job is SIGTERMed with no diagnostic, which reads as
# a mystery rather than as "out of memory". Lower it where the machine is
# small; the default suits a workstation.
BUILD_JOBS="${COVERAGE_BUILD_JOBS:-4}"

# Counts phases that failed to build or ran a red test; reported at the end so
# the exit status says whether the number rests on every workspace.
PHASE_FAILURES=0
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
    # Why: the export below picks the newest binary per crate basename, so a
    # test executable left behind by a crate that no longer exists (or was
    # renamed) is never superseded and keeps feeding stale coverage mappings —
    # files that were deleted count as 0% and functions whose hashes changed
    # count as uncovered. Dropping every executable first costs one relink per
    # test binary; the rlibs stay cached.
    "$FIND" "$tdir/debug/deps" -maxdepth 1 -type f -executable \
        ! -name '*.d' ! -name '*.so' ! -name '*.rlib' -delete 2>/dev/null || true
    # Why `if` rather than a bare call: this script runs under `set -e`, and
    # the phases below deliberately survive a red workspace. The construct
    # this replaces was `|| echo "warning: ..."`, which suppressed the abort
    # but also discarded the status, so a failed phase and a clean one were
    # indistinguishable to the caller.
    local status=0
    if CARGO_BUILD_RUSTC_WRAPPER="" \
        RUSTC_WRAPPER="" \
        CARGO_TARGET_DIR="$tdir" \
        LLVM_PROFILE_FILE="$PROFDIR/%m%c.profraw" \
        RUSTFLAGS="-C instrument-coverage -C llvm-args=--runtime-counter-relocation" \
        SQLX_OFFLINE=true \
        cargo nextest run --no-fail-fast --build-jobs "$BUILD_JOBS" "$@"
    then
        status=0
    else
        status=$?
    fi
    # nextest exit codes, treated by consequence rather than lumped together:
    #   0  everything ran
    #   4  the workspace defines no tests. True of the root workspace here —
    #      it is instrumented for its objects, not for tests — so it is the
    #      expected answer, not a failure.
    #   *  a build failure or a red test. The report is still produced, since a
    #      partial number beats none while iterating, but the run is marked so
    #      the caller can tell. A CI job that reported a healthy percentage
    #      while a whole workspace failed to compile is what this exists to
    #      prevent.
    case "$status" in
        0|4) ;;
        *)
            echo "warning: workspace failed (nextest exit $status) — continuing to coverage report" >&2
            PHASE_FAILURES=$((PHASE_FAILURES + 1))
            ;;
    esac
}

echo "==> [1/2] Instrumented tests: root workspace"
run_instrumented "$TBASE-root" --workspace --tests

echo "==> [2/2] Instrumented tests: tests/ workspace"
run_instrumented "$TBASE-tests" --manifest-path tests/Cargo.toml --workspace

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

# Test binaries land in <target>/debug/deps. Emit candidates from every
# workspace, then dedupe the combined stream by crate basename. Prefer the
# tests workspace because that is where the executed tests live; root supplies
# only crates absent from it. Within a workspace, keep the
# newest build. Dedupe must be global: doing it once per target directory
# feeds llvm-cov multiple feature/build variants of the same crate. Their
# function names overlap but their mapping hashes differ, so llvm-cov discards
# profile records with "functions have mismatched data" and under-reports the
# very tests this job is meant to measure.
collect_bin_candidates() {
    "$FIND" "$1/debug/deps" -maxdepth 1 -executable -type f ! -name '*.d' ! -name '*.so' \
        -printf '%T@ %p\n' 2>/dev/null | sort -rn
}
BINS="$(
    {
        collect_bin_candidates "$TBASE-tests"
        collect_bin_candidates "$TBASE-root"
    } | awk '{ base=$2; sub(".*/", "", base); sub(/-[0-9a-f]+$/, "", base); if (!seen[base]++) print $2 }'
)"
BIN_COUNT=$(printf '%s\n' "$BINS" | sed '/^$/d' | wc -l)
if [ "$BIN_COUNT" -eq 0 ]; then
    echo "error: no instrumented test objects found" >&2
    exit 1
fi
echo "==> Reporting over $BIN_COUNT globally deduplicated test objects"
OBJ_ARGS=()
for b in $BINS; do OBJ_ARGS+=(--object "$b"); done

# Denominator exclusions, kept explicit per-file (per-directory only for test
# code) so ordinary testable code added alongside them still counts:
#   src/main.rs, src/lib.rs        — process entry / pure re-export shims
#   extensions/**/extension.rs     — inventory registration glue, no logic
#   */build.rs                     — build scripts
# Keep this regex in sync between coverage.sh and `just coverage-html`.
# systemprompt-core/ is the sibling checkout pulled in via [patch.crates-io];
# it has its own coverage CI and must not dilute this repo's denominator.
# extensions/mcp/*/src/main.rs are `tokio::main` shells that build a server and
# serve it over stdio — the same kind of process entry as the CLI mains beside
# them. They are excluded for a second reason too: whether their object is
# picked up depends on which binaries a run happened to build, so leaving them
# in made a crate's number move by several points when an unrelated test crate
# took a new dependency.
# The repo's own src/main.rs and src/lib.rs are anchored to ROOT (escaped for
# the regex) because a clone's directory name is not fixed.
ROOT_RE="$(printf '%s' "$ROOT" | sed 's/[][\.*^$()+?{}|]/\\&/g')"
IGNORE_RE="(\.cargo|/rustc/|/registry/|/debug/build/|/tests/|/target/|systemprompt-core/|\.vendor/|${ROOT_RE}/src/(main|lib)\.rs|extensions/(cli|mcp)/[^/]+/src/main\.rs|extensions/cli/[^/]+/src/commands/|extensions/.*/extension\.rs|build\.rs)"

echo "==> Coverage report"
WARNINGS="$ROOT/coverage-report/llvm-cov-warnings.txt"
"$LLVM_COV" report \
    --instr-profile="$ROOT/coverage-report/tests.profdata" \
    "${OBJ_ARGS[@]}" \
    --ignore-filename-regex="$IGNORE_RE" \
    2> >(tee "$WARNINGS" >&2) \
    | tee "$ROOT/coverage-report/report.txt"

if grep -q 'mismatched data' "$WARNINGS"; then
    echo "error: llvm-cov rejected profile data; duplicate or incompatible instrumented objects remain" >&2
    echo "       inspect coverage-report/llvm-cov-warnings.txt" >&2
    exit 1
fi

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

if [ "$PHASE_FAILURES" -gt 0 ]; then
    echo "error: $PHASE_FAILURES workspace(s) did not run cleanly — the figure above is measured over a partial build" >&2
    exit 1
fi
exit 0
