#!/usr/bin/env python3
"""Handlebars field references that resolve to nothing.

Strict mode makes a missing TOP-LEVEL field a 500, which is loud. A misspelled
field inside an {{#each}} renders as empty string and nothing notices: no
compiler sees it, no gate sees it, and a spec asserting the page renders sees a
perfectly good page with one blank column. The result is decided by something
other than what the test claims to measure, and it stays green.

Two things this gets right that a first attempt does not, both of which produce
CONFIDENT WRONG OUTPUT rather than an error:

  * Visibility. `pub(crate)` is the dominant visibility in the admin crate and
    the view structs are mostly bare `name: Type`, which serde serialises just
    the same. A regex matching only `pub name:` reads most structs as having no
    fields and then flags every reference into them.

  * Partial parameters. `{{> components/kpi testid=x}}` binds `testid` inside
    that partial; it is an argument, not a context field, and no amount of
    struct knowledge resolves it. Rather than a hand-tuned skip list, the
    parameter names are DERIVED: any hash key passed to a partial anywhere in
    the corpus is a legitimate name inside it.

Reports by default; --strict exits non-zero on any finding. Originated by
P7-analytics, who wrote the first version and worked out both traps.
"""
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TEMPLATES = sorted(
    list((ROOT / "storage/files/admin/templates").rglob("*.hbs"))
    + list((ROOT / "storage/files/admin/partials").rglob("*.hbs"))
)
SOURCES = list((ROOT / "extensions/web").rglob("*.rs"))

STRUCT = re.compile(r"struct\s+\w+(?:<[^>]*>)?\s*\{(.*?)\n\}", re.S)
FIELD = re.compile(r"^\s*(?:pub(?:\([\w:]+\))?\s+)?(\w+)\s*:", re.M)
RENAME = re.compile(r'rename\s*=\s*"(\w+)"')
INSERT = re.compile(r'insert\(\s*"(\w+)"')
INVOKE = re.compile(r"\{\{[#~]?>\s*([\w/-]+)([^}]*)\}\}")
HASHKEY = re.compile(r"(\w+)=")
BLOCKPARAM = re.compile(r"\{\{#\w+[^}]*\bas\s*\|([^|]+)\|")
PATH = re.compile(r"\{\{[\{~]?\s*[#/^]?\s*([A-Za-z_][\w.@/-]*)")

HELPERS = set(
    "formatDate formatNumber relativeTime initials truncate json concat toLowerCase "
    "toUpperCase default governanceColor css_version eq gt not add sub formatUsd "
    "percent deltaPct shortId navActive".split()
)
BUILTIN = set(
    "if unless each with lookup log else this true false inline block partial "
    "content actions filters meta head_extra scripts body layout".split()
)
SHELL = set(
    "branding current_user marketplace scope_selector demo_help page title "
    "breadcrumbs user csrf_token flash nav request env version".split()
)


def known_field_names() -> set:
    names = set()
    for path in SOURCES:
        text = path.read_text(errors="replace")
        names |= set(RENAME.findall(text)) | set(INSERT.findall(text))
        for struct in STRUCT.finditer(text):
            names |= set(FIELD.findall(struct.group(1)))
    return names


def partial_parameters() -> dict:
    params = {}
    for path in TEMPLATES:
        for name, rest in INVOKE.findall(path.read_text(errors="replace")):
            params.setdefault(name.split("/")[-1], set()).update(HASHKEY.findall(rest))
    return params


def main() -> int:
    fields = known_field_names()
    params = partial_parameters()
    findings = {}

    for path in TEMPLATES:
        text = path.read_text(errors="replace")
        local = set(params.get(path.stem, set()))
        for captured in BLOCKPARAM.findall(text):
            local |= set(captured.split())
        invoked = {name for name, _ in INVOKE.findall(text)}
        for raw in PATH.findall(text):
            if raw in invoked or "/" in raw:
                continue
            segment = raw.replace("../", "").split(".")[0].strip()
            if not segment or segment.startswith("@"):
                continue
            if segment in HELPERS or segment in BUILTIN or segment in SHELL:
                continue
            if segment in local or segment in fields:
                continue
            findings.setdefault(str(path.relative_to(ROOT)), set()).add(segment)

    for template, names in sorted(findings.items()):
        print(f"{template}: {', '.join(sorted(names))}")

    total = sum(len(v) for v in findings.values())
    print(
        f"\ncheck-template-fields: {len(fields)} field names, "
        f"{len(params)} partials with parameters, "
        f"{total} unresolved in {len(findings)} of {len(TEMPLATES)} templates"
    )
    if total and "--strict" in sys.argv:
        print("\nEach name above is referenced by a template and defined nowhere.")
        print("It renders as empty string, so the page looks right with one value missing.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
