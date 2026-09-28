#!/usr/bin/env bash
# Cross-file referential integrity for the services/ YAML tree.
#
# Catches at commit time what otherwise only fails (or silently stops
# matching) at boot: access-control rules pointing at ids that no resource
# defines, plugins and marketplaces including members no file declares (the
# composition rule ServicesConfig::validate() enforces — a dangling include
# stops the tree composing, which once surfaced on the access-control page as
# "rules.yaml names a marketplace that does not exist"), and MCP port
# declarations drifting between services/mcp/ and the extension manifest.
set -uo pipefail
cd "$(dirname "$0")/.."

python3 - <<'EOF'
import pathlib
import sys

import yaml

root = pathlib.Path(".")
errors = []


def load(path):
    try:
        return yaml.safe_load(path.read_text()) or {}
    except yaml.YAMLError as e:
        errors.append(f"{path}: unparseable YAML: {e}")
        return {}


skills = {
    load(p).get("id")
    for p in root.glob("services/skills/*/config.yaml")
}
agents = set()
for p in root.glob("services/agents/*.yaml"):
    agents.update((load(p).get("agents") or {}).keys())
mcp_servers = set()
for p in root.glob("services/mcp/*.yaml"):
    mcp_servers.update((load(p).get("mcp_servers") or {}).keys())
marketplaces = {
    (load(p).get("marketplace") or {}).get("id")
    for p in root.glob("services/marketplaces/*/config.yaml")
}
plugins = set()
for p in root.glob("services/plugins/*/config.yaml"):
    doc = load(p)
    # Two shapes are in use: a `plugins:` map keyed by id, and a single
    # `plugin:` block that carries its own `id`. Both name the same thing.
    plugins.update((doc.get("plugins") or {}).keys())
    single = (doc.get("plugin") or {}).get("id")
    if single:
        plugins.add(single)

# Cross-file includes. A plugin lists skills, agents and mcp servers by id; a
# marketplace lists plugins by id. Composition refuses an id nothing declares.
def includes(block):
    block = block or {}
    if block.get("source", "explicit") != "explicit":
        return []
    return block.get("include") or []


for p in root.glob("services/plugins/*/config.yaml"):
    doc = load(p)
    blocks = list((doc.get("plugins") or {}).values())
    if doc.get("plugin"):
        blocks.append(doc["plugin"])
    for block in blocks:
        pid = block.get("id", p.parent.name)
        for key, pool in (("skills", skills), ("agents", agents), ("mcp_servers", mcp_servers)):
            for member in includes(block.get(key)):
                if member not in pool:
                    errors.append(
                        f"{p}: plugin '{pid}': {key}.include references unknown "
                        f"{key[:-1]} '{member}'"
                    )

for p in root.glob("services/marketplaces/*/config.yaml"):
    mp = load(p).get("marketplace") or {}
    if not mp.get("id"):
        errors.append(f"{p}: marketplace declares no id")
    for member in includes(mp.get("plugins")):
        if member not in plugins:
            errors.append(
                f"{p}: marketplace '{mp.get('id')}': plugins.include references "
                f"unknown plugin '{member}'"
            )
    for member in includes(mp.get("mcp_servers")):
        if member not in mcp_servers:
            errors.append(
                f"{p}: marketplace '{mp.get('id')}': mcp_servers.include references "
                f"unknown mcp_server '{member}'"
            )

known = {
    "skill": skills,
    "agent": agents,
    "mcp_server": mcp_servers,
    "marketplace": marketplaces,
    "plugin": plugins,
}
# services/access-control/rules.yaml is the ONE declarative source of
# entitlement. Every entity it names must exist, every group/project it names
# must be declared, every entity must say why, and no marketplace config may
# carry an `access:` block of its own — that second truth is exactly what this
# file replaced.
RULES = root / "services/access-control/rules.yaml"
BANDS = {"role", "group", "project", "connector"}
# Nothing registers a `hook` entity and gateway_route ids are generated, so a
# literal id of either kind would be minted rather than validated. Only the
# glob is accepted for them, and only for them.
glob_only = {"gateway_route", "hook"}

groups_doc = load(root / "services/web/config/groups.yaml")
group_ids = {g.get("id") for g in (groups_doc.get("groups") or [])} | {"unassigned"}
project_ids = {p.get("id") for p in (groups_doc.get("projects") or [])}
member_ids = {"group": group_ids, "project": project_ids}


def band_values(spec):
    if isinstance(spec, dict):
        return spec.get("values") or [], spec.get("why")
    return spec or [], None


