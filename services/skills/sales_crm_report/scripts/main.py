# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - Main Entry Point
=====================================
CLI interface for generating, publishing, and emailing the CRM report.
"""

import argparse
import locale
import sys
import traceback
from datetime import datetime, timedelta
from pathlib import Path

from . import config
from .odoo_client import OdooClient
from . import odoo_publisher
from . import data_processor
from . import report_generator
from . import email_service


def main():
    # Set Spanish locale for month names
    for loc in ("es_ES.UTF-8", "es_ES", "Spanish_Spain"):
        try:
            locale.setlocale(locale.LC_TIME, loc)
            break
        except locale.Error:
            continue

    parser = argparse.ArgumentParser(description="INDAWS CRM Report Generator")
    parser.add_argument("--test", action="store_true", help="Test Odoo connection only")
    parser.add_argument("--test-email", action="store_true", help="Test SMTP connection only")
    parser.add_argument("--odoo", action="store_true", help="Post to Odoo website and create activities")
    parser.add_argument("--email", action="store_true", help="Send team report via email")
    parser.add_argument("--email-salespeople", action="store_true", help="Send personalized emails to salespeople")
    parser.add_argument("--output", default="report.html", help="Output HTML file (default: report.html)")
    parser.add_argument("--health-check", action="store_true", help="Connect and print basic stats")
    args = parser.parse_args()

    # Handle --test-email first (no Odoo connection needed)
    if args.test_email:
        return 0 if email_service.test_smtp_connection() else 1

    # Connect to Odoo
    print("Connecting to Odoo...")
    client = OdooClient()
    try:
        client.connect()
        print(f"[OK] Connected (uid: {client.uid})")
    except Exception as e:
        print(f"[FAIL] Connection failed: {e}")
        return 1

    # Handle --test
    if args.test:
        print("[OK] Connection test successful")
        return 0

    reference_date = datetime.now()

    # Handle --health-check
    if args.health_check:
        cutoff = reference_date - timedelta(days=30 * config.MONTHS_LOOKBACK)
        leads, stages, sources, users = client.fetch_all_data(cutoff)
        print(f"[OK] Health check passed")
        print(f"   - Leads: {len(leads)}")
        print(f"   - Stages: {len(stages)}")
        print(f"   - Sources: {len(sources)}")
        print(f"   - Users: {len(users)}")
        return 0

    # Fetch data
    cutoff_date = reference_date - timedelta(days=30 * config.MONTHS_LOOKBACK)
    print(f"Fetching data (since {cutoff_date.strftime('%Y-%m-%d')})...")
    leads, stages, sources, users = client.fetch_all_data(cutoff_date)
    print(f"   - {len(leads)} leads in period")

    # Fetch sales orders for the current month
    current_month_start = reference_date.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
    orders = client.fetch_sales_orders(current_month_start)
    print(f"   - {len(orders)} confirmed orders since {current_month_start.strftime('%Y-%m-%d')}")

    # Process data
    print(f"Processing data (reference: {reference_date.strftime('%Y-%m-%d')})...")
    data = data_processor.process_all(leads, stages, sources, users, reference_date, orders=orders)

    # Generate HTML
    html = report_generator.generate_team_report(data, reference_date)

    # Save to file
    output_path = Path(args.output)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(html)
    print(f"[OK] Local Report: {output_path.absolute()}")

    # Publish to Odoo
    if args.odoo:
        print("Posting to Odoo Website...")
        try:
            odoo_publisher.publish_report(client, html, reference_date)
        except Exception as e:
            print(f"[FAIL] Odoo integration failed: {e}")
            traceback.print_exc()

    # Send team email
    if args.email:
        print("Sending email report...")
        if email_service.send_team_report(html, reference_date):
            print("[OK] Email delivery complete")
        else:
            print("[WARN] Email delivery failed (report still generated locally)")

    # Send personalized emails
    if args.email_salespeople:
        print("Sending personalized emails to salespeople...")
        email_service.send_personalized_emails(data.get('enriched', []), data, reference_date)

    # Print summary
    print("")
    print("Summary:")
    print(f"   - Total active (filtered): {data.get('total_active', 0):,}")
    print(f"   - Won this month: {data.get('won_last_month', 0)} (EUR {data.get('won_revenue', 0):,.0f})")
    print(f"   - Zombies: {data.get('zombie_count', 0)}")

    return 0


if __name__ == "__main__":
    sys.exit(main())
