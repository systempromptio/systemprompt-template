# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - Data Processor
===================================
Decomposition of the monolithic process_data() into focused functions.
Each function handles a specific KPI domain.
"""

from collections import defaultdict
from datetime import datetime, timedelta

from . import config
from .utils import parse_date, truncate_text


# =============================================================================
# HELPERS (module-private)
# =============================================================================

def _is_won(stage_name):
    """Check if a stage name matches won keywords (case-insensitive)."""
    lower = stage_name.lower()
    return any(k.lower() in lower for k in config.STAGE_WON_KEYWORDS)


def _is_lost(stage_name):
    """Check if a stage name matches lost keywords (case-insensitive)."""
    lower = stage_name.lower()
    return any(k.lower() in lower for k in config.STAGE_LOST_KEYWORDS)


def _is_waiting(stage_name):
    lower = stage_name.lower()
    return any(k.lower() in lower for k in config.STAGE_WAITING_KEYWORDS)


def _is_proposal(stage_name):
    lower = stage_name.lower()
    return any(k.lower() in lower for k in config.STAGE_PROPOSAL_KEYWORDS)


def _is_early(stage_name):
    lower = stage_name.lower()
    return any(k.lower() in lower for k in config.STAGE_EARLY_KEYWORDS)


def _is_closed(stage_name):
    """Won or lost."""
    return _is_won(stage_name) or _is_lost(stage_name)


def _active_leads(enriched):
    """Return list of active (not won/lost, not archived) leads."""
    return [
        l for l in enriched
        if not _is_closed(l['stage'])
        and l.get('active')
    ]


# =============================================================================
# 1. ENRICH LEADS
# =============================================================================

def enrich_leads(leads, stages, sources, users, ref_date):
    """Enrich raw Odoo leads with computed fields and aggregate metrics.

    Returns:
        tuple: (enriched_list, sp_metrics, product_stats)
    """
    enriched = []
    sp_metrics = defaultdict(lambda: {'won': 0, 'lost': 0, 'active': 0, 'forecast': 0})
    product_stats = defaultdict(lambda: {'count': 0, 'revenue': 0})

    for lead in leads:
        stage_id = lead.get('stage_id')
        source_id = lead.get('source_id')
        user_id = lead.get('user_id')
        salesperson = users.get(user_id[0] if user_id else None, 'Sin asignar')
        lead_name = lead.get('name', 'Sin nombre')

        # 1. Filter salespeople
        if salesperson in config.EXCLUDED_SALESPEOPLE:
            continue
        if salesperson not in config.SALESPEOPLE_OBJECTIVES:
            continue

        # 2. Categorize product
        product_cat = "Otros"
        for cat, keywords in config.PRODUCT_CATEGORIES.items():
            if any(k in lead_name.lower() for k in keywords):
                product_cat = cat
                break

        # 3. Process dates and revenue
        revenue = lead.get('expected_revenue', 0) or 0
        prob = (lead.get('probability', 0) or 0) / 100
        create_date = parse_date(lead.get('create_date'))
        write_date = parse_date(lead.get('write_date'))
        date_closed = parse_date(lead.get('date_closed'))
        deadline = parse_date(lead.get('date_deadline'))

        stage_name = stages.get(stage_id[0] if stage_id else None, 'Sin etapa')

        # 4. Determine status (case-insensitive)
        stage_lower = stage_name.lower()
        is_won = any(k.lower() in stage_lower for k in config.STAGE_WON_KEYWORDS)
        is_lost = any(k.lower() in stage_lower for k in config.STAGE_LOST_KEYWORDS)
        is_waiting = any(k.lower() in stage_lower for k in config.STAGE_WAITING_KEYWORDS)
        is_proposal = any(k.lower() in stage_lower for k in config.STAGE_PROPOSAL_KEYWORDS)

        # Excluded stages
        is_frozen = "congelado" in stage_lower
        is_on_hold = "en espera" in stage_lower
        is_academy = "academy" in stage_lower
        is_collaborators = "colaboradores" in stage_lower
        is_analysis = ("en análisis" in stage_lower or "análisis" in stage_lower) and not is_won

        # Override probability for analysis stage
        if is_analysis:
            prob = 0.95

        # Activity status
        days_inactive = (ref_date - (write_date or create_date)).days
        alert_level = "ok"
        needs_proposal_alert = False

        is_ignored_for_alerts = (
            is_won or is_lost or is_frozen
            or is_collaborators or is_analysis
            or is_on_hold or is_academy
            or not lead.get('active')
        )

        if not is_ignored_for_alerts:
            if days_inactive > config.DAYS_CRITICAL:
                alert_level = "critical"
            elif days_inactive > config.DAYS_WARNING:
                alert_level = "warning"

            if is_proposal and days_inactive > config.PROPOSAL_FOLLOWUP_DAYS:
                needs_proposal_alert = True

        # 5. Aggregate metrics
        if is_won:
            sp_metrics[salesperson]['won'] += 1
            product_stats[product_cat]['revenue'] += revenue
        elif is_lost:
            sp_metrics[salesperson]['lost'] += 1
        else:
            sp_metrics[salesperson]['active'] += 1
            if deadline and deadline.month == ref_date.month and not is_frozen:
                weighted = revenue * prob
                sp_metrics[salesperson]['forecast'] += weighted
            product_stats[product_cat]['count'] += 1

        enriched.append({
            'name': lead_name,
            'partner': lead.get('partner_name', ''),
            'stage': stage_name,
            'source': sources.get(source_id[0] if source_id else None, 'Sin origen'),
            'salesperson': salesperson,
            'revenue': revenue,
            'probability': prob,
            'deadline': deadline,
            'created': create_date,
            'updated': write_date,
            'closed': date_closed,
            'alert': alert_level,
            'days_inactive': days_inactive,
            'product': product_cat,
            'is_proposal': is_proposal,
            'needs_proposal_alert': needs_proposal_alert,
            'is_excluded': (
                is_frozen or is_collaborators or is_analysis
                or is_on_hold or is_academy or not lead.get('active')
            ),
            'demo_realizada': lead.get('x_studio_demo_realizada'),
            'fecha_demo': parse_date(lead.get('x_studio_fecha_demo')),
            'active': lead.get('active'),
        })

    return enriched, sp_metrics, product_stats


# =============================================================================
# 2. TEMPORAL METRICS
# =============================================================================

def calculate_temporal_metrics(enriched, ref_date):
    """Calculate 30/60/90 day averages, weekly trends, and period comparisons.

    Returns:
        dict with keys: avg_30, avg_60, avg_90, weekly_trends, this_week,
             last_week, new_leads_list, month_comparison, quarter_comparison
    """
    def count_in_period(days):
        cutoff = ref_date - timedelta(days=days)
        return len([l for l in enriched if l['created'] and l['created'] >= cutoff])

    avg_30 = count_in_period(30)
    avg_60 = count_in_period(60)
    avg_90 = count_in_period(90)

    # 4-week trend
    week_start = (ref_date - timedelta(days=ref_date.weekday())).replace(
        hour=0, minute=0, second=0, microsecond=0
    )
    weekly_trends = []
    for i in range(4):
        w_start = week_start - timedelta(days=7 * i)
        w_end = w_start + timedelta(days=7)
        count = len([l for l in enriched if l['created'] and w_start <= l['created'] < w_end])
        weekly_trends.append({
            'week': f"S-{i}" if i > 0 else "Esta",
            'count': count,
            'start': w_start,
        })
    weekly_trends.reverse()  # oldest first

    # Weekly capture
    last_week_start = week_start - timedelta(days=7)
    this_week = len([l for l in enriched if l['created'] and l['created'] >= week_start])
    last_week = len([
        l for l in enriched
        if l['created'] and last_week_start <= l['created'] < week_start
    ])

    # New leads list (sorted by source, name)
    new_leads_list = [l for l in enriched if l['created'] and l['created'] >= week_start]
    new_leads_list.sort(key=lambda x: (x['source'], x['name']))

    # Month comparison
    current_month_start = ref_date.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
    prev_month_end = current_month_start - timedelta(days=1)
    prev_month_start = prev_month_end.replace(day=1)

    current_month_leads = [l for l in enriched if l['created'] and l['created'] >= current_month_start]
    prev_month_leads = [
        l for l in enriched
        if l['created'] and prev_month_start <= l['created'] < current_month_start
    ]

    month_comparison = {
        'current': len(current_month_leads),
        'previous': len(prev_month_leads),
        'delta': len(current_month_leads) - len(prev_month_leads),
    }

    # Quarter comparison
    current_quarter = (ref_date.month - 1) // 3
    quarter_start_month = current_quarter * 3 + 1
    current_quarter_start = ref_date.replace(
        month=quarter_start_month, day=1, hour=0, minute=0, second=0, microsecond=0
    )

    prev_quarter_end = current_quarter_start - timedelta(days=1)
    prev_quarter_start_month = ((prev_quarter_end.month - 1) // 3) * 3 + 1
    prev_quarter_start = prev_quarter_end.replace(month=prev_quarter_start_month, day=1)

    current_quarter_leads = [l for l in enriched if l['created'] and l['created'] >= current_quarter_start]
    prev_quarter_leads = [
        l for l in enriched
        if l['created'] and prev_quarter_start <= l['created'] < current_quarter_start
    ]

    quarter_comparison = {
        'current': len(current_quarter_leads),
        'previous': len(prev_quarter_leads),
        'delta': len(current_quarter_leads) - len(prev_quarter_leads),
    }

    return {
        'avg_30': avg_30,
        'avg_60': avg_60,
        'avg_90': avg_90,
        'weekly_trends': weekly_trends,
        'this_week': this_week,
        'last_week': last_week,
        'new_leads_list': new_leads_list,
        'month_comparison': month_comparison,
        'quarter_comparison': quarter_comparison,
    }


# =============================================================================
# 3. WON DEALS
# =============================================================================

def calculate_won_deals(enriched, ref_date):
    """Won deals this month (accumulated).

    Returns:
        tuple: (won_month_list, won_revenue)
    """
    current_month_start = ref_date.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
    won_month_list = []
    for l in enriched:
        if _is_won(l['stage']):
            closed_dt = l['closed'] or l['updated']
            if closed_dt and current_month_start <= closed_dt:
                won_month_list.append(l)
    won_revenue = sum(l['revenue'] for l in won_month_list)
    return won_month_list, won_revenue


# =============================================================================
# 4. SOURCE RANKING
# =============================================================================

def calculate_source_ranking(enriched):
    """Top 8 sources from active leads (not won/lost).

    Returns:
        list of (source_name, count) tuples, descending by count.
    """
    source_counts = defaultdict(int)
    for l in enriched:
        if not _is_closed(l['stage']):
            source_counts[l['source']] += 1
    return sorted(source_counts.items(), key=lambda x: -x[1])[:8]


# =============================================================================
# 5. FORECAST
# =============================================================================

def calculate_forecast(enriched, ref_date):
    """Current-month forecast with concentration risk.

    Returns:
        dict with keys: forecast_leads, forecast_total, top_deal,
             concentration_risk
    """
    forecast_leads = []
    for l in enriched:
        if (l['deadline']
                and l['deadline'].month == ref_date.month
                and l['deadline'].year == ref_date.year):
            if not _is_closed(l['stage']) and "congelado" not in l['stage'].lower():
                weighted = l['revenue'] * l['probability']
                forecast_leads.append({**l, 'weighted': weighted})
    forecast_leads.sort(key=lambda x: -x['weighted'])

    total_forecast = sum(l['weighted'] for l in forecast_leads)
    top_deal = forecast_leads[0] if forecast_leads else None
    concentration_risk = (
        (top_deal['weighted'] / total_forecast * 100)
        if top_deal and total_forecast > 0 else 0
    )

    return {
        'forecast_leads': forecast_leads,
        'forecast_total': total_forecast,
        'top_deal': top_deal,
        'concentration_risk': concentration_risk,
    }


# =============================================================================
# 6. DEMO CONVERSION
# =============================================================================

def calculate_demo_conversion(enriched, ref_date):
    """90-day demo conversion rate.

    Returns:
        tuple: (demo_candidates, global_conversion_val)
    """
    demo_window_start = ref_date - timedelta(days=90)
    demo_candidates = []

    for l in enriched:
        if not l.get('demo_realizada'):
            continue
        relevant_date = l.get('fecha_demo') or l.get('created')
        if relevant_date and relevant_date >= demo_window_start:
            demo_candidates.append(l)

    demo_won = len([l for l in demo_candidates if _is_won(l['stage'])])
    demo_total = len(demo_candidates)
    global_conversion_val = (demo_won / demo_total * 100) if demo_total > 0 else 0

    return demo_candidates, global_conversion_val


# =============================================================================
# 7. OBJECTIVES
# =============================================================================

def calculate_objectives(enriched, sp_metrics, forecast_leads, demo_candidates,
                         sales_orders_map, ref_date):
    """Per-salesperson objectives and KPI table data.

    Returns:
        list of dicts, one per salesperson.
    """
    objectives_data = []
    for sp, target in config.SALESPEOPLE_OBJECTIVES.items():
        metrics = sp_metrics[sp]

        pipeline_forecast = metrics['forecast']
        actual_sales = sales_orders_map[sp]['amount']

        total_forecast = pipeline_forecast + actual_sales
        gap = total_forecast - target

        # Demo-based conversion (90d)
        sp_demo_candidates = [l for l in demo_candidates if l['salesperson'] == sp]
        sp_demo_won = [l for l in sp_demo_candidates if _is_won(l['stage'])]
        sp_demo_won_count = len(sp_demo_won)
        sp_demo_total = len(sp_demo_candidates)

        conversion = (sp_demo_won_count / sp_demo_total * 100) if sp_demo_total > 0 else 0

        # Top 3 opportunities (forecast_leads already sorted desc)
        sp_opps = [l for l in forecast_leads if l['salesperson'] == sp]
        top_opps = sp_opps[:3]

        objectives_data.append({
            'name': sp,
            'target': target,
            'forecast': total_forecast,
            'pipeline_forecast': pipeline_forecast,
            'actual_sales': actual_sales,
            'gap': gap,
            'achievement': (total_forecast / target * 100) if target > 0 else 0,
            'conversion': conversion,
            'active_deals': metrics['active'],
            'top_opportunities': top_opps,
            'forecast_count': len(sp_opps),
        })

    return objectives_data


# =============================================================================
# 8. ACTIVITY
# =============================================================================

def calculate_activity(enriched, ref_date):
    """7-day activity counts per salesperson.

    Returns:
        tuple: (activity_list, activity_counts_dict)
            activity_list: [(sp_name, count), ...] for all configured salespeople
            activity_counts_dict: {sp_name: count}
    """
    seven_days = ref_date - timedelta(days=7)
    activity_counts = defaultdict(int)
    for l in enriched:
        if l['updated'] and l['updated'] >= seven_days:
            activity_counts[l['salesperson']] += 1

    activity_list = [(sp, activity_counts[sp]) for sp in config.SALESPEOPLE_OBJECTIVES]
    return activity_list, dict(activity_counts)


# =============================================================================
# 9. ALERTS
# =============================================================================

def calculate_alerts(enriched):
    """Zombies, warnings, and proposal alerts.

    Returns:
        tuple: (zombies, warnings, proposal_alerts)
    """
    zombies = [l for l in enriched if l['alert'] == 'critical']
    warnings = [l for l in enriched if l['alert'] == 'warning']
    proposal_alerts = [l for l in enriched if l['needs_proposal_alert']]

    zombies.sort(key=lambda x: -x['days_inactive'])
    proposal_alerts.sort(key=lambda x: -x['days_inactive'])

    return zombies, warnings, proposal_alerts


# =============================================================================
# 10. AGING
# =============================================================================

def calculate_aging(enriched):
    """Opportunity aging analysis.

    Returns:
        dict with keys: age_buckets, stage_avg_age, yellow_zone,
             salesperson_avg_age
    """
    age_buckets = {'0-7d': 0, '8-14d': 0, '15-30d': 0, '31-60d': 0, '60+d': 0}
    stage_ages = defaultdict(list)
    yellow_zone = []
    salesperson_ages = defaultdict(list)

    for l in enriched:
        if _is_closed(l['stage']) or l.get('is_excluded'):
            continue

        age_days = l['days_inactive']

        # Bucket distribution
        if age_days <= 7:
            age_buckets['0-7d'] += 1
        elif age_days <= 14:
            age_buckets['8-14d'] += 1
        elif age_days <= 30:
            age_buckets['15-30d'] += 1
        elif age_days <= 60:
            age_buckets['31-60d'] += 1
        else:
            age_buckets['60+d'] += 1

        stage_ages[l['stage']].append(age_days)

        # Yellow zone (approaching zombie)
        if 5 <= age_days <= 6:
            yellow_zone.append(l)

        salesperson_ages[l['salesperson']].append(age_days)

    # Average age by stage (top 10)
    stage_avg_age = {
        stage: sum(ages) / len(ages) if ages else 0
        for stage, ages in stage_ages.items()
    }
    stage_avg_age_sorted = sorted(stage_avg_age.items(), key=lambda x: -x[1])[:10]

    # Average age by salesperson
    salesperson_avg_age = {
        sp: sum(ages) / len(ages) if ages else 0
        for sp, ages in salesperson_ages.items()
    }

    return {
        'age_buckets': age_buckets,
        'stage_avg_age': stage_avg_age_sorted,
        'yellow_zone': yellow_zone[:15],
        'salesperson_avg_age': salesperson_avg_age,
    }


# =============================================================================
# 11. VELOCITY
# =============================================================================

def calculate_velocity(enriched, ref_date):
    """Sales velocity metrics.

    Returns:
        dict with keys: avg_cycle_time, sp_avg_velocity, stage_duration
    """
    # Won deals in last 90 days
    cutoff_90d = ref_date - timedelta(days=90)
    won_deals_90d = [
        l for l in enriched
        if _is_won(l['stage'])
        and l['closed'] and l['closed'] >= cutoff_90d
    ]

    # Average cycle time (created -> closed)
    cycle_times = []
    for deal in won_deals_90d:
        if deal['created'] and deal['closed']:
            cycle_time = (deal['closed'] - deal['created']).days
            cycle_times.append(cycle_time)
            deal['cycle_time'] = cycle_time

    avg_cycle_time = sum(cycle_times) / len(cycle_times) if cycle_times else 0

    # Velocity by salesperson
    sp_velocity = defaultdict(list)
    for deal in won_deals_90d:
        if 'cycle_time' in deal:
            sp_velocity[deal['salesperson']].append(deal['cycle_time'])

    sp_avg_velocity = {
        sp: sum(times) / len(times) if times else 0
        for sp, times in sp_velocity.items()
    }

    # Days in current stage (active deals)
    stage_duration = []
    for l in enriched:
        if not (_is_closed(l['stage']) or l.get('is_excluded')):
            stage_duration.append({
                'name': l['name'],
                'stage': l['stage'],
                'days_in_stage': l['days_inactive'],
                'salesperson': l['salesperson'],
                'revenue': l['revenue'],
            })
    stage_duration.sort(key=lambda x: -x['days_in_stage'])

    return {
        'avg_cycle_time': avg_cycle_time,
        'sp_avg_velocity': sp_avg_velocity,
        'stage_duration': stage_duration[:20],
    }


# =============================================================================
# 12. PROPOSAL PERFORMANCE
# =============================================================================

def calculate_proposal_performance(enriched, ref_date):
    """Proposal conversion and timing metrics.

    Returns:
        dict with keys: proposal_conversion_rate, avg_proposal_decision_time,
             sp_proposal_stats, proposals_active_count
    """
    proposals_active = [
        l for l in enriched
        if l['is_proposal'] and not _is_closed(l['stage'])
    ]

    # Historical proposals (won or lost, last 90 days)
    cutoff_90d = ref_date - timedelta(days=90)
    proposals_historical = []
    for l in enriched:
        stage_lower = l['stage'].lower()
        name_lower = l['name'].lower()
        is_proposal_stage = any(k.lower() in stage_lower for k in config.STAGE_PROPOSAL_KEYWORDS)
        has_proposal_in_name = any(
            k in name_lower for k in ['propuesta', 'proposal', 'cotización', 'quote']
        )

        is_won = _is_won(l['stage'])
        is_lost = _is_lost(l['stage'])

        if is_proposal_stage or ((is_won or is_lost) and has_proposal_in_name):
            closed_date = l['closed'] or l['updated']
            if closed_date and closed_date >= cutoff_90d:
                proposals_historical.append(l)

    # Conversion metrics
    proposals_won = len([p for p in proposals_historical if _is_won(p['stage'])])
    proposals_lost = len([p for p in proposals_historical if _is_lost(p['stage'])])
    proposals_total_decided = proposals_won + proposals_lost

    proposal_conversion_rate = (
        (proposals_won / proposals_total_decided * 100)
        if proposals_total_decided > 0 else 0
    )

    # Average time from proposal to decision
    proposal_decision_times = []
    for p in proposals_historical:
        is_decided = _is_closed(p['stage'])
        if is_decided and p['closed'] and p['created']:
            decision_time = (p['closed'] - p['created']).days
            proposal_decision_times.append(decision_time)

    avg_proposal_decision_time = (
        sum(proposal_decision_times) / len(proposal_decision_times)
        if proposal_decision_times else 0
    )

    # Proposals by salesperson
    sp_proposals = defaultdict(lambda: {'active': 0, 'won': 0, 'lost': 0})
    for p in proposals_historical:
        if p['is_proposal'] and not _is_closed(p['stage']):
            sp_proposals[p['salesperson']]['active'] += 1
        elif _is_won(p['stage']):
            sp_proposals[p['salesperson']]['won'] += 1
        elif _is_lost(p['stage']):
            sp_proposals[p['salesperson']]['lost'] += 1

    sp_proposal_stats = []
    for sp in config.SALESPEOPLE_OBJECTIVES:
        stats = sp_proposals[sp]
        total = stats['won'] + stats['lost']
        conversion = (stats['won'] / total * 100) if total > 0 else 0
        sp_proposal_stats.append({
            'name': sp,
            'active': stats['active'],
            'won': stats['won'],
            'lost': stats['lost'],
            'conversion': conversion,
        })

    return {
        'proposal_conversion_rate': proposal_conversion_rate,
        'avg_proposal_decision_time': avg_proposal_decision_time,
        'sp_proposal_stats': sp_proposal_stats,
        'proposals_active_count': len(proposals_active),
        'proposals_won': proposals_won,
        'proposals_lost': proposals_lost,
    }


# =============================================================================
# 13. AT-RISK OPPORTUNITIES
# =============================================================================

def calculate_at_risk(enriched, ref_date):
    """Identify at-risk opportunities by multiple criteria.

    Returns:
        list of enriched leads with added 'risk_reasons' key, limit 15.
    """
    at_risk = []
    for l in enriched:
        if _is_closed(l['stage']) or l.get('is_excluded'):
            continue

        risk_reasons = []

        # High-value deal with no activity >3 days
        if l['revenue'] > config.HIGH_VALUE_THRESHOLD and l['days_inactive'] > 3:
            risk_reasons.append(
                f"Alto valor sin actividad ({l['days_inactive']}d)"
            )

        # Approaching month-end deadline with low probability
        if l['deadline']:
            days_to_deadline = (l['deadline'] - ref_date).days
            if 0 < days_to_deadline <= 7 and l['probability'] < 0.5:
                risk_reasons.append(
                    f"Cierre en {days_to_deadline}d, prob {l['probability'] * 100:.0f}%"
                )

        # Stage mismatch: high value but early stage
        if (l['revenue'] > config.HIGH_VALUE_THRESHOLD
                and _is_early(l['stage'])
                and l['days_inactive'] > 14):
            risk_reasons.append(
                f"Alto valor en etapa temprana ({l['stage']})"
            )

        if risk_reasons:
            at_risk.append({**l, 'risk_reasons': risk_reasons})

    at_risk.sort(key=lambda x: -x['revenue'])
    return at_risk[:15]


# =============================================================================
# 14. DEMO METRICS
# =============================================================================

def calculate_demo_metrics(enriched, ref_date):
    """Monthly demo counts by salesperson and source.

    Returns:
        dict with keys: target, current, by_salesperson, by_source
    """
    current_month_start = datetime(ref_date.year, ref_date.month, 1)
    demos_count = 0
    demos_by_sp = defaultdict(int)
    demos_by_source = defaultdict(int)

    for l in enriched:
        if l.get('fecha_demo'):
            demo_date = l['fecha_demo']
            if demo_date >= current_month_start:
                demos_count += 1
                demo_user = l.get('demo_realizada')
                sp_name = demo_user[1] if demo_user else l['salesperson']
                demos_by_sp[sp_name] += 1
                demos_by_source[l['source']] += 1

    return {
        'target': config.DEMOS_MONTHLY_TARGET,
        'current': demos_count,
        'by_salesperson': dict(demos_by_sp),
        'by_source': dict(demos_by_source),
    }


# =============================================================================
# 15. EXECUTIVE INSIGHTS
# =============================================================================

def calculate_executive_insights(active_leads, weekly_trends, ref_date):
    """High-level metrics, risks, and projections for the executive summary.

    Returns:
        dict with keys: status, risks, momentum, projections
    """
    insights = {
        'status': 'NEUTRAL',
        'risks': [],
        'momentum': 'STABLE',
        'projections': {},
    }

    # 1. Objective forecasting
    target_weekly = config.WEEKLY_LEAD_TARGET
    run_rate_4w = sum(w['count'] for w in weekly_trends) / 4 if weekly_trends else 0

    insights['projections'] = {
        'run_rate_4w': run_rate_4w,
        'target_weekly': target_weekly,
        'gap': run_rate_4w - target_weekly,
    }

    if run_rate_4w >= target_weekly:
        insights['status'] = 'ON TRACK'
    elif run_rate_4w >= target_weekly * 0.8:
        insights['status'] = 'WARNING'
    else:
        insights['status'] = 'AT RISK'

    # 2. Risk analysis

    # A. Concentration risk
    total_revenue = sum(l.get('revenue', 0) for l in active_leads)
    sorted_by_revenue = sorted(active_leads, key=lambda x: x.get('revenue', 0), reverse=True)
    top_3_revenue = sum(l.get('revenue', 0) for l in sorted_by_revenue[:3])

    if total_revenue > 0:
        concentration_ratio = top_3_revenue / total_revenue
        if concentration_ratio > 0.5:
            insights['risks'].append(
                f"Concentracion Alta: Top 3 oportunidades representan "
                f"{concentration_ratio * 100:.0f}% del pipeline."
            )

    # B. Momentum
    current_week_count = weekly_trends[-1]['count'] if weekly_trends else 0
    if run_rate_4w > 0:
        momentum_ratio = (current_week_count - run_rate_4w) / run_rate_4w
        if momentum_ratio > 0.2:
            insights['momentum'] = 'ACCELERATING'
        elif momentum_ratio < -0.2:
            insights['momentum'] = 'DECELERATING'
            insights['risks'].append(
                f"Desaceleracion: Generacion de leads cayendo un "
                f"{abs(momentum_ratio) * 100:.0f}% vs media 4s."
            )

    # C. Data quality risk
    incomplete_leads = 0
    for l in active_leads:
        if not l.get('deadline') or not l.get('revenue') or l.get('revenue') <= 0:
            incomplete_leads += 1

    if len(active_leads) > 0:
        dq_ratio = incomplete_leads / len(active_leads)
        if dq_ratio > 0.2:
            insights['risks'].append(
                f"Calidad del Dato: {dq_ratio * 100:.0f}% de oportunidades "
                f"activas con datos incompletos."
            )

    return insights


# =============================================================================
# 16. DATA QUALITY ISSUES
# =============================================================================

def get_data_quality_issues(active_leads):
    """Analyze active leads for missing critical data, grouped by Salesperson -> Stage.

    Returns:
        dict: { salesperson: { 'total': int, 'stages': { stage: [{'name', 'problems'}] } } }
    """
    issues = {}
    excluded_keywords = [
        "congelado", "lead", "cualificado-asignado",
        "1er contacto", "en espera", "academy",
    ]

    for l in active_leads:
        if any(k in l['stage'].lower() for k in excluded_keywords) or l.get('is_excluded'):
            continue

        problems = []
        if not l.get('deadline'):
            problems.append("Sin fecha")
        if not l.get('revenue') or l.get('revenue') <= 0:
            problems.append("Sin ingresos")
        if l.get('probability') is None:
            problems.append("Sin probabilidad")

        if problems:
            sp = l.get('salesperson', "Sin Asignar")
            if sp not in issues:
                issues[sp] = {'total': 0, 'stages': defaultdict(list)}

            issues[sp]['total'] += 1
            issues[sp]['stages'][l['stage']].append({
                'name': l['name'],
                'problems': ", ".join(problems),
            })

    return issues


# =============================================================================
# 17. PIPELINE (helper used by process_all)
# =============================================================================

def _calculate_pipeline(enriched):
    """Pipeline by salesperson (for stacked bars).

    Returns:
        dict: { salesperson: { stage: count } }
    """
    pipeline = defaultdict(lambda: defaultdict(int))
    for l in enriched:
        if not _is_closed(l['stage']):
            pipeline[l['salesperson']][l['stage']] += 1
    return {sp: dict(pipeline[sp]) for sp in config.SALESPEOPLE_OBJECTIVES}


# =============================================================================
# 18. QUALITY ISSUES (raw list, for the quality_issues key)
# =============================================================================

def _calculate_quality_issues_list(enriched):
    """Raw quality issues list (not grouped by SP)."""
    quality_issues = []
    for l in enriched:
        if _is_won(l['stage']) or _is_lost(l['stage']) or _is_early(l['stage']):
            continue
        issues = []
        if not l['revenue'] or l['revenue'] == 0:
            issues.append('Sin ingresos')
        if not l['deadline']:
            issues.append('Sin fecha cierre')
        if issues:
            quality_issues.append({**l, 'issues': issues})
    return quality_issues


# =============================================================================
# ORCHESTRATOR
# =============================================================================

def process_all(leads, stages, sources, users, ref_date, orders=None):
    """Orchestrate all data processing and return a single result dict.

    This produces the SAME dict structure as the old process_data() function.
    """
    # Process sales orders
    sales_orders_map = defaultdict(lambda: {'count': 0, 'amount': 0.0})
    if orders:
        for order in orders:
            user_id = order.get('user_id')
            sp_name = users.get(user_id[0] if user_id else None, 'Sin asignar')
            sales_orders_map[sp_name]['count'] += 1
            sales_orders_map[sp_name]['amount'] += order.get('amount_untaxed', 0.0)

    # 1. Enrich leads
    enriched, sp_metrics, product_stats = enrich_leads(
        leads, stages, sources, users, ref_date
    )

    # 2. Temporal metrics
    temporal = calculate_temporal_metrics(enriched, ref_date)

    # 3. Won deals
    won_month_list, won_revenue = calculate_won_deals(enriched, ref_date)

    # 4. Source ranking
    top_sources = calculate_source_ranking(enriched)

    # 5. Forecast
    forecast_data = calculate_forecast(enriched, ref_date)
    forecast_leads = forecast_data['forecast_leads']

    # 6. Demo conversion
    demo_candidates, global_conversion_val = calculate_demo_conversion(enriched, ref_date)

    # 7. Objectives
    objectives_data = calculate_objectives(
        enriched, sp_metrics, forecast_leads, demo_candidates,
        sales_orders_map, ref_date,
    )

    # 8. Activity
    activity_list, activity_counts = calculate_activity(enriched, ref_date)

    # 9. Alerts
    zombies, warnings, proposal_alerts = calculate_alerts(enriched)

    # 10. Aging
    aging = calculate_aging(enriched)

    # 11. Velocity
    velocity = calculate_velocity(enriched, ref_date)

    # 12. Proposal performance
    proposals = calculate_proposal_performance(enriched, ref_date)

    # 13. At-risk
    at_risk = calculate_at_risk(enriched, ref_date)

    # 14. Demo metrics
    demos_data = calculate_demo_metrics(enriched, ref_date)

    # Active leads (strict filter)
    active_leads = [
        l for l in enriched
        if l['alert'] != 'old_closed'
        and not _is_closed(l['stage'])
        and l.get('active')
    ]
    active_leads_count = len(active_leads)

    # 15. Executive insights
    executive_summary = calculate_executive_insights(
        active_leads, temporal['weekly_trends'], ref_date
    )

    # 16. Data quality
    data_quality = get_data_quality_issues(active_leads)

    # Pipeline (stacked bars)
    pipeline = _calculate_pipeline(enriched)

    # Quality issues (raw list)
    quality_issues = _calculate_quality_issues_list(enriched)

    # Run rate
    weekly_trends = temporal['weekly_trends']
    run_rate_4w = sum(w['count'] for w in weekly_trends) / 4 if weekly_trends else 0

    return {
        'global_conversion': global_conversion_val,
        'demos_data': demos_data,
        'enriched': enriched,
        'total_active': active_leads_count,
        'this_week': temporal['this_week'],
        'last_week': temporal['last_week'],
        'won_last_month': len(won_month_list),
        'won_revenue': won_revenue,
        'avg_30': temporal['avg_30'],
        'avg_60': temporal['avg_60'],
        'avg_90': temporal['avg_90'],
        'weekly_trends': weekly_trends,
        'run_rate_4w': run_rate_4w,
        'month_comparison': temporal['month_comparison'],
        'quarter_comparison': temporal['quarter_comparison'],
        'executive_summary': executive_summary,
        'data_quality': data_quality,
        'objectives': objectives_data,
        'activity': activity_list,
        'activity_counts': activity_counts,
        'sources': top_sources,
        'pipeline': pipeline,
        'products': dict(product_stats),
        'forecast_leads': forecast_leads[:15],
        'forecast_total': forecast_data['forecast_total'],
        'top_deal': forecast_data['top_deal'],
        'concentration_risk': forecast_data['concentration_risk'],
        'zombies': zombies[:15],
        'proposal_alerts': proposal_alerts,
        'quality_issues': quality_issues[:15],
        'zombie_count': len(zombies),
        'warning_count': len(warnings),
        'quality_count': len(quality_issues),
        'new_leads_list': temporal['new_leads_list'],
        'won_deals_list': won_month_list,
        'total_objective': sum(config.SALESPEOPLE_OBJECTIVES.values()),
        'age_buckets': aging['age_buckets'],
        'stage_avg_age': aging['stage_avg_age'],
        'yellow_zone': aging['yellow_zone'],
        'salesperson_avg_age': aging['salesperson_avg_age'],
        'avg_cycle_time': velocity['avg_cycle_time'],
        'sp_avg_velocity': velocity['sp_avg_velocity'],
        'stage_duration': velocity['stage_duration'],
        'proposals_active_count': proposals['proposals_active_count'],
        'proposals_won': proposals['proposals_won'],
        'proposals_lost': proposals['proposals_lost'],
        'proposal_conversion_rate': proposals['proposal_conversion_rate'],
        'avg_proposal_decision_time': proposals['avg_proposal_decision_time'],
        'sp_proposal_stats': proposals['sp_proposal_stats'],
        'at_risk': at_risk,
        'sales_orders': dict(sales_orders_map),
        'detailed_orders': orders or [],
    }