rules_doc = load(RULES) if RULES.exists() else {}
for decl in rules_doc.get("entities") or []:
    ref = str(decl.get("entity", ""))
    if "/" not in ref:
        errors.append(f"rules.yaml: entity '{ref}' must be written as <kind>/<id>")
        continue
    etype, eid = ref.split("/", 1)
    if not str(decl.get("why") or "").strip():
        errors.append(f"rules.yaml: {ref}: `why` is required")
    if decl.get("default", "closed") not in ("open", "closed"):
        errors.append(f"rules.yaml: {ref}: default must be open or closed")
    if etype in glob_only:
        if eid != "*":
            errors.append(
                f"rules.yaml: {ref}: {etype} ids are generated, never written — use {etype}/*"
            )
    elif "*" in eid:
        errors.append(f"rules.yaml: {ref}: only gateway_route and hook take a glob")
    else:
        pool = known.get(etype)
        owner = decl.get("owner")
        # An entity a remote bundle owns (`owner: bundle:<name>`) may be absent
        # from this tree: composition forbids an id both local and bundled, so
        # the kit's marketplace is declared here and arrives with the bundle.
        if owner is not None:
            if not (isinstance(owner, str) and owner.startswith("bundle:") and owner[7:].strip()):
                errors.append(f"rules.yaml: {ref}: owner must be written as bundle:<name>")
            if etype not in ("marketplace", "plugin", "skill"):
                errors.append(f"rules.yaml: {ref}: only a marketplace, plugin or skill can name an owner")
            if pool is not None and eid in pool:
                errors.append(
                    f"rules.yaml: {ref}: is defined in this tree and names owner {owner} — "
                    f"composition refuses an id that is both local and bundled"
                )
        elif pool is None:
            errors.append(f"rules.yaml: {ref}: unknown entity kind '{etype}'")
        elif eid not in pool:
            errors.append(f"rules.yaml: {ref}: matches no defined resource")
    allow = decl.get("allow") or {}
    deny = decl.get("deny") or {}
    if not allow and not deny:
        errors.append(f"rules.yaml: {ref}: declares no allow and no deny")
    for verb, bands in (("allow", allow), ("deny", deny)):
        for band, spec in bands.items():
            if band not in BANDS:
                errors.append(f"rules.yaml: {ref}: unknown band '{band}' under {verb}")
                continue
            values, why = band_values(spec)
            if not values:
                errors.append(f"rules.yaml: {ref}: {verb}.{band} names no subjects")
            if isinstance(spec, dict) and not str(why or "").strip():
                errors.append(f"rules.yaml: {ref}: {verb}.{band} has a `why` key that is empty")
            for value in values:
                if band in member_ids and value not in member_ids[band]:
                    errors.append(
                        f"rules.yaml: {ref}: {band} '{value}' is not declared in "
                        f"services/web/config/groups.yaml"
                    )
    for band in set(allow) & set(deny):
        both = set(band_values(allow[band])[0]) & set(band_values(deny[band])[0])
        for value in sorted(both):
            errors.append(f"rules.yaml: {ref}: {band} '{value}' is both allowed and denied")

for p in root.glob("services/marketplaces/*/config.yaml"):
    if "access" in (load(p).get("marketplace") or {}):
        errors.append(
            f"{p}: marketplace configs carry no `access:` block — declare "
            f"marketplace/<id> in services/access-control/rules.yaml instead"
        )

for svc_path in root.glob("services/mcp/*.yaml"):
    for name, cfg in (load(svc_path).get("mcp_servers") or {}).items():
        manifest_path = root / "extensions/mcp" / name / "manifest.yaml"
        if not manifest_path.is_file():
            continue
        service_port = cfg.get("port")
        manifest_port = (load(manifest_path).get("extension") or {}).get("port")
        if manifest_port is not None and service_port != manifest_port:
            errors.append(
                f"{svc_path}: port {service_port} disagrees with "
                f"{manifest_path}: port {manifest_port}"
            )

# The checked-in marketplace JSON under storage/files/plugins/.claude-plugin/
# is generated from services config (core: plugins/generate/marketplace.rs).
# It went stale once (phantom per-plugin agents survived a config rewrite), so
# pin its plugin list and version to the marketplace config here.
import json

for mp_path in root.glob("services/marketplaces/*/config.yaml"):
    mp = (load(mp_path) or {}).get("marketplace") or {}
    mp_id = mp.get("id")
    json_path = (
        root / "storage/files/plugins/.claude-plugin" / f"marketplace-{mp_id}.json"
    )
    if not json_path.is_file():
        continue
    generated = json.loads(json_path.read_text())
    declared = list((mp.get("plugins") or {}).get("include") or [])
    emitted = [p.get("name") for p in generated.get("plugins") or []]
    if declared != emitted:
        errors.append(
            f"{json_path}: plugin list {emitted} is stale — marketplace config "
            f"declares {declared}; regenerate the marketplace JSON"
        )
    declared_version = mp.get("version")
    emitted_version = (generated.get("metadata") or {}).get("version")
    if declared_version != emitted_version:
        errors.append(
            f"{json_path}: version {emitted_version} is stale — marketplace "
            f"config declares {declared_version}"
        )

if errors:
    print("services validation FAILED:")
    for e in errors:
        print(f"  {e}")
    sys.exit(1)
print("services validation OK")
EOF
