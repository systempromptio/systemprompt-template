#!/usr/bin/env python3
"""Unit tests for calculate_crm_health.py"""

import json
import os
import sys
import unittest
from datetime import date

sys.path.insert(0, os.path.dirname(__file__))
from calculate_crm_health import calculate, clamp, parse_date, get_id, get_name, get_estado


FIXTURES_DIR = os.path.join(os.path.dirname(__file__), "fixtures")


def load_fixture(name="sample_crm_data.json"):
    with open(os.path.join(FIXTURES_DIR, name), "r", encoding="utf-8") as f:
        return json.load(f)


def make_empty_data():
    return {
        "leads_active": [],
        "leads_won": [],
        "leads_lost": [],
        "leads_all": [],
        "sale_orders": [],
        "stages": [],
        "users": [],
        "summary": {
            "has_demo_field": False,
            "lost_available": True,
        },
    }


def make_minimal_pipeline(n_leads=5, ref_date=None):
    """Create minimal pipeline data with n_leads in pipeline stages."""
    if ref_date is None:
        ref_date = date(2026, 2, 15)
    leads = []
    for i in range(n_leads):
        leads.append({
            "id": 1000 + i,
            "name": f"Opp Test {i}",
            "user_id": [10, "Test User"],
            "stage_id": [3, "Propuesta enviada"],
            "probability": 50,
            "expected_revenue": 10000,
            "date_deadline": f"{ref_date.year}-{ref_date.month:02d}-28",
            "create_date": f"{ref_date.year}-01-01",
            "write_date": str(ref_date),
            "date_last_stage_update": str(ref_date),
            "date_closed": False,
            "x_studio_fecha_demo": False,
            "x_studio_last_update": str(ref_date),
            "active": True,
        })
    return {
        "leads_active": leads,
        "leads_won": [],
        "leads_lost": [],
        "leads_all": leads[:],
        "sale_orders": [],
        "stages": [
            {"id": 3, "name": "Propuesta enviada", "sequence": 50},
        ],
        "users": [{"id": 10, "name": "Test User", "login": "test@test.com"}],
        "summary": {"has_demo_field": True, "lost_available": True},
    }


# --- Helper tests ---

class TestHelpers(unittest.TestCase):

    def test_clamp(self):
        self.assertEqual(clamp(50, 0, 100), 50)
        self.assertEqual(clamp(-5, 0, 100), 0)
        self.assertEqual(clamp(150, 0, 100), 100)

    def test_parse_date_valid(self):
        self.assertEqual(parse_date("2026-02-15"), date(2026, 2, 15))
        self.assertEqual(parse_date("15/02/2026"), date(2026, 2, 15))

    def test_parse_date_invalid(self):
        self.assertIsNone(parse_date(None))
        self.assertIsNone(parse_date(False))
        self.assertIsNone(parse_date(""))
        self.assertIsNone(parse_date("not-a-date"))

    def test_get_id(self):
        self.assertEqual(get_id([10, "Name"]), 10)
        self.assertEqual(get_id(10), 10)
        self.assertIsNone(get_id(False))
        self.assertIsNone(get_id(None))

    def test_get_name(self):
        self.assertEqual(get_name([10, "Name"]), "Name")
        self.assertEqual(get_name("Direct"), "Direct")
        self.assertIsNone(get_name(False))

    def test_get_estado(self):
        self.assertEqual(get_estado(85)[0], "SALUDABLE")
        self.assertEqual(get_estado(70)[0], "EN RIESGO")
        self.assertEqual(get_estado(45)[0], "CRITICO")
        self.assertEqual(get_estado(20)[0], "EMERGENCIA")


# --- Empty pipeline ---

class TestEmptyPipeline(unittest.TestCase):

    def test_empty_data_returns_zero_score(self):
        data = make_empty_data()
        result = calculate(data, ref_date=date(2026, 2, 15))
        self.assertEqual(result["health_score"]["score"], 0)
        self.assertEqual(result["health_score"]["estado"], "EMERGENCIA")

    def test_empty_data_scenario(self):
        data = make_empty_data()
        result = calculate(data, ref_date=date(2026, 2, 15))
        self.assertEqual(result["scenario"]["code"], "LOW_PIPELINE")

    def test_empty_data_has_alert(self):
        data = make_empty_data()
        result = calculate(data, ref_date=date(2026, 2, 15))
        self.assertTrue(
            any("No hay oportunidades" in a for a in result["alerts"])
        )


# --- Healthy pipeline ---

