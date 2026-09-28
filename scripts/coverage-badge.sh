#!/usr/bin/env bash
# Render the README's coverage badge from coverage/baseline.json.
#
# The badge is static markup rather than a third-party service: coverage here
# is measured locally by scripts/coverage.sh and recorded in a tracked
# baseline, so the number is already in the repository and there is nothing to
# gain by shipping this codebase's file paths to an external host to have it
# read back.
#
# That only works if the badge cannot drift from the baseline, which is what
# `--check` is for: it runs as a source gate, so a `just coverage-baseline`
# that moved the number and left the README behind fails the build rather than
# quietly advertising a figure nobody measured.
#
# Checking is the default, because this runs in the `just lint-gates` array
# alongside two dozen other read-only checks and a gate that rewrote a tracked
# file as a side effect of being run would be a trap.
#
#   scripts/coverage-badge.sh           fail if it disagrees with the baseline
#   scripts/coverage-badge.sh --write   rewrite the badge in place
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
MODE="${1:---check}"

python3 - "$ROOT" "$MODE" <<'PY'
import json, pathlib, re, sys

root, mode = pathlib.Path(sys.argv[1]), sys.argv[2]
baseline_path = root / "coverage" / "baseline.json"
readme_path = root / "README.md"

if not baseline_path.exists():
    print("coverage-badge: no coverage/baseline.json - nothing to render")
    sys.exit(0)

total = json.loads(baseline_path.read_text())["total"]

# Why: the thresholds are the ones the eye already reads on a badge — green is
# "healthy", amber "watch it", red "a problem" — and they sit either side of
# the floor this repo actually enforces rather than at conventional round
# numbers.
colour = "16a34a" if total >= 80 else "f97316" if total >= 60 else "dc2626"
badge = (
    f"[![Coverage {total}%]"
    f"(https://img.shields.io/badge/coverage-{total}%25-{colour}?style=flat-square)]"
    f"(coverage/baseline.json)"
)

readme = readme_path.read_text()
pattern = re.compile(r"\[!\[Coverage [^\]]*\]\(https://img\.shields\.io/badge/coverage-[^)]*\)\]\([^)]*\)")

if not pattern.search(readme):
    if mode == "--check":
        print(
            "coverage-badge: README.md carries no coverage badge.\n"
            "  Add one with: scripts/coverage-badge.sh",
            file=sys.stderr,
        )
        sys.exit(1)
    print("coverage-badge: README.md carries no coverage badge to update", file=sys.stderr)
    sys.exit(1)

updated = pattern.sub(badge, readme, count=1)

if mode == "--check":
    if updated != readme:
        print(
            f"coverage-badge: the README badge disagrees with coverage/baseline.json ({total}%).\n"
            "  Refresh it with: just coverage-badge",
            file=sys.stderr,
        )
        sys.exit(1)
    print(f"coverage-badge: OK (README agrees with the baseline at {total}%)")
    sys.exit(0)

readme_path.write_text(updated)
print(f"coverage-badge: README badge set to {total}%")
PY
