
# Timesheet Report

Automated timesheet reporting system for Foodles. Generates weekly summaries with billable/internal split, anomaly detection, quality flags, and trend comparison.

## Overview

Queries Odoo's `account.analytic.line` for timesheet entries, `hr.leave` for absences, and `project.project` for billable classification. Produces a clean, actionable report for supervisors.

## Prerequisites

- **odoo-pilot** skill (same repo)
- Odoo >= 19.0 (JSON2 protocol)
- Environment variables: `ODOO_URL`, `ODOO_DB`, `ODOO_KEY`, `ODOO_USER`

## Company Policy Reference

- **Schedule**: L-J 9:00-18:00 (8h), V 8:00-15:00 (5.5h)
- **Expected weekly hours**: 37.5h
- **Breaks**: Lunch + pauses — must be logged
- **Billable target (Marketing/Elena)**: 40h/month facturable
- **Internal allowance (consultors)**: 8h/month max for internal tasks
- **Absences**: Vacations and sick leave in Odoo `hr.leave`

## Billable Classification

- **Internal**: `project.is_internal_project = true` OR project name starts with "Interno"
- **Facturable (client)**: Everything else (client projects, implantaciones, consultorías)

## Report Features

### Weekly Report (every Monday 8:00 CET)
1. **Score visual**: 🟢 >95% horas (>35.6h), 🟡 80-95% (30-35.6h), 🔴 <80% (<30h)
2. **Per employee**:
   - Total hours + % of expected (adjusted for absences)
   - Facturable vs Interno split (hours + %)
   - Top projects/tasks by hours
   - Daily distribution (detect missing days)
   - Alerts: missing days, <30h, vague entries
3. **vs Previous week**: delta comparison
4. **Quality flags**: entries with "/" or single-word vague descriptions
5. **Task concentration**: alert if any task category > 40% of total

### Monthly Report (first Monday of month, 8:00 CET)
- Same as weekly but aggregated for the full month
- Billable target tracking (Elena: 40h/month facturable)
- Executive summary for Víctor (all teams, when expanded)

## Current Reporting Structure (Pilot: Marketing)

| Employee | ID | Supervisor | Supervisor ID | Channel |
|----------|-----|------------|---------------|---------|
| Elena Soler González | 206 | Víctor Peris | 2 | Telegram + Email |
| Álvaro Royo Niederleytner | 203 | Víctor Peris | 2 | Telegram + Email |

**Delivery**: Email is primary channel. Telegram as secondary for Víctor.

## Future Expansion

When scaling beyond Marketing:
- Each supervisor receives their direct reports' timesheets via email
- Víctor receives a monthly executive summary of all teams
- Consult `hr.employee.parent_id` to auto-resolve supervisor chains

## Usage

```bash
# Weekly report (auto-detects last week)
./scripts/generate_report.sh

# Specific week
./scripts/generate_report.sh 2026-01-27

# Monthly report
./scripts/generate_report.sh --monthly 2026-01

# Output: JSON (for programmatic use)
./scripts/generate_report.sh --format json
```

## Adding Employees

1. Find employee in Odoo:
   ```bash
   odoo-pilot/scripts/search_records.sh hr.employee '[["name","ilike","name"]]' '["name","department_id","parent_id"]' 5
   ```
2. Add to config in `scripts/config.json`
3. Supervisor auto-resolved from `parent_id` in Odoo

## Cron Configuration

- **Weekly**: Monday 8:00 CET → email to supervisor + Telegram to Víctor
- **Monthly**: First Monday of month 8:00 CET → email to supervisor + Telegram to Víctor
