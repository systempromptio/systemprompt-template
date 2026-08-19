#!/usr/bin/env bash
# Pre-merge gate enforcing the extension-crate layering (ported from core's
# lint-layers.sh, adapted to this repo's topology):
#
#   facade (src/) -> aggregator (extensions/web) -> app (site, jobs)
#     -> domain (admin, content) -> shared (web/shared, mcp/shared)
#
# extensions/cli/* and extensions/mcp/* (non-shared) are entry-tier peers of
# the aggregator. Three properties must hold exactly:
#
#   1. No dependency points upward through the layer stack.
#   2. No dependency cycles.
#   3. No domain -> domain dependencies (admin and content are peers; shared
#      carries anything they both need).
#
# Layer membership is read from each crate's position on disk, so a crate
# moved between directories is re-classified automatically. Only normal and
# build dependencies count: dev-dependencies are not part of the shipped graph.
set -euo pipefail

cd "$(dirname "$0")/.."

command -v python3 >/dev/null || { echo "lint-layers: python3 not found"; exit 1; }

cargo metadata --no-deps --format-version 1 | python3 -c '
import json, re, sys
from collections import defaultdict

ORDER = {"shared": 0, "domain": 1, "app": 2, "entry": 3, "facade": 4}

RULES = [
    (r"/extensions/(web|mcp)/shared/", "shared"),
    (r"/extensions/web/(admin|content)/", "domain"),
    (r"/extensions/web/(site|jobs)/", "app"),
    (r"/extensions/web/Cargo\.toml$", "entry"),
    (r"/extensions/(cli|mcp)/", "entry"),
]

md = json.load(sys.stdin)
pkgs = {p["name"]: p for p in md["packages"]}
local = set(pkgs)

layer = {}
for name, pkg in pkgs.items():
    path = pkg["manifest_path"]
    for pat, lyr in RULES:
        if re.search(pat, path):
            layer[name] = lyr
            break
    else:
        layer[name] = "facade" if "/extensions/" not in path else None

unknown = sorted(n for n, l in layer.items() if l not in ORDER)
if unknown:
    for n in unknown:
        print(f"  {n}: unrecognised layer for {pkgs[n]['manifest_path']}")
    print("lint-layers: FAIL — crate outside the known layer taxonomy")
    sys.exit(1)

deps = defaultdict(set)
for name, pkg in pkgs.items():
    for d in pkg["dependencies"]:
        if d["name"] in local and d["name"] != name and d["kind"] in (None, "build"):
            deps[name].add(d["name"])

violations = []
for name in sorted(local):
    for dep in sorted(deps[name]):
        if ORDER[layer[dep]] > ORDER[layer[name]]:
            violations.append(f"  {name} ({layer[name]}) -> {dep} ({layer[dep]})")
        elif layer[name] == "domain" and layer[dep] == "domain":
            violations.append(
                f"  {name} (domain) -> {dep} (domain): domain crates must not depend on each other"
            )

WHITE, GREY, BLACK = 0, 1, 2
colour = defaultdict(int)
stack = []
cycles = []

def visit(node):
    colour[node] = GREY
    stack.append(node)
    for dep in sorted(deps[node]):
        if colour[dep] == GREY:
            cycles.append(" -> ".join(stack[stack.index(dep):] + [dep]))
        elif colour[dep] == WHITE:
            visit(dep)
    stack.pop()
    colour[node] = BLACK

for name in sorted(local):
    if colour[name] == WHITE:
        visit(name)

if violations:
    print("Dependencies pointing upward through the layer stack:")
    print("\n".join(violations))
if cycles:
    print("Dependency cycles:")
    for c in cycles:
        print(f"  {c}")

if violations or cycles:
    print(f"lint-layers: FAIL — {len(violations)} layer violation(s), {len(cycles)} cycle(s)")
    sys.exit(1)

print(f"lint-layers: OK — {len(local)} crates, no upward dependencies, no cycles, domain isolation holds")
'
