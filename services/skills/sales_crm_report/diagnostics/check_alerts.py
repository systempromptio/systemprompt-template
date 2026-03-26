# -*- coding: utf-8 -*-
"""Check Odoo view content for report sections."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

import re
from scripts.odoo_client import OdooClient
from scripts import config


def main():
    print("=== CHECK ALERTS IN ODOO VIEW ===")
    print("")

    client = OdooClient()
    client.connect()
    print(f"[OK] Connected (uid: {client.uid})")
    print("")

    db = config.ODOO_DB
    uid = client.uid
    password = client.password

    # Fetch the report view by ID
    view = client.models.execute_kw(
        db, uid, password, "ir.ui.view", "search_read",
        [[["id", "=", 12139]]],
        {"fields": ["arch_db"]},
    )

    if not view:
        print("[FAIL] View with ID 12139 not found")
        return

    content = view[0]["arch_db"]

    print("=== VERIFYING CONTENT IN ODOO ===")

    # Check for key sections
    sections_to_check = [
        ("Farming", True),
        ("Todas las propuestas al dia", False),
        ("Tendencia de Captacion", True),
        ("Top Origenes", True),
        ("Calidad del Dato", True),
    ]

    for section_name, expect_present in sections_to_check:
        found = section_name in content
        if found == expect_present:
            print(f"[OK] '{section_name}' {'found' if found else 'not found'} (expected)")
        else:
            print(f"[WARN] '{section_name}' {'found' if found else 'not found'} (unexpected)")

    # Find the alerts section
    alerts_match = re.search(r"Alertas Propuestas.*?</div>", content, re.DOTALL)
    if alerts_match:
        print("")
        print("=== ALERTS SECTION ===")
        alert_text = alerts_match.group(0)[:800]
        print(alert_text)
    else:
        print("")
        print("[INFO] No 'Alertas Propuestas' section found in view")


if __name__ == "__main__":
    main()
