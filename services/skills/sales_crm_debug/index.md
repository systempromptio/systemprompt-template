
# CRM Debug Utilities

**CONFIDENTIAL - SALES TEAM ONLY**
**Document ID:** SALES-SKILL-003
**Classification:** Critical Sales Know-How

## Overview

A collection of 8 diagnostic scripts for troubleshooting CRM report issues. Each script targets a specific area and produces focused output.

## When to Use This Skill

- Report shows unexpected numbers or missing data
- Salesperson names do not match between Odoo and config
- Proposal alerts are not triggering correctly
- Forecast calculations seem wrong
- Odoo website page is not updating
- Duplicate pages or views detected

## Available Diagnostics

### 1. Check Proposals

```bash
python skills/sales-crm-report/diagnostics/check_proposals.py
```

Verifies proposal stage matching against configured keywords. Shows which leads are in proposal stages and whether they should trigger alerts.

### 2. Check Alerts

```bash
python skills/sales-crm-report/diagnostics/check_alerts.py
```

Inspects the Odoo view content to verify that expected report sections are present.

### 3. Check Names

```bash
python skills/sales-crm-report/diagnostics/check_names.py
```

Compares salesperson names between Odoo and config. Detects mismatches that cause leads to be excluded from the report.

### 4. Check Odoo View

```bash
python skills/sales-crm-report/diagnostics/check_odoo_view.py
```

Shows the status of the report website page and view in Odoo. Lists all matching pages and views with their IDs and update dates.

### 5. Debug Metrics

```bash
python skills/sales-crm-report/diagnostics/debug_metrics.py
```

Traces the forecast calculation step by step. Outputs to `debug_forecast_result.txt` for detailed analysis.

### 6. Trace Proposal

```bash
python skills/sales-crm-report/diagnostics/trace_proposal.py
```

Shows the proposal detection logic for each lead. Traces keyword matching to explain why a lead is or is not flagged as a proposal.

### 7. Find Duplicates

```bash
python skills/sales-crm-report/diagnostics/find_duplicates.py
```

Searches for duplicate report pages and views in Odoo. Duplicates can cause the wrong page to be updated.

### 8. Verify Generation

```bash
python skills/sales-crm-report/diagnostics/verify_generation.py
```

Generates a report with mock data to verify the HTML generation works without needing a live Odoo connection.

## Prerequisites

All scripts (except verify_generation.py) require Odoo connection environment variables:

| Variable | Required |
|---|---|
| `ODOO_URL` | Yes |
| `ODOO_DB` | Yes |
| `ODOO_USER` | Yes |
| `ODOO_KEY` | Yes |

## Success Criteria

- Script runs without errors
- Output clearly indicates the source of any issue
- Actionable recommendations provided
