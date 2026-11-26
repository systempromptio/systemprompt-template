#!/usr/bin/env python3
"""Fetch every console page a signed-in admin can reach from the sidebar.

    console-crawl.py BASE COOKIE

Reads /admin, takes every link in the admin navigation (the flavour's own
dashboard.yaml, as the server rendered it), fetches each, then fetches every
same-origin /admin link those pages carry, one hop deep. Prints one line per
page: `nav CODE PATH` for a sidebar page, `link CODE PATH` for a page one hop
beneath it. A strict-mode template reading a field the flavour's view model
does not supply answers 500, and a link to a page the flavour does not compile
answers 404; the smoke fails both, so this is the render and link check for
every page a flavour compiles and links."""
import html.parser, sys, urllib.error, urllib.parse, urllib.request

base, cookie = sys.argv[1:3]
SKIP = ("/admin/api/", "/admin/login", "/admin/logout")


class Links(html.parser.HTMLParser):
    def __init__(self, nav_only):
        super().__init__()
        self.nav_only, self.depth, self.hrefs = nav_only, 0, []

    def handle_starttag(self, tag, attrs):
        a = dict(attrs)
        if tag == "nav" and (self.depth or a.get("aria-label") == "Admin navigation"):
            self.depth += 1
        if tag == "a" and (self.depth or not self.nav_only) and a.get("href"):
            self.hrefs.append(a["href"])

    def handle_endtag(self, tag):
        if tag == "nav" and self.depth:
            self.depth -= 1


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k): return None


opener = urllib.request.build_opener(NoRedirect)


def get(path):
    r = urllib.request.Request(base + path, headers={"Cookie": cookie, "Accept": "text/html"})
    try:
        resp = opener.open(r, timeout=30)
    except urllib.error.HTTPError as e:
        resp = e
    body = resp.read()
    kind = resp.headers.get("Content-Type", "")
    return resp.status, body.decode("utf-8", "replace") if "html" in kind else ""


def admin_paths(page, nav_only):
    p = Links(nav_only)
    p.feed(page)
    out = []
    for href in p.hrefs:
        u = urllib.parse.urlsplit(href)
        if u.scheme or u.netloc or not u.path.startswith("/admin") or u.path.startswith(SKIP):
            continue
        path = u.path + ("?" + u.query if u.query else "")
        if path not in out:
            out.append(path)
    return out


status, home = get("/admin")
if status != 200:
    sys.exit(f"/admin -> {status}")
seen = set()
pages = []
for path in admin_paths(home, True):
    seen.add(path)
    status, body = get(path)
    print(f"nav {status} {path}", flush=True)
    pages.append(body)
for body in pages:
    for path in admin_paths(body, False):
        if path in seen:
            continue
        seen.add(path)
        status, _ = get(path)
        print(f"link {status} {path}", flush=True)
