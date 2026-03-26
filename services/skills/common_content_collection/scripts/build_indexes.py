#!/usr/bin/env python3
"""
Build markdown index files from collected JSON data.

This script:
1. Reads videos.json, cases.json, modules.json
2. Creates cross-references between modules, cases, and videos
3. Generates searchable markdown tables for each content type

Usage:
    python build_indexes.py
"""

import json
from datetime import datetime
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent
SKILL_DIR = SCRIPT_DIR.parent
DATA_DIR = SKILL_DIR / "data"
REFS_DIR = SKILL_DIR / "references"


def load_json(filename):
    """Load JSON data file."""
    filepath = DATA_DIR / filename
    if not filepath.exists():
        print(f"  Warning: {filename} not found, using empty list")
        return []
    with open(filepath, "r", encoding="utf-8") as f:
        return json.load(f)


def save_json(data, filename):
    """Save JSON data file."""
    filepath = DATA_DIR / filename
    with open(filepath, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, indent=2)


def truncate(text, max_length=80):
    """Truncate text to max length."""
    if not text:
        return ""
    if len(text) <= max_length:
        return text
    return text[:max_length - 3] + "..."


def build_cross_references(videos, cases, modules):
    """Build cross-references between content types."""
    # Create lookup maps
    module_by_name = {}
    for module in modules:
        module_by_name[module["technical_name"]] = module
        module_by_name[module["name"].lower()] = module
        module["related_cases"] = []
        module["related_videos"] = []

    # Link cases to modules
    for case in cases:
        for module_ref in case.get("modules_used", []):
            if module_ref in module_by_name:
                module = module_by_name[module_ref]
                if case["id"] not in module["related_cases"]:
                    module["related_cases"].append(case["id"])

    # Link videos to modules based on topics
    for video in videos:
        video_topics = set(t.lower() for t in video.get("topics", []))
        for module in modules:
            module_keywords = set([
                module["technical_name"].replace("indaws_", ""),
                module["name"].lower().replace("indaws ", "")
            ])
            if video_topics & module_keywords:
                if video["id"] not in module["related_videos"]:
                    module["related_videos"].append(video["id"])

    return videos, cases, modules


def build_videos_index(videos):
    """Generate videos-index.md."""
    lines = [
        "# YouTube Videos Index",
        "",
        f"Last updated: {datetime.now().strftime('%Y-%m-%d')}",
        f"Total videos: {len(videos)}",
        "",
        "## Quick Reference",
        "",
        "| Title | Topics | Duration | Date |",
        "|-------|--------|----------|------|",
    ]

    for video in videos:
        title = truncate(video["title"], 50)
        topics = ", ".join(video.get("topics", [])[:3])
        duration = video.get("duration", "").replace("PT", "").replace("H", "h ").replace("M", "m ").replace("S", "s")
        date = video.get("publish_date", "")
        lines.append(f"| [{title}]({video['url']}) | {topics} | {duration} | {date} |")

    lines.extend([
        "",
        "## Search Patterns",
        "",
        "To find videos about a specific topic:",
        "```",
        'grep -i "inventory" data/videos.json',
        'grep -i "manufacturing" data/videos.json',
        "```",
        "",
        "## Data Schema",
        "",
        "Each video record in `data/videos.json` contains:",
        "- `id`: YouTube video ID",
        "- `title`: Video title",
        "- `description`: Full description",
        "- `publish_date`: YYYY-MM-DD format",
        "- `duration`: ISO 8601 duration (PT#M#S)",
        "- `url`: Full YouTube URL",
        "- `thumbnail`: Path to thumbnail image",
        "- `topics`: Array of detected topics",
    ])

    return "\n".join(lines)


def build_cases_index(cases):
    """Generate cases-index.md."""
    lines = [
        "# Case Studies Index",
        "",
        f"Last updated: {datetime.now().strftime('%Y-%m-%d')}",
        f"Total case studies: {len(cases)}",
        "",
        "## Quick Reference",
        "",
        "| Client | Modules Used | Date |",
        "|--------|--------------|------|",
    ]

    for case in cases:
        client = truncate(case.get("client", case["title"]), 40)
        modules = ", ".join(m.replace("indaws_", "") for m in case.get("modules_used", [])[:3])
        date = case.get("publish_date", "")
        lines.append(f"| [{client}]({case['url']}) | {modules} | {date} |")

    lines.extend([
        "",
        "## Search Patterns",
        "",
        "To find cases by industry or module:",
        "```",
        'grep -i "manufacturing" data/cases.json',
        'grep -i "indaws_inventory" data/cases.json',
        "```",
        "",
        "## Data Schema",
        "",
        "Each case record in `data/cases.json` contains:",
        "- `id`: Odoo blog post ID",
        "- `title`: Full case study title",
        "- `client`: Client name",
        "- `url`: Full URL to blog post",
        "- `publish_date`: YYYY-MM-DD format",
        "- `summary`: Brief summary (first 500 chars)",
        "- `content`: Full cleaned content",
        "- `modules_used`: Array of module technical names",
        "- `industry`: Industry category (may be empty)",
    ])

    return "\n".join(lines)