class TestHealthyPipeline(unittest.TestCase):

    def setUp(self):
        self.data = load_fixture()
        self.ref_date = date(2026, 2, 15)

    def test_score_is_valid_range(self):
        result = calculate(self.data, ref_date=self.ref_date)
        score = result["health_score"]["score"]
        self.assertGreaterEqual(score, 0)
        self.assertLessEqual(score, 100)

    def test_pipeline_metrics_populated(self):
        result = calculate(self.data, ref_date=self.ref_date)
        pm = result["pipeline_metrics"]
        self.assertGreater(pm["total_pipeline"], 0)
        self.assertGreater(pm["pipeline_revenue"], 0)
        self.assertGreater(pm["total_won"], 0)

    def test_win_rate_calculated(self):
        result = calculate(self.data, ref_date=self.ref_date)
        pm = result["pipeline_metrics"]
        # 18 total opps >= 5, so win_rate should be calculated (not default 50)
        self.assertTrue(pm["win_rate_sufficient"])
        # 5 won / 18 total = 27.8%
        self.assertAlmostEqual(pm["win_rate"], 27.8, places=1)

    def test_penalties_are_non_negative(self):
        result = calculate(self.data, ref_date=self.ref_date)
        pen = result["penalties"]
        self.assertGreaterEqual(pen["rendimiento"]["total"], 0)
        self.assertGreaterEqual(pen["gobernanza"]["total"], 0)
        self.assertGreaterEqual(pen["total"], 0)

    def test_score_equals_100_minus_penalties(self):
        result = calculate(self.data, ref_date=self.ref_date)
        score = result["health_score"]["score"]
        total_pen = result["penalties"]["total"]
        expected = max(0, min(100, round(100 - total_pen)))
        self.assertEqual(score, expected)

    def test_salesperson_scores_present(self):
        result = calculate(self.data, ref_date=self.ref_date)
        sp = result["salesperson_scores"]
        self.assertGreater(len(sp), 0)
        for s in sp:
            self.assertIn("score", s)
            self.assertIn("name", s)
            self.assertGreaterEqual(s["score"], 0)
            self.assertLessEqual(s["score"], 100)


# --- Demo analysis ---

