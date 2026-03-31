
# ENTERPRISE DEMO CRM Weekly Report

**CONFIDENTIAL - SALES TEAM ONLY**
**Document ID:** SALES-SKILL-001
**Classification:** Critical Sales Know-How

## Overview

This skill generates the weekly commercial control report for the ENTERPRISE DEMO sales team. It connects to the Odoo CRM via XML-RPC, calculates over 21 KPIs, and produces a brand-compliant HTML report. The report can be published to the Odoo website, sent as a team email, and delivered as personalized summaries to each salesperson.

## When to Use This Skill

- The user asks to generate the weekly CRM report
- The user needs to check pipeline health or commercial metrics
- The user wants to publish the report to Odoo or send it via email
- Monday morning automation (GitHub Actions workflow)

## Prerequisites

### Environment Variables (Required)

| Variable | Description | Example |
|---|---|---|
| `ODOO_URL` | Odoo instance URL | `https://www.enterprise-demo.es/` |
| `ODOO_DB` | Odoo database name | `master` |
| `ODOO_USER` | Odoo API username | `user@enterprise-demo.es` |
| `ODOO_KEY` | Odoo API key | (API key) |
| `EMAIL_SMTP_PASSWORD` | SMTP password for Gmail | (app password) |

### Optional Environment Variables

| Variable | Default | Description |
|---|---|---|
| `EMAIL_SMTP_SERVER` | `smtp.gmail.com` | SMTP server |
| `EMAIL_SMTP_PORT` | `587` | SMTP port |
| `EMAIL_USE_TLS` | `true` | Enable STARTTLS |
| `EMAIL_FROM` | `victor@enterprise-demo.es` | Sender address |
| `EMAIL_USERNAME` | `victor@enterprise-demo.es` | SMTP login |

## Workflow (Mandatory Sequence)

### 1. Test Connection

```bash
python skills/sales-crm-report/scripts/main.py --test
```

Verifies Odoo connectivity before generating a report.

### 2. Generate Local Report

```bash
python skills/sales-crm-report/scripts/main.py --output report.html
```

Generates the HTML report and saves it locally.

### 3. Publish to Odoo Website

```bash
python skills/sales-crm-report/scripts/main.py --odoo
```

Publishes the report to the Odoo website at `/reporte-crm-semanal` and creates review activities for each team member.

### 4. Send Team Email

```bash
python skills/sales-crm-report/scripts/main.py --email
```

Sends the full report to all configured recipients.

### 5. Send Personalized Emails

```bash
python skills/sales-crm-report/scripts/main.py --email-salespeople
```

Sends individual summaries to each salesperson with their specific KPIs, alerts, and action items.

### 6. Full Pipeline (Production)

```bash
python skills/sales-crm-report/scripts/main.py --odoo --email --email-salespeople
```

Runs the complete pipeline: generate, publish, and deliver.

## Module Architecture

```
skills/sales-crm-report/
  scripts/
    config.py           - Centralized configuration and brand constants
    utils.py            - Shared helpers (parse_date, truncate_text)
    odoo_client.py      - XML-RPC connection and data fetching
    data_processor.py   - KPI engine with 17 focused functions
    report_generator.py - Brand-compliant HTML generation
    email_service.py    - SMTP delivery (team + personalized)
    odoo_publisher.py   - Odoo website publishing + activities
    main.py             - CLI entry point
  diagnostics/          - 8 debug utilities
  references/           - KPI documentation
```

## Configuration Reference

Edit `skills/sales-crm-report/scripts/config.py` to update:

- **SALESPEOPLE_OBJECTIVES**: Monthly targets per salesperson
- **SALESPEOPLE_EMAILS**: Email addresses for personalized reports
- **EXCLUDED_SALESPEOPLE**: Users to exclude from analysis
- **STAGE_*_KEYWORDS**: Stage classification keywords
- **PRODUCT_CATEGORIES**: Product categorization rules
- **Thresholds**: DAYS_WARNING (5), DAYS_CRITICAL (7), PROPOSAL_FOLLOWUP_DAYS (3)

## Brand Compliance

This report follows the Enterprise Demo brand guidelines:

- **Font**: Dosis (no other fonts)
- **Colors**: Blue Lilac #6B68FA, Blue Space #1C265D, Warm Yellow #E5B92B, Light Sky #8AC2DB
- **Zero emojis**: Text labels and CSS badges only
- **Language**: Spanish

## Success Criteria

- Report generates without errors
- All 21+ KPIs calculate correctly
- HTML renders in email clients (Gmail, Outlook)
- Odoo website page updates correctly
- Activities created for all team members
- Zero emojis in output
- Dosis font used throughout
