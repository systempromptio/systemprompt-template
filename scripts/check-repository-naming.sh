#!/usr/bin/env bash
# Check that repository function names match what they return.
#
# The convention is documented in CLAUDE.md: `Vec<T>` is `list_`, `Option<T>`
# is `find_`, a single value is `get_`. It had drifted because one prefix —
# `fetch_` — was doing all three jobs, 85 times against 14 `get_` and 7
# `find_`. A reader could not tell from `fetch_summary` whether a missing row
# came back as `None` or as an error without opening the file.
#
# Banning `fetch_` alone would not hold the line: nothing would stop a
# `get_top_users() -> Vec<T>`. So the prefix is checked against the real return
# type. Mutations (`insert_`/`update_`/`delete_`/`set_`/`count_`) name an
# action rather than a shape and are left alone.
#
# Exemption: annotate with `// lint-ok: repository-naming` on the line above,
# and say why.
set -uo pipefail

REPO_DIR="${REPO_DIR:-extensions/web/admin/src/repositories}"

[ -d "$REPO_DIR" ] || { echo "check-repository-naming: no $REPO_DIR - nothing to check"; exit 0; }

python3 - "$REPO_DIR" <<'PY'
import re, sys, pathlib

root = pathlib.Path(sys.argv[1])
SHAPE_PREFIXES = ("list_", "find_", "get_")
violations = []

def return_type(src, open_paren):
    """Text between the closing paren of the arg list and the body brace."""
    i, depth = open_paren, 0
    while i < len(src):
        if src[i] == '(':
            depth += 1
        elif src[i] == ')':
            depth -= 1
            if depth == 0:
                break
        i += 1
    tail = src[i + 1:]
    # A `where` clause or generics can sit between; the body brace ends it.
    head = tail.split('{')[0].strip()
    return ' '.join(head[2:].split()) if head.startswith('->') else '()'

def shape_of(rt):
    """list | find | get, from the type the caller actually receives."""
    inner = rt
    m = re.match(r'Result\s*<(.*)>$', rt, re.S)
    if m:
        inner, depth = m.group(1), 0
        for j, c in enumerate(inner):
            if c == '<':
                depth += 1
            elif c == '>':
                depth -= 1
            elif c == ',' and depth == 0:
                inner = inner[:j]
                break
    inner = inner.strip()
    if inner.startswith('Vec<'):
        return 'list'
    if inner.startswith('Option<'):
        return 'find'
    # A page and its total, or a map keyed by id, is still many things.
    if inner.startswith('(Vec<') or inner.startswith(('HashMap<', 'BTreeMap<')):
        return 'list'
    return 'get'

for path in sorted(root.rglob('*.rs')):
    src = path.read_text()
    lines = src.splitlines()
    for m in re.finditer(r'\bfn ([a-z][a-z0-9_]*)\s*(?:<[^>]*>)?\s*\(', src):
        name = m.group(1)
        line_no = src[:m.start()].count('\n') + 1
        if line_no > 1 and 'lint-ok: repository-naming' in lines[line_no - 2]:
            continue
        where = f"{path}:{line_no}"
        if name.startswith('fetch_'):
            violations.append((where, name, "`fetch_` says nothing about what comes back"))
            continue
        prefix = next((p for p in SHAPE_PREFIXES if name.startswith(p)), None)
        if prefix is None:
            continue
        want = shape_of(return_type(src, m.end() - 1))
        if want != prefix[:-1]:
            violations.append((where, name, f"returns a {want}, so it is `{want}_`"))

if violations:
    print("check-repository-naming: name(s) that do not match their return type:", file=sys.stderr)
    for where, name, why in violations:
        print(f"  {name} — {why}", file=sys.stderr)
        print(f"    {where}", file=sys.stderr)
    print("", file=sys.stderr)
    print("See the naming table in CLAUDE.md, or annotate with", file=sys.stderr)
    print("'// lint-ok: repository-naming' and a reason.", file=sys.stderr)
    sys.exit(1)

print("check-repository-naming: OK (every repository name matches its return type)")
PY
