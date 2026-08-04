#!/usr/bin/env python3
"""Reverse-direction asset gate: every shipped asset must be reachable.

The existing gates prove markup -> asset (nothing referenced is missing).
This gate proves asset -> markup, per file:

  CSS   a stylesheet is dead when zero of its class selectors occur in any
        template, JS, or Rust source outside CSS files themselves
  JS    a script is dead when unreachable from every <script src> root via
        the ES-module import graph (absolute /js/... and relative specifiers)
  HBS   an admin partial is dead when no {{> name}} include names it
  FONT  a font directory is dead when no @font-face src references it
  IMG   an image is dead when its basename appears in no source file

Exemptions: scripts/asset-reachability-exemptions.txt (one path per line,
relative to repo root, '#' comments).
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
CSS_DIR = ROOT / "storage/files/css"
JS_DIR = ROOT / "storage/files/js"
FONT_DIR = ROOT / "storage/files/fonts"
IMG_DIR = ROOT / "storage/files/images"
PARTIALS_DIR = ROOT / "storage/files/admin/partials"

MARKUP_GLOBS = [
    ("services/web/templates", "*.html"),
    ("storage/files/admin", "*.hbs"),
    ("storage/files/js", "*.js"),
    ("extensions", "*.rs"),
    ("bridge", "*.rs"),
]


def read(path: Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace")


def load_exemptions() -> set:
    path = ROOT / "scripts/asset-reachability-exemptions.txt"
    if not path.exists():
        return set()
    lines = (l.strip() for l in read(path).splitlines())
    return {l for l in lines if l and not l.startswith("#")}


def markup_corpus() -> str:
    chunks = []
    for base, pattern in MARKUP_GLOBS:
        for f in (ROOT / base).rglob(pattern):
            chunks.append(read(f))
    return "\n".join(chunks)


CLASS_RE = re.compile(r"\.([A-Za-z_][A-Za-z0-9_-]{2,})")
CSS_KEYWORDS = {"active", "hidden", "open", "selected", "disabled", "error"}


def dead_css(corpus: str) -> list:
    dead = []
    for css in sorted(CSS_DIR.rglob("*.css")):
        if css.name == "admin-bundle.css":
            continue
        classes = {c for c in CLASS_RE.findall(read(css)) if c not in CSS_KEYWORDS}
        if not classes:
            continue
        if not any(c in corpus for c in classes):
            dead.append(css)
    return dead


SCRIPT_SRC_RE = re.compile(r"""<script[^>]+src=["']([^"'?]+)""")
IMPORT_RE = re.compile(
    r"""\bfrom\s*['"]([^'"]+)['"]|import\s*\(?\s*['"]([^'"]+)['"]"""
)


def resolve_js(spec: str, importer: Path):
    if spec.startswith("/js/"):
        cand = JS_DIR / spec[len("/js/") :]
    elif spec.startswith("."):
        cand = (importer.parent / spec).resolve()
    else:
        return None
    if cand.suffix != ".js":
        cand = cand.with_suffix(".js")
    return cand if cand.exists() else None


def dead_js() -> list:
    roots = []
    for base in ("services/web/templates", "storage/files/admin"):
        for f in (ROOT / base).rglob("*"):
            if f.suffix in (".html", ".hbs"):
                text = read(f).replace("{{JS_BASE_PATH}}", "/js")
                for src in SCRIPT_SRC_RE.findall(text):
                    if src.startswith("/js/"):
                        cand = JS_DIR / src[len("/js/") :]
                        if cand.exists():
                            roots.append(cand)
    seen = set()
    stack = list(roots)
    while stack:
        f = stack.pop()
        if f in seen:
            continue
        seen.add(f)
        for m in IMPORT_RE.findall(read(f)):
            spec = m[0] or m[1]
            target = resolve_js(spec, f)
            if target and target not in seen:
                stack.append(target)
    return [f for f in sorted(JS_DIR.rglob("*.js")) if f not in seen]


PARTIAL_RE = re.compile(r"\{\{#?>\s*([\w/-]+)")


def dead_partials() -> list:
    if not PARTIALS_DIR.exists():
        return []
    used = set()
    for f in (ROOT / "storage/files/admin").rglob("*.hbs"):
        used.update(PARTIAL_RE.findall(read(f)))
    for f in (ROOT / "extensions").rglob("*.rs"):
        used.update(PARTIAL_RE.findall(read(f)))
    dead = []
    for p in sorted(PARTIALS_DIR.rglob("*.hbs")):
        name = p.stem
        rel = p.relative_to(PARTIALS_DIR).with_suffix("").as_posix()
        if name not in used and rel not in used and name != "layout":
            dead.append(p)
    return dead


def dead_fonts() -> list:
    if not FONT_DIR.exists():
        return []
    css = "\n".join(read(f) for f in CSS_DIR.rglob("*.css"))
    return [d for d in sorted(FONT_DIR.iterdir()) if d.is_dir() and d.name not in css]


def dead_images(corpus: str) -> list:
    if not IMG_DIR.exists():
        return []
    extra = []
    for base, pattern in [
        ("services", "*.yaml"),
        ("services", "*.md"),
        ("storage/files/css", "*.css"),
        ("deploy", "*"),
        (".", "*.md"),
    ]:
        for f in (ROOT / base).rglob(pattern):
            if f.is_file():
                extra.append(read(f))
    full = corpus + "\n".join(extra)
    return [f for f in sorted(IMG_DIR.rglob("*")) if f.is_file() and f.name not in full]


def main() -> int:
    exempt = load_exemptions()
    corpus = markup_corpus()
    failures = []
    for label, files in (
        ("dead CSS (no class used anywhere)", dead_css(corpus)),
        ("unreachable JS (no import path from any <script> root)", dead_js()),
        ("orphaned admin partial (no {{> include}})", dead_partials()),
        ("unreferenced font directory", dead_fonts()),
        ("unreferenced image", dead_images(corpus)),
    ):
        for f in files:
            rel = f.relative_to(ROOT).as_posix()
            if rel not in exempt:
                failures.append(f"{label}: {rel}")
    if failures:
        print("asset reachability FAILED:")
        for line in failures:
            print(f"  {line}")
        print(f"{len(failures)} unreachable asset(s). Delete them or exempt in scripts/asset-reachability-exemptions.txt")
        return 1
    print("asset reachability OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