class TestDemoAnalysis(unittest.TestCase):

    def test_demo_analysis_present_when_field_exists(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        self.assertIsNotNone(result["demo_analysis"])
        self.assertIn("demo_rate", result["demo_analysis"])

    def test_demo_analysis_none_when_field_missing(self):
        data = load_fixture()
        data["summary"]["has_demo_field"] = False
        result = calculate(data, ref_date=date(2026, 2, 15))
        self.assertIsNone(result["demo_analysis"])
        self.assertTrue(
            any("x_studio_fecha_demo" in a for a in result["alerts"])
        )

    def test_win_rate_con_demo(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        da = result["demo_analysis"]
        if da and da["win_rate_con_demo"] is not None:
            self.assertGreaterEqual(da["win_rate_con_demo"], 0)
            self.assertLessEqual(da["win_rate_con_demo"], 100)


# --- Insufficient win rate data ---

class TestInsufficientWinRate(unittest.TestCase):

    def test_fewer_than_5_opps_gives_neutral_win_rate(self):
        data = make_minimal_pipeline(n_leads=3)
        # Only 3 leads in leads_all, < 5
        result = calculate(data, ref_date=date(2026, 2, 15))
        pm = result["pipeline_metrics"]
        self.assertEqual(pm["win_rate"], 50.0)
        self.assertFalse(pm["win_rate_sufficient"])

    def test_win_rate_penalty_zero_when_insufficient(self):
        data = make_minimal_pipeline(n_leads=3)
        result = calculate(data, ref_date=date(2026, 2, 15))
        pen = result["penalties"]["rendimiento"]
        self.assertEqual(pen["win_rate_bajo"], 0.0)


# --- Governance issues ---

class TestGovernance(unittest.TestCase):

    def test_detects_missing_deadline(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        gov = result["governance"]
        # Opp 107 (Eta) has no deadline
        self.assertGreater(gov["sin_fecha_cierre"]["count"], 0)

    def test_detects_missing_revenue(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        gov = result["governance"]
        # Opp 107 (Eta) has 0 revenue
        self.assertGreater(gov["sin_revenue"]["count"], 0)

    def test_detects_missing_user(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        gov = result["governance"]
        # Opp 107 (Eta) has user_id=false
        self.assertGreater(gov["sin_comercial"]["count"], 0)

    def test_detects_overdue(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        gov = result["governance"]
        # Opp 108 (Theta) deadline=2026-01-15, overdue on 2026-02-15
        self.assertGreater(gov["vencidas_abiertas"]["count"], 0)

    def test_five_governance_types(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        gov = result["governance"]
        expected_keys = {
            "sin_fecha_cierre",
            "sin_revenue",
            "sin_comercial",
            "vencidas_abiertas",
            "sin_actividad_30d",
        }
        self.assertEqual(set(gov.keys()), expected_keys)


# --- Forecast ---

class TestForecast(unittest.TestCase):

    def test_forecast_calculated(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        fc = result["forecast"]
        self.assertGreater(fc["opportunities_count"], 0)
        self.assertGreater(fc["forecast_weighted"], 0)

    def test_coverage_with_target(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15), monthly_target=100000)
        fc = result["forecast"]
        self.assertIsNotNone(fc["coverage_pct"])

    def test_coverage_without_target(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        fc = result["forecast"]
        self.assertIsNone(fc["coverage_pct"])


# --- Scenario detection ---

class TestScenarioDetection(unittest.TestCase):

    def test_low_pipeline_scenario(self):
        data = make_minimal_pipeline(n_leads=3)
        result = calculate(data, ref_date=date(2026, 2, 15))
        self.assertEqual(result["scenario"]["code"], "LOW_PIPELINE")

    def test_healthy_scenario_possible(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        # Score may vary but scenario should be one of the 6 valid ones
        valid = {
            "HEALTHY_PIPELINE", "LOW_PIPELINE", "STALE_PIPELINE",
            "LOW_WIN_RATE", "GOVERNANCE_ISSUES", "UNBALANCED_TEAM",
        }
        self.assertIn(result["scenario"]["code"], valid)

    def test_no_demos_scenario_does_not_exist(self):
        """NO_DEMOS scenario was removed - it must never appear."""
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        self.assertNotEqual(result["scenario"]["code"], "NO_DEMOS")


# --- Dashboard extra ---

class TestDashboardExtra(unittest.TestCase):

    def test_closed_orders_present(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        de = result["dashboard_extra"]
        self.assertIn("closed_orders_this_month", de)
        self.assertIn("closed_orders_prev_month", de)

    def test_closed_orders_this_month(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        this_month = result["dashboard_extra"]["closed_orders_this_month"]
        # SO/2026/001 and SO/2026/002 have date_order in Feb 2026
        self.assertEqual(this_month["count"], 2)
        self.assertEqual(this_month["revenue"], 70000.0)

    def test_closed_orders_prev_month(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        prev_month = result["dashboard_extra"]["closed_orders_prev_month"]
        # SO/2026/003 and SO/2026/004 have date_order in Jan 2026
        self.assertEqual(prev_month["count"], 2)
        self.assertEqual(prev_month["revenue"], 34000.0)

    def test_new_opps_this_month(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        de = result["dashboard_extra"]
        # Opp Kappa (create_date=2026-02-10) is the only one created in Feb
        self.assertGreaterEqual(de["new_opps_this_month"], 1)


# --- Output structure ---

class TestOutputStructure(unittest.TestCase):

    def test_all_top_level_keys(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        required_keys = {
            "health_score", "scenario", "pipeline_metrics", "penalties",
            "forecast", "demo_analysis", "stages_summary", "by_salesperson",
            "salesperson_scores", "team_balance", "governance",
            "dashboard_extra", "alerts", "validation",
        }
        self.assertEqual(required_keys, required_keys & set(result.keys()))

    def test_json_serializable(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        # Must not raise
        serialized = json.dumps(result, ensure_ascii=False)
        self.assertIsInstance(serialized, str)


# --- Stale thresholds per stage ---

class TestStaleThresholds(unittest.TestCase):

    def test_stale_detection_with_inactive_lead(self):
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        pm = result["pipeline_metrics"]
        # Opp 108 (Theta) has x_studio_last_update=2026-01-10, stage 3 threshold=10d
        # 2026-02-15 - 2026-01-10 = 36 days > 10 -> stale
        self.assertGreater(pm["stale_count"], 0)

    def test_en_espera_has_20d_threshold(self):
        """Opp Zeta is in En Espera (id=7, 20d threshold),
        last_update=2025-12-20, ref=2026-02-15 -> 57 days > 20 -> stale."""
        data = load_fixture()
        result = calculate(data, ref_date=date(2026, 2, 15))
        gov = result["governance"]["sin_actividad_30d"]
        zeta_inactive = any(
            "Zeta" in item.get("name", "") for item in gov["items"]
        )
        self.assertTrue(zeta_inactive)


if __name__ == "__main__":
    unittest.main()
