#!/usr/bin/env python3
"""Check a kit tree for the shapes core's importer accepts silently but the
platform serves badly, and for the ones it refuses with an offset instead of a
reason.

Copied verbatim into every kit repository from the instance repository's
`deploy/kit/tools/`; the kits are the same shape by construction, so this file
is edited there and copied, never edited in place.

Core requires exactly one frontmatter key, `description`, and derives a skill's
id from its DIRECTORY name, not from `name`. Everything else degrades quietly:
a skill with no `title` is listed as its de-hyphenated id, one with no
`category` inherits the plugin's, one with no `display_category` groups under
"General" and sorts last on the public skills page. A reference the body cites
but does not carry is simply absent from the bundle. This script makes each of
those visible, as a GitHub annotation on the file and line that causes it.

Errors are release blockers; warnings are shape drift worth fixing. `--strict`
exits 1 on any error, which is how CI consumes it.
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path

import yaml

# Why: the importer reads these from frontmatter because a kit carries no
# config.yaml. `just kit-export` writes exactly this set, so it is the shape a
# hand-authored skill is measured against.
CANONICAL_KEYS = ("name", "title", "description", "tags", "category", "display_category")
FRONTMATTER = re.compile(r"\A---\r?\n(.*?)\r?\n---\r?\n?", re.DOTALL)
# Why: only material the skill SHIPS is checkable. A body also cites files in
# the user's own repository at runtime (`state.json`, `sfdx-project.json`,
# `docs/PROJECT_SCHEMA.md`) — those are not ours to find. A `references/` path,
# or one climbing out of the skill directory, is a claim about the bundle.
CITATION = re.compile(r"`([^`\n]*references/[^`\n]+|\.\.\/[^`\n]+)`")


class Findings:
    """Every problem, as an annotation and a summary row."""

    def __init__(self):
        self.errors = []
        self.warnings = []

    def add(self, severity, path, line, message, fix):
        (self.errors if severity == "error" else self.warnings).append(
            {"path": str(path), "line": line, "message": message, "fix": fix}
        )

    def error(self, path, line, message, fix):
        self.add("error", path, line, message, fix)

    def warn(self, path, line, message, fix):
        self.add("warning", path, line, message, fix)


def frontmatter_of(text):
    """The parsed frontmatter, its raw block, and the line the body starts on."""
    match = FRONTMATTER.match(text)
    if match is None:
        return None, None, 1
    return match.group(1), match.group(0), match.group(0).count("\n") + 1


def line_of_key(raw, key):
    """The 1-based line a frontmatter key sits on, for the annotation."""
    if raw is None:
        return 1
    for offset, line in enumerate(raw.splitlines(), start=2):
        if line.startswith(f"{key}:"):
            return offset
    return 1


def check_skill(skill_dir, plugin_id, prefix, out):
    """One skill directory: it must carry a SKILL.md, that file must parse, and
    everything it cites must sit beside it."""
    rel = skill_dir.relative_to(skill_dir.parents[3])
    md = skill_dir / "SKILL.md"
    if not md.is_file():
        carried = sorted(p.name for p in skill_dir.iterdir())
        out.error(
            rel,
            1,
            f"skill directory has no SKILL.md, so core does not import it and "
            f"nothing it holds ({', '.join(carried) or 'nothing'}) reaches the bundle",
            "add a SKILL.md, or move these files into the skills that cite them",
        )
        return None

    rel_md = rel / "SKILL.md"
    text = md.read_text(encoding="utf-8")
    raw, block, body_line = frontmatter_of(text)
    if raw is None:
        out.error(rel_md, 1, "no YAML frontmatter", "open the file with a --- delimited block")
        return None

    try:
        front = yaml.safe_load(raw)
    except yaml.YAMLError as exc:
        mark = getattr(exc, "problem_mark", None)
        line = (mark.line + 2) if mark is not None else 1
        problem = getattr(exc, "problem", str(exc))
        out.error(
            rel_md,
            line,
            f"frontmatter is not valid YAML: {problem}",
            'quote the value — a bare colon inside a description starts a nested mapping',
        )
        return None

    if not isinstance(front, dict):
        out.error(rel_md, 1, f"frontmatter is {type(front).__name__}, not a mapping", "write key: value pairs")
        return None

    if not str(front.get("description") or "").strip():
        out.error(
            rel_md,
            line_of_key(raw, "description"),
            "no non-empty 'description' — the one key core requires",
            "describe when the model should reach for this skill",
        )

    directory = skill_dir.name
    if prefix and not directory.startswith(prefix):
        out.error(
            rel,
            1,
            f"skill id '{directory}' does not carry the '{prefix}' prefix this plugin declares; "
            f"ids are claimed globally, so a generic id collides with another kit at boot",
            f"rename the directory to '{prefix}{directory}'",
        )

    name = front.get("name")
    if name is not None and name != directory:
        out.warn(
            rel_md,
            line_of_key(raw, "name"),
            f"'name: {name}' does not match the directory '{directory}', which is the real id",
            f"set name: {directory}",
        )

    missing = [k for k in CANONICAL_KEYS if k not in front]
    if missing:
        out.warn(
            rel_md,
            1,
            f"frontmatter omits {', '.join(missing)} — the skill is listed by its "
            f"de-hyphenated id and groups under the plugin category",
            "match the shape `just kit-export` writes: " + ", ".join(CANONICAL_KEYS),
        )

    body = text[len(block):] if block else text
    for offset, line in enumerate(body.splitlines(), start=body_line):
        for cited in CITATION.findall(line):
            if ".." in Path(cited).parts:
                out.error(
                    rel_md,
                    offset,
                    f"cites '{cited}', which escapes the skill directory; composition renames "
                    f"skill directories to snake_case, so a relative path to a sibling skill "
                    f"never resolves on the instance",
                    "copy the file into this skill's own references/ and cite it directly",
                )
                continue
            if not (skill_dir / cited).exists():
                out.error(
                    rel_md,
                    offset,
                    f"cites '{cited}', which this skill does not carry",
                    f"add {cited} beside SKILL.md, or drop the citation",
                )
    return directory


def check_plugin(plugin_dir, marketplace_entry, prefixes, out):
    """One plugin: a category somewhere, a manifest that matches disk, and no
    two skills covering the same topic."""
    plugin_id = plugin_dir.name
    rel = plugin_dir.relative_to(plugin_dir.parents[1])
    sidecar_path = plugin_dir / ".claude-plugin" / "systemprompt.yaml"
    sidecar = {}
    if sidecar_path.is_file():
        try:
            sidecar = yaml.safe_load(sidecar_path.read_text(encoding="utf-8")) or {}
        except yaml.YAMLError as exc:
            out.error(rel / ".claude-plugin/systemprompt.yaml", 1, f"not valid YAML: {exc}", "fix the syntax")

    category = (sidecar.get("plugin") or {}).get("category") or (marketplace_entry or {}).get("category")
    if not category:
        out.error(
            rel / ".claude-plugin/systemprompt.yaml",
            1,
            "plugin declares no category in its sidecar or its marketplace.json entry; "
            "core's importer treats a missing category as a strict-mode failure",
            "set plugin.category",
        )

    skills_dir = plugin_dir / "skills"
    if not skills_dir.is_dir():
        out.error(rel, 1, "plugin ships no skills/ directory", "add skills/, or drop the plugin")
        return

    prefix = prefixes.get(plugin_id, "")
    ids = []
    for skill_dir in sorted(p for p in skills_dir.iterdir() if p.is_dir()):
        found = check_skill(skill_dir, plugin_id, prefix, out)
        if found:
            ids.append(found)

    # Why: `core-logic` and `sf-core-logic` are two ids to core and two skills
    # to the model, which then has to choose between them.
    topics = {}
    for skill_id in ids:
        topic = skill_id[len(prefix):] if prefix and skill_id.startswith(prefix) else skill_id
        topics.setdefault(topic, []).append(skill_id)
    for topic, members in sorted(topics.items()):
        if len(members) > 1:
            out.error(
                rel / "skills",
                1,
                f"{len(members)} skills cover the topic '{topic}': {', '.join(sorted(members))}",
                "keep one; a duplicate topic makes the model choose between two answers",
            )


def check_manifests(root, out):
    """marketplace.json against disk, and the per-plugin skill prefixes."""
    manifest_path = root / ".claude-plugin" / "marketplace.json"
    if not manifest_path.is_file():
        out.error(".claude-plugin/marketplace.json", 1, "missing", "a kit owns exactly one marketplace manifest")
        return None, {}
    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        out.error(".claude-plugin/marketplace.json", exc.lineno, f"not valid JSON: {exc.msg}", "fix the syntax")
        return None, {}

    entries = {e.get("name"): e for e in manifest.get("plugins", [])}
    on_disk = {p.name for p in (root / "plugins").iterdir() if p.is_dir()} if (root / "plugins").is_dir() else set()
    for declared in sorted(set(entries) - on_disk):
        out.error(".claude-plugin/marketplace.json", 1, f"declares plugin '{declared}', which is not in plugins/", "add it or drop the entry")
    for present in sorted(on_disk - set(entries)):
        out.error(".claude-plugin/marketplace.json", 1, f"plugins/{present} is not declared in the manifest, so it never ships", "add its entry")

    prefixes = {}
    config_path = root / "tools" / "kit-sanitize.json"
    if config_path.is_file():
        try:
            prefixes = json.loads(config_path.read_text(encoding="utf-8")).get("skill_prefix", {})
        except json.JSONDecodeError as exc:
            out.error("tools/kit-sanitize.json", exc.lineno, f"not valid JSON: {exc.msg}", "fix the syntax")
    return manifest, prefixes


def check_no_access(root, out):
    """An access block anywhere is a second truth for who reaches the kit."""
    for path in sorted(root.rglob("systemprompt.yaml")):
        if ".git" in path.parts:
            continue
        for offset, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            if re.match(r"\s*access:", line):
                out.error(
                    path.relative_to(root),
                    offset,
                    "carries an access: block — who reaches a kit is declared on the instance",
                    "declare marketplace/<id> with owner: bundle:<kit> in "
                    "the instance repository's services/access-control/rules.yaml",
                )


def check_version(root, manifest, out):
    """A content change under an unchanged version publishes nothing."""
    version = ((manifest or {}).get("metadata") or {}).get("version")
    if not version:
        out.error(".claude-plugin/marketplace.json", 1, "metadata.version is missing; CI names the release tag after it", "set metadata.version")
        return
    released = {t.strip() for t in os.environ.get("KIT_EXISTING_TAGS", "").splitlines() if t.strip()}
    if released and f"v{version}" in released:
        out.warn(
            ".claude-plugin/marketplace.json",
            1,
            f"metadata.version {version} is already released as v{version}, so this push publishes nothing",
            "bump metadata.version to publish these changes",
        )


def emit(out, summary_path):
    """GitHub annotations on stdout, and a table in the job summary."""
    for severity, items in (("error", out.errors), ("warning", out.warnings)):
        for item in items:
            message = f"{item['message']} — fix: {item['fix']}"
            # Annotations are one line; a newline would truncate the rest.
            message = message.replace("\n", " ")
            print(f"::{severity} file={item['path']},line={item['line']}::{message}")

    lines = ["## Kit sanitation", ""]
    if not out.errors and not out.warnings:
        lines.append("Clean — every skill parses, carries what it cites, and is uniquely named.")
    else:
        lines.append(f"**{len(out.errors)} error(s), {len(out.warnings)} warning(s)**")
        lines += ["", "| severity | file | line | problem | fix |", "|---|---|---|---|---|"]
        for severity, items in (("error", out.errors), ("warning", out.warnings)):
            for item in items:
                cells = [severity, f"`{item['path']}`", str(item["line"]), item["message"], item["fix"]]
                lines.append("| " + " | ".join(c.replace("|", "\\|") for c in cells) + " |")
    report = "\n".join(lines) + "\n"
    print(report)
    if summary_path:
        with open(summary_path, "a", encoding="utf-8") as handle:
            handle.write(report)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", default=".", help="the kit tree to check")
    parser.add_argument("--strict", action="store_true", help="exit 1 when there is any error")
    args = parser.parse_args()

    root = Path(args.root).resolve()
    out = Findings()
    manifest, prefixes = check_manifests(root, out)
    check_no_access(root, out)
    check_version(root, manifest, out)

    entries = {e.get("name"): e for e in (manifest or {}).get("plugins", [])}
    plugins_dir = root / "plugins"
    if plugins_dir.is_dir():
        for plugin_dir in sorted(p for p in plugins_dir.iterdir() if p.is_dir()):
            check_plugin(plugin_dir, entries.get(plugin_dir.name), prefixes, out)

    emit(out, os.environ.get("GITHUB_STEP_SUMMARY"))
    if out.errors:
        print(f"::error::kit sanitation found {len(out.errors)} error(s); the release is blocked until they are fixed")
        if args.strict:
            return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
