# -*- coding: utf-8 -*-
"""Debug forecast calculations by tracing lead-level data."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from datetime import datetime, timedelta
from collections import Counter
from scripts.odoo_client import OdooClient
from scripts import config
from scripts.utils import parse_date


def main():
    print("=== DEBUG FORECAST METRICS ===")
    print("")

    client = OdooClient()
    client.connect()
    print(f"[OK] Connected (uid: {client.uid})")

    ref_date = datetime.now()
    cutoff_date = ref_date - timedelta(days=30 * config.MONTHS_LOOKBACK)

    print(f"Reference date: {ref_date.strftime('%Y-%m-%d')}")
    print(f"Cutoff date: {cutoff_date.strftime('%Y-%m-%d')}")
    print("")

    leads = client.fetch_leads(cutoff_date)
    users = client.fetch_users()
    stages = client.fetch_stages()

    print(f"Total leads fetched: {len(leads)}")
    print("")

    stats = {
        "total_active": 0,
        "with_deadline": 0,
        "current_month_match": 0,
        "frozen_excluded": 0,
        "forecast_total": 0.0,
    }

    month_distribution = Counter()

    header = f"{'Salesperson':<20} | {'Lead':<30} | {'Deadline':<10} | {'Frozen':<6} | {'Match?'} | {'Amount'}"
    print(header)
    print("-" * len(header))

    for lead in leads:
        user_id = lead.get("user_id")
        salesperson = users.get(user_id[0] if user_id else None, "Sin asignar")

        if salesperson not in config.SALESPEOPLE_OBJECTIVES:
            continue

        stage_id = lead.get("stage_id")
        stage_name = stages.get(stage_id[0] if stage_id else None, "Sin etapa")
        stage_lower = stage_name.lower()

        is_won = any(k.lower() in stage_lower for k in config.STAGE_WON_KEYWORDS)
        is_lost = any(k.lower() in stage_lower for k in config.STAGE_LOST_KEYWORDS)
        is_frozen = "congelado" in stage_lower

        if is_won or is_lost:
            continue

        stats["total_active"] += 1
        deadline_str = lead.get("date_deadline")
        deadline = parse_date(deadline_str)

        if deadline:
            stats["with_deadline"] += 1
            month_key = f"{deadline.year}-{deadline.month:02d}"
            month_distribution[month_key] += 1

            if deadline.month == ref_date.month and not is_frozen:
                stats["current_month_match"] += 1
                revenue = lead.get("expected_revenue", 0) or 0
                prob = (lead.get("probability", 0) or 0) / 100
                weighted = revenue * prob
                stats["forecast_total"] += weighted
                print(
                    f"{salesperson:<20} | {lead.get('name', '')[:30]:<30} | "
                    f"{str(deadline_str):<10} | {str(is_frozen):<6} | YES    | {weighted:.0f}"
                )
            elif deadline.month == ref_date.month and is_frozen:
                stats["frozen_excluded"] += 1
                print(
                    f"{salesperson:<20} | {lead.get('name', '')[:30]:<30} | "
                    f"{str(deadline_str):<10} | {str(is_frozen):<6} | NO(Fr) | 0"
                )

    print("")
    print("--- STATISTICS ---")
    print(f"Total Active (Sales Filtered): {stats['total_active']}")
    print(f"With Deadline: {stats['with_deadline']}")
    print(f"Match Current Month ({ref_date.month}): {stats['current_month_match']}")
    print(f"Excluded Frozen (Current Month): {stats['frozen_excluded']}")
    print(f"Forecast Total: {stats['forecast_total']:,.2f}")

    print("")
    print("--- DEADLINE DISTRIBUTION ---")
    for m in sorted(month_distribution.keys()):
        print(f"  {m}: {month_distribution[m]}")


if __name__ == "__main__":
    main()
