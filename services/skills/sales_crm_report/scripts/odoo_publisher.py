# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - Odoo Website Publisher
===========================================
Creates/updates Odoo website pages and creates activities
for the sales team when a new report is published.
"""

import re
from datetime import datetime

from . import config


def post_to_odoo_website(client, html_content, ref_date):
    """Create or update an Odoo website page with the report.

    Extracts body and style from the full HTML, wraps them in a QWeb
    template, and upserts both the ir.ui.view and website.page records.
    Returns the page URL.
    """
    page_key = "indaws_crm_report_weekly"
    view_name = f"CRM Report {ref_date.strftime('%Y-%m-%d')}"

    # Extract content between <body> and </body>
    body_match = re.search(r"<body[^>]*>(.*?)</body>", html_content, re.DOTALL)
    body_content = body_match.group(1) if body_match else html_content

    # Extract styles
    style_match = re.search(r"<style>(.*?)</style>", html_content, re.DOTALL)
    report_style = style_match.group(1) if style_match else ""

    # Build QWeb template
    arch = f"""
    <t t-name="website.{page_key}">
        <t t-call="website.layout">
            <style>
                {report_style}
            </style>
            <div id="wrap">
                {body_content}
            </div>
        </t>
    </t>
    """

    db = config.ODOO_DB
    uid = client.uid
    password = client.password
    models = client.models

    # 1. Find or create the view
    view_domain = [("key", "=", f"website.{page_key}")]
    view_ids = models.execute_kw(
        db, uid, password, "ir.ui.view", "search", [view_domain]
    )

    if view_ids:
        models.execute_kw(
            db, uid, password, "ir.ui.view", "write",
            [view_ids[0], {"arch_db": arch, "name": view_name}],
        )
        view_id = view_ids[0]
        print(f"[OK] Odoo View updated (ID: {view_id})")
    else:
        view_id = models.execute_kw(
            db, uid, password, "ir.ui.view", "create",
            [{
                "name": view_name,
                "type": "qweb",
                "key": f"website.{page_key}",
                "arch_db": arch,
            }],
        )
        print(f"[OK] Odoo View created (ID: {view_id})")

    # 2. Find or create the page
    page_domain = [("view_id", "=", view_id)]
    page_ids = models.execute_kw(
        db, uid, password, "website.page", "search", [page_domain]
    )

    page_vals = {
        "view_id": view_id,
        "url": "/reporte-crm-semanal",
        "is_published": False,
        "name": view_name,
    }

    if page_ids:
        models.execute_kw(
            db, uid, password, "website.page", "write", [page_ids[0], page_vals]
        )
        page_id = page_ids[0]
        print(f"[OK] Odoo Page updated (ID: {page_id})")
    else:
        page_id = models.execute_kw(
            db, uid, password, "website.page", "create", [page_vals]
        )
        print(f"[OK] Odoo Page created (ID: {page_id})")

    return page_vals["url"]


def create_activities(client, url, user_ids):
    """Create 'To Do' activities for each team member on a tracking lead.

    Finds or creates a CRM lead named 'REPORTE CRM SEMANAL - SEGUIMIENTO'
    and creates one mail.activity per user, skipping duplicates.
    """
    db = config.ODOO_DB
    uid = client.uid
    password = client.password
    models = client.models

    lead_name = "REPORTE CRM SEMANAL - SEGUIMIENTO"
    lead_ids = models.execute_kw(
        db, uid, password, "crm.lead", "search",
        [[["name", "=", lead_name], ["probability", "<", 100]]],
    )

    report_link = f'<p>Link al ultimo reporte: <a href="https://www.indaws.es{url}">Ver Reporte</a></p>'

    if lead_ids:
        lead_id = lead_ids[0]
        models.execute_kw(
            db, uid, password, "crm.lead", "write",
            [lead_id, {"description": report_link}],
        )
    else:
        lead_id = models.execute_kw(
            db, uid, password, "crm.lead", "create",
            [{
                "name": lead_name,
                "description": report_link,
                "team_id": 1,
            }],
        )

    # Get activity type 'To Do'
    activity_type_ids = models.execute_kw(
        db, uid, password, "mail.activity.type", "search",
        [[["name", "ilike", "To Do"]]],
    )
    if not activity_type_ids:
        print("[WARN] Activity type 'To Do' not found, skipping activities")
        return
    activity_type_id = activity_type_ids[0]

    # Get model ID for crm.lead
    model_ids = models.execute_kw(
        db, uid, password, "ir.model", "search",
        [[["model", "=", "crm.lead"]]],
    )
    if not model_ids:
        print("[WARN] ir.model for crm.lead not found, skipping activities")
        return
    model_id = model_ids[0]

    today = datetime.now().strftime("%Y-%m-%d")

    count = 0
    for u_id in user_ids:
        try:
            existing = models.execute_kw(
                db, uid, password, "mail.activity", "search",
                [[
                    ["res_id", "=", lead_id],
                    ["res_model_id", "=", model_id],
                    ["user_id", "=", u_id],
                    ["summary", "=", "Repasar el REPORTE CRM"],
                    ["date_deadline", "=", today],
                ]],
            )
            if not existing:
                models.execute_kw(
                    db, uid, password, "mail.activity", "create",
                    [{
                        "res_id": lead_id,
                        "res_model_id": model_id,
                        "activity_type_id": activity_type_id,
                        "summary": "Repasar el REPORTE CRM",
                        "note": (
                            f'<p>El reporte semanal ya esta disponible. '
                            f'<a href="https://www.indaws.es{url}">Abrir reporte aqui</a>.</p>'
                        ),
                        "date_deadline": today,
                        "user_id": u_id,
                    }],
                )
                count += 1
        except Exception as e:
            print(f"[WARN] Failed to create activity for user {u_id}: {e}")

    print(f"[OK] Created {count} activities for the team on Lead {lead_id}")


def publish_report(client, html_content, ref_date):
    """Orchestrator: publish to Odoo website and create team activities.

    1. Post HTML to Odoo website page
    2. Get user IDs for salespeople
    3. Create activities for each team member
    """
    report_url = post_to_odoo_website(client, html_content, ref_date)

    print("   Finding team members...")
    user_ids = client.get_user_ids()

    print("   Creating activities...")
    create_activities(client, report_url, user_ids)

    print(f"[OK] Odoo integration complete")
    print(f"   Link: https://www.indaws.es{report_url}")
    return report_url
