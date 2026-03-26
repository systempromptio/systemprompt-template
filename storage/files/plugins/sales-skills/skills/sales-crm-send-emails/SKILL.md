---
name: "CRM Send Emails"
description: "Send CRM reports via email. Delivers full team report and personalized individual summaries to each salesperson"
---

|---|
| `EMAIL_SMTP_SERVER` | `smtp.gmail.com` | SMTP server |
| `EMAIL_SMTP_PORT` | `587` | SMTP port |
| `EMAIL_USE_TLS` | `true` | Enable STARTTLS |
| `EMAIL_FROM` | `victor@foodles.com` | Sender address |
| `EMAIL_USERNAME` | `victor@foodles.com` | SMTP login |
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
