#!/usr/bin/env python3
"""
Collect and categorize all public URLs from the Indaws website sitemap.

Fetches sitemap.xml, parses every URL, and categorizes them by type
(page, blog, case study, slide, shop, job, event, forum, other).
No API keys required -- uses the public sitemap.

Supports incremental updates: only adds new URLs found since last run.

Usage:
    python collect_sitemap.py
"""

import json
import os
import re
import sys
import xml.etree.ElementTree as ET
from pathlib import Path
from urllib.request import urlopen, Request
from urllib.error import URLError

try:
    from dotenv import load_dotenv
except ImportError:
    load_dotenv = None

SCRIPT_DIR = Path(__file__).parent
SKILL_DIR = SCRIPT_DIR.parent
DATA_DIR = SKILL_DIR / "data"

SITEMAP_URL = "https://www.indaws.es/sitemap.xml"

# URL categorization patterns (order matters -- first match wins)
CATEGORY_PATTERNS = [
    ("blog",       re.compile(r"^/blog/blog-indaws-1/")),
    ("case_study", re.compile(r"^/blog/referencias-indaws-16/")),
    ("blog_other", re.compile(r"^/blog/")),
    ("slide",      re.compile(r"^/slides/")),
    ("shop",       re.compile(r"^/shop/")),
    ("job",        re.compile(r"^/jobs/")),
    ("event",      re.compile(r"^/event/")),
    ("forum",      re.compile(r"^/forum/")),
    ("helpdesk",   re.compile(r"^/helpdesk/")),
]


def load_env():
    """Load environment variables from .env file."""
    env_path = SKILL_DIR / ".env"
    if env_path.exists() and load_dotenv:
        load_dotenv(env_path)
    elif env_path.exists():
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, value = line.split("=", 1)
                    value = value.strip().strip('"').strip("'")
                    os.environ[key.strip()] = value


def fetch_sitemap(url):
    """Fetch and parse sitemap XML."""
    print(f"Fetching sitemap: {url}")
    req = Request(url, headers={"User-Agent": "IndawsContentCollector/1.0"})
    try:
        with urlopen(req, timeout=30) as response:
            xml_data = response.read()
    except URLError as e:
        print(f"ERROR: Failed to fetch sitemap: {e}", file=sys.stderr)
        sys.exit(1)

    # Parse XML (handle namespace)
    root = ET.fromstring(xml_data)
    ns = {"sm": "http://www.sitemaps.org/schemas/sitemap/0.9"}

    entries = []
    for url_elem in root.findall("sm:url", ns):
        loc = url_elem.find("sm:loc", ns)
        lastmod = url_elem.find("sm:lastmod", ns)
        priority = url_elem.find("sm:priority", ns)
        changefreq = url_elem.find("sm:changefreq", ns)

        if loc is not None and loc.text:
            entries.append({
                "url": loc.text.strip(),
                "lastmod": lastmod.text.strip() if lastmod is not None and lastmod.text else "",
                "priority": float(priority.text.strip()) if priority is not None and priority.text else 0.5,
                "changefreq": changefreq.text.strip() if changefreq is not None and changefreq.text else "",
            })

    return entries


def categorize_url(url):
    """Categorize a URL by its path pattern."""
    from urllib.parse import urlparse
    path = urlparse(url).path

    for category, pattern in CATEGORY_PATTERNS:
        if pattern.search(path):
            return category

    # Pages with /odoo-* paths are service/landing pages
    if re.match(r"^/odoo-", path):
        return "service"

    # Root-level pages (not ending in common file extensions)
    if path.count("/") <= 1:
        return "page"

    return "page"


def extract_title_from_url(url):
    """Extract a human-readable title hint from the URL slug."""
    from urllib.parse import urlparse
    path = urlparse(url).path.rstrip("/")
    slug = path.split("/")[-1] if path else ""

    # Remove trailing numeric ID (e.g., "mi-articulo-123" -> "mi-articulo")
    slug = re.sub(r"-\d+$", "", slug)

    # Convert slug to title-like string
    title = slug.replace("-", " ").replace("_", " ").strip()
    return title.title() if title else ""


def load_existing_sitemap():
    """Load existing sitemap data for incremental updates."""
    sitemap_path = DATA_DIR / "sitemap.json"
    if sitemap_path.exists():
        with open(sitemap_path, encoding="utf-8") as f:
            return json.load(f)
    return []


def main():
    load_env()
    DATA_DIR.mkdir(parents=True, exist_ok=True)

    # Load existing data for incremental merge
    existing = load_existing_sitemap()
    existing_urls = {entry["url"] for entry in existing}
    print(f"Existing URLs in database: {len(existing_urls)}")

    # Fetch fresh sitemap
    raw_entries = fetch_sitemap(SITEMAP_URL)
    print(f"URLs found in sitemap: {len(raw_entries)}")

    # Process and categorize
    new_count = 0
    updated_count = 0
    all_entries = {entry["url"]: entry for entry in existing}

    for raw in raw_entries:
        url = raw["url"]
        category = categorize_url(url)
        title_hint = extract_title_from_url(url)

        entry = {
            "url": url,
            "category": category,
            "title_hint": title_hint,
            "lastmod": raw["lastmod"],
            "priority": raw["priority"],
            "changefreq": raw["changefreq"],
        }

        if url in all_entries:
            # Update lastmod if changed
            if raw["lastmod"] and raw["lastmod"] != all_entries[url].get("lastmod"):
                all_entries[url]["lastmod"] = raw["lastmod"]
                updated_count += 1
        else:
            all_entries[url] = entry
            new_count += 1

    # Convert to sorted list
    sitemap = sorted(all_entries.values(), key=lambda x: x.get("lastmod", ""), reverse=True)

    # Print category summary
    categories = {}
    for entry in sitemap:
        cat = entry["category"]
        categories[cat] = categories.get(cat, 0) + 1

    print(f"\nCategory breakdown:")
    for cat, count in sorted(categories.items(), key=lambda x: -x[1]):
        print(f"  {cat}: {count}")

    print(f"\nNew URLs: {new_count}")
    print(f"Updated URLs: {updated_count}")

    # Save
    output_path = DATA_DIR / "sitemap.json"
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(sitemap, f, ensure_ascii=False, indent=2)

    print(f"\nSaved {len(sitemap)} URLs to {output_path}")


if __name__ == "__main__":
    main()
