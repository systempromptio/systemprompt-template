#!/usr/bin/env bash
# Coverage floor + ratchet, enforced against the tracked coverage/baseline.json.
#
# Fails when:
#   - total line coverage < floor (baseline "floor" field), or
#   - total drops more than 0.5pt below the recorded baseline total, or
#   - any per-crate figure drops more than 1.0pt below its recorded baseline.
#
# Raising the numbers is a deliberate, committed act: run
# `UPDATE_BASELINE=1 scripts/coverage-check.sh` (or `just coverage-baseline`)
# and commit the rewritten coverage/baseline.json.
#
# CI without a tracked baseline (.github/workflows/coverage.yml) must not be
# red forever for want of a file only a person can commit, so two opt-in modes
# let it carry its own baseline between runs:
#   COVERAGE_BASELINE=<path>  read/write this baseline instead of the tracked one
#   COVERAGE_ESTABLISH=1      no baseline yet: record this measurement as the
#                             baseline and pass (first run reports and establishes)
#   COVERAGE_RATCHET=1        after a pass, raise the stored baseline to any
#                             figure that improved (never lowers it)
# The tracked coverage/baseline.json, once committed, always wins in CI.
#
# Reads coverage-report/summary.json produced by scripts/coverage.sh.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUMMARY="$ROOT/coverage-report/summary.json"
TRACKED_BASELINE="$ROOT/coverage/baseline.json"
BASELINE="${COVERAGE_BASELINE:-$TRACKED_BASELINE}"

if [ ! -f "$SUMMARY" ]; then
    echo "error: $SUMMARY not found — run 'just coverage' first" >&2
    exit 1
fi

# Aggregate per-file llvm-cov summaries into per-crate line coverage. Crate
# key = path relative to the repo root, truncated to the crate directory
# (extensions/web/admin, extensions/mcp/shared, bridge, src, ...).
python3 - "$SUMMARY" "$BASELINE" "$TRACKED_BASELINE" <<'PY'
import json, os, re, sys

summary_path, baseline_path, tracked_path = sys.argv[1], sys.argv[2], sys.argv[3]
root = os.path.dirname(os.path.dirname(os.path.abspath(summary_path)))
update = os.environ.get("UPDATE_BASELINE") == "1"
establish = os.environ.get("COVERAGE_ESTABLISH") == "1"
ratchet = os.environ.get("COVERAGE_RATCHET") == "1"

with open(summary_path) as f:
    data = json.load(f)["data"][0]

total = data["totals"]["lines"]["percent"]

CRATE_PATTERNS = [
    re.compile(r"^(extensions/(?:web/(?:admin|site|content|jobs|shared)|web|mcp/[^/]+|cli/[^/]+))/"),
    re.compile(r"^(bridge)/"),
    re.compile(r"^(src)/"),
]

crates = {}
for entry in data.get("files", []):
    rel = os.path.relpath(entry["filename"], root)
    if rel.startswith(".."):
        continue
    for pat in CRATE_PATTERNS:
        m = pat.match(rel)
        if m:
            covered, count = crates.setdefault(m.group(1), [0, 0])
            lines = entry["summary"]["lines"]
            crates[m.group(1)] = [covered + lines["covered"], count + lines["count"]]
            break

per_crate = {
    k: round(100.0 * cov / cnt, 2) if cnt else 100.0
    for k, (cov, cnt) in sorted(crates.items())
}

print(f"total: {total:.2f}%")
for k, v in per_crate.items():
    print(f"  {v:6.2f}%  {k}")

if update:
    baseline = {"floor": 0.0, "total": round(total, 2), "per_crate": per_crate}
    if os.path.exists(baseline_path):
        with open(baseline_path) as f:
            previous = json.load(f)
        baseline["floor"] = previous.get("floor", 0.0)
        # Why: a coverage run that produced no usable objects reports 0.00%,
        # and recording that silently lowers the bar for everyone afterwards.
        # It has happened: two concurrent runs deleted each other's
        # instrumented binaries, every object was lost, and the "measurement"
        # was written straight into the baseline. A run that lost most of its
        # data is a broken run, never a real regression, so refuse it here and
        # let a genuine drop be recorded by hand.
        floor = previous.get("total", 0.0) / 2.0
        if total < floor:
            print(
                f"refusing to record {total:.2f}% as the baseline: that is less than half "
                f"the recorded {previous.get('total', 0.0):.2f}%, which means the run lost "
                f"its coverage data rather than the code losing its tests.\n"
                f"Re-run `just coverage`, "
                f"check nothing else is running coverage at the same time, and if the drop "
                f"is genuine edit {os.path.relpath(baseline_path, root)} by hand.",
                file=sys.stderr,
            )
            sys.exit(1)
    os.makedirs(os.path.dirname(baseline_path), exist_ok=True)
    with open(baseline_path, "w") as f:
        json.dump(baseline, f, indent=2)
        f.write("\n")
    print(f"baseline updated: {os.path.relpath(baseline_path, root)} "
          f"(floor {baseline['floor']}, total {baseline['total']})")
    sys.exit(0)

