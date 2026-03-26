# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - HTML Report Generator
==========================================
Generates brand-compliant HTML reports for team and individual salespeople.
Uses Bogle font, INDAWS brand palette, and zero emojis.
"""

from datetime import datetime

from . import config
from .utils import truncate_text


# =============================================================================
# CSS STYLES
# =============================================================================

def _get_css_styles():
    """Return the full CSS stylesheet for the report.

    Uses Bogle font (brand requirement), INDAWS color palette,
    and semantic alert colors for data visualization.
    """
    return f"""
    <style>
        @import url('{config.FONT_IMPORT_URL}');

        body {{
            font-family: '{config.FONT_FAMILY}', {config.FONT_FALLBACK};
            background-color: #F8FAFC;
            color: #1E293B;
            margin: 0;
            padding: 20px;
        }}

        .header {{
            text-align: center;
            margin-bottom: 30px;
        }}
        .header h1 {{
            color: {config.BRAND_COLORS['blue_space']};
            margin: 0;
            font-weight: 800;
            font-size: 1.8rem;
            letter-spacing: -0.5px;
        }}
        .header p {{
            color: #64748B;
            margin: 5px 0 0 0;
            font-size: 0.9rem;
        }}

        .container {{
            max-width: 1000px;
            margin: 0 auto;
        }}

        .card {{
            background: white;
            border-radius: 12px;
            padding: 25px;
            margin-bottom: 25px;
            box-shadow: 0 4px 6px -1px rgba(0,0,0,0.05), 0 2px 4px -1px rgba(0,0,0,0.03);
        }}
        .card.highlight {{
            border-top: 5px solid {config.BRAND_COLORS['blue_lilac']};
        }}

        h3 {{
            margin-top: 0;
            color: #1E293B;
            font-size: 1.1rem;
            border-bottom: 2px solid #F1F5F9;
            padding-bottom: 10px;
            margin-bottom: 20px;
            display: flex;
            justify-content: space-between;
            align-items: center;
        }}

        table {{
            width: 100%;
            border-collapse: separate;
            border-spacing: 0;
        }}
        th {{
            background: #F8FAFC;
            color: #64748B;
            font-weight: 600;
            text-transform: uppercase;
            font-size: 0.7rem;
            padding: 12px 10px;
            border-bottom: 2px solid #E2E8F0;
            text-align: left;
        }}
        td {{
            padding: 12px 10px;
            border-bottom: 1px solid #F1F5F9;
            font-size: 0.85rem;
            vertical-align: middle;
        }}
        tr:last-child td {{
            border-bottom: none;
        }}

        .num {{
            text-align: right;
        }}
        .center {{
            text-align: center;
        }}

        .success {{
            color: {config.ALERT_COLORS['success']};
        }}
        .warning {{
            color: {config.ALERT_COLORS['warning']};
        }}
        .danger {{
            color: {config.ALERT_COLORS['danger']};
        }}

        .badge {{
            display: inline-block;
            padding: 2px 8px;
            border-radius: 12px;
            font-size: 0.7rem;
            font-weight: 700;
        }}
        .badge-green {{
            background: #DCFCE7;
            color: #166534;
        }}
        .badge-red {{
            background: #FEE2E2;
            color: #991B1B;
        }}
        .badge-yellow {{
            background: #FEF3C7;
            color: #92400E;
        }}
        .badge-neutral {{
            background: #F1F5F9;
            color: #475569;
        }}

        .metrics-grid {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 15px;
            margin-bottom: 30px;
        }}
        .metric-card {{
            background: white;
            padding: 20px;
            border-radius: 12px;
            text-align: center;
            box-shadow: 0 1px 3px rgba(0,0,0,0.05);
            border-bottom: 3px solid transparent;
        }}
        .metric-val {{
            font-size: 2rem;
            font-weight: 800;
            color: {config.BRAND_COLORS['blue_space']};
            line-height: 1;
            margin-bottom: 5px;
        }}
        .metric-label {{
            font-size: 0.8rem;
            color: #64748B;
            font-weight: 600;
            text-transform: uppercase;
            letter-spacing: 0.5px;
        }}

        .activity-row {{
            display: flex;
            align-items: center;
            gap: 10px;
            margin-bottom: 6px;
        }}
        .activity-row .sp-name {{
            width: 140px;
            font-size: 0.8rem;
            font-weight: 600;
        }}
        .activity-row .bar-container {{
            flex: 1;
            height: 18px;
            background: #E2E8F0;
            border-radius: 4px;
            overflow: hidden;
        }}
        .activity-row .bar {{
            height: 100%;
            border-radius: 4px;
        }}
        .activity-row .count {{
            width: 30px;
            font-weight: 700;
            text-align: right;
            font-size: 0.85rem;
        }}

        .sp-name {{
            font-weight: 600;
            color: #1E293B;
        }}
    </style>
    """


# =============================================================================
# EXECUTIVE SUMMARY
# =============================================================================

def _build_executive_summary(data, ref_date):
    """Build the executive summary card with status, projections, alerts, and weekly flow."""
    exec_data = data.get('executive_summary', {})

    status_colors = {
        'ON TRACK': config.ALERT_COLORS['success'],
        'WARNING': config.ALERT_COLORS['warning'],
        'AT RISK': config.ALERT_COLORS['danger'],
        'NEUTRAL': config.ALERT_COLORS['neutral'],
    }
    status_bg = {
        'ON TRACK': '#DCFCE7',
        'WARNING': '#FEF3C7',
        'AT RISK': '#FEE2E2',
        'NEUTRAL': '#F1F5F9',
    }

    current_status = exec_data.get('status', 'NEUTRAL')
    status_color = status_colors.get(current_status, '#64748B')
    status_bg_color = status_bg.get(current_status, '#F1F5F9')

    # Risks list
    risks_html = ""
    if exec_data.get('risks'):
        risks_html = '<ul style="margin:5px 0 0 0; padding-left:20px; color:#7F1D1D;">'
        for risk in exec_data['risks']:
            risks_html += f'<li style="margin-bottom:2px;">{risk}</li>'
        risks_html += '</ul>'
    else:
        risks_html = '<div style="color:#166534; font-size:0.85rem; margin-top:5px;">[OK] Sin riesgos criticos detectados.</div>'

    # Momentum text (no emojis)
    momentum_map = {
        'ACCELERATING': 'Acelerando',
        'DECELERATING': 'Desacelerando',
        'STABLE': 'Estable',
    }
    momentum_text = momentum_map.get(exec_data.get('momentum', 'STABLE'), 'Estable')

    # Projections data
    projections = exec_data.get('projections', {})
    run_rate = projections.get('run_rate_4w', 0)
    target_weekly = projections.get('target_weekly', 0)
    proj_gap = projections.get('gap', 0)
    gap_color = config.ALERT_COLORS['danger'] if proj_gap < 0 else config.ALERT_COLORS['success']

    # Weekly flow: new leads
    new_leads_list = data.get('new_leads_list', [])
    if new_leads_list:
        new_leads_html = (
            '<ul style="margin:0;padding-left:15px;font-size:0.8rem;color:#4B5563;">'
            + ''.join(
                f"<li>{truncate_text(l['name'], 25)}</li>"
                for l in new_leads_list[:5]
            )
            + '</ul>'
        )
    else:
        new_leads_html = '<div style="font-size:0.8rem;color:#9CA3AF;font-style:italic;">Sin nuevos leads</div>'

    # Weekly flow: won deals
    won_deals_list = data.get('won_deals_list', [])
    if won_deals_list:
        won_deals_html = (
            '<ul style="margin:0;padding-left:15px;font-size:0.8rem;color:#4B5563;">'
            + ''.join(
                f"<li>{truncate_text(l['name'], 25)} ({_fmt_eur(l['revenue'])})</li>"
                for l in won_deals_list
            )
            + '</ul>'
        )
    else:
        won_deals_html = '<div style="font-size:0.8rem;color:#9CA3AF;font-style:italic;">Sin cierres</div>'

    return f'''
    <div class="card" style="border-left: 5px solid {status_color}; background: linear-gradient(to right, {status_bg_color}33, #ffffff);">
        <div style="display:flex; justify-content:space-between; align-items:center;">
            <h2 style="margin:0; color:#1E293B; font-size:1.1rem;">Resumen Ejecutivo</h2>
            <span style="background:{status_color}; color:white; padding:2px 8px; border-radius:12px; font-size:0.7rem; font-weight:bold;">{current_status}</span>
        </div>

        <div style="display:grid; grid-template-columns: 1fr 1fr 1fr; gap:20px; margin-top:15px;">
            <!-- Column 1: Projections -->
            <div>
                <div style="font-size:0.75rem; font-weight:700; color:#64748B; text-transform:uppercase; margin-bottom:5px;">Prediccion de Objetivos</div>

                <div style="margin-bottom:8px;">
                    <div style="font-size:0.8rem; color:#64748B;">Generacion de Oportunidades (Run Rate)</div>
                    <div style="font-size:1.1rem; color:#334155; font-weight:700;">
                        {run_rate:.1f} <span style="font-size:0.8rem; font-weight:400; color:#64748B;">nuevas/sem</span>
                    </div>
                    <div style="font-size:0.75rem; color:{gap_color};">
                        Objetivo: {target_weekly} (Gap: {proj_gap:.1f})
                    </div>
                    <div style="font-size:0.7rem; color:#94A3B8; margin-top:2px;">*Basado en media ultimas 4 semanas</div>
                </div>

                <div style="font-size:0.8rem; color:#64748B;">
                    Momentum (Tendencia): <strong>{momentum_text}</strong>
                </div>
            </div>

            <!-- Column 2: Alerts -->
            <div style="border-left:1px solid #E2E8F0; padding-left:15px;">
                <div style="font-size:0.75rem; font-weight:700; color:#64748B; text-transform:uppercase;">Alertas y Riesgos Detectados</div>
                <div style="font-size:0.85rem;">
                    {risks_html}
                </div>
            </div>

            <!-- Column 3: Weekly Flow -->
            <div style="border-left:1px solid #E2E8F0; padding-left:15px;">
                <div style="font-size:0.75rem; font-weight:700; color:#64748B; text-transform:uppercase; margin-bottom:5px;">Flujo Semanal</div>

                <div style="margin-bottom:8px;">
                    <div style="font-size:0.75rem; font-weight:700; color:{config.BRAND_COLORS['blue_lilac']};">NUEVOS LEADS ({len(new_leads_list)})</div>
                    {new_leads_html}
                </div>

                <div>
                     <div style="font-size:0.75rem; font-weight:700; color:{config.ALERT_COLORS['success']};">GANADOS (MES) ({_fmt_eur(data.get('won_revenue', 0))})</div>
                     {won_deals_html}
                </div>
            </div>
        </div>
    </div>
    '''


# =============================================================================
# KPI GRID
# =============================================================================

def _build_kpi_grid(data, ref_date):
    """Build the 6-metric KPI card grid at the top of the report."""
    total_target = data.get('total_objective', 0)
    total_forecast = sum(o['forecast'] for o in data['objectives'])
    global_achievement = (total_forecast / total_target * 100) if total_target > 0 else 0

    # Count all action items
    action_count = (
        len(data.get('at_risk', []))
        + len(data.get('proposal_alerts', []))
        + min(len(data.get('zombies', [])), 10)
    )

    return f'''
    <div class="metrics-grid">
        <div class="metric-card" style="border-color: {config.BRAND_COLORS['blue_lilac']};">
            <div class="metric-val">{global_achievement:.0f}%</div>
            <div class="metric-label">Objetivo Mensual</div>
        </div>
        <div class="metric-card" style="border-color: {config.ALERT_COLORS['warning']};">
            <div class="metric-val">{_fmt_eur(total_forecast)}</div>
            <div class="metric-label">Forecast ({ref_date.strftime('%B')})</div>
        </div>
        <div class="metric-card" style="border-color: #EC4899;">
            <div class="metric-val">{data['global_conversion']:.1f}%</div>
            <div class="metric-label">Conversion Global</div>
        </div>
        <div class="metric-card" style="border-color: {config.ALERT_COLORS['success']};">
            <div class="metric-val">{data['demos_data']['current']} <span style="font-size:1rem; color:#9CA3AF;">/ {data['demos_data']['target']}</span></div>
            <div class="metric-label">Demos Realizadas (Mes)</div>
        </div>
        <div class="metric-card" style="border-color: #3b82f6;">
            <div class="metric-val">{data['avg_cycle_time']:.0f}d</div>
            <div class="metric-label">Ciclo de Venta</div>
        </div>
        <div class="metric-card" style="border-color: {config.ALERT_COLORS['danger']};">
            <div class="metric-val" style="color:{config.ALERT_COLORS['danger']};">{action_count}</div>
            <div class="metric-label">Acciones Prioritarias</div>
        </div>
    </div>
    '''


# =============================================================================
# UNIFIED SALES PERFORMANCE TABLE
# =============================================================================

def _build_sales_performance_table(data, ref_date):
    """Build the unified sales performance table with all KPIs per salesperson."""
    prop_stats_map = {item['name']: item for item in data.get('sp_proposal_stats', [])}

    rows_html = ""
    total_target = 0
    total_forecast = 0

    for sp_name in config.SALESPEOPLE_OBJECTIVES:
        target = config.SALESPEOPLE_OBJECTIVES.get(sp_name, 0)
        total_target += target

        # Forecast from objectives data
        obj_data = next((o for o in data['objectives'] if o['name'] == sp_name), {})
        weighted_val = obj_data.get('forecast', 0)
        total_forecast += weighted_val
        gap = weighted_val - target
        achievement = (weighted_val / target * 100) if target > 0 else 0

        # Color classes
        color_class = "success" if achievement >= 100 else "warning" if achievement >= 80 else "danger"
        text_color = config.ALERT_COLORS['success'] if achievement >= 100 else config.ALERT_COLORS['warning'] if achievement >= 80 else config.ALERT_COLORS['danger']

        # Activity (7d)
        activity_count = data.get('activity_counts', {}).get(sp_name, 0)
        act_bar_width = min((activity_count / 20) * 100, 100)

        # Demos
        demos_count = data['demos_data']['by_salesperson'].get(sp_name, 0)

        # Proposals
        prop_stats = prop_stats_map.get(sp_name, {'active': 0})
        prop_text = f"{prop_stats['active']} Activas"

        # Conversion
        conversion = obj_data.get('conversion', 0)
        conv_color = config.ALERT_COLORS['success'] if conversion >= 20 else config.ALERT_COLORS['warning'] if conversion >= 10 else config.ALERT_COLORS['danger']

        # Velocity / Cycle
        velocity = data.get('sp_avg_velocity', {}).get(sp_name, 0)
        vel_color = config.ALERT_COLORS['success'] if velocity < 30 else config.ALERT_COLORS['warning'] if velocity < 60 else config.ALERT_COLORS['danger']

        # Aging
        age_avg = data.get('salesperson_avg_age', {}).get(sp_name, 0)
        age_color = config.ALERT_COLORS['success'] if age_avg < 14 else config.ALERT_COLORS['warning'] if age_avg < 30 else config.ALERT_COLORS['danger']

        # Sales orders
        sp_sales = data.get('sales_orders', {}).get(sp_name, {})
        sales_amount = sp_sales.get('amount', 0)
        sales_count = sp_sales.get('count', 0)

        rows_html += f"""
        <tr>
            <td style="font-weight:600; color:#1E293B;">{sp_name}</td>
            <td class="num">{_fmt_eur(target)}</td>
            <td class="num" style="background-color: #F8FAFC;">
                <div style="font-weight:700;">{_fmt_eur(weighted_val)}</div>
                <div style="font-size:0.75rem; color: {'green' if gap >= 0 else 'red'};">Gap: {_fmt_eur(gap)}</div>
                <div style="font-size:0.7rem; color: #64748B;">({obj_data.get('forecast_count', 0)} oportunidades)</div>
            </td>
            <td class="num">
                <div style="background: #E2E8F0; width: 60px; height: 6px; border-radius: 3px; display:inline-block; margin-right:5px;">
                    <div style="width: {min(achievement, 100)}%; height: 100%; background: {text_color}; border-radius: 3px;"></div>
                </div>
                <span style="color:{text_color}; font-weight:700;">{achievement:.0f}%</span>
            </td>
            <td style="vertical-align: middle;">
                <div style="display:flex; align-items:center; gap:8px;">
                     <span style="font-weight:700; width:20px;">{activity_count}</span>
                     <div style="flex:1; height:4px; background:#E2E8F0; border-radius:2px; max-width:50px;">
                        <div style="width:{act_bar_width}%; height:100%; background:{config.BRAND_COLORS['blue_lilac']}; border-radius:2px;"></div>
                     </div>
                </div>
            </td>
            <td class="center" style="font-weight:700; color:{conv_color};">{conversion:.0f}%</td>
            <td class="center" style="font-weight:700; color:{vel_color};">{velocity:.0f} d</td>
            <td class="center" style="font-weight:700; color:{config.BRAND_COLORS['blue_space']};">{demos_count}</td>
            <td class="center">
                <div style="font-weight:700; color:{config.BRAND_COLORS['blue_space']};">{_fmt_eur(sales_amount)}</div>
                <div style="font-size:0.7rem; color:#64748B;">{sales_count} pedidos</div>
            </td>
            <td class="center" style="line-height:1.2;">{prop_text}</td>
            <td class="center" style="font-weight:700; color:{age_color};">{age_avg:.0f} d</td>
        </tr>
        """

    # Total row
    global_achievement = (total_forecast / total_target * 100) if total_target > 0 else 0
    global_cls = "success" if global_achievement >= 100 else "warning" if global_achievement >= 80 else "danger"
    total_demos = data['demos_data']['current']
    total_sales_amount = sum(o.get('amount_untaxed', 0) for o in data.get('detailed_orders', []))
    total_sales_count = len(data.get('detailed_orders', []))

    rows_html += f'''
    <tr style="background-color: #f8fafc; font-weight: bold; border-top: 2px solid #e2e8f0;">
        <td>TOTAL</td>
        <td class="num">{_fmt_eur(total_target)}</td>
        <td class="num">{_fmt_eur(total_forecast)}</td>
        <td class="num {global_cls}">{global_achievement:.1f}%</td>
        <td class="center">-</td>
        <td class="center">{data['global_conversion']:.0f}%</td>
        <td class="center">{data['avg_cycle_time']:.0f} d</td>
        <td class="center">{total_demos}</td>
        <td class="center" style="color:{config.BRAND_COLORS['blue_space']};">
            <div>{_fmt_eur(total_sales_amount)}</div>
            <div style="font-size:0.7rem; font-weight:400;">{total_sales_count}</div>
        </td>
        <td class="center">-</td>
        <td class="center">-</td>
    </tr>
    '''

    month_label = ref_date.strftime('%B').capitalize()
    return f'''
    <div class="card highlight">
        <h3>Rendimiento de Equipo (Mes Actual) -- Datos CRM + Ventas</h3>
        <table>
            <thead>
                <tr>
                    <th width="15%">Comercial</th>
                    <th width="10%" class="num">Objetivo</th>
                    <th width="15%" class="num">Forecast (Gap)</th>
                    <th width="15%" class="num">% Cump.</th>
                    <th width="15%">Actividad (7d)</th>
                    <th width="8%" class="center" title="% Ganados vs Totales">Conv. %</th>
                    <th width="8%" class="center" title="Ciclo medio de venta">Ciclo (d)</th>
                    <th width="8%" class="center">Demos (Mes)</th>
                    <th width="10%" class="center" title="Pedidos Confirmados (Mes)">Ventas ({month_label})</th>
                    <th width="10%" class="center" title="Activas / Conv. Rate">Propuestas</th>
                    <th width="10%" class="center">Aging (d)</th>
                </tr>
            </thead>
            <tbody>
                {rows_html}
            </tbody>
        </table>

        <div style="padding: 10px 15px; background: #F8FAFC; border-top: 1px solid #E2E8F0; border-radius: 0 0 12px 12px; font-size: 0.75rem; color: #64748B; display: flex; justify-content: flex-end; align-items: center; gap: 15px;">
             <span><strong>Leyenda:</strong></span>
             <span><strong>Forecast:</strong> Ponderado (Valor x Probabilidad)</span>
             <span><strong>Conv. %:</strong> Ganados / Demos Totales (90d)</span>
             <span><strong>Ciclo:</strong> Dias hasta cierre (Ganados, 90d)</span>
             <span><strong>Demos:</strong> Realizadas este mes</span>
        </div>
    </div>
    '''


# =============================================================================
# SALES DETAIL (Orders by Salesperson)
# =============================================================================

def _build_sales_detail(data, ref_date):
    """Build the detailed sales orders section grouped by salesperson."""
    if not data.get('detailed_orders'):
        return '''
        <div class="card">
            <h3>Detalle de Ventas -- Pedidos Confirmados</h3>
            <div style="padding:20px; text-align:center; color:#94A3B8;">No hay ventas confirmadas este mes.</div>
        </div>
        '''

    # Group orders by salesperson
    orders_by_sp = {}
    for o in data['detailed_orders']:
        user_id = o.get('user_id')
        sp_name = user_id[1] if user_id else 'Sin Asignar'
        orders_by_sp.setdefault(sp_name, []).append(o)

    # Sort salespeople by total amount descending
    sorted_sps = sorted(
        orders_by_sp.keys(),
        key=lambda sp: sum(x['amount_untaxed'] for x in orders_by_sp[sp]),
        reverse=True,
    )

    detail_html = ""
    for sp in sorted_sps:
        sp_orders = sorted(orders_by_sp[sp], key=lambda x: x['date_order'], reverse=True)
        total_sp = sum(x['amount_untaxed'] for x in sp_orders)

        rows = ""
        for order in sp_orders:
            date_str = order['date_order'][:10]
            try:
                dt = datetime.strptime(date_str, '%Y-%m-%d')
                date_fmt = dt.strftime('%d/%m')
            except (ValueError, TypeError):
                date_fmt = date_str

            partner = order.get('partner_id')
            partner_name = partner[1] if isinstance(partner, list) and len(partner) > 1 else 'Cliente Desconocido'

            rows += f'''
                <tr>
                    <td style="color:#64748B;">{date_fmt}</td>
                    <td>
                        <div style="font-weight:600; color:#334155;">{partner_name}</div>
                        <div style="font-size:0.75rem; color:#94A3B8;">{order['name']}</div>
                    </td>
                    <td class="num" style="font-weight:700; color:#1E293B;">{_fmt_eur(order['amount_untaxed'])}</td>
                </tr>
            '''

        detail_html += f'''
        <div style="margin-bottom: 20px;">
            <div style="display:flex; justify-content:space-between; align-items:center; background:#F1F5F9; padding:10px 15px; border-radius:8px 8px 0 0; border:1px solid #E2E8F0;">
                <h4 style="margin:0; color:#1E293B; font-size:0.95rem;">{sp}</h4>
                <span style="font-weight:700; color:{config.ALERT_COLORS['success']};">{_fmt_eur(total_sp)}</span>
            </div>
            <table style="width:100%; font-size:0.85rem; border:1px solid #E2E8F0; border-top:none; border-radius:0 0 8px 8px;">
                <tr style="background:white;">
                    <th style="width:15%; background:white; color:#94A3B8; font-weight:400; font-size:0.75rem;">Fecha</th>
                    <th style="width:40%; background:white; color:#94A3B8; font-weight:400; font-size:0.75rem;">Cliente / Pedido</th>
                    <th style="width:25%; background:white; color:#94A3B8; font-weight:400; font-size:0.75rem; text-align:right;">Importe</th>
                </tr>
                {rows}
            </table>
        </div>
        '''

    total_all = sum(o.get('amount_untaxed', 0) for o in data.get('detailed_orders', []))
    month_label = ref_date.strftime('%B').capitalize()

    return f'''
    <div class="card">
         <h3>
            <span>Detalle de Ventas ({month_label}) -- Pedidos Confirmados</span>
            <span style="font-size:1rem; background:#DCFCE7; color:#166534; padding:4px 12px; border-radius:20px;">
                Total: {_fmt_eur(total_all)}
            </span>
         </h3>
         {detail_html}
    </div>
    '''


# =============================================================================
# CONSOLIDATED ACTION ITEMS
# =============================================================================

def _build_action_items(data):
    """Build the consolidated action items table sorted by priority: risk > proposal > zombie."""
    all_actions = []

    # At-risk deals (highest priority)
    for r in data.get('at_risk', []):
        reason = r.get('risk_reasons', ['Riesgo'])[0] if isinstance(r.get('risk_reasons'), list) else 'Riesgo'
        all_actions.append({
            'type': 'risk',
            'label': 'CRITICO',
            'color': config.ALERT_COLORS['danger'],
            'name': r['name'],
            'sp': r['salesperson'],
            'val': r.get('stage', ''),
            'revenue': r.get('revenue', 0),
            'probability': r.get('probability', 0),
        })

    # Proposal alerts (medium priority)
    for p in data.get('proposal_alerts', []):
        all_actions.append({
            'type': 'proposal',
            'label': 'ALERTA',
            'color': '#D97706',
            'name': p['name'],
            'sp': p['salesperson'],
            'val': f"{p['days_inactive']}d",
            'revenue': p.get('revenue', 0),
            'probability': p.get('probability', 0),
        })

    # Zombies (lower priority, limit to 10)
    for z in data.get('zombies', [])[:10]:
        all_actions.append({
            'type': 'zombie',
            'label': 'ZOMBIE',
            'color': config.ALERT_COLORS['neutral'],
            'name': z['name'],
            'sp': z['salesperson'],
            'val': f"{z['days_inactive']}d",
            'revenue': z.get('revenue', 0),
            'probability': z.get('probability', 0),
        })

    # Sort: risk > proposal > zombie
    priority_map = {'risk': 0, 'proposal': 1, 'zombie': 2}
    all_actions.sort(key=lambda x: priority_map.get(x['type'], 99))

    action_rows = ""
    if all_actions:
        for item in all_actions[:20]:
            prob_pct = item['probability'] * 100 if item['probability'] <= 1 else item['probability']
            action_rows += f"""
             <tr style="border-left: 3px solid {item['color']};">
                <td><span class="badge" style="background:{item['color']}22; color:{item['color']};">{item['label']}</span></td>
                <td style="font-weight:600;">{truncate_text(item['name'], 45)}</td>
                <td>{item['sp']}</td>
                <td class="num">{_fmt_eur(item['revenue'])}</td>
                <td class="center">{prob_pct:.0f}%</td>
                <td class="center" style="color: #4B5563;">{item['val']}</td>
             </tr>
             """
    else:
        action_rows = '<tr><td colspan="6" class="center" style="padding:20px; color:green;">[OK] Todo limpio. Buen trabajo.</td></tr>'

    return f'''
    <div class="card">
        <h3>Acciones Requeridas (Top Prioridad)</h3>
        <table style="font-size:0.8rem;">
            <thead>
                <tr>
                    <th width="15%">Tipo</th>
                    <th width="35%">Oportunidad</th>
                    <th width="15%">Resp.</th>
                    <th width="15%" class="num">Importe</th>
                    <th width="10%" class="center">Prob.</th>
                    <th width="10%" class="center">Estado</th>
                </tr>
            </thead>
            <tbody>
                {action_rows}
            </tbody>
        </table>
    </div>
    '''


# =============================================================================
# DATA QUALITY
# =============================================================================

def _build_data_quality(data):
    """Build the data quality section with horizontally scrollable cards per salesperson."""
    dq_data = data.get('data_quality', {})

    html = '<div class="card" style="margin-top:20px; overflow:hidden;">'
    html += '<h3>Calidad del Dato: Detalle por Etapa</h3>'
    html += '<p style="font-size:0.8rem; color:#64748B; margin-bottom:15px;">Desglose de oportunidades activas incompletas (excluyendo leads, congeladas y perdidas). Prioridad: Altas.</p>'

    if not dq_data:
        html += '<p style="color:#10B981;">[OK] Todo correcto. No hay incidencias de calidad detectadas.</p>'
    else:
        for sp, sp_data in dq_data.items():
            incident_word = "incidencia" if sp_data['total'] == 1 else "incidencias"
            html += f'''
            <div style="display:flex; flex-direction:column; margin-bottom:25px; border-bottom:1px solid #F1F5F9; padding-bottom:15px;">
                <div style="display:flex; align-items:center; gap:10px; margin-bottom:10px;">
                    <h4 style="margin:0; color:#1E293B; font-size:1rem;">{sp}</h4>
                    <span style="background:#FEE2E2; color:#B91C1C; padding:2px 8px; border-radius:12px; font-size:0.75rem; font-weight:bold;">{sp_data['total']} {incident_word}</span>
                </div>

                <div style="display:flex; gap:15px; overflow-x:auto; padding-bottom:10px; -webkit-overflow-scrolling: touch;">
            '''

            sorted_stages = sorted(sp_data['stages'].items(), key=lambda x: -len(x[1]))
            for stage, issues in sorted_stages:
                issues_count = len(issues)
                top_issues = issues[:5]

                html += f'''
                <div style="min-width:280px; max-width:300px; background:#F8FAFC; padding:12px; border-radius:8px; border:1px solid #E2E8F0; flex-shrink:0;">
                    <div style="font-weight:700; color:#475569; font-size:0.85rem; margin-bottom:8px; border-bottom:1px solid #E2E8F0; padding-bottom:5px; display:flex; justify-content:space-between;">
                        <span>{stage}</span>
                        <span style="color:#64748B; font-weight:400;">({issues_count})</span>
                    </div>
                    <ul style="margin:0; padding-left:15px; font-size:0.75rem;">
                '''

                for i in top_issues:
                    prob_text = f" <span style='color:{config.ALERT_COLORS['warning']};'>({i['problems']})</span>"
                    name_display = truncate_text(i['name'], 35)
                    html += f'<li style="color:#64748B; margin-bottom:4px; line-height:1.3;">{name_display}{prob_text}</li>'

                if issues_count > 5:
                    html += f'<li style="color:#94A3B8; font-style:italic; list-style:none; margin-top:5px; font-size:0.7rem;">... y {issues_count - 5} oportunidades mas</li>'

                html += '</ul></div>'

            html += '</div></div>'

    html += '</div>'
    return html


# =============================================================================
# FOOTER
# =============================================================================

def _build_footer(ref_date):
    """Build the report footer with version and generation timestamp."""
    now = datetime.now()
    return f'''
    <div style="text-align:center; font-size:0.75rem; color:#94A3B8; margin-top:30px;">
        INDAWS CRM Intelligence v5.0 -- Generado el {now.strftime('%d/%m/%Y %H:%M')}
    </div>
    '''


# =============================================================================
# MAIN REPORT GENERATION
# =============================================================================

def generate_team_report(data, ref_date):
    """Generate the full team HTML report.

    Combines all sections into a complete HTML document with brand-compliant
    styling: Bogle font, INDAWS color palette, zero emojis.

    Args:
        data: Enriched data dict from data_processor.
        ref_date: Reference datetime for the report period.

    Returns:
        Complete HTML string ready to send or save.
    """
    css = _get_css_styles()
    exec_summary = _build_executive_summary(data, ref_date)
    kpi_grid = _build_kpi_grid(data, ref_date)
    sales_table = _build_sales_performance_table(data, ref_date)
    sales_detail = _build_sales_detail(data, ref_date)
    action_items = _build_action_items(data)
    data_quality = _build_data_quality(data)
    footer = _build_footer(ref_date)

    return f'''<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    {css}
</head>
<body>

    <div class="header">
        <h1>Control Comercial Semanal</h1>
        <p>Semana del {ref_date.strftime('%d/%m')} -- Datos en tiempo real</p>
    </div>

    <div class="container">

        {exec_summary}

        {kpi_grid}

        {sales_table}

        {sales_detail}

        {action_items}

        {data_quality}

        {footer}

    </div>
</body>
</html>'''


# =============================================================================
# PERSONALIZED EMAIL REPORT
# =============================================================================

def generate_personalized_report(sp_metrics, ref_date):
    """Generate a personalized HTML email for a single salesperson.

    Args:
        sp_metrics: Dict with individual salesperson metrics
            (from email_service.calculate_salesperson_metrics).
        ref_date: Reference datetime for the report period.

    Returns:
        Complete HTML string for the personalized email.
    """
    name = sp_metrics['name']
    first_name = name.split()[0]
    achievement = sp_metrics['achievement']
    target = sp_metrics['target']
    forecast = sp_metrics['forecast']
    gap = sp_metrics['gap']

    # Performance indicator -- text badges, no colored circles
    if achievement >= 100:
        perf_color = "#10B981"
        perf_badge = '<span style="background:#DCFCE7; color:#166534; padding:3px 10px; border-radius:12px; font-size:0.8rem; font-weight:700;">EN OBJETIVO</span>'
        perf_text = "En objetivo"
    elif achievement >= 80:
        perf_color = config.ALERT_COLORS['warning']
        perf_badge = '<span style="background:#FEF3C7; color:#92400E; padding:3px 10px; border-radius:12px; font-size:0.8rem; font-weight:700;">CERCA DEL OBJETIVO</span>'
        perf_text = "Cerca del objetivo"
    else:
        perf_color = "#EF4444"
        perf_badge = '<span style="background:#FEE2E2; color:#991B1B; padding:3px 10px; border-radius:12px; font-size:0.8rem; font-weight:700;">POR DEBAJO</span>'
        perf_text = "Por debajo del objetivo"

    # Motivational message (dynamic)
    if achievement < 100:
        motivation = f"""
        <div style="background:#FEF2F2;border-left:4px solid #EF4444;padding:15px;margin:20px 0;">
            <strong style="color:#991B1B;">ATENCION: Necesitas {_fmt_eur(abs(gap))} mas para alcanzar tu objetivo</strong>
            <p style="margin:10px 0 0 0;color:#7F1D1D;">
                Prioriza tus zombies y propuestas sin seguimiento. Cada dia cuenta.
            </p>
        </div>
        """
    else:
        risks = []
        if sp_metrics.get('concentration', 0) > 50:
            risks.append(f"ALERTA: <strong>Concentracion alta:</strong> Tu top deal representa el {sp_metrics['concentration']:.0f}% del forecast")
        if sp_metrics.get('recent_leads_count', 0) < 5:
            risks.append(f"ALERTA: <strong>Pipeline futuro debil:</strong> Solo {sp_metrics['recent_leads_count']} leads nuevos en 30 dias")
        if sp_metrics.get('stalled_count', 0) > 0:
            risks.append(f"ALERTA: <strong>Deals estancados:</strong> {sp_metrics['stalled_count']} oportunidades sin actividad >14 dias")

        if risks:
            risks_html = "<br>".join(risks)
            motivation = f"""
            <div style="background:#ECFDF5;border-left:4px solid #10B981;padding:15px;margin:20px 0;">
                <strong style="color:#065F46;">[OK] Vas por buen camino.</strong>
                <p style="margin:10px 0 0 0;color:#064E3B;">
                    Pero no te relajes. Aqui hay algunos riesgos que debes vigilar:
                </p>
                <p style="margin:10px 0 0 0;color:#065F46;font-size:0.9rem;">
                    {risks_html}
                </p>
            </div>
            """
        else:
            motivation = f"""
            <div style="background:#ECFDF5;border-left:4px solid #10B981;padding:15px;margin:20px 0;">
                <strong style="color:#065F46;">EXCELENTE: Trabajo sobresaliente.</strong>
                <p style="margin:10px 0 0 0;color:#064E3B;">
                    Estas superando tu objetivo y tu pipeline esta saludable. Sigue asi.
                </p>
            </div>
            """

    # Top 5 actions -- text labels instead of emoji icons
    actions = []

    # Priority 1: Critical zombies
    for z in sp_metrics.get('zombies', [])[:3]:
        actions.append({
            'label': 'CRITICO:',
            'label_color': config.ALERT_COLORS['danger'],
            'lead': z['name'],
            'value': z.get('revenue', 0),
            'reason': f"Zombie critico ({z.get('days_inactive', 0)} dias sin actividad)",
            'action': 'Contactar HOY',
        })

    # Priority 2: Proposal alerts
    for p in sp_metrics.get('proposals', [])[:2]:
        if len(actions) >= 5:
            break
        actions.append({
            'label': 'ALERTA:',
            'label_color': '#D97706',
            'lead': p['name'],
            'value': p.get('revenue', 0),
            'reason': f"Propuesta sin seguimiento ({p.get('days_inactive', 0)} dias)",
            'action': 'Hacer seguimiento inmediato',
        })

    # Priority 3: High-value warnings (5-6 days)
    for w in sp_metrics.get('warnings', [])[:2]:
        if len(actions) >= 5:
            break
        if w.get('revenue', 0) > 5000:
            actions.append({
                'label': 'ZONA AMARILLA:',
                'label_color': config.ALERT_COLORS['warning'],
                'lead': w['name'],
                'value': w.get('revenue', 0),
                'reason': f"Zona amarilla ({w.get('days_inactive', 0)} dias)",
                'action': 'Actualizar antes de que se vuelva zombie',
            })

    # Priority 4: Stalled high-value deals
    for s in sp_metrics.get('stalled_deals', []):
        if len(actions) >= 5:
            break
        if s.get('revenue', 0) > 10000:
            actions.append({
                'label': 'ESTANCADO:',
                'label_color': config.ALERT_COLORS['neutral'],
                'lead': s['name'],
                'value': s.get('revenue', 0),
                'reason': f"Estancado {s.get('days_inactive', 0)} dias en '{s.get('stage', '')}'",
                'action': 'Revisar estrategia',
            })

    # Build actions HTML table rows
    actions_html = ""
    for act in actions[:5]:
        actions_html += f"""
        <tr>
            <td style="padding:8px;border-bottom:1px solid #E5E7EB;font-weight:700;color:{act['label_color']};white-space:nowrap;">{act['label']}</td>
            <td style="padding:8px;border-bottom:1px solid #E5E7EB;"><strong>{truncate_text(act['lead'], 40)}</strong></td>
            <td style="padding:8px;border-bottom:1px solid #E5E7EB;text-align:right;">{_fmt_eur(act['value'])}</td>
            <td style="padding:8px;border-bottom:1px solid #E5E7EB;font-size:0.85rem;color:#6B7280;">{act['reason']}</td>
            <td style="padding:8px;border-bottom:1px solid #E5E7EB;font-size:0.85rem;color:#10B981;"><strong>{act['action']}</strong></td>
        </tr>
        """

    if not actions:
        actions_html = """
        <tr>
            <td colspan="5" style="padding:20px;text-align:center;color:#6B7280;">
                [OK] No tienes acciones criticas pendientes. Excelente gestion del pipeline.
            </td>
        </tr>
        """

    date_str = ref_date.strftime('%d/%m/%Y')

    return f"""
    <!DOCTYPE html>
    <html>
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1.0">
        <style>
            @import url('{config.FONT_IMPORT_URL}');
        </style>
    </head>
    <body style="font-family:'{config.FONT_FAMILY}',{config.FONT_FALLBACK};margin:0;padding:0;background:#F3F4F6;">
        <div style="max-width:700px;margin:0 auto;background:white;padding:30px;">

            <!-- Header -->
            <div style="text-align:center;margin-bottom:30px;border-bottom:3px solid {config.BRAND_COLORS['blue_lilac']};padding-bottom:20px;">
                <h1 style="margin:0;color:{config.BRAND_COLORS['blue_space']};font-size:1.8rem;">Tu Resumen CRM Semanal</h1>
                <p style="margin:10px 0 0 0;color:#6B7280;font-size:0.95rem;">Semana del {date_str}</p>
            </div>

            <!-- Personal Greeting -->
            <p style="font-size:1.1rem;color:#1F2937;margin-bottom:25px;">
                Hola <strong>{first_name}</strong>,
            </p>

            <!-- Performance Overview -->
            <div style="background:#F9FAFB;border:2px solid {perf_color};border-radius:8px;padding:20px;margin-bottom:25px;">
                <h2 style="margin:0 0 15px 0;color:#1F2937;font-size:1.3rem;">{perf_badge} Tu Rendimiento</h2>
                <table style="width:100%;border-collapse:collapse;">
                    <tr>
                        <td style="padding:8px 0;color:#6B7280;">Objetivo Mensual:</td>
                        <td style="padding:8px 0;text-align:right;"><strong>{_fmt_eur(target)}</strong></td>
                    </tr>
                    <tr>
                        <td style="padding:8px 0;color:#6B7280;">Prevision Ponderada:</td>
                        <td style="padding:8px 0;text-align:right;">
                             <strong style="color:{perf_color};">{_fmt_eur(forecast)}</strong>
                             <div style="font-size:0.75rem;color:#6B7280;">({sp_metrics.get('forecast_count', 0)} oportunidades)</div>
                        </td>
                    </tr>
                    <tr>
                        <td style="padding:8px 0;color:#6B7280;">Cumplimiento:</td>
                        <td style="padding:8px 0;text-align:right;"><strong style="color:{perf_color};font-size:1.2rem;">{achievement:.0f}%</strong></td>
                    </tr>
                    <tr style="border-top:2px solid #E5E7EB;">
                        <td style="padding:8px 0;color:#6B7280;">Leads Activos:</td>
                        <td style="padding:8px 0;text-align:right;"><strong>{sp_metrics.get('active_count', 0)}</strong></td>
                    </tr>
                    <tr>
                        <td style="padding:8px 0;color:#6B7280;">Ganados este mes:</td>
                        <td style="padding:8px 0;text-align:right;"><strong style="color:#10B981;">{len(sp_metrics.get('won_this_month', []))} ({_fmt_eur(sp_metrics.get('won_revenue', 0))})</strong></td>
                    </tr>
                </table>
            </div>

            <!-- Motivational Message -->
            {motivation}

            <!-- Alerts -->
            <div style="margin-bottom:25px;">
                <h2 style="margin:0 0 15px 0;color:#1F2937;font-size:1.3rem;">Tus Alertas</h2>
                <div style="display:grid;grid-template-columns:repeat(3,1fr);gap:10px;">
                    <div style="background:#FEE2E2;padding:15px;border-radius:6px;text-align:center;">
                        <div style="font-size:0.75rem;font-weight:700;color:#991B1B;text-transform:uppercase;margin-bottom:5px;">Zombies</div>
                        <div style="font-size:1.5rem;font-weight:bold;color:#991B1B;">{len(sp_metrics.get('zombies', []))}</div>
                    </div>
                    <div style="background:#FED7AA;padding:15px;border-radius:6px;text-align:center;">
                        <div style="font-size:0.75rem;font-weight:700;color:#9A3412;text-transform:uppercase;margin-bottom:5px;">Propuestas</div>
                        <div style="font-size:1.5rem;font-weight:bold;color:#9A3412;">{len(sp_metrics.get('proposals', []))}</div>
                    </div>
                    <div style="background:#DBEAFE;padding:15px;border-radius:6px;text-align:center;">
                        <div style="font-size:0.75rem;font-weight:700;color:#1E40AF;text-transform:uppercase;margin-bottom:5px;">Calidad</div>
                        <div style="font-size:1.5rem;font-weight:bold;color:#1E40AF;">{len(sp_metrics.get('quality_issues', []))}</div>
                    </div>
                </div>
            </div>

            <!-- Top 5 Actions -->
            <div style="margin-bottom:25px;">
                <h2 style="margin:0 0 15px 0;color:#1F2937;font-size:1.3rem;">Top 5 Acciones Esta Semana</h2>
                <table style="width:100%;border-collapse:collapse;background:#F9FAFB;border-radius:6px;overflow:hidden;">
                    <thead>
                        <tr style="background:#E5E7EB;">
                            <th style="padding:10px;text-align:left;font-size:0.85rem;color:#374151;">Prioridad</th>
                            <th style="padding:10px;text-align:left;font-size:0.85rem;color:#374151;">Lead</th>
                            <th style="padding:10px;text-align:right;font-size:0.85rem;color:#374151;">Valor</th>
                            <th style="padding:10px;text-align:left;font-size:0.85rem;color:#374151;">Razon</th>
                            <th style="padding:10px;text-align:left;font-size:0.85rem;color:#374151;">Accion</th>
                        </tr>
                    </thead>
                    <tbody>
                        {actions_html}
                    </tbody>
                </table>
            </div>

            <!-- Footer -->
            <div style="text-align:center;padding-top:20px;border-top:2px solid #E5E7EB;color:#6B7280;font-size:0.85rem;">
                <p style="margin:0 0 10px 0;">Este es tu resumen personalizado. Para ver el reporte completo del equipo:</p>
                <a href="https://www.indaws.es/crm-report-weekly" style="display:inline-block;background:{config.BRAND_COLORS['blue_lilac']};color:white;padding:10px 20px;border-radius:6px;text-decoration:none;font-weight:bold;">Ver Reporte Completo</a>
                <p style="margin:15px 0 0 0;color:#9CA3AF;">Generado automaticamente -- INDAWS CRM v5.0</p>
            </div>

        </div>
    </body>
    </html>
    """


# =============================================================================
# HELPERS
# =============================================================================

def _fmt_eur(value):
    """Format a numeric value as EUR currency string."""
    try:
        return f"\u20ac{value:,.0f}"
    except (TypeError, ValueError):
        return "\u20ac0"