def build_modules_index(modules):
    """Generate modules-index.md."""
    lines = [
        "# Indaws Core Modules Index",
        "",
        f"Last updated: {datetime.now().strftime('%Y-%m-%d')}",
        f"Total modules: {len(modules)}",
        "",
        "## Quick Reference",
        "",
        "| Module | Key Features | Cases | Videos |",
        "|--------|--------------|-------|--------|",
    ]

    for module in modules:
        name = module["name"]
        features = ", ".join(f[:30] for f in module.get("features", [])[:2])
        cases_count = len(module.get("related_cases", []))
        videos_count = len(module.get("related_videos", []))
        url = module.get("url", "")
        if url:
            lines.append(f"| [{name}]({url}) | {features} | {cases_count} | {videos_count} |")
        else:
            lines.append(f"| {name} | {features} | {cases_count} | {videos_count} |")

    lines.extend([
        "",
        "## Search Patterns",
        "",
        "To find module details:",
        "```",
        'grep -i "inventory" data/modules.json',
        "```",
        "",
        "## Data Schema",
        "",
        "Each module record in `data/modules.json` contains:",
        "- `id`: Odoo product ID",
        "- `name`: Display name",
        "- `technical_name`: Technical identifier (snake_case)",
        "- `code`: Product code",
        "- `description`: Full description",
        "- `features`: Array of feature descriptions",
        "- `price`: List price",
        "- `url`: Product page URL",
        "- `related_cases`: Array of case study IDs using this module",
        "- `related_videos`: Array of video IDs about this module",
    ])

    return "\n".join(lines)


def build_sitemap_index(sitemap):
    """Generate sitemap-index.md."""
    # Count by category
    categories = {}
    for entry in sitemap:
        cat = entry.get("category", "other")
        categories[cat] = categories.get(cat, 0) + 1

    lines = [
        "# Website Sitemap Index",
        "",
        f"Last updated: {datetime.now().strftime('%Y-%m-%d')}",
        f"Total URLs: {len(sitemap)}",
        "",
        "## Categories",
        "",
        "| Category | Count | Description |",
        "|----------|-------|-------------|",
    ]

    category_desc = {
        "blog": "Blog articles (blog-indaws-1)",
        "case_study": "Client success stories (referencias-indaws-16)",
        "blog_other": "Other blog categories",
        "slide": "eLearning slides and courses",
        "page": "Static pages and landing pages",
        "service": "Odoo service pages (/odoo-*)",
        "shop": "Product catalog pages",
        "job": "Job listings",
        "event": "Events and webinars",
        "forum": "Community forum",
        "helpdesk": "Support and knowledge base",
    }

    for cat, count in sorted(categories.items(), key=lambda x: -x[1]):
        desc = category_desc.get(cat, cat)
        lines.append(f"| {cat} | {count} | {desc} |")

    # Blog articles (for interlinking)
    lines.extend([
        "",
        "## Blog Articles (for interlinking)",
        "",
        "| Title | URL | Last Modified |",
        "|-------|-----|---------------|",
    ])
    blogs = [e for e in sitemap if e["category"] == "blog"]
    for entry in blogs[:100]:  # Limit to 100 for readability
        title = truncate(entry.get("title_hint", ""), 50)
        url = entry["url"]
        lastmod = entry.get("lastmod", "")
        lines.append(f"| {title} | [{url}]({url}) | {lastmod} |")
    if len(blogs) > 100:
        lines.append(f"| ... | *{len(blogs) - 100} more blogs* | |")

    # Service & landing pages (for interlinking)
    lines.extend([
        "",
        "## Service & Landing Pages",
        "",
        "| Title | URL | Last Modified |",
        "|-------|-----|---------------|",
    ])
    pages = [e for e in sitemap if e["category"] in ("page", "service")]
    for entry in pages:
        title = truncate(entry.get("title_hint", ""), 50)
        url = entry["url"]
        lastmod = entry.get("lastmod", "")
        lines.append(f"| {title} | [{url}]({url}) | {lastmod} |")

    lines.extend([
        "",
        "## Search Patterns",
        "",
        "To find URLs by category or keyword:",
        "```",
        'grep \'"category": "blog"\' data/sitemap.json',
        'grep \'"category": "service"\' data/sitemap.json',
        'grep -i "crm" data/sitemap.json',
        "```",
        "",
        "## Data Schema",
        "",
        "Each entry in `data/sitemap.json` contains:",
        "- `url`: Full URL",
        "- `category`: page, blog, case_study, service, slide, shop, job, event, forum, helpdesk, blog_other",
        "- `title_hint`: Title extracted from URL slug",
        "- `lastmod`: Last modified date (YYYY-MM-DD)",
        "- `priority`: Sitemap priority (0.0-1.0)",
        "- `changefreq`: Change frequency hint",
    ])

    return "\n".join(lines)


def main():
    REFS_DIR.mkdir(parents=True, exist_ok=True)

    print("Loading data files...")
    videos = load_json("videos.json")
    cases = load_json("cases.json")
    modules = load_json("modules.json")
    sitemap = load_json("sitemap.json")

    print("Building cross-references...")
    videos, cases, modules = build_cross_references(videos, cases, modules)

    # Save updated modules with cross-references
    if modules:
        save_json(modules, "modules.json")

    print("Generating index files...")

    # Videos index
    if videos:
        videos_md = build_videos_index(videos)
        (REFS_DIR / "videos-index.md").write_text(videos_md, encoding="utf-8")
        print(f"  Created videos-index.md ({len(videos)} videos)")

    # Cases index
    if cases:
        cases_md = build_cases_index(cases)
        (REFS_DIR / "cases-index.md").write_text(cases_md, encoding="utf-8")
        print(f"  Created cases-index.md ({len(cases)} cases)")

    # Modules index
    if modules:
        modules_md = build_modules_index(modules)
        (REFS_DIR / "modules-index.md").write_text(modules_md, encoding="utf-8")
        print(f"  Created modules-index.md ({len(modules)} modules)")

    # Sitemap index
    if sitemap:
        sitemap_md = build_sitemap_index(sitemap)
        (REFS_DIR / "sitemap-index.md").write_text(sitemap_md, encoding="utf-8")
        print(f"  Created sitemap-index.md ({len(sitemap)} URLs)")

    print("\nDone!")


if __name__ == "__main__":
    main()
