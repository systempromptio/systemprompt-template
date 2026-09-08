#!/usr/bin/env bash
# Check that every static asset a page requests is published by the manifest.
#
# CSS and JS sources live under storage/files/ and reach web/dist/ only if they
# are listed in the asset manifests (extensions/web/site/src/assets/ and
# extensions/web/admin/src/assets.rs). That
# list is hand-maintained, so a new file lands in storage, the template points at
# it, and the browser 404s — with no build, lint, or test signal. This gate reads
# every asset URL the templates and JS modules reference and fails if the source
# is missing or the served path is not in the manifest.
#
# References come from three places: static `src=`/`href=` URLs in the admin
# templates and partials, the `{{CSS_BASE_PATH}}` / `{{JS_BASE_PATH}}` URLs in
# the public-site partials (include_str!-compiled by extensions/web/site), and
# relative ES-module imports between the JS modules themselves.
#
# The manifest corpus is every served path advertised by web_assets(): macro
# invocations (`page_js!` -> js/pages/, `svc_js!` -> js/services/, `css!` ->
# css/) plus plain `AssetDefinition::js(..., "served/path")` literals.
#
# Exemption: list a served path (one per line, `#` comments) in
# scripts/admin-asset-exemptions.txt for assets served by other means. Never
# exempt an asset that genuinely 404s.
set -uo pipefail

TPL_DIR="${TPL_DIR:-storage/files/admin/templates}"
PARTIAL_DIR="${PARTIAL_DIR:-storage/files/admin/partials}"
SITE_PARTIAL_DIR="${SITE_PARTIAL_DIR:-services/web/templates/partials}"
STORAGE_FILES="${STORAGE_FILES:-storage/files}"
ASSET_DIRS="${ASSET_DIRS:-extensions/web/site/src/assets extensions/web/admin/src/assets.rs}"
EXEMPT_FILE="${EXEMPT_FILE:-scripts/admin-asset-exemptions.txt}"

[ -d "$TPL_DIR" ] || { echo "check-admin-template-assets: no $TPL_DIR - nothing to check"; exit 0; }

python3 - "$TPL_DIR" "$PARTIAL_DIR" "$SITE_PARTIAL_DIR" "$STORAGE_FILES" "$ASSET_DIRS" "$EXEMPT_FILE" <<'PY'
import re, sys, pathlib

tpl_dir, partial_dir, site_partial_dir, storage_files, asset_dirs, exempt_file = sys.argv[1:7]

exempt = set()
p = pathlib.Path(exempt_file)
if p.exists():
    for line in p.read_text().splitlines():
        line = line.split('#', 1)[0].strip()
        if line:
            exempt.add(line)

# Manifest corpus: the served path of every asset web_assets() advertises. Each
# macro hard-codes its own `concat!` prefix, so a bare file name resolves only
# once the macro it appears in is known.
MACRO_PREFIX = {'page_js': 'js/pages/', 'svc_js': 'js/services/', 'site_js': 'js/site/', 'css': 'css/'}
MACRO = re.compile(r'\b(page_js|svc_js|site_js|css)!\s*\([^,]+,\s*"([^"]+)"')
DIRECT = re.compile(r'AssetDefinition::(?:js|css|builder)\s*\([^,]+,\s*"([^"]+)"')

served = set()
manifest_files = []
for entry in asset_dirs.split():
    path = pathlib.Path(entry)
    manifest_files.extend(sorted(path.glob('*.rs')) if path.is_dir() else [path])
for f in manifest_files:
    text = f.read_text()
    for name, leaf in MACRO.findall(text):
        served.add(MACRO_PREFIX[name] + leaf)
    served.update(DIRECT.findall(text))

if not served:
    print(f"check-admin-template-assets: no assets parsed from {asset_dirs}", file=sys.stderr)
    sys.exit(1)

# Reference corpus. Every entry is a served path relative to the web root, i.e.
# `js/pages/admin-models.js`, mapping to storage/files/js/pages/admin-models.js.
ATTR = re.compile(r'(?:src|href)\s*=\s*"([^"]*\.(?:js|css)(?:\?[^"]*)?)"')
IMPORT = re.compile(r'''\bfrom\s+['"]([^'"]+\.js)['"]''')

def clean(url):
    return url.split('?', 1)[0].split('#', 1)[0]

def template_refs(text):
    for raw in ATTR.findall(text):
        url = clean(raw)
        # Public-site partials address the roots through placeholders.
        url = url.replace('{{CSS_BASE_PATH}}', '/css').replace('{{JS_BASE_PATH}}', '/js')
        if '{{' in url:
            continue
        if not url.startswith(('/js/', '/css/')):
            continue
        yield url.lstrip('/')

refs = {}  # served path -> set of referencing files

def record(path, url):
    refs.setdefault(url, set()).add(str(path))

templates = sorted(pathlib.Path(tpl_dir).rglob('*.hbs'))
templates += sorted(pathlib.Path(partial_dir).rglob('*.hbs'))
site_partials = pathlib.Path(site_partial_dir)
if site_partials.is_dir():
    templates += sorted(site_partials.rglob('*.html'))

for path in templates:
    for url in template_refs(path.read_text()):
        record(path, url)

# Relative imports between modules: resolve against the importing file so the
# result is a served path on the same web root.
js_root = pathlib.Path(storage_files) / 'js'
for path in sorted(js_root.rglob('*.js')):
    for spec in IMPORT.findall(path.read_text()):
        if not spec.startswith('.'):
            continue
        target = (path.parent / spec).resolve()
        try:
            rel = target.relative_to(pathlib.Path(storage_files).resolve())
        except ValueError:
            continue
        record(path, rel.as_posix())

missing_source = []
unpublished = []
for url in sorted(refs):
    if url in exempt:
        continue
    if not (pathlib.Path(storage_files) / url).is_file():
        missing_source.append(url)
    elif url not in served:
        unpublished.append(url)

if missing_source or unpublished:
    print("check-admin-template-assets: asset reference(s) that will 404:", file=sys.stderr)
    for url in missing_source:
        print(f"  /{url} — no source at {storage_files}/{url}", file=sys.stderr)
        for who in sorted(refs[url]):
            print(f"      referenced by {who}", file=sys.stderr)
    for url in unpublished:
        print(f"  /{url} — source exists but is not in the asset manifest", file=sys.stderr)
        for who in sorted(refs[url]):
            print(f"      referenced by {who}", file=sys.stderr)
    print("", file=sys.stderr)
    total = len(missing_source) + len(unpublished)
    print(f"{total} broken asset reference(s).", file=sys.stderr)
    print(f"Register the asset in {asset_dirs}/, fix the URL, or if it is served by", file=sys.stderr)
    print(f"other means list it in {exempt_file}.", file=sys.stderr)
    sys.exit(1)

print(f"check-admin-template-assets: OK ({len(refs)} asset reference(s) resolve)")
PY
