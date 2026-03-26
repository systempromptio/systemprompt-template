#!/usr/bin/env python3
"""
Master collection script - runs all collectors and rebuilds indexes.

Required environment variables:
    - YOUTUBE_API_KEY: YouTube Data API v3 key
    - ODOO_URL: Odoo instance URL (e.g., https://indaws.es)
    - ODOO_DB: Odoo database name
    - ODOO_KEY: Odoo API key

Optional:
    - ODOO_PROTOCOL: jsonrpc or json2 (default: auto-detect)

Usage:
    # Set environment variables first
    export YOUTUBE_API_KEY="your_key"
    export ODOO_URL="https://indaws.es"
    export ODOO_DB="indaws_production"
    export ODOO_KEY="your_odoo_key"

    # Run collection
    python collect_all.py

    # Run specific collectors only
    python collect_all.py --youtube
    python collect_all.py --cases
    python collect_all.py --modules
    python collect_all.py --indexes-only
"""

import argparse
import os
import subprocess
import sys
from datetime import datetime
from pathlib import Path

SCRIPT_DIR = Path(__file__).parent


def run_script(name, description):
    """Run a collector script and return success status."""
    script_path = SCRIPT_DIR / name
    print(f"\n{'='*60}")
    print(f"Running: {description}")
    print(f"Script: {script_path}")
    print(f"{'='*60}")

    result = subprocess.run(
        [sys.executable, str(script_path)],
        cwd=SCRIPT_DIR.parent
    )

    if result.returncode != 0:
        print(f"\nWARNING: {name} failed with exit code {result.returncode}")
        return False
    return True


def check_youtube_env():
    """Check YouTube API environment."""
    return bool(os.environ.get("YOUTUBE_API_KEY"))


def check_odoo_env():
    """Check Odoo environment variables."""
    required = ["ODOO_URL", "ODOO_DB", "ODOO_KEY"]
    return all(os.environ.get(var) for var in required)


def main():
    parser = argparse.ArgumentParser(description="Collect all Indaws content")
    parser.add_argument("--youtube", action="store_true", help="Collect YouTube videos only")
    parser.add_argument("--cases", action="store_true", help="Collect case studies only")
    parser.add_argument("--modules", action="store_true", help="Collect modules only")
    parser.add_argument("--sitemap", action="store_true", help="Collect sitemap only")
    parser.add_argument("--indexes-only", action="store_true", help="Rebuild indexes only (no collection)")
    args = parser.parse_args()

    # If no specific flags, run everything
    run_all = not (args.youtube or args.cases or args.modules or args.sitemap or args.indexes_only)

    print(f"Content Collection - {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Working directory: {SCRIPT_DIR.parent}")

    success_count = 0
    total_count = 0

    # YouTube collection
    if run_all or args.youtube:
        total_count += 1
        if check_youtube_env():
            if run_script("collect_youtube.py", "YouTube Video Collection"):
                success_count += 1
        else:
            print("\nSKIPPED: YouTube collection (YOUTUBE_API_KEY not set)")

    # Odoo collections
    odoo_ready = check_odoo_env()

    if run_all or args.cases:
        total_count += 1
        if odoo_ready:
            if run_script("collect_cases.py", "Case Studies Collection"):
                success_count += 1
        else:
            print("\nSKIPPED: Cases collection (Odoo env vars not set)")

    if run_all or args.modules:
        total_count += 1
        if odoo_ready:
            if run_script("collect_modules.py", "Modules Collection"):
                success_count += 1
        else:
            print("\nSKIPPED: Modules collection (Odoo env vars not set)")

    # Sitemap collection (no env vars required)
    if run_all or args.sitemap:
        total_count += 1
        if run_script("collect_sitemap.py", "Sitemap Collection"):
            success_count += 1

    # Always rebuild indexes (unless explicitly skipped by running only collectors)
    if run_all or args.indexes_only or success_count > 0:
        total_count += 1
        if run_script("build_indexes.py", "Index Generation"):
            success_count += 1

    # Summary
    print(f"\n{'='*60}")
    print(f"Collection Complete: {success_count}/{total_count} tasks succeeded")
    print(f"{'='*60}")

    if success_count < total_count:
        print("\nSome tasks failed. Check the output above for details.")
        print("\nRequired environment variables:")
        print("  YOUTUBE_API_KEY - YouTube Data API v3 key")
        print("  ODOO_URL        - Odoo instance URL")
        print("  ODOO_DB         - Odoo database name")
        print("  ODOO_KEY        - Odoo API key")
        sys.exit(1)

    print("\nNext steps:")
    print("1. Review the collected data in data/*.json")
    print("2. Review the generated indexes in references/*-index.md")
    print("3. Package the skill: python /path/to/package_skill.py .")


if __name__ == "__main__":
    main()
