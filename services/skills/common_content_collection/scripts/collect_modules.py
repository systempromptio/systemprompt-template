#!/usr/bin/env python3
"""
Collect Indaws Core module information from Odoo product catalog.

Requires:
    - ODOO_URL, ODOO_DB, ODOO_KEY environment variables
    - odoo-pilot skill scripts in PATH or sibling directory

Usage:
    python collect_modules.py
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

CATEGORY_NAME = "Indaws Core"


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


def get_category_id():
    """Find the category ID for Indaws Core."""
    categories = run_odoo_search(
        "product.category",
        [["name", "ilike", CATEGORY_NAME]],
        ["id", "name"]
    )

    if not categories:
        print(f"ERROR: Category '{CATEGORY_NAME}' not found", file=sys.stderr)
        sys.exit(1)

    # Find exact or best match
    for cat in categories:
        if cat["name"].lower() == CATEGORY_NAME.lower():
            return cat["id"]

    # Return first match if no exact match
    return categories[0]["id"]


def clean_html(html_content):
    """Remove HTML tags and clean up content."""
    if not html_content:
        return ""

    text = re.sub(r'<[^>]+>', ' ', html_content)
    text = re.sub(r'\s+', ' ', text).strip()
    return text


def extract_features(description):
    """Extract feature bullet points from description."""
    if not description:
        return []

    features = []

    # Look for bullet points or numbered lists
    lines = description.split('\n')
    for line in lines:
        line = clean_html(line).strip()
        # Match lines starting with -, *, numbers, or common feature indicators
        if re.match(r'^[-*\u2022]|\d+[.)]', line):
            feature = re.sub(r'^[-*\u2022\d.)\s]+', '', line).strip()
            if feature and len(feature) > 5:
                features.append(feature)

    # If no bullet points found, try to extract key phrases
    if not features:
        clean_desc = clean_html(description)
        # Split by common separators
        parts = re.split(r'[.;]', clean_desc)
        for part in parts[:5]:  # Take first 5 sentences as potential features
            part = part.strip()
            if part and len(part) > 10 and len(part) < 200:
                features.append(part)

    return features[:10]  # Limit to 10 features


def extract_technical_name(name):
    """Generate a technical name from product name."""
    # Convert "Indaws Inventory" to "indaws_inventory"
    tech_name = name.lower()
    tech_name = re.sub(r'[^a-z0-9\s]', '', tech_name)
    tech_name = re.sub(r'\s+', '_', tech_name)
    return tech_name


def main():
    check_env()

    DATA_DIR.mkdir(parents=True, exist_ok=True)

    print("Finding category ID...")
    category_id = get_category_id()
    print(f"  Category ID: {category_id}")

    print("Fetching products...")
    products = run_odoo_search(
        "product.template",
        [["categ_id", "=", category_id], ["name", "ilike", "[CORE]"]],
        ["id", "name", "description", "description_sale", "list_price", "default_code", "website_url"]
    )

    print(f"  Found {len(products)} [CORE] modules")

    odoo_url = os.environ.get("ODOO_URL", "").rstrip("/")

    modules = []
    for product in products:
        name = product.get("name", "")

        # Combine descriptions
        description = product.get("description_sale") or product.get("description") or ""

        # Build URL
        website_url = product.get("website_url", "")
        if website_url and not website_url.startswith("http"):
            full_url = f"{odoo_url}{website_url}"
        else:
            full_url = website_url or ""

        module = {
            "id": product["id"],
            "name": name,
            "technical_name": extract_technical_name(name),
            "code": product.get("default_code", ""),
            "description": clean_html(description),
            "features": extract_features(description),
            "price": product.get("list_price", 0),
            "url": full_url,
            "related_cases": [],  # Will be populated by build_indexes.py
            "related_videos": [],  # Will be populated by build_indexes.py
        }
        modules.append(module)

    # Sort by name
    modules.sort(key=lambda x: x["name"])

    # Save to JSON
    output_path = DATA_DIR / "modules.json"
    with open(output_path, "w", encoding="utf-8") as f:
        json.dump(modules, f, ensure_ascii=False, indent=2)

    print(f"\nSaved {len(modules)} modules to {output_path}")


if __name__ == "__main__":
    main()
