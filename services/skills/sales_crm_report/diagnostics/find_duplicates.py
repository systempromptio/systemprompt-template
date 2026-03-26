# -*- coding: utf-8 -*-
"""Find duplicate report pages and views in Odoo."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.odoo_client import OdooClient
from scripts import config


def main():
    print("=== FIND DUPLICATE REPORT PAGES / VIEWS ===")
    print("")

    client = OdooClient()
    client.connect()
    print(f"[OK] Connected (uid: {client.uid})")
    print("")

    db = config.ODOO_DB
    uid = client.uid
    password = client.password

    # Search for all pages with the report URL
    print("=== PAGES WITH URL /reporte-crm-semanal ===")
    pages = client.models.execute_kw(
        db, uid, password, "website.page", "search_read",
        [[["url", "ilike", "reporte-crm"]]],
        {"fields": ["id", "name", "url", "view_id", "is_published", "write_date"]},
    )

    print(f"Found {len(pages)} pages:")
    for p in pages:
        view_info = p["view_id"]
        view_str = f"View {view_info[0]} ({view_info[1]})" if view_info else "NO VIEW"
        print(f"  ID: {p['id']} | URL: {p['url']} | {view_str} | Updated: {p['write_date']}")

    if len(pages) > 1:
        print(f"[WARN] {len(pages)} duplicate pages found!")
    elif len(pages) == 1:
        print("[OK] Single page, no duplicates")
    else:
        print("[INFO] No pages found")

    # Search for all views with the report key
    print("")
    print("=== VIEWS WITH KEY 'indaws_crm_report' ===")
    views = client.models.execute_kw(
        db, uid, password, "ir.ui.view", "search_read",
        [[["key", "ilike", "indaws_crm_report"]]],
        {"fields": ["id", "name", "key", "write_date"]},
    )

    print(f"Found {len(views)} views:")
    for v in views:
        print(f"  ID: {v['id']} | Key: {v['key']} | Name: {v['name']} | Updated: {v['write_date']}")

    if len(views) > 1:
        print(f"[WARN] {len(views)} duplicate views found!")
    elif len(views) == 1:
        print("[OK] Single view, no duplicates")
    else:
        print("[INFO] No views found")

    # Cross-reference: check for orphaned pages or views
    print("")
    print("=== CROSS-REFERENCE CHECK ===")
    page_view_ids = {p["view_id"][0] for p in pages if p["view_id"]}
    view_ids = {v["id"] for v in views}

    orphan_views = view_ids - page_view_ids
    if orphan_views:
        print(f"[WARN] Views without pages: {orphan_views}")
    else:
        print("[OK] All views are linked to pages")


if __name__ == "__main__":
    main()
