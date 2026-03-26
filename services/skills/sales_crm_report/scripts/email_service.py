# -*- coding: utf-8 -*-
"""
INDAWS CRM Report - Email Service
==================================
SMTP email delivery for team reports and personalized salesperson emails.
All credentials loaded from environment variables via config module.
"""

import smtplib
from datetime import datetime, timedelta
from email import encoders
from email.mime.base import MIMEBase
from email.mime.multipart import MIMEMultipart
from email.mime.text import MIMEText

from . import config
from .utils import truncate_text


# =============================================================================
# SMTP CONNECTION
# =============================================================================

def _create_smtp_connection():
    """Create and return an authenticated SMTP connection.

    Uses config values for server, port, TLS, and credentials.
    Raises on connection or authentication failure.

    Returns:
        smtplib.SMTP: Authenticated SMTP server object.
    """
    server = smtplib.SMTP(config.EMAIL_SMTP_SERVER, config.EMAIL_SMTP_PORT, timeout=30)
    server.set_debuglevel(0)

    if config.EMAIL_USE_TLS:
        print(f"  Starting TLS on {config.EMAIL_SMTP_SERVER}:{config.EMAIL_SMTP_PORT}...")
        server.starttls()

    print(f"  Authenticating as {config.EMAIL_USERNAME}...")
    server.login(config.EMAIL_USERNAME, config.EMAIL_PASSWORD)

    return server


# =============================================================================
# TEAM REPORT EMAIL
# =============================================================================

def send_team_report(html_content, ref_date):
    """Send the full team HTML report to all configured recipients.

    Args:
        html_content: Complete HTML string of the team report.
        ref_date: Reference datetime for subject line formatting.

    Returns:
        True on success, False on failure.
    """
    if not config.EMAIL_PASSWORD:
        print("[FAIL] EMAIL_SMTP_PASSWORD not configured in environment variables.")
        return False

    if not config.EMAIL_RECIPIENTS:
        print("[FAIL] No recipients configured in EMAIL_RECIPIENTS.")
        return False

    date_str = ref_date.strftime('%d/%m/%Y')
    subject = f"{config.EMAIL_SUBJECT_PREFIX} - {date_str}"

    # Build message
    msg = MIMEMultipart('alternative')
    msg['Subject'] = subject
    msg['From'] = config.EMAIL_FROM
    msg['To'] = ', '.join(config.EMAIL_RECIPIENTS)

    # Plain text fallback
    text_part = MIMEText(
        f"Reporte CRM Semanal INDAWS - {date_str}\n\n"
        "Este email contiene el reporte en formato HTML.\n"
        "Si no puedes verlo, abre el archivo adjunto.",
        'plain', 'utf-8',
    )

    # HTML content
    html_part = MIMEText(html_content, 'html', 'utf-8')

    msg.attach(text_part)
    msg.attach(html_part)

    # Attach HTML file as backup
    attachment = MIMEBase('text', 'html')
    attachment.set_payload(html_content.encode('utf-8'))
    encoders.encode_base64(attachment)
    attachment.add_header(
        'Content-Disposition', 'attachment',
        filename=f'reporte_crm_{ref_date.strftime("%Y%m%d")}.html',
    )
    msg.attach(attachment)

    try:
        print(f"Connecting to {config.EMAIL_SMTP_SERVER}:{config.EMAIL_SMTP_PORT}...")
        server = _create_smtp_connection()

        print(f"Sending team report to {len(config.EMAIL_RECIPIENTS)} recipient(s)...")
        server.sendmail(config.EMAIL_FROM, config.EMAIL_RECIPIENTS, msg.as_string())
        server.quit()

        print(f"[OK] Team report sent to: {', '.join(config.EMAIL_RECIPIENTS)}")
        return True

    except smtplib.SMTPAuthenticationError as e:
        print(f"[FAIL] SMTP authentication error: {e}")
        print("  Check EMAIL_USERNAME and EMAIL_SMTP_PASSWORD environment variables.")
        return False
    except smtplib.SMTPConnectError as e:
        print(f"[FAIL] Could not connect to SMTP server: {e}")
        return False
    except smtplib.SMTPException as e:
        print(f"[FAIL] SMTP error: {e}")
        return False
    except Exception as e:
        print(f"[FAIL] Error sending team report: {e}")
        return False


