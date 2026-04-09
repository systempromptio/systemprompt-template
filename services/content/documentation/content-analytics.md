---
title: "Content Analytics"
description: "Link performance, campaign tracking, and content journey analytics for the systemprompt.io AI governance platform."
author: "systemprompt.io"
slug: "content-analytics"
keywords: "content analytics, link tracking, campaign performance, content journey, conversion"
kind: "guide"
public: true
tags: ["analytics", "content", "tracking"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand link performance tracking and conversion metrics"
  - "Use campaign analytics to measure content distribution effectiveness"
  - "Analyse content journeys to see how users navigate between pages"
related_docs:
  - title: "Dashboard"
    url: "/documentation/dashboard"
  - title: "Metrics Reference"
    url: "/documentation/metrics-reference"
  - title: "CLI Analytics"
    url: "/documentation/cli-analytics"
---

# Content Analytics

The platform tracks content engagement through link analytics, campaign performance, and content journey mapping. These views help you understand how users discover and interact with your content.

## Link Performance

The `v_link_performance` view provides per-link metrics:

| Metric | Description |
|--------|-------------|
| **Clicks** | Total click count for each tracked link |
| **Unique Visitors** | Distinct users who clicked the link |
| **Conversion Rate** | Percentage of clicks that led to a target action |
| **Session Data** | Average session duration after clicking |
| **Referrer** | Where the click originated |

Use link performance to identify which content drives the most engagement and which links underperform.

## Campaign Performance

The `v_campaign_performance` view aggregates metrics at the campaign level:

| Metric | Description |
|--------|-------------|
| **Total Clicks** | Sum of all link clicks within the campaign |
| **Unique Users** | Distinct users who engaged with the campaign |
| **Top Links** | Highest-performing links in the campaign |
| **Time Distribution** | Click distribution over time |

Campaign tracking works with UTM parameters and internal campaign IDs to attribute engagement to specific distribution efforts.

## Content Journeys

The `v_content_journey` view maps how users navigate between content pages:

| Field | Description |
|-------|-------------|
| **Source Page** | The page where the user started |
| **Destination Page** | The page they navigated to |
| **Journey Count** | How many users followed this path |
| **Average Time** | Average time between source and destination |

Content journeys reveal natural navigation patterns and help identify where users drop off or where internal linking is most effective.

## Content Performance Metrics

The `content_performance_metrics` table stores engagement data per content page:

| Metric | Description |
|--------|-------------|
| **Views** | Total page views |
| **Unique Visitors** | Distinct visitors |
| **Time on Page** | Average time spent reading |
| **Shares** | Share action count |
| **Comments** | Comment count |
| **Search Impressions** | Appearances in search results |
| **Trend Direction** | Whether engagement is increasing, stable, or decreasing |

## Click Stream

The `v_link_click_stream` view provides a real-time feed of link clicks with:

- Device type (desktop, mobile, tablet)
- Geographic location
- Referrer URL
- Timestamp
- User identifier

Use the click stream for real-time monitoring of content distribution campaigns or to investigate individual user journeys.

## Querying Content Analytics

Content analytics are accessible through the CLI:

```bash
# Content engagement overview
systemprompt analytics content stats

# Top performing content
systemprompt analytics content top --limit 10

# Content trends over time
systemprompt analytics content trends --since 7d

# Export for analysis
systemprompt analytics content stats --export content-report.csv
```
