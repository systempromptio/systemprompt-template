# -*- coding: utf-8 -*-
"""Trace proposal detection logic for leads in stage 8."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from datetime import datetime
from scripts.odoo_client import OdooClient
from scripts import config


def main():
    print("=== TRACE PROPOSAL DETECTION ===")
    print("")

    client = OdooClient()
    client.connect()
    print(f"[OK] Connected (uid: {client.uid})")
    print("")

    db = config.ODOO_DB
    uid = client.uid
    password = client.password

    stages = client.fetch_stages()
    users = client.fetch_users()

    # Get leads in stage 8 (Demo Propuesta)
    leads = client.models.execute_kw(
        db, uid, password, "crm.lead", "search_read",
        [[["stage_id", "=", 8], ["probability", ">", 0]]],
        {"fields": ["name", "user_id", "stage_id", "write_date"]},
    )

    print(f"Keywords: {config.STAGE_PROPOSAL_KEYWORDS}")
    print("")

    ref_date = datetime.now()
    proposal_count = 0
    needs_alert = 0

    for lead in leads:
        user_id = lead.get("user_id")
        stage_id = lead.get("stage_id")
        salesperson = users.get(user_id[0] if user_id else None, "Sin asignar")
        stage_name = stages.get(stage_id[0] if stage_id else None, "Sin etapa")

        if salesperson not in config.SALESPEOPLE_OBJECTIVES:
            continue

        is_proposal = any(kw in stage_name for kw in config.STAGE_PROPOSAL_KEYWORDS)

        print(f"Lead: {lead['name'][:40]}")
        print(f"  Stage: '{stage_name}'")
        print(f"  is_proposal: {is_proposal}")

        for kw in config.STAGE_PROPOSAL_KEYWORDS:
            match = kw in stage_name
            print(f"    '{kw}' in '{stage_name}': {match}")

        if is_proposal:
            proposal_count += 1
            write_date = datetime.fromisoformat(lead["write_date"].replace("Z", "+00:00"))
            days_inactive = (ref_date - write_date).days

            if days_inactive > config.PROPOSAL_FOLLOWUP_DAYS:
                needs_alert += 1
                print(f"  [ALERT] needs_proposal_alert: True ({days_inactive} days)")
            else:
                print(f"  [OK] Within followup window ({days_inactive} days)")
        print("")

    print("=== TOTAL ===")
    print(f"Proposals from active salespeople: {proposal_count}")
    print(f"Proposals needing alert: {needs_alert}")


if __name__ == "__main__":
    main()
