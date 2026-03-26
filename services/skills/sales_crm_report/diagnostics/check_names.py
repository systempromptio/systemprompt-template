# -*- coding: utf-8 -*-
"""Verify salesperson names in Odoo match the config."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from scripts.odoo_client import OdooClient
from scripts import config


def main():
    print("=== CHECK SALESPERSON NAMES ===")
    print("")

    client = OdooClient()
    client.connect()
    print(f"[OK] Connected (uid: {client.uid})")
    print("")

    db = config.ODOO_DB
    uid = client.uid
    password = client.password

    # Get leads in stage 8 (Demo Propuesta)
    leads = client.models.execute_kw(
        db, uid, password, "crm.lead", "search_read",
        [[["stage_id", "=", 8]]],
        {"fields": ["name", "user_id"]},
    )

    print(f"=== LEADS IN STAGE 8 (Demo Propuesta) ===")
    print(f"Total: {len(leads)}")

    salespeople_found = set()
    for lead in leads:
        user = lead.get("user_id")
        sp_name = user[1] if user else "Sin asignar"
        salespeople_found.add(sp_name)

    print("")
    print("=== SALESPEOPLE WITH PROPOSALS ===")
    for sp in sorted(salespeople_found):
        in_config = sp in config.SALESPEOPLE_OBJECTIVES
        status = "[OK] IN CONFIG" if in_config else "[WARN] NOT IN CONFIG"
        print(f"  [{sp}] {status}")

    print("")
    print("=== SALESPEOPLE IN CONFIG ===")
    for sp in config.SALESPEOPLE_OBJECTIVES.keys():
        found = sp in salespeople_found
        status = "[OK] Found in Odoo" if found else "[INFO] Not in current proposals"
        print(f"  [{sp}] {status}")


if __name__ == "__main__":
    main()
