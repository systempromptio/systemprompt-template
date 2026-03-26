---
name: "Timesheet Report"
description: "Generate weekly and monthly timesheet reports from Odoo with billable ratios, anomaly detection, and delivery via email or Telegram"
---

----|-----|------------|---------------|---------|
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
