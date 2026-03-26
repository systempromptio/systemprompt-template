#!/usr/bin/env python3
# =============================================================================
# calculate_crm_health.py - Motor de calculo de salud del CRM
# =============================================================================
# Calcula el CRM Health Score global (0-100), scoring individual por comercial,
# analisis de demos, prevision mensual, problemas de gobernanza y escenario
# de diagnostico.
#
# Uso:
#   python3 calculate_crm_health.py --input crm_data.json --target 230000
#   python3 calculate_crm_health.py --input crm_data.json --config stages.json
#
# Config JSON (stages.json):
#   {
#     "early_stage_ids": [1, 22, 19],
#     "pipeline_stage_ids": [24, 8, 2, 9, 3, 7, 18, 23],
#     "stale_field": "x_studio_last_update",
#     "stale_thresholds": { "24": 10, "8": 10, "7": 20, "23": 30 }
#   }
#
# Input: JSON de fetch_crm_data.sh
# Output: JSON con metricas calculadas a stdout
# =============================================================================

import argparse
import json
import sys
from datetime import date, datetime, timedelta


# --- Default config (Indaws) ---
DEFAULT_CONFIG = {
    "early_stage_ids": [1, 22, 19],
    "pipeline_stage_ids": [24, 8, 2, 9, 3, 7, 18, 23],
    "won_stage_id": 4,
    "lost_stage_id": 5,
    "stale_field": "x_studio_last_update",
    "min_pipeline_leads": 3,
    "stale_thresholds": {
        24: 10,   # Pte. Asignar consultor
        8: 10,    # Demo propuesta
        2: 10,    # Demo realizada
        18: 10,   # Upselling
        9: 10,    # Presupuesto creado
        3: 10,    # Propuesta enviada
        7: 20,    # En Espera
        23: 30,   # En analisis
    },
    "stage_names": {
        1: "Lead",
        22: "Cualificado-Asignado",
        19: "1er contacto",
        24: "Pte. Asignar consultor",
        8: "Demo propuesta",
        2: "Demo realizada",
        9: "Presupuesto creado",
        3: "Propuesta enviada",
        7: "En Espera",
        18: "Upselling",
        23: "En analisis",
        4: "Ganado",
        5: "Congelado",
    },
}


# --- Helpers ---

def clamp(val, lo, hi):
    return max(lo, min(hi, val))


def round1(val):
    return round(val, 1)


def round2(val):
    return round(val, 2)


def parse_date(val):
    if not val or val is False:
        return None
    s = str(val).strip()[:10]
    for fmt in ("%Y-%m-%d", "%d/%m/%Y"):
        try:
            return datetime.strptime(s, fmt).date()
        except ValueError:
            continue
    return None


def get_id(field):
    """Extract numeric ID from Odoo many2one field (can be [id, name] or int)."""
    if isinstance(field, list) and len(field) >= 1:
        return field[0]
    if isinstance(field, (int, float)) and field:
        return int(field)
    return None


def get_name(field):
    """Extract display name from Odoo many2one field."""
    if isinstance(field, list) and len(field) >= 2:
        return field[1]
    if isinstance(field, str):
        return field
    return None


def get_estado(score):
    if score >= 80:
        return "SALUDABLE", "\U0001f7e2"
    elif score >= 60:
        return "EN RIESGO", "\U0001f7e1"
    elif score >= 40:
        return "CRITICO", "\U0001f7e0"
    else:
        return "EMERGENCIA", "\U0001f534"


def build_health_bar(score):
    filled = round(score / 100 * 30)
    bar = "\u2588" * filled + "\u2592" * (30 - filled)
    return bar


def resolve_stage_name(lead_or_id, stage_names_map):
    """Resolve stage name using config mapping, fallback to Odoo name."""
    if isinstance(lead_or_id, dict):
        sid = get_id(lead_or_id.get("stage_id"))
        odoo_name = get_name(lead_or_id.get("stage_id")) or ""
    else:
        sid = lead_or_id
        odoo_name = ""
    if sid and sid in stage_names_map:
        return stage_names_map[sid]
    return odoo_name or f"Etapa {sid}"


def get_activity_date(lead, stale_field):
    """Get the most relevant activity date for a lead."""
    # Prefer configured stale_field (e.g. x_studio_last_update)
    d = parse_date(lead.get(stale_field))
    if d:
        return d
    # Fallback to write_date
    d = parse_date(lead.get("write_date"))
    if d:
        return d
    # Fallback to date_last_stage_update
    return parse_date(lead.get("date_last_stage_update"))


def _normalize_stale_thresholds(raw):
    """Ensure stale_thresholds keys are int (JSON keys come as str)."""
    if not raw:
        return {}
    return {int(k): v for k, v in raw.items()}


# =============================================================================
# Main calculation
# =============================================================================

