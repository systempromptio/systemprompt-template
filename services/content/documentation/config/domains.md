---
title: "Custom Domains"
description: "Configure custom domains for SystemPrompt Cloud with automatic TLS certificates via Let's Encrypt."
author: "SystemPrompt Team"
slug: "config/domains"
keywords: "domains, custom domain, tls, ssl, dns, https"
image: "/files/images/docs/config-domains.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Custom Domains

Configure custom domains for your SystemPrompt Cloud deployment. Automatic TLS certificates are provisioned via Let's Encrypt.

## Setting a Custom Domain

```bash
# Set your custom domain
systemprompt cloud domain set api.example.com

# Set apex domain
systemprompt cloud domain set example.com
```

## DNS Configuration

After setting your domain, configure DNS records:

### For Subdomains (Recommended)

Create a CNAME record pointing to your cloud tenant:

| Type | Name | Value |
|------|------|-------|
| CNAME | api | `your-tenant.systemprompt.cloud` |

### For Apex Domains

Create an A record pointing to the SystemPrompt IP:

| Type | Name | Value |
|------|------|-------|
| A | @ | `<provided-ip>` |

Get your tenant's IP:

```bash
systemprompt cloud domain info
```

## TLS Certificate Provisioning

After DNS is configured, TLS certificates are automatically provisioned:

1. SystemPrompt detects DNS propagation
2. Let's Encrypt challenge is completed
3. Certificate is issued and installed
4. HTTPS is enabled automatically

This process typically takes 2-5 minutes after DNS propagation.

## Checking Domain Status

```bash
# View domain configuration
systemprompt cloud domain show

# Check certificate status
systemprompt cloud domain status
```

## Domain Status Values

| Status | Description |
|--------|-------------|
| `pending_dns` | Waiting for DNS records |
| `provisioning` | Certificate being issued |
| `active` | Domain fully configured |
| `error` | Configuration failed |

## Multiple Domains

Add multiple domains to a single deployment:

```bash
# Add primary domain
systemprompt cloud domain set api.example.com

# Add additional domain
systemprompt cloud domain add www.example.com

# List all domains
systemprompt cloud domain list
```

## Removing a Custom Domain

```bash
# Remove a domain
systemprompt cloud domain remove api.example.com

# Revert to default subdomain
systemprompt cloud domain reset
```

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| `pending_dns` stuck | DNS not propagated | Wait 24-48 hours, check with `dig` |
| Certificate error | DNS misconfigured | Verify CNAME/A record is correct |
| HTTPS not working | Certificate pending | Wait for provisioning, check status |
| Domain conflict | Domain already in use | Contact support |

### Checking DNS Propagation

```bash
# Check CNAME record
dig CNAME api.example.com

# Check A record
dig A example.com

# Check from multiple locations
dig @8.8.8.8 CNAME api.example.com
```

## Quick Reference

| Task | Command |
|------|---------|
| Set domain | `systemprompt cloud domain set <domain>` |
| Check status | `systemprompt cloud domain status` |
| View info | `systemprompt cloud domain show` |
| Add domain | `systemprompt cloud domain add <domain>` |
| Remove domain | `systemprompt cloud domain remove <domain>` |
| List domains | `systemprompt cloud domain list` |
| Reset | `systemprompt cloud domain reset` |