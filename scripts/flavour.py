#!/usr/bin/env python3
"""The one reader of flavours/: every script and recipe asks this file.

  flavour.py list [--artefact KIND]   names, in index order
  flavour.py table                    name and description, aligned
  flavour.py get <name> <key>         a value; lists print one item per line,
                                      booleans print true/false, a dotted key
                                      reads a table (smoke.routes_present);
                                      artefact prints image when unset
  flavour.py features <name>          cargo_feature plus extra_features, comma-joined
                                      (image flavours only)
  flavour.py nav-titles <name>        the console sidebar section titles the
                                      flavour's dashboard.yaml configures
  flavour.py matrix [--artefact KIND] the index as a JSON array of
                                      {"flavour", "artefact"} objects (CI matrices)
  flavour.py check                    index, directories, features and services agree

A flavour's artefact is what it produces: image (a server image, the default)
or desktop (installers built from a cargo workspace of its own). KIND keeps
only the flavours of that artefact.
"""
import json
import re
import sys
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parent.parent
FLAVOURS = ROOT / "flavours"
ARTEFACTS = ("image", "desktop")
COMMON = {
    "name": str,
    "description": str,
}
IMAGE = {
    "cargo_feature": str,
    "extra_features": list,
    "binaries": list,
    "services": list,
    "requires_ai_key": bool,
    "site": bool,
    "runtime": str,
}
SMOKE = {
    "routes_present": list,
    "routes_absent": list,
    "cli_groups_present": list,
    "cli_groups_absent": list,
    "capabilities": list,
    "mcp_servers": list,
    "models_endpoint": bool,
}
OPTIONAL_SMOKE = {
    "root_redirect": str,
    "console_pages": list,
    "routes_absent_signed_in": list,
}
RUNTIMES = ("slim", "full")
DESKTOP = {
    "workspace": str,
}
IMAGE_FILES = ("profile.overlay.yaml", "services")


def load(path):
    with path.open(encoding="utf-8") as handle:
        return yaml.safe_load(handle) or {}


def index():
    return load(FLAVOURS / "index.yaml")["flavours"]


def flavour(name):
    if name not in index():
        sys.exit(f"unknown flavour '{name}' (known: {', '.join(index())})")
    data = load(FLAVOURS / name / "flavour.yaml")
    data.setdefault("artefact", "image")
    return data


def selected(kind=None):
    """Index order; with a kind, only the flavours of that artefact."""
    return [name for name in index() if kind is None or flavour(name)["artefact"] == kind]


def cargo_features():
    """The keys of the root Cargo.toml [features] table.

    A line scan, not a TOML parse: the table holds only `name = [...]`
    entries, and reading it this way keeps the script on any python3 with
    PyYAML.
    """
    names, inside = set(), False
    for line in (ROOT / "Cargo.toml").read_text(encoding="utf-8").splitlines():
        if line.startswith("["):
            inside = line.strip() == "[features]"
            continue
        match = re.match(r"([A-Za-z0-9_-]+)\s*=", line) if inside else None
        if match:
            names.add(match.group(1))
    return names


def render(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list):
        return "\n".join(str(item) for item in value)
    return str(value)


def get(name, key):
    value = flavour(name)
    for part in key.split("."):
        if not isinstance(value, dict) or part not in value:
            sys.exit(f"flavour '{name}' has no key '{key}'")
        value = value[part]
    return render(value)


def features(name):
    data = flavour(name)
    if data["artefact"] != "image":
        sys.exit(
            f"flavour '{name}' is a {data['artefact']} artefact: it is built from its own "
            f"workspace ({data.get('workspace')}), not from root cargo features"
        )
    return ",".join([data["cargo_feature"], *data["extra_features"]])


def nav_titles(name):
    path = FLAVOURS / name / "services/web/config/dashboard.yaml"
    if not path.is_file():
        path = ROOT / "services/web/config/dashboard.yaml"
    return "\n".join(section["title"] for section in load(path).get("nav", []))


