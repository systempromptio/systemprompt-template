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
# Reads coverage-report/summary.json produced by scripts/coverage.sh.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUMMARY="$ROOT/coverage-report/summary.json"
BASELINE="$ROOT/coverage/baseline.json"

if [ ! -f "$SUMMARY" ]; then
    echo "error: $SUMMARY not found — run 'just coverage' first" >&2
    exit 1
fi

# Aggregate per-file llvm-cov summaries into per-crate line coverage. Crate
# key = path relative to the repo root, truncated to the crate directory
# (extensions/web/admin, extensions/mcp/shared, bridge, src, ...).
python3 - "$SUMMARY" "$BASELINE" <<'PY'
import json, os, re, sys

summary_path, baseline_path = sys.argv[1], sys.argv[2]
root = os.path.dirname(os.path.dirname(os.path.abspath(summary_path)))
update = os.environ.get("UPDATE_BASELINE") == "1"

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
            baseline["floor"] = json.load(f).get("floor", 0.0)
    os.makedirs(os.path.dirname(baseline_path), exist_ok=True)
    with open(baseline_path, "w") as f:
        json.dump(baseline, f, indent=2)
        f.write("\n")
    print(f"baseline updated: {os.path.relpath(baseline_path, root)} "
          f"(floor {baseline['floor']}, total {baseline['total']})")
    sys.exit(0)

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

better = total - baseline["total"]
if better > 0.5:
    print(f"note: total is {better:.2f}pt above baseline — ratchet up with "
          f"'just coverage-baseline' and commit the result.")
print("coverage-check passed")
PY