# =============================================================================
# SMTP CONNECTION TEST
# =============================================================================

def test_smtp_connection():
    """Test SMTP connection without sending any email.

    Returns:
        True if connection and auth succeed, False otherwise.
    """
    if not config.EMAIL_PASSWORD:
        print("[FAIL] EMAIL_SMTP_PASSWORD not configured in environment variables.")
        return False

    try:
        print(f"Testing connection to {config.EMAIL_SMTP_SERVER}:{config.EMAIL_SMTP_PORT}...")
        server = _create_smtp_connection()
        server.quit()

        print("[OK] SMTP connection successful.")
        print(f"  Server: {config.EMAIL_SMTP_SERVER}")
        print(f"  Port: {config.EMAIL_SMTP_PORT}")
        print(f"  TLS: {'Yes' if config.EMAIL_USE_TLS else 'No'}")
        print(f"  User: {config.EMAIL_USERNAME}")
        return True

    except Exception as e:
        print(f"[FAIL] Connection error: {e}")
        return False


# =============================================================================
# SALESPERSON METRICS CALCULATION
# =============================================================================

def calculate_salesperson_metrics(salesperson, enriched_leads, data, ref_date):
    """Calculate individual metrics for a specific salesperson.

    Filters the enriched leads for one salesperson and computes objectives,
    zombies, warnings, proposals, quality issues, won deals, top opportunities,
    concentration risk, and stalled deals.

    Args:
        salesperson: Full name of the salesperson.
        enriched_leads: List of all enriched lead dicts from data_processor.
        data: Full data dict from data_processor.
        ref_date: Reference datetime for the report period.

    Returns:
        Dict with all individual metrics, or None if objectives data not found.
    """
    # Find objectives data
    sp_objectives = next(
        (obj for obj in data['objectives'] if obj['name'] == salesperson),
        None,
    )
    if not sp_objectives:
        return None

    # Filter leads for this salesperson
    sp_leads = [l for l in enriched_leads if l['salesperson'] == salesperson]

    # Active leads (not won/lost)
    won_lost_keywords = config.STAGE_WON_KEYWORDS + config.STAGE_LOST_KEYWORDS
    sp_active = [
        l for l in sp_leads
        if not any(k in l.get('stage', '') for k in won_lost_keywords)
    ]

    # Zombies (critical alerts)
    sp_zombies = [l for l in sp_leads if l.get('alert') == 'critical']

    # Warnings (yellow zone)
    sp_warnings = [l for l in sp_leads if l.get('alert') == 'warning']

    # Proposal alerts
    sp_proposals = [l for l in sp_leads if l.get('proposal_alert', False)]

    # Quality issues
    sp_quality = [l for l in sp_leads if l.get('has_quality_issue', False)]

    # Won deals this month (accumulated)
    current_month_start = ref_date.replace(day=1, hour=0, minute=0, second=0, microsecond=0)
    sp_won_this_month = []
    for l in sp_leads:
        is_won = any(k in l.get('stage', '') for k in config.STAGE_WON_KEYWORDS)
        if is_won:
            closed_dt = l.get('closed') or l.get('updated')
            if closed_dt and closed_dt >= current_month_start:
                sp_won_this_month.append(l)

    # Top opportunities sorted by weighted value
    sp_active_sorted = sorted(sp_active, key=lambda x: -x.get('weighted', 0))
    top_opportunities = sp_active_sorted[:5]

    # Concentration risk
    sp_forecast = sp_objectives.get('forecast', 0)
    top_deal = sp_active_sorted[0] if sp_active_sorted else None
    concentration = (
        (top_deal.get('weighted', 0) / sp_forecast * 100)
        if top_deal and sp_forecast > 0
        else 0
    )

    # Recent leads (last 30 days)
    thirty_days_ago = ref_date - timedelta(days=30)
    recent_leads = [
        l for l in sp_active
        if l.get('created') and l['created'] >= thirty_days_ago
    ]

    # Stalled deals (>14 days inactive)
    stalled = [l for l in sp_active if l.get('days_inactive', 0) > 14]

    return {
        'name': salesperson,
        'email': config.SALESPEOPLE_EMAILS.get(salesperson),
        'target': sp_objectives['target'],
        'forecast': sp_objectives['forecast'],
        'gap': sp_objectives.get('gap', 0),
        'achievement': sp_objectives['achievement'],
        'forecast_count': sp_objectives.get('forecast_count', 0),
        'active_count': len(sp_active),
        'won_this_month': sp_won_this_month,
        'won_revenue': sum(l.get('revenue', 0) for l in sp_won_this_month),
        'zombies': sp_zombies,
        'warnings': sp_warnings,
        'proposals': sp_proposals,
        'quality_issues': sp_quality,
        'top_opportunities': top_opportunities,
        'concentration': concentration,
        'top_deal': top_deal,
        'recent_leads_count': len(recent_leads),
        'stalled_count': len(stalled),
        'stalled_deals': stalled[:5],
        'conversion': sp_objectives.get('conversion', 0),
    }


