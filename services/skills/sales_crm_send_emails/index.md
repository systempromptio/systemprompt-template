
# CRM Email Delivery

**CONFIDENTIAL - SALES TEAM ONLY**
**Document ID:** SALES-SKILL-004
**Classification:** Critical Sales Know-How

## Overview

Handles email delivery of CRM reports through two channels: the full team report sent to all recipients, and personalized individual summaries sent to each salesperson with their specific KPIs and action items.

## When to Use This Skill

- Send the weekly CRM report to the team
- Deliver personalized summaries to salespeople
- Test SMTP configuration before sending
- Troubleshoot email delivery issues

## Workflow

### 1. Test Email Connection

Always test before sending:

```bash
python skills/sales-crm-report/scripts/main.py --test-email
```

Verifies SMTP credentials and connectivity without sending any email.

### 2. Send Team Report

```bash
python skills/sales-crm-report/scripts/main.py --email
```

Generates the report and sends it to all recipients in `EMAIL_RECIPIENTS`.

The email includes:
- Full HTML report inline
- HTML file attachment as backup
- Plain text fallback

### 3. Send Personalized Emails

```bash
python skills/sales-crm-report/scripts/main.py --email-salespeople
```

Sends individual summaries to each salesperson configured in `SALESPEOPLE_EMAILS`. Each email contains:
- Personal greeting
- Individual performance metrics (objective, forecast, achievement)
- Alert counts (zombies, proposals, quality issues)
- Top 5 prioritized action items
- Motivational message based on performance

### 4. Send Everything

```bash
python skills/sales-crm-report/scripts/main.py --email --email-salespeople
```

## Configuration

### Recipients

Edit `skills/sales-crm-report/scripts/config.py`:

```python
EMAIL_RECIPIENTS = [
    "user1@enterprise-demo.es",
    "user2@enterprise-demo.es",
]

SALESPEOPLE_EMAILS = {
    "Salesperson Name": "email@enterprise-demo.es",
}
```

### SMTP Settings

| Variable | Default | Description |
|---|---|---|
| `EMAIL_SMTP_SERVER` | `smtp.gmail.com` | SMTP server |
| `EMAIL_SMTP_PORT` | `587` | SMTP port |
| `EMAIL_USE_TLS` | `true` | Enable STARTTLS |
| `EMAIL_FROM` | `victor@enterprise-demo.es` | Sender address |
| `EMAIL_USERNAME` | `victor@enterprise-demo.es` | SMTP login |
| `EMAIL_SMTP_PASSWORD` | (required) | SMTP password |

### Gmail App Password

For Gmail, you need an App Password (not your regular password):

1. Go to Google Account > Security > 2-Step Verification
2. At the bottom, select "App passwords"
3. Generate a new app password for "Mail"
4. Use this as `EMAIL_SMTP_PASSWORD`

## Troubleshooting

| Issue | Check |
|---|---|
| Authentication error | Verify EMAIL_USERNAME and EMAIL_SMTP_PASSWORD |
| Connection timeout | Check EMAIL_SMTP_SERVER and EMAIL_SMTP_PORT |
| Email not received | Check spam folder, verify recipient address |
| TLS error | Ensure EMAIL_USE_TLS matches server requirements |

## Success Criteria

- SMTP test passes
- Team report delivered to all recipients
- Personalized emails sent to all salespeople
- No authentication or connection errors
