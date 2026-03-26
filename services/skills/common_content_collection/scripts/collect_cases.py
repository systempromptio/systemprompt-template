#!/usr/bin/env python3
"""
Collect case studies from Indaws Odoo blog.

Requires:
    - ODOO_URL, ODOO_DB, ODOO_KEY environment variables
    - odoo-pilot skill scripts in PATH or sibling directory

Usage:
    python collect_cases.py
"""

import json
import os
import re
import subprocess
import sys
from pathlib import Path

try:
    from dotenv import load_dotenv
except ImportError:
    load_dotenv = None

SCRIPT_DIR = Path(__file__).parent
SKILL_DIR = SCRIPT_DIR.parent
DATA_DIR = SKILL_DIR / "data"
ODOO_PILOT_DIR = SKILL_DIR.parent / "odoo-pilot" / "scripts"

# Blog: referencias-indaws-16 -> blog_id = 16, company_id = 1 (INDAWS)
BLOG_ID = 16
COMPANY_ID = 1
BLOG_SLUG = "referencias-indaws-16"


def load_env():
    """Load environment variables from .env file."""
    env_path = SKILL_DIR / ".env"
    if env_path.exists() and load_dotenv:
        load_dotenv(env_path)
    elif env_path.exists():
        # Fallback: manually parse .env file
        with open(env_path) as f:
            for line in f:
                line = line.strip()
                if line and not line.startswith("#") and "=" in line:
                    key, value = line.split("=", 1)
                    value = value.strip().strip('"').strip("'")
                    os.environ[key.strip()] = value


def check_env():
    """Check required environment variables."""
    load_env()
    required = ["ODOO_URL", "ODOO_DB", "ODOO_KEY"]
    missing = [var for var in required if not os.environ.get(var)]
    if missing:
        print(f"ERROR: Missing environment variables: {', '.join(missing)}", file=sys.stderr)
        sys.exit(1)


def run_odoo_search(model, domain, fields):
    """Run odoo-pilot search_records script."""
    script_path = ODOO_PILOT_DIR / "search_records.sh"

    if not script_path.exists():
        print(f"ERROR: odoo-pilot script not found: {script_path}", file=sys.stderr)
        sys.exit(1)

    cmd = [
        str(script_path),
        model,
        json.dumps(domain),
        json.dumps(fields)
    ]

    result = subprocess.run(cmd, capture_output=True, text=True)

    if result.returncode != 0:
        print(f"ERROR: Odoo search failed: {result.stderr}", file=sys.stderr)
        sys.exit(1)

    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        print(f"ERROR: Invalid JSON response: {result.stdout[:500]}", file=sys.stderr)
        sys.exit(1)


def clean_html(html_content):
    """Remove HTML tags and clean up content."""
    if not html_content:
        return ""

    # Remove HTML tags
    text = re.sub(r'<[^>]+>', ' ', html_content)
    # Normalize whitespace
    text = re.sub(r'\s+', ' ', text).strip()
    return text


def extract_summary(content, max_length=500):
    """Extract a summary from the content."""
    clean = clean_html(content)
    if len(clean) <= max_length:
        return clean
    return clean[:max_length].rsplit(' ', 1)[0] + "..."


def extract_modules_from_content(title, content):
    """Try to extract mentioned Indaws modules from content."""
    modules = []
    module_keywords = {
        "indaws_inventory": ["inventory", "inventario", "stock", "almacen"],
        "indaws_mrp": ["mrp", "manufacturing", "fabricacion", "produccion"],
        "indaws_purchase": ["purchase", "compras", "aprovisionamiento"],
        "indaws_sale": ["sales", "ventas", "comercial"],
        "indaws_quality": ["quality", "calidad", "qc"],
        "indaws_maintenance": ["maintenance", "mantenimiento"],
        "indaws_project": ["project", "proyecto"],
        "indaws_hr": ["hr", "recursos humanos", "nomina"],
    }

    text = (title + " " + clean_html(content)).lower()

    for module, keywords in module_keywords.items():
        for keyword in keywords:
            if keyword in text:
                modules.append(module)
                break

    return list(set(modules))


def main():
    check_env()

    DATA_DIR.mkdir(parents=True, exist_ok=True)

    odoo_url = os.environ.get("ODOO_URL", "").rstrip("/")

    print(f"Using blog ID: {BLOG_ID} (company_id: {COMPANY_ID})")

    print("Fetching blog posts...")
    posts = run_odoo_search(
        "blog.post",
        [["blog_id", "=", BLOG_ID], ["is_published", "=", True]],
        ["id", "name", "subtitle", "content", "website_url", "create_date", "author_id", "tag_ids"]
    )

    print(f"  Found {len(posts)} posts")

    cases = []
    for post in posts:
        # Build full URL
        website_url = post.get("website_url", "")
        if website_url and not website_url.startswith("http"):
            full_url = f"{odoo_url}{website_url}"
        else:
            full_url = website_url or f"{odoo_url}/blog/{BLOG_SLUG}/{post['id']}"

        # Extract client name from title (often "Client - Description" format)
        title = post.get("name", "")
        client = title.split(" - ")[0] if " - " in title else title.split(":")[0] if ":" in title else title

        case = {
            "id": post["id"],
            "title": title,
            "client": client.strip(),
            "subtitle": post.get("subtitle", ""),
            "url": full_url,
            "publish_date": post.get("create_date", "")[:10] if post.get("create_date") else "",
            "summary": extract_summary(post.get("content", "")),
            "content": clean_html(post.get("content", "")),
            "modules_used": extract_modules_from_content(title, post.get("content", "")),
            "industry": "",  # Would need manual tagging or extraction
        }
        cases.append(case)

    # Sort by publish date descending
    cases.sort(key=lambda x: x["publish_date"], reverse=True)

    # Save to JSON
    output_path = DATA_DIR / "cases.json"
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(cases, f, ensure_ascii=False, indent=2)

    print(f"\nSaved {len(cases)} case studies to {output_path}")


if __name__ == "__main__":
    main()
