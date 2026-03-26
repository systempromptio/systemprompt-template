# -*- coding: utf-8 -*-
"""Check proposal stages and alerts in Odoo."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from datetime import datetime
from scripts.odoo_client import OdooClient
from scripts import config


def main():
    print("=== CHECK PROPOSALS ===")
    print("")

    client = OdooClient()
    client.connect()
    print(f"[OK] Connected (uid: {client.uid})")
    print("")

    # Get all CRM stages
    stages = client.fetch_stages()
    print("=== STAGES IN ODOO ===")
    for sid, sname in sorted(stages.items()):
        print(f"  {sid}: {sname}")

    # Find stages matching proposal keywords
    print("")
    print("=== PROPOSAL STAGE VERIFICATION ===")
    print(f"Keywords in config: {config.STAGE_PROPOSAL_KEYWORDS}")

    matching_stages = []
    for sid, sname in stages.items():
        for kw in config.STAGE_PROPOSAL_KEYWORDS:
            if kw.lower() in sname.lower():
                matching_stages.append({"id": sid, "name": sname})
                print(f"[OK] Match: '{sname}' contains '{kw}'")

    if not matching_stages:
        print("[FAIL] No stages match the proposal keywords")
        return

    # Fetch leads in proposal stages
    stage_ids = [s["id"] for s in matching_stages]
    db = config.ODOO_DB
    uid = client.uid
    password = client.password

    leads = client.models.execute_kw(
        db, uid, password, "crm.lead", "search_read",
        [[["stage_id", "in", stage_ids], ["probability", "<", 100], ["probability", ">", 0]]],
        {"fields": ["name", "stage_id", "write_date", "user_id"]},
    )

    print("")
    print(f"=== LEADS IN PROPOSAL STAGE ({len(leads)} total) ===")

    ref_date = datetime.now()
    for lead in leads:
        write_date = datetime.fromisoformat(lead["write_date"].replace("Z", "+00:00"))
        days_inactive = (ref_date - write_date).days
        user = lead["user_id"][1] if lead["user_id"] else "Sin asignar"

        if days_inactive > config.PROPOSAL_FOLLOWUP_DAYS:
            status = "[ALERT]"
        else:
            status = "[OK]"
        print(f"  {status} | {days_inactive}d | {user} | {lead['name'][:40]}")


if __name__ == "__main__":
    main()