# =============================================================================
# PERSONALIZED EMAILS
# =============================================================================

def send_personalized_emails(enriched_leads, data, ref_date):
    """Send personalized CRM summary emails to all salespeople.

    For each salesperson in SALESPEOPLE_OBJECTIVES, calculates individual metrics,
    generates a personalized HTML report, and sends it via SMTP.

    Args:
        enriched_leads: List of all enriched lead dicts from data_processor.
        data: Full data dict from data_processor.
        ref_date: Reference datetime for the report period.

    Returns:
        True if all emails sent successfully, False otherwise.
    """
    # Import here to avoid circular import at module level
    from . import report_generator

    if not config.EMAIL_PASSWORD:
        print("[FAIL] EMAIL_SMTP_PASSWORD not configured.")
        return False

    success_count = 0
    total_count = len(config.SALESPEOPLE_OBJECTIVES)

    print(f"Sending personalized emails to {total_count} salesperson(s)...")

    for salesperson in config.SALESPEOPLE_OBJECTIVES:
        email = config.SALESPEOPLE_EMAILS.get(salesperson)
        if not email:
            print(f"  [SKIP] {salesperson}: email not configured.")
            continue

        # Calculate individual metrics
        sp_metrics = calculate_salesperson_metrics(
            salesperson, enriched_leads, data, ref_date,
        )
        if not sp_metrics:
            print(f"  [SKIP] {salesperson}: could not calculate metrics.")
            continue

        # Generate personalized HTML
        html_content = report_generator.generate_personalized_report(sp_metrics, ref_date)

        # Build email message
        date_str = ref_date.strftime('%d/%m/%Y')
        subject = f"Tu Resumen CRM Semanal - {date_str}"

        msg = MIMEMultipart('alternative')
        msg['Subject'] = subject
        msg['From'] = config.EMAIL_FROM
        msg['To'] = email

        text_part = MIMEText(
            f"Resumen CRM Semanal para {salesperson} - {date_str}\n\n"
            "Este email contiene tu resumen personalizado en formato HTML.",
            'plain', 'utf-8',
        )
        html_part = MIMEText(html_content, 'html', 'utf-8')

        msg.attach(text_part)
        msg.attach(html_part)

        # Send
        try:
            server = _create_smtp_connection()
            server.sendmail(config.EMAIL_FROM, [email], msg.as_string())
            server.quit()
            print(f"  [OK] {salesperson} ({email})")
            success_count += 1
        except Exception as e:
            print(f"  [FAIL] {salesperson} ({email}): {e}")

    print(f"Result: {success_count}/{total_count} personalized emails sent.")
    return success_count == total_count
