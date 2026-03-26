# -*- coding: utf-8 -*-
"""Check website view and page status in Odoo."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.odoo_client import OdooClient
from scripts import config


def main():
    print("=== CHECK ODOO VIEW / PAGE STATUS ===")
    print("")

    client = OdooClient()
    client.connect()
    print(f"[OK] Connected (uid: {client.uid})")
    print("")

    db = config.ODOO_DB
    uid = client.uid
    password = client.password

    # Search pages by URL
    print("=== SEARCHING PAGES WITH URL 'reporte-crm' ===")
    pages = client.models.execute_kw(
        db, uid, password, "website.page", "search_read",
        [[["url", "ilike", "reporte-crm"]]],
        {"fields": ["id", "name", "url", "view_id", "website_id", "is_published", "write_date"]},
    )

    print(f"Found {len(pages)} pages:")
    for p in pages:
        website = p.get("website_id")
        website_name = website[1] if website else "NO WEBSITE"
        print(f"  ID: {p['id']} | {p['name']}")
        print(f"    URL: {p['url']}")
        print(f"    Website: {website_name}")
        print(f"    View ID: {p['view_id']}")
        print(f"    Published: {p['is_published']}")
        print(f"    Updated: {p['write_date']}")
        print("")

    # Search views by key
    print("=== SEARCHING VIEWS WITH KEY 'indaws_crm_report' ===")
    views = client.models.execute_kw(
        db, uid, password, "ir.ui.view", "search_read",
        [[["key", "ilike", "indaws_crm_report"]]],
        {"fields": ["id", "name", "key", "write_date"]},
    )

    print(f"Found {len(views)} views:")
    for v in views:
        print(f"  ID: {v['id']} | {v['name']} | Key: {v['key']} | Updated: {v['write_date']}")

    if not pages and not views:
        print("[WARN] No report pages or views found in Odoo")
    else:
        print("")
        print("[OK] View/page check complete")


if __name__ == "__main__":
    main()