def calculate(data, ref_date=None, monthly_target=None, stale_days=21, config=None):
    if ref_date is None:
        ref_date = date.today()

    if config is None:
        config = DEFAULT_CONFIG

    early_ids = set(config.get("early_stage_ids", []))
    pipeline_ids = set(config.get("pipeline_stage_ids", []))
    stale_field = config.get("stale_field", "x_studio_last_update")
    stage_names = config.get("stage_names", {})
    # Normalize stage_names keys to int (JSON keys come as str)
    stage_names = {int(k): v for k, v in stage_names.items()}
    min_pipeline_leads = config.get("min_pipeline_leads", 3)
    stale_thresholds = _normalize_stale_thresholds(
        config.get("stale_thresholds", DEFAULT_CONFIG.get("stale_thresholds", {}))
    )

    alerts = []
    warnings = []

    # --- Extract data ---
    leads_active = data.get("leads_active", [])
    leads_won = data.get("leads_won", [])
    leads_lost = data.get("leads_lost", [])
    leads_all = data.get("leads_all", [])
    sale_orders = data.get("sale_orders", [])
    stages = data.get("stages", [])
    users = data.get("users", [])
    summary = data.get("summary", {})

    has_demo_field = summary.get("has_demo_field", False)
    lost_available = summary.get("lost_available", True)

    # Build stage lookup
    stage_map = {}
    for s in stages:
        sid = s.get("id")
        if sid:
            stage_map[sid] = s

    # Build user lookup
    user_map = {}
    for u in users:
        uid = u.get("id")
        if uid:
            user_map[uid] = u

    # --- Classify leads by stage ---
    leads_early = []      # Early stages: no governance penalty
    leads_pipeline = []   # Main pipeline: full analysis
    leads_other = []      # Ignored stages

    for l in leads_active:
        sid = get_id(l.get("stage_id"))
        if sid in early_ids:
            leads_early.append(l)
        elif sid in pipeline_ids:
            leads_pipeline.append(l)
        else:
            leads_other.append(l)

    total_active = len(leads_active)
    total_pipeline = len(leads_pipeline)
    total_early = len(leads_early)
    total_other = len(leads_other)
    total_won = len(leads_won)
    total_lost = len(leads_lost)
    total_all_opps = len(leads_all)

    # Revenue only from pipeline leads
    pipeline_revenue = sum(l.get("expected_revenue") or 0 for l in leads_pipeline)

    # Date helpers
    current_month_start = date(ref_date.year, ref_date.month, 1)
    if ref_date.month == 12:
        next_month_start = date(ref_date.year + 1, 1, 1)
    else:
        next_month_start = date(ref_date.year, ref_date.month + 1, 1)

    if ref_date.month == 1:
        prev_month_start = date(ref_date.year - 1, 12, 1)
    else:
        prev_month_start = date(ref_date.year, ref_date.month - 1, 1)

    # =========================================================================
    # 1. PIPELINE METRICS (based on pipeline leads only)
    # =========================================================================

    # Win rate = ganadas (12m) / total oportunidades (activas + archivadas)
    if total_all_opps >= 5:
        win_rate = round1(total_won / total_all_opps * 100)
        win_rate_sufficient = True
    else:
        win_rate = 50.0
        win_rate_sufficient = False
        if total_all_opps > 0:
            warnings.append(
                f"Datos insuficientes para win rate ({total_all_opps} opp totales). "
                f"Se usa valor neutro (50%)."
            )

    # Stale opportunities (per-stage thresholds)
    stale_count = 0
    for l in leads_pipeline:
        sid = get_id(l.get("stage_id"))
        threshold = stale_thresholds.get(sid, stale_days)
        cutoff = ref_date - timedelta(days=threshold)
        ad = get_activity_date(l, stale_field)
        if ad and ad < cutoff:
            stale_count += 1

    stale_pct = round1(stale_count / total_pipeline * 100) if total_pipeline > 0 else 0.0

    # Overdue opportunities (pipeline only)
    overdue_count = 0
    overdue_leads = []
    for l in leads_pipeline:
        dl = parse_date(l.get("date_deadline"))
        if dl and dl < ref_date:
            overdue_count += 1
            overdue_leads.append({
                "name": l.get("name", ""),
                "user": get_name(l.get("user_id")) or "Sin asignar",
                "date_deadline": str(dl),
                "days_overdue": (ref_date - dl).days,
                "expected_revenue": l.get("expected_revenue") or 0,
                "stage": resolve_stage_name(l, stage_names),
            })

    overdue_leads.sort(key=lambda x: -x["days_overdue"])
    overdue_pct = round1(overdue_count / total_pipeline * 100) if total_pipeline > 0 else 0.0

    # Inactive 30+ days (pipeline only)
    inactive_cutoff = ref_date - timedelta(days=30)
    inactive_count = 0
    for l in leads_pipeline:
        ad = get_activity_date(l, stale_field)
        if ad and ad < inactive_cutoff:
            inactive_count += 1

    inactive_pct = round1(inactive_count / total_pipeline * 100) if total_pipeline > 0 else 0.0

    pipeline_metrics = {
        "total_all_leads": total_active,
        "total_pipeline": total_pipeline,
        "total_early": total_early,
        "total_other": total_other,
        "total_won": total_won,
        "total_lost": total_lost,
        "total_all_opps": total_all_opps,
        "pipeline_revenue": round2(pipeline_revenue),
        "win_rate": win_rate,
        "win_rate_sufficient": win_rate_sufficient,
        "stale_count": stale_count,
        "stale_pct": stale_pct,
        "stale_thresholds": stale_thresholds,
        "stale_field": stale_field,
        "overdue_count": overdue_count,
        "overdue_pct": overdue_pct,
        "inactive_30d_count": inactive_count,
        "inactive_30d_pct": inactive_pct,
    }

    # Win rate con demo (se calcula despues del demo analysis y se inyecta)
    # Placeholder - se rellena tras la seccion 2
    win_rate_con_demo = None
    win_rate_con_demo_won = 0
    win_rate_con_demo_total = 0

    # =========================================================================
    # 2. DEMO ANALYSIS (pipeline leads only)
    # =========================================================================
    demo_analysis = None

    if has_demo_field:
        # Qualified = pipeline leads (all pipeline stages are "qualified")
        qualified = leads_pipeline
        total_qualified = len(qualified)
        total_with_demo = sum(
            1 for l in qualified
            if l.get("x_studio_fecha_demo") and l.get("x_studio_fecha_demo") is not False
        )
        demo_rate = round1(total_with_demo / total_qualified * 100) if total_qualified > 0 else 0.0

        # Demos this month (from active leads, not just pipeline)
        current_month = ref_date.month
        current_year = ref_date.year
        demos_this_month = 0
        for l in leads_active:
            dd = parse_date(l.get("x_studio_fecha_demo"))
            if dd and dd.month == current_month and dd.year == current_year:
                demos_this_month += 1

        # Demo conversion using all_12m totals
        won_with_demo = 0
        won_without_demo = 0
        demo_to_close_days = []

        for l in leads_won:
            demo_date = l.get("x_studio_fecha_demo")
            close_date = parse_date(l.get("date_closed")) or parse_date(l.get("write_date"))
            if demo_date and demo_date is not False:
                won_with_demo += 1
                dd = parse_date(demo_date)
                if dd and close_date and close_date > dd:
                    demo_to_close_days.append((close_date - dd).days)
            else:
                won_without_demo += 1

        # Total with/without demo from all opportunities dataset
        all_with_demo = sum(
            1 for l in leads_all
            if l.get("x_studio_fecha_demo") and l.get("x_studio_fecha_demo") is not False
        )
        all_without_demo = total_all_opps - all_with_demo

        conversion_with_demo = round1(
            won_with_demo / all_with_demo * 100
        ) if all_with_demo > 0 else 0.0
        conversion_without_demo = round1(
            won_without_demo / all_without_demo * 100
        ) if all_without_demo > 0 else 0.0

        avg_demo_to_close = round1(
            sum(demo_to_close_days) / len(demo_to_close_days)
        ) if demo_to_close_days else 0.0

        # Demos by salesperson (this month)
        demos_by_user = {}
        for l in leads_active:
            uid = get_id(l.get("user_id"))
            if not uid:
                continue
            dd = parse_date(l.get("x_studio_fecha_demo"))
            if dd and dd.month == current_month and dd.year == current_year:
                uname = get_name(l.get("user_id")) or f"UID {uid}"
                demos_by_user.setdefault(uname, 0)
                demos_by_user[uname] += 1

        demo_analysis = {
            "total_qualified": total_qualified,
            "total_with_demo": total_with_demo,
            "demo_rate": demo_rate,
            "demos_this_month": demos_this_month,
            "demos_by_user": demos_by_user,
            "won_with_demo": won_with_demo,
            "won_without_demo": won_without_demo,
            "conversion_with_demo": conversion_with_demo,
            "conversion_without_demo": conversion_without_demo,
            "avg_demo_to_close_days": avg_demo_to_close,
        }
        # Win rate con demo (global)
        win_rate_con_demo_won = won_with_demo
        win_rate_con_demo_total = all_with_demo
        if all_with_demo > 0:
            win_rate_con_demo = round1(won_with_demo / all_with_demo * 100)
        else:
            win_rate_con_demo = None

        demo_analysis["win_rate_con_demo"] = win_rate_con_demo
        demo_analysis["win_rate_con_demo_won"] = win_rate_con_demo_won
        demo_analysis["win_rate_con_demo_total"] = win_rate_con_demo_total

    else:
        alerts.append("Campo x_studio_fecha_demo no encontrado. Analisis de demos omitido.")

    # Inject win_rate_con_demo into pipeline_metrics for dashboard visibility
    pipeline_metrics["win_rate_con_demo"] = win_rate_con_demo

    # =========================================================================
    # 3. MONTHLY FORECAST (pipeline leads only) - show ALL, no limit
    # =========================================================================
    opp_this_month = []
    forecast_total = 0.0

    for l in leads_pipeline:
        dl = parse_date(l.get("date_deadline"))
        if dl and current_month_start <= dl < next_month_start:
            rev = l.get("expected_revenue") or 0
            prob = l.get("probability") or 0
            weighted = round2(rev * prob / 100)
            forecast_total += weighted
            opp_this_month.append({
                "name": l.get("name", ""),
                "user": get_name(l.get("user_id")) or "Sin asignar",
                "expected_revenue": rev,
                "probability": prob,
                "weighted_revenue": weighted,
                "stage": resolve_stage_name(l, stage_names),
            })

    opp_this_month.sort(key=lambda x: -x["weighted_revenue"])
    forecast_total = round2(forecast_total)

    coverage_pct = None
    if monthly_target and monthly_target > 0:
        coverage_pct = round1(forecast_total / monthly_target * 100)

    forecast = {
        "month": ref_date.strftime("%Y-%m"),
        "opportunities_count": len(opp_this_month),
        "forecast_weighted": forecast_total,
        "monthly_target": monthly_target,
        "coverage_pct": coverage_pct,
        "opportunities": opp_this_month,
    }

    # =========================================================================
    # 4. PIPELINE BY STAGE (active leads for visibility) + closing_this_month
    # =========================================================================
    by_stage = {}
    for l in leads_active:
        stage_id = get_id(l.get("stage_id"))
        sname = resolve_stage_name(l, stage_names)
        if sname not in by_stage:
            stage = stage_map.get(stage_id, {})
            category = "pipeline" if stage_id in pipeline_ids else (
                "early" if stage_id in early_ids else "other"
            )
            by_stage[sname] = {
                "name": sname,
                "stage_id": stage_id,
                "sequence": stage.get("sequence", 0),
                "category": category,
                "count": 0,
                "revenue": 0.0,
                "weighted_revenue": 0.0,
                "closing_this_month": 0,
            }
        by_stage[sname]["count"] += 1
        by_stage[sname]["revenue"] += l.get("expected_revenue") or 0
        prob = l.get("probability") or 0
        by_stage[sname]["weighted_revenue"] += (l.get("expected_revenue") or 0) * prob / 100

        # Count closing this month
        dl = parse_date(l.get("date_deadline"))
        if dl and current_month_start <= dl < next_month_start:
            by_stage[sname]["closing_this_month"] += 1

    stages_summary = sorted(by_stage.values(), key=lambda x: x["sequence"])
    for s in stages_summary:
        s["revenue"] = round2(s["revenue"])
        s["weighted_revenue"] = round2(s["weighted_revenue"])

    # =========================================================================
    # 5. GOVERNANCE ISSUES (pipeline leads only)
    # Removed: calificadas_sin_demo, duplicadas_partner
    # =========================================================================
    gov_sin_fecha = []
    gov_sin_revenue = []
    gov_sin_user = []
    gov_vencidas = overdue_leads
    gov_inactivas = []

    for l in leads_pipeline:
        name = l.get("name", "")
        user = get_name(l.get("user_id")) or "Sin asignar"
        stage = resolve_stage_name(l, stage_names)

        # Sin fecha cierre (only pipeline, not early)
        if not l.get("date_deadline"):
            gov_sin_fecha.append({"name": name, "user": user, "stage": stage})

        # Sin revenue (only pipeline)
        rev = l.get("expected_revenue") or 0
        if rev == 0:
            gov_sin_revenue.append({"name": name, "user": user, "stage": stage})

        # Sin comercial
        if not l.get("user_id") or l.get("user_id") is False:
            gov_sin_user.append({
                "name": name,
                "stage": stage,
                "expected_revenue": l.get("expected_revenue") or 0,
            })

        # Sin actividad 30d (using stale_field)
        ad = get_activity_date(l, stale_field)
        if ad and ad < inactive_cutoff:
            gov_inactivas.append({
                "name": name,
                "user": user,
                "stage": stage,
                "last_activity": str(ad),
                "days_inactive": (ref_date - ad).days,
            })

    gov_inactivas.sort(key=lambda x: -x["days_inactive"])

    governance = {
        "sin_fecha_cierre": {"count": len(gov_sin_fecha), "items": gov_sin_fecha[:15]},
        "sin_revenue": {"count": len(gov_sin_revenue), "items": gov_sin_revenue[:15]},
        "sin_comercial": {"count": len(gov_sin_user), "items": gov_sin_user[:10]},
        "vencidas_abiertas": {"count": len(gov_vencidas), "items": gov_vencidas[:15]},
        "sin_actividad_30d": {"count": len(gov_inactivas), "items": gov_inactivas[:15]},
    }

    # =========================================================================
    # 6. PENALTIES (based on pipeline leads)
    # Removed: pen_sin_demos (demo rate no penaliza)
    # =========================================================================

    # --- Rendimiento ---
    if win_rate_sufficient and win_rate < 30:
        pen_win_rate = round1(min(15, max(0, (30 - win_rate)) * 0.5))
    else:
        pen_win_rate = 0.0

    pen_stale = round1(min(20, stale_pct * 0.3))
    pen_overdue = round1(min(20, overdue_pct * 0.4))

    if coverage_pct is not None and coverage_pct < 100:
        pen_pipeline_bajo = round1(min(15, max(0, (100 - coverage_pct)) * 0.15))
    else:
        pen_pipeline_bajo = 0.0

    pen_rendimiento_total = round1(
        pen_win_rate + pen_stale + pen_overdue + pen_pipeline_bajo
    )

    # --- Gobernanza (pipeline leads only) ---
    pct_sin_fecha = round1(len(gov_sin_fecha) / total_pipeline * 100) if total_pipeline > 0 else 0.0
    pct_sin_revenue = round1(len(gov_sin_revenue) / total_pipeline * 100) if total_pipeline > 0 else 0.0

    pen_sin_fecha = round1(min(10, pct_sin_fecha * 0.15))
    pen_sin_revenue = round1(min(10, pct_sin_revenue * 0.15))
    pen_sin_comercial = round1(min(10, len(gov_sin_user) * 3))
    pen_sin_actividad = round1(min(10, inactive_pct * 0.15))

    pen_gobernanza_total = round1(
        pen_sin_fecha + pen_sin_revenue + pen_sin_comercial + pen_sin_actividad
    )

    penalties = {
        "rendimiento": {
            "win_rate_bajo": pen_win_rate,
            "pipeline_estancado": pen_stale,
            "opp_vencidas": pen_overdue,
            "pipeline_bajo": pen_pipeline_bajo,
            "total": pen_rendimiento_total,
        },
        "gobernanza": {
            "sin_fecha_cierre": pen_sin_fecha,
            "sin_revenue": pen_sin_revenue,
            "sin_comercial": pen_sin_comercial,
            "sin_actividad_30d": pen_sin_actividad,
            "total": pen_gobernanza_total,
        },
        "total": round1(pen_rendimiento_total + pen_gobernanza_total),
    }

    # =========================================================================
    # 7. HEALTH SCORE
    # =========================================================================
    if total_pipeline == 0:
        score = 0
        estado = "EMERGENCIA"
        icono = "\U0001f534"
        alerts.append("No hay oportunidades en etapas de pipeline.")
    else:
        raw_score = 100 - pen_rendimiento_total - pen_gobernanza_total
        score = clamp(round(raw_score), 0, 100)
        estado, icono = get_estado(score)

    if not win_rate_sufficient:
        alerts.append(
            f"Datos insuficientes para win rate ({total_all_opps} opp totales). Score parcial."
        )
    if not lost_available:
        alerts.append("No se encontraron oportunidades en etapa Congelado.")

    barra = build_health_bar(score)

    health_score = {
        "score": score,
        "estado": estado,
        "icono": icono,
        "barra": barra,
    }

    # =========================================================================
    # 8. SALESPERSON SCORING (pipeline leads only)
    # Removed: p_demos penalty (demo rate no penaliza)
    # Added: overdue_count, overdue_prev_month_count, closing_this_month
    # =========================================================================
    user_leads = {}   # uid -> list of pipeline leads
    user_won = {}
    user_lost = {}
    user_all_total = {}

    for l in leads_pipeline:
        uid = get_id(l.get("user_id"))
        if uid:
            user_leads.setdefault(uid, []).append(l)

    for l in leads_won:
        uid = get_id(l.get("user_id"))
        if uid:
            user_won.setdefault(uid, []).append(l)

    for l in leads_lost:
        uid = get_id(l.get("user_id"))
        if uid:
            user_lost.setdefault(uid, []).append(l)

    for l in leads_all:
        uid = get_id(l.get("user_id"))
        if uid:
            user_all_total.setdefault(uid, []).append(l)

    # Filter salespeople by minimum pipeline leads
    all_uids = set(
        uid for uid, leads in user_leads.items()
        if len(leads) >= min_pipeline_leads
    )
    num_salespeople = len(all_uids)
    pipeline_revenue_scored = sum(
        sum(l.get("expected_revenue") or 0 for l in user_leads[uid])
        for uid in all_uids
    )
    avg_pipeline = round2(pipeline_revenue_scored / num_salespeople) if num_salespeople > 0 else 0.0
    avg_win_rate = win_rate

    salesperson_scores = []

    for uid in sorted(all_uids):
        uname = user_map.get(uid, {}).get("name") or f"UID {uid}"
        my_leads = user_leads.get(uid, [])
        my_won = user_won.get(uid, [])
        my_lost = user_lost.get(uid, [])
        my_all_total = user_all_total.get(uid, [])
        my_count = len(my_leads)

        # Pipeline penalty
        my_pipeline = sum(l.get("expected_revenue") or 0 for l in my_leads)
        my_pipeline_weighted = sum(
            (l.get("expected_revenue") or 0) * (l.get("probability") or 0) / 100
            for l in my_leads
        )
        if avg_pipeline > 0:
            ratio = my_pipeline / avg_pipeline
            p_pipeline = round1(min(20, max(0, (1 - ratio)) * 20))
        else:
            p_pipeline = 0.0

        # Win rate penalty (formula: won 12m / total all opps)
        my_won_count = len(my_won)
        my_lost_count = len(my_lost)
        my_total_all_count = len(my_all_total)
        if my_total_all_count >= 3:
            my_wr = round1(my_won_count / my_total_all_count * 100)
            p_wr = round1(min(20, max(0, (avg_win_rate - my_wr)) * 0.5))
        else:
            my_wr = None
            p_wr = 5.0

        # Demo rate (informational only, no penalty)
        if has_demo_field and my_count > 0:
            my_with_demo = sum(
                1 for l in my_leads
                if l.get("x_studio_fecha_demo") and l.get("x_studio_fecha_demo") is not False
            )
            my_demo_pct = round1(my_with_demo / my_count * 100)
        else:
            my_demo_pct = None

        # Win rate con demo (won with demo / total with demo for this user)
        my_wr_con_demo = None
        if has_demo_field:
            my_won_with_demo = sum(
                1 for l in my_won
                if l.get("x_studio_fecha_demo") and l.get("x_studio_fecha_demo") is not False
            )
            my_total_with_demo = sum(
                1 for l in my_all_total
                if l.get("x_studio_fecha_demo") and l.get("x_studio_fecha_demo") is not False
            )
            if my_total_with_demo >= 3:
                my_wr_con_demo = round1(my_won_with_demo / my_total_with_demo * 100)
            elif my_total_with_demo > 0:
                my_wr_con_demo = round1(my_won_with_demo / my_total_with_demo * 100)

        # Governance penalty (pipeline leads)
        my_sin_fecha = sum(1 for l in my_leads if not l.get("date_deadline"))
        my_sin_rev = sum(1 for l in my_leads if not (l.get("expected_revenue") or 0))
        if my_count > 0:
            pf = my_sin_fecha / my_count * 100
            pr = my_sin_rev / my_count * 100
            p_gov = round1(min(15, (pf + pr) * 0.15))
        else:
            p_gov = 0.0

        # Stale penalty (per-stage thresholds)
        my_stale = 0
        for l in my_leads:
            sid = get_id(l.get("stage_id"))
            threshold = stale_thresholds.get(sid, stale_days)
            cutoff = ref_date - timedelta(days=threshold)
            ad = get_activity_date(l, stale_field)
            if ad and ad < cutoff:
                my_stale += 1
        my_stale_pct = round1(my_stale / my_count * 100) if my_count > 0 else 0.0
        p_stale = round1(min(15, my_stale_pct * 0.2))

        # Activity penalty (using stale_field)
        my_inactive = sum(
            1 for l in my_leads
            if (get_activity_date(l, stale_field) or ref_date) < inactive_cutoff
        )
        my_inact_pct = round1(my_inactive / my_count * 100) if my_count > 0 else 0.0
        p_activity = round1(min(15, my_inact_pct * 0.2))

        # Overdue counts
        my_overdue = 0
        my_overdue_prev = 0
        for l in my_leads:
            dl = parse_date(l.get("date_deadline"))
            if dl and dl < ref_date:
                my_overdue += 1
                if dl < current_month_start:
                    my_overdue_prev += 1

        # Closing this month
        my_closing_this_month = 0
        for l in my_leads:
            dl = parse_date(l.get("date_deadline"))
            if dl and current_month_start <= dl < next_month_start:
                my_closing_this_month += 1

        total_pen = round1(p_pipeline + p_wr + p_gov + p_stale + p_activity)
        my_score = clamp(round(100 - total_pen), 0, 100)
        my_estado, my_icono = get_estado(my_score)

        salesperson_scores.append({
            "user_id": uid,
            "name": uname,
            "score": my_score,
            "estado": my_estado,
            "icono": my_icono,
            "pipeline_leads": my_count,
            "pipeline_revenue": round2(my_pipeline),
            "pipeline_weighted": round2(my_pipeline_weighted),
            "won": my_won_count,
            "lost": my_lost_count,
            "win_rate": my_wr,
            "win_rate_con_demo": my_wr_con_demo,
            "demo_rate": my_demo_pct,
            "stale_pct": my_stale_pct,
            "inactive_pct": my_inact_pct,
            "overdue_count": my_overdue,
            "overdue_prev_month_count": my_overdue_prev,
            "closing_this_month": my_closing_this_month,
            "penalties": {
                "pipeline": p_pipeline,
                "win_rate": p_wr,
                "gobernanza": p_gov,
                "stale": p_stale,
                "actividad": p_activity,
                "total": total_pen,
            },
        })

    salesperson_scores.sort(key=lambda x: -x["score"])

    # Team balance
    if len(salesperson_scores) >= 3:
        score_max = max(s["score"] for s in salesperson_scores)
        score_min = min(s["score"] for s in salesperson_scores)
        score_spread = score_max - score_min
    elif salesperson_scores:
        score_max = salesperson_scores[0]["score"]
        score_min = salesperson_scores[-1]["score"]
        score_spread = score_max - score_min
    else:
        score_max = score_min = score_spread = 0

    # =========================================================================
    # 9. SCENARIO DETECTION (removed NO_DEMOS)
    # =========================================================================
    if total_pipeline < 10 or (coverage_pct is not None and coverage_pct < 50):
        scenario_code = "LOW_PIPELINE"
        scenario_desc = "Pipeline insuficiente para alcanzar objetivos comerciales"
    elif score >= 80:
        scenario_code = "HEALTHY_PIPELINE"
        scenario_desc = "CRM en buen estado, pipeline sano"
    elif stale_pct > 40:
        scenario_code = "STALE_PIPELINE"
        scenario_desc = "Pipeline inflado con oportunidades sin actividad reciente"
    elif win_rate_sufficient and win_rate < 15:
        scenario_code = "LOW_WIN_RATE"
        scenario_desc = "Tasa de cierre baja, se pierden demasiadas oportunidades"
    elif pen_gobernanza_total > 15:
        scenario_code = "GOVERNANCE_ISSUES"
        scenario_desc = "Datos del CRM incompletos, baja calidad de informacion"
    elif len(salesperson_scores) >= 3 and score_spread > 40:
        scenario_code = "UNBALANCED_TEAM"
        scenario_desc = "Gran disparidad de rendimiento entre comerciales"
    else:
        issues = []
        if pen_stale > 5:
            issues.append(("STALE_PIPELINE", pen_stale))
        if pen_win_rate > 5:
            issues.append(("LOW_WIN_RATE", pen_win_rate))
        if pen_gobernanza_total > 10:
            issues.append(("GOVERNANCE_ISSUES", pen_gobernanza_total))

        if issues:
            issues.sort(key=lambda x: -x[1])
            scenario_code = issues[0][0]
            descs = {
                "STALE_PIPELINE": "Pipeline con oportunidades estancadas",
                "LOW_WIN_RATE": "Tasa de cierre por debajo del esperado",
                "GOVERNANCE_ISSUES": "Problemas de calidad de datos",
            }
            scenario_desc = descs.get(scenario_code, "Multiples problemas detectados")
        else:
            scenario_code = "HEALTHY_PIPELINE"
            scenario_desc = "Sin desviaciones criticas detectadas"

    scenario = {
        "code": scenario_code,
        "description": scenario_desc,
    }

    # =========================================================================
    # 10. PIPELINE BY SALESPERSON (updated fields)
    # =========================================================================
    by_salesperson = []
    for sp in salesperson_scores:
        by_salesperson.append({
            "name": sp["name"],
            "pipeline_leads": sp["pipeline_leads"],
            "pipeline_revenue": sp["pipeline_revenue"],
            "pipeline_weighted": sp["pipeline_weighted"],
            "won": sp["won"],
            "lost": sp["lost"],
            "win_rate": sp["win_rate"],
            "win_rate_con_demo": sp["win_rate_con_demo"],
            "closing_this_month": sp["closing_this_month"],
        })

    # =========================================================================
    # 11. DASHBOARD EXTRA METRICS
    # =========================================================================

    # Closed orders this month / prev month (from sale.order, not crm.lead)
    closed_this_month = []
    closed_prev_month = []
    for so in sale_orders:
        do = parse_date(so.get("date_order"))
        if do:
            if current_month_start <= do < next_month_start:
                closed_this_month.append(so)
            elif prev_month_start <= do < current_month_start:
                closed_prev_month.append(so)

    closed_this_month_revenue = sum(so.get("amount_untaxed") or 0 for so in closed_this_month)
    closed_prev_month_revenue = sum(so.get("amount_untaxed") or 0 for so in closed_prev_month)

    # Per commercial
    closed_by_user_this_month = {}
    for so in closed_this_month:
        uid = get_id(so.get("user_id"))
        uname = get_name(so.get("user_id")) or f"UID {uid}"
        if uid:
            if uname not in closed_by_user_this_month:
                closed_by_user_this_month[uname] = {"count": 0, "revenue": 0.0}
            closed_by_user_this_month[uname]["count"] += 1
            closed_by_user_this_month[uname]["revenue"] += so.get("amount_untaxed") or 0

    closed_by_user_prev_month = {}
    for so in closed_prev_month:
        uid = get_id(so.get("user_id"))
        uname = get_name(so.get("user_id")) or f"UID {uid}"
        if uid:
            if uname not in closed_by_user_prev_month:
                closed_by_user_prev_month[uname] = {"count": 0, "revenue": 0.0}
            closed_by_user_prev_month[uname]["count"] += 1
            closed_by_user_prev_month[uname]["revenue"] += so.get("amount_untaxed") or 0

    # Round revenues
    for v in closed_by_user_this_month.values():
        v["revenue"] = round2(v["revenue"])
    for v in closed_by_user_prev_month.values():
        v["revenue"] = round2(v["revenue"])

    # New opps this month / prev month (from all leads)
    new_opps_this_month = 0
    new_opps_prev_month = 0
    for l in leads_all:
        cd = parse_date(l.get("create_date"))
        if cd:
            if current_month_start <= cd < next_month_start:
                new_opps_this_month += 1
            elif prev_month_start <= cd < current_month_start:
                new_opps_prev_month += 1

    # Assigned by user (current and prev month, based on create_date)
    assigned_by_user_this_month = {}
    assigned_by_user_prev_month = {}
    for l in leads_all:
        cd = parse_date(l.get("create_date"))
        uid = get_id(l.get("user_id"))
        uname = get_name(l.get("user_id")) or f"UID {uid}"
        if cd and uid:
            if current_month_start <= cd < next_month_start:
                assigned_by_user_this_month.setdefault(uname, 0)
                assigned_by_user_this_month[uname] += 1
            elif prev_month_start <= cd < current_month_start:
                assigned_by_user_prev_month.setdefault(uname, 0)
                assigned_by_user_prev_month[uname] += 1

    dashboard_extra = {
        "closed_orders_this_month": {
            "count": len(closed_this_month),
            "revenue": round2(closed_this_month_revenue),
            "by_user": closed_by_user_this_month,
        },
        "closed_orders_prev_month": {
            "count": len(closed_prev_month),
            "revenue": round2(closed_prev_month_revenue),
            "by_user": closed_by_user_prev_month,
        },
        "new_opps_this_month": new_opps_this_month,
        "new_opps_prev_month": new_opps_prev_month,
        "assigned_by_user_this_month": assigned_by_user_this_month,
        "assigned_by_user_prev_month": assigned_by_user_prev_month,
    }

    # =========================================================================
    # 12. VALIDATION
    # =========================================================================
    validation = {
        "has_demo_field": has_demo_field,
        "has_monthly_target": monthly_target is not None and monthly_target > 0,
        "lost_available": lost_available,
        "win_rate_sufficient": win_rate_sufficient,
        "stage_config": config,
        "warnings": warnings,
    }

    # =========================================================================
    # OUTPUT
    # =========================================================================
    return {
        "health_score": health_score,
        "scenario": scenario,
        "pipeline_metrics": pipeline_metrics,
        "penalties": penalties,
        "forecast": forecast,
        "demo_analysis": demo_analysis,
        "stages_summary": stages_summary,
        "by_salesperson": by_salesperson,
        "salesperson_scores": salesperson_scores,
        "team_balance": {
            "num_salespeople": num_salespeople,
            "score_max": score_max,
            "score_min": score_min,
            "score_spread": score_spread,
            "avg_pipeline": avg_pipeline,
        },
        "governance": governance,
        "dashboard_extra": dashboard_extra,
        "alerts": alerts,
        "validation": validation,
    }


