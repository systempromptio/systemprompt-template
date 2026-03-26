# -*- coding: utf-8 -*-
"""Verify report generation with mock data (no Odoo connection needed)."""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from datetime import datetime


def main():
    print("=== VERIFY REPORT GENERATION (MOCK DATA) ===")
    print("")

    # Import modules
    try:
        from scripts import data_processor
        print("[OK] data_processor imported")
    except ImportError as e:
        print(f"[FAIL] data_processor import failed: {e}")
        return

    try:
        from scripts import report_generator
        print("[OK] report_generator imported")
    except ImportError as e:
        print(f"[FAIL] report_generator import failed: {e}")
        return

    # Mock data dict
    data = {
        "objectives": [
            {"name": "Salesperson 1", "target": 10000, "forecast": 8000, "gap": -2000,
             "achievement": 80, "conversion": 25, "active_deals": 10},
            {"name": "Salesperson 2", "target": 10000, "forecast": 12000, "gap": 2000,
             "achievement": 120, "conversion": 40, "active_deals": 15},
        ],
        "total_active": 25,
        "activity": [("Salesperson 1", 10), ("Salesperson 2", 20)],
        "products": {"Cat 1": {"count": 5, "revenue": 5000}},
        "proposal_alerts": [],
        "zombies": [],
        "zombie_count": 0,
        "warning_count": 0,
        "quality_count": 0,
        "sources": [],
        "month_comparison": {"current": 100, "previous": 90, "delta": 10},
        "quarter_comparison": {"current": 300, "previous": 250, "delta": 50},
        "won_revenue": 50000,
        "won_last_month": 5,
        "avg_30": 10,
        "avg_60": 20,
        "avg_90": 30,
        "activity_counts": {"Salesperson 1": 10, "Salesperson 2": 20},
        "weekly_trends": [{"week": "S-1", "count": 10}, {"week": "Esta", "count": 12}],
        "quality_issues": [],
        "concentration_risk": 10,
        "top_deal": None,
        "new_leads_list": [],
        "won_deals_list": [],
        "total_objective": 20000,
        "age_buckets": {"0-7d": 5, "8-14d": 2, "15-30d": 1, "31-60d": 0, "60+d": 0},
        "salesperson_avg_age": {},
        "stage_avg_age": [],
        "yellow_zone": [],
        "forecast_leads": [
            {"name": "Lead 1", "salesperson": "Salesperson 1", "revenue": 10000,
             "weighted": 5000, "probability": 0.5},
        ],
        "sp_avg_velocity": {},
        "stage_duration": [],
        "sp_proposal_stats": [],
        "at_risk": [],
        "demos_data": {"current": 5, "target": 10, "by_salesperson": {}, "by_source": {}},
        "global_conversion": 34.5,
        "avg_cycle_time": 45.2,
        "executive_summary": {
            "status": "ON TRACK",
            "risks": [],
            "momentum": "STABLE",
            "projections": {"run_rate_4w": 10, "target_weekly": 10, "gap": 0},
        },
        "data_quality": {},
        "enriched": [],
    }

    ref_date = datetime.now()

    # Test report generation
    try:
        html = report_generator.generate_team_report(data, ref_date)
        output_path = Path(__file__).resolve().parent / "verification_report.html"
        with open(output_path, "w", encoding="utf-8") as f:
            f.write(html)
        print(f"[OK] Verification report generated: {output_path}")
        print(f"   HTML size: {len(html):,} characters")
    except Exception as e:
        print(f"[FAIL] Report generation failed: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    main()
