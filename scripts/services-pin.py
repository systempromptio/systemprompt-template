#!/usr/bin/env python3
"""Pin a kit's services bundle into a profile's `services.sources[]`.

    scripts/services-pin.py <kit> <sha256 digest | channel tag> [profile.yaml]

The kit's image, public key and pull mode come from deploy/kit/known-kits.json;
the profile is edited as text so its comments survive. If the source entry
exists its reference is rewritten; if not, the entry (and the `sources:`
block when absent) is inserted under `services:`. The instance never
rewrites its own profile — this is the one place a reference becomes code.

A digest pins one upload. A channel tag (the kit's `channel` in
known-kits.json, e.g. `stable`) follows whatever the kit's CI last published,
so a release reaches the instance with one Import on /admin/sync and no re-pin.
A kit whose package is public (`pull: public`) gets no `auth_secret`.
"""
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KITS = ROOT / "deploy/kit/known-kits.json"


def die(msg: str) -> None:
    print(f"services-pin: {msg}", file=sys.stderr)
    sys.exit(1)


def main() -> None:
    if len(sys.argv) < 3:
        die("usage: services-pin.py <kit> <sha256 digest> [profile.yaml]")
    name, target = sys.argv[1], sys.argv[2]
    profile = Path(sys.argv[3]) if len(sys.argv) > 3 else ROOT / ".systemprompt/profiles/production/profile.yaml"
    kits = {k["name"]: k for k in json.loads(KITS.read_text())["kits"]}
    kit = kits.get(name) or die(f"no kit '{name}' in {KITS.relative_to(ROOT)}")
    if not kit.get("public_key"):
        die(f"kit '{name}' has no public_key in known-kits.json — record the kit's signing key first")
    if not profile.is_file():
        die(f"{profile} does not exist")

    digest = target.removeprefix("sha256:")
    if re.fullmatch(r"[0-9a-f]{64}", digest):
        reference, shown = f"{kit['image']}@sha256:{digest}", f"sha256:{digest}"
    elif target == kit.get("channel"):
        reference, shown = f"{kit['image']}:{target}", f"channel {target}"
    else:
        die(f"'{target}' is neither a sha256 digest nor kit '{name}'s channel ({kit.get('channel') or 'none'})")

    lines = profile.read_text().split("\n")
    entry = [
        f"    - name: {name}",
        "      oci:",
        f"        reference: {reference}",
    ]
    if kit.get("pull") != "public":
        entry.append("        auth_secret: ghcr_pull_token")
    entry += [
        "        verify:",
        f'          ed25519_public_keys: ["{kit["public_key"]}"]',
    ]

    services_at = next((i for i, l in enumerate(lines) if l.rstrip() == "services:"), None)
    if services_at is None:
        die(f"{profile} has no top-level services: key")
    block_end = next(
        (i for i in range(services_at + 1, len(lines)) if lines[i] and not lines[i].startswith(" ") and not lines[i].startswith("#")),
        len(lines),
    )
    block = range(services_at + 1, block_end)

    name_at = next((i for i in block if lines[i].strip() == f"- name: {name}"), None)
    if name_at is not None:
        ref_at = next((i for i in range(name_at + 1, block_end) if lines[i].strip().startswith("reference:")), None)
        if ref_at is None:
            die(f"source '{name}' has no oci.reference line")
        indent = lines[ref_at][: len(lines[ref_at]) - len(lines[ref_at].lstrip())]
        lines[ref_at] = f"{indent}reference: {reference}"
        action = "re-pinned"
    else:
        sources_at = next((i for i in block if lines[i].rstrip() == "  sources:"), None)
        if sources_at is None:
            insert_at = services_at + 1
            lines[insert_at:insert_at] = ["  sources:"] + entry
            if not any(lines[i].strip().startswith("on_fetch_failure:") for i in block):
                lines.insert(insert_at + 1 + len(entry), "  on_fetch_failure: use_bundled")
        else:
            end = sources_at + 1
            while end < block_end and (lines[end].startswith("    ") or not lines[end].strip()):
                end += 1
            lines[end:end] = entry
        action = "pinned"

    profile.write_text("\n".join(lines))
    print(f"{action} {name} -> {shown} in {profile}")
    print("next: just deploy, or Import sources on /admin/sync (POST /api/v1/admin/services/refresh)")


if __name__ == "__main__":
    main()