def check_image(name, base, data, features_declared, problems):
    if not (base / "profile.overlay.yaml").is_file():
        problems.append(f"flavours/{name}/profile.overlay.yaml is missing")
    for key, kind in IMAGE.items():
        if not isinstance(data.get(key), kind):
            problems.append(f"flavours/{name}: '{key}' must be a {kind.__name__}")
    smoke = data.get("smoke", {})
    for key, kind in SMOKE.items():
        if not isinstance(smoke.get(key), kind):
            problems.append(f"flavours/{name}: 'smoke.{key}' must be a {kind.__name__}")
    for key, kind in OPTIONAL_SMOKE.items():
        if key in smoke and not isinstance(smoke[key], kind):
            problems.append(f"flavours/{name}: 'smoke.{key}' must be a {kind.__name__}")
    if data.get("runtime") not in RUNTIMES:
        problems.append(f"flavours/{name}: runtime must be one of {RUNTIMES}")
    if not data.get("binaries"):
        problems.append(f"flavours/{name}: binaries is empty")
    for feature in [data.get("cargo_feature"), *data.get("extra_features", [])]:
        if feature not in features_declared:
            problems.append(f"flavours/{name}: feature '{feature}' is not in Cargo.toml [features]")
    for subtree in data.get("services", []):
        if not (ROOT / "services" / subtree).is_dir():
            problems.append(f"flavours/{name}: services/{subtree} does not exist")
    overrides = base / "services"
    if overrides.exists() and not any(overrides.rglob("*")):
        problems.append(f"flavours/{name}/services/ is empty; delete it")


def check_desktop(name, base, data, problems):
    """A desktop flavour names its workspace and nothing a server image reads."""
    for key, kind in DESKTOP.items():
        if not isinstance(data.get(key), kind):
            problems.append(f"flavours/{name}: '{key}' must be a {kind.__name__}")
    for key in (*IMAGE, "smoke"):
        if key in data:
            problems.append(f"flavours/{name}: '{key}' is read for image flavours only; a desktop flavour does not set it")
    for leftover in IMAGE_FILES:
        if (base / leftover).exists():
            problems.append(f"flavours/{name}/{leftover} is read for image flavours only; delete it")
    workspace = data.get("workspace")
    if isinstance(workspace, str) and not (ROOT / workspace / "Cargo.toml").is_file():
        problems.append(f"flavours/{name}: workspace '{workspace}' has no Cargo.toml")


def check():
    problems = []
    names = index()
    if len(names) != len(set(names)):
        problems.append("flavours/index.yaml lists a flavour twice")
    dirs = sorted(p.name for p in FLAVOURS.iterdir() if p.is_dir())
    for missing in sorted(set(names) - set(dirs)):
        problems.append(f"index names '{missing}' but flavours/{missing}/ does not exist")
    for extra in sorted(set(dirs) - set(names)):
        problems.append(f"flavours/{extra}/ is not listed in flavours/index.yaml")
    features_declared = cargo_features()
    for name in [n for n in names if n in dirs]:
        base = FLAVOURS / name
        for required in ("flavour.yaml", "README.md"):
            if not (base / required).is_file():
                problems.append(f"flavours/{name}/{required} is missing")
        if not (base / "flavour.yaml").is_file():
            continue
        data = load(base / "flavour.yaml")
        for key, kind in COMMON.items():
            if not isinstance(data.get(key), kind):
                problems.append(f"flavours/{name}: '{key}' must be a {kind.__name__}")
        if data.get("name") != name:
            problems.append(f"flavours/{name}: name is '{data.get('name')}'")
        artefact = data.get("artefact", "image")
        if artefact == "image":
            check_image(name, base, data, features_declared, problems)
        elif artefact == "desktop":
            check_desktop(name, base, data, problems)
        else:
            problems.append(f"flavours/{name}: artefact must be one of {ARTEFACTS}")
    for problem in problems:
        print(f"check-flavours: {problem}", file=sys.stderr)
    if problems:
        return 1
    print(f"check-flavours: OK ({len(names)} flavour(s): {', '.join(names)})")
    return 0


def main(argv):
    command = argv[1] if len(argv) > 1 else ""
    kind = None
    if command in ("list", "matrix") and len(argv) == 4 and argv[2] == "--artefact":
        if argv[3] not in ARTEFACTS:
            sys.exit(f"unknown artefact '{argv[3]}' (known: {', '.join(ARTEFACTS)})")
        kind = argv[3]
        argv = argv[:2]
    if command == "list" and len(argv) == 2:
        print("\n".join(selected(kind)))
    elif command == "table" and len(argv) == 2:
        rows = [(n, flavour(n)["description"]) for n in index()]
        width = max(len(n) for n, _ in rows)
        for name, description in rows:
            print(f"{name.ljust(width)}  {description}")
    elif command == "get" and len(argv) == 4:
        print(get(argv[2], argv[3]))
    elif command == "features" and len(argv) == 3:
        print(features(argv[2]))
    elif command == "nav-titles" and len(argv) == 3:
        print(nav_titles(argv[2]))
    elif command == "matrix" and len(argv) == 2:
        print(json.dumps([{"flavour": n, "artefact": flavour(n)["artefact"]} for n in selected(kind)]))
    elif command == "check" and len(argv) == 2:
        return check()
    else:
        sys.exit(__doc__)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