def write_baseline(path, baseline):
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "w") as f:
        json.dump(baseline, f, indent=2)
        f.write("\n")

if not os.path.exists(baseline_path) and establish:
    # Why the zero guard: a run that lost its objects reports 0.00%, and
    # establishing that would make every later run pass trivially.
    if total <= 0.0:
        print("refusing to establish a 0.00% baseline: the run produced no coverage data",
              file=sys.stderr)
        sys.exit(1)
    write_baseline(baseline_path, {"floor": 0.0, "total": round(total, 2), "per_crate": per_crate})
    print(f"::notice title=Coverage baseline established::{total:.2f}% recorded as the "
          f"first baseline; later runs ratchet against it. Commit it as "
          f"coverage/baseline.json with 'just coverage-baseline' to make it durable.")
    print("coverage-check passed (baseline established)")
    sys.exit(0)

if not os.path.exists(baseline_path) and os.path.abspath(baseline_path) != os.path.abspath(tracked_path):
    print(f"error: {baseline_path} missing — set COVERAGE_ESTABLISH=1 to record it",
          file=sys.stderr)
    sys.exit(1)

if not os.path.exists(baseline_path):
    # The baseline is tracked, so "missing" almost always means it was deleted
    # from the working tree rather than never recorded — a stray `rm -rf
    # coverage*` catches it alongside the gitignored coverage-report/, and the
    # gate then fails for a reason that has nothing to do with the code under
    # test. Restore it from HEAD and say so loudly; only give up if git cannot
    # produce it either, which is the genuinely-never-recorded case.
    import subprocess
    rel = os.path.relpath(baseline_path, root)
    restored = subprocess.run(
        ["git", "-C", root, "checkout", "--", rel],
        capture_output=True, text=True).returncode == 0
    if restored and os.path.exists(baseline_path):
        print(f"warning: {rel} was missing from the working tree and has been "
              f"restored from HEAD. Something deleted it — check for a cleanup "
              f"that globs 'coverage*' rather than the gitignored "
              f"'coverage-report/'.", file=sys.stderr)
    else:
        print(f"error: {baseline_path} missing and not recoverable from git — "
              f"record it with 'just coverage-baseline'", file=sys.stderr)
        sys.exit(1)

with open(baseline_path) as f:
    baseline = json.load(f)

failures = []
floor = baseline.get("floor", 0.0)
if total < floor:
    failures.append(f"total {total:.2f}% is below the floor {floor:.2f}%")
if total < baseline["total"] - 0.5:
    failures.append(
        f"total {total:.2f}% dropped >0.5pt below baseline {baseline['total']:.2f}%")
for crate, base in baseline.get("per_crate", {}).items():
    now = per_crate.get(crate)
    if now is None:
        failures.append(f"{crate}: no coverage data (baseline {base:.2f}%)")
    elif now < base - 1.0:
        failures.append(f"{crate}: {now:.2f}% dropped >1pt below baseline {base:.2f}%")

if failures:
    print("coverage-check FAILED:", file=sys.stderr)
    for msg in failures:
        print(f"  - {msg}", file=sys.stderr)
    print("If the drop is deliberate, re-record with 'just coverage-baseline' "
          "and commit coverage/baseline.json.", file=sys.stderr)
    sys.exit(1)

if ratchet:
    raised = dict(baseline)
    raised["total"] = max(baseline["total"], round(total, 2))
    raised["per_crate"] = {
        crate: max(baseline.get("per_crate", {}).get(crate, 0.0), now)
        for crate, now in per_crate.items()
    }
    if raised != baseline:
        write_baseline(baseline_path, raised)
        print(f"baseline ratcheted: total {baseline['total']:.2f}% -> {raised['total']:.2f}%")

better = total - baseline["total"]
if better > 0.5 and not ratchet:
    print(f"note: total is {better:.2f}pt above baseline — ratchet up with "
          f"'just coverage-baseline' and commit the result.")
print("coverage-check passed")
PY