def main():
    parser = argparse.ArgumentParser(
        description="Calcula metricas de salud del CRM de Odoo."
    )
    parser.add_argument(
        "--input", "-i",
        help="Fichero JSON de entrada (default: stdin)",
        default=None,
    )
    parser.add_argument(
        "--date", "-d",
        help="Fecha de referencia YYYY-MM-DD (default: hoy)",
        default=None,
    )
    parser.add_argument(
        "--target", "-t",
        help="Objetivo mensual de ventas (revenue)",
        type=float,
        default=None,
    )
    parser.add_argument(
        "--stale-days",
        help="Dias sin actividad por defecto para opp estancada - fallback si no hay umbral de etapa (default: 21)",
        type=int,
        default=21,
    )
    parser.add_argument(
        "--min-pipeline",
        help="Minimo de opp pipeline para incluir comercial en scoring (default: 3)",
        type=int,
        default=None,
    )
    parser.add_argument(
        "--config", "-c",
        help="Fichero JSON con configuracion de etapas (default: config Indaws)",
        default=None,
    )
    args = parser.parse_args()

    # Read input
    if args.input:
        with open(args.input, "r", encoding="utf-8") as f:
            data = json.load(f)
    else:
        data = json.load(sys.stdin)

    # Parse reference date
    ref_date = None
    if args.date:
        try:
            ref_date = datetime.strptime(args.date, "%Y-%m-%d").date()
        except ValueError:
            print(f"Error: fecha invalida: {args.date}", file=sys.stderr)
            sys.exit(1)

    # Load config
    config = None
    if args.config:
        with open(args.config, "r", encoding="utf-8") as f:
            config = json.load(f)

    # Override min_pipeline_leads from CLI
    if args.min_pipeline is not None:
        if config is None:
            config = dict(DEFAULT_CONFIG)
        config["min_pipeline_leads"] = args.min_pipeline

    result = calculate(data, ref_date, args.target, args.stale_days, config)
    json.dump(result, sys.stdout, ensure_ascii=False, indent=2)
    print()


if __name__ == "__main__":
    main()
