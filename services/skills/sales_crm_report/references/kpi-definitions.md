# KPI Definitions - FOODLES CRM Report

Reference document with definitions for all KPIs calculated by the CRM report system.

---

## Executive Summary KPIs

### 1. Pipeline Status

- **Formula**: Based on run rate vs weekly target
- **Values**: ON TRACK (>= target), WARNING (>= 80% target), AT RISK (< 80% target)
- **Source**: `calculate_executive_insights()`
- **Interpretation**: Overall health of lead generation

### 2. Run Rate (4-week average)

- **Formula**: Sum of leads created in last 4 weeks / 4
- **Unit**: Leads per week
- **Target**: 10 leads/week
- **Source**: `weekly_trends` data

### 3. Momentum

- **Formula**: (current week - 4w average) / 4w average
- **Values**: ACCELERATING (> +20%), STABLE (-20% to +20%), DECELERATING (< -20%)
- **Interpretation**: Direction of lead generation trend

---

## Sales Performance KPIs

### 4. Monthly Objective Achievement

- **Formula**: (Pipeline Forecast + Actual Sales) / Monthly Target x 100
- **Unit**: Percentage
- **Thresholds**: Green >= 100%, Yellow >= 80%, Red < 80%
- **Per**: Salesperson + Global total

### 5. Pipeline Forecast (Weighted)

- **Formula**: SUM(Expected Revenue x Probability) for leads with deadline in current month
- **Unit**: Euros
- **Excludes**: Won, Lost, and Frozen stage leads
- **Per**: Salesperson

### 6. Actual Sales

- **Formula**: SUM(amount_untaxed) from confirmed sale.order records in current month
- **Unit**: Euros
- **Source**: `fetch_sales_orders()`

### 7. Gap to Target

- **Formula**: (Pipeline Forecast + Actual Sales) - Monthly Target
- **Unit**: Euros
- **Interpretation**: Positive = on track, Negative = needs attention

---

## Conversion KPIs

### 8. Demo Conversion Rate (90 days)

- **Formula**: Won deals with demo / Total deals with demo (90-day window)
- **Unit**: Percentage
- **Target**: 30% (TARGET_CONVERSION_RATE)
- **Filter**: Only leads where `x_studio_demo_realizada` is set
- **Per**: Salesperson + Global

### 9. Proposal Conversion Rate (90 days)

- **Formula**: Won proposals / (Won + Lost proposals) in 90-day window
- **Unit**: Percentage
- **Source**: `calculate_proposal_performance()`

---

## Activity KPIs

### 10. Weekly Activity Count

- **Formula**: Count of leads with `write_date` in last 7 days
- **Unit**: Number of touched leads
- **Per**: Salesperson

### 11. Average Sales Cycle

- **Formula**: AVG(date_closed - create_date) for won deals in last 90 days
- **Unit**: Days
- **Per**: Salesperson + Global

### 12. Demos Realized (Monthly)

- **Formula**: Count of leads with `x_studio_fecha_demo` in current month
- **Unit**: Count
- **Target**: 16 per month (DEMOS_MONTHLY_TARGET)
- **Per**: Salesperson + by source

---

## Pipeline Health KPIs

### 13. Active Leads Count

- **Formula**: Count of leads not Won, not Lost, and `active=True`
- **Unit**: Count
- **Excludes**: Archived leads

### 14. Concentration Risk

- **Formula**: Top deal weighted value / Total forecast x 100
- **Unit**: Percentage
- **Threshold**: > 50% triggers alert
- **Interpretation**: High value = pipeline depends too much on one deal

### 15. Opportunity Aging

- **Formula**: Days since last update (`write_date`)
- **Buckets**: 0-7d, 8-14d, 15-30d, 31-60d, 60+d
- **Per**: Stage + Salesperson (average)

### 16. Yellow Zone

- **Formula**: Count of leads with 5-6 days inactive
- **Interpretation**: Leads about to become zombies (7+ days)

---

## Alert KPIs

### 17. Zombie Count

- **Formula**: Count of leads with `days_inactive > DAYS_CRITICAL` (7 days)
- **Excludes**: Won, Lost, Frozen, Collaborators, Analysis, On Hold, Academy, archived
- **Priority**: Sorted by days inactive (worst first)

### 18. Proposal Follow-up Alerts

- **Formula**: Leads in proposal stage with `days_inactive > PROPOSAL_FOLLOWUP_DAYS` (3 days)
- **Independent**: from zombie logic (a proposal at 4 days triggers alert even though < 7 days)

### 19. At-Risk Opportunities

- **Signals**:
  - High value (> 15k) with no activity > 3 days
  - Approaching deadline (< 7 days) with probability < 50%
  - High value in early stage with > 14 days inactive
- **Sorted**: By revenue descending

---

## Data Quality KPIs

### 20. Data Quality Issues

- **Formula**: Count of active leads missing: expected_revenue, date_deadline, or probability
- **Excludes**: Early stages (Nuevo, Cualificado, 1er Contacto), Frozen, On Hold
- **Grouped by**: Salesperson, then Stage

### 21. Source Ranking

- **Formula**: Count of active leads by source (utm.source)
- **Display**: Top 8 sources
- **Excludes**: Won and Lost leads

---

## Temporal Comparison KPIs

### 22. Month-over-Month Comparison

- **Formula**: Leads created in current month vs previous month
- **Unit**: Count + delta

### 23. Quarter-over-Quarter Comparison

- **Formula**: Leads created in current quarter vs previous quarter
- **Unit**: Count + delta

### 24. Weekly Trend (4-week bars)

- **Formula**: Leads created per week for last 4 weeks
- **Display**: Bar chart with week labels (S-3, S-2, S-1, Esta)

---

## Notes

- All date-based calculations use `datetime.now()` as reference date
- Probability values from Odoo are divided by 100 (Odoo stores 0-100, we use 0-1)
- Revenue is `expected_revenue` field (untaxed)
- "Active" means both pipeline-active (not won/lost) AND Odoo `active=True` (not archived)
- Analysis stage leads have probability overridden to 0.95
