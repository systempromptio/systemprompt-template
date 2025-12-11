# World-Class Analytics Implementation Plan

## Executive Summary

This document outlines a comprehensive plan to transform SystemPrompt Admin MCP tools into a world-class analytics platform. The goal is to provide deep, actionable insights across all dimensions of the platform: traffic, content performance, user behavior, engagement, and system health.

## Vision: World-Class Analytics Platform

### What "World-Class" Means

A world-class analytics platform provides:
1. **Complete visibility** into all user interactions and system behavior
2. **Actionable insights** that drive decision-making
3. **Real-time monitoring** with historical trend analysis
4. **Granular segmentation** across multiple dimensions
5. **Performance attribution** linking metrics to outcomes
6. **Predictive capabilities** to anticipate trends

### Current State vs Target State

| Dimension | Current State | Target State |
|-----------|---------------|--------------|
| **Traffic** | Basic session counts | Full visitor journey, cohort analysis, attribution |
| **Content** | Simple view counts | Engagement metrics, performance scoring, SEO insights |
| **Users** | Session listings | Behavior patterns, segmentation, lifecycle analysis |
| **Bot Traffic** | Not separated | Cleanly separated with SEO monitoring |
| **Trends** | Basic daily counts | Multi-dimensional time-series with forecasting |
| **Performance** | Error counts | Full performance SLI/SLO monitoring |
| **Attribution** | Referrer only | Full UTM tracking, campaign ROI, channel analysis |

---

## Data Architecture Analysis

### Available Data Sources

#### 1. Session Data (`user_sessions`)
**Rich metrics per session:**
- **Identity:** session_id, user_id, user_type, fingerprint_hash
- **Timing:** started_at, last_activity_at, ended_at, duration_seconds
- **Activity:** request_count, task_count, message_count, ai_request_count
- **Quality:** avg_response_time_ms, success_rate, error_count
- **Cost:** total_tokens_used, total_ai_cost_cents
- **Device:** device_type, browser, os
- **Location:** country, region, city, ip_address
- **Attribution:** referrer_source, referrer_url, utm_source, utm_medium, utm_campaign
- **Content:** landing_page, entry_url, endpoints_accessed
- **Bot Detection:** is_bot (66.7% of traffic identified)

#### 2. Content Data (`content_view_events`)
**Per-view engagement tracking:**
- content_id, session_id, user_id
- time_on_page_seconds, scroll_depth_percent
- referrer_source, referrer_url
- user_agent, ip_country
- viewed_at timestamp

#### 3. Content Metadata (`markdown_content`)
**Content characteristics:**
- slug, title, description, excerpt
- content_type, category_id, author
- published_at, keywords
- source_id (origin tracking)

#### 4. Pre-Aggregated Metrics
**Performance tables:**
- `content_performance_metrics` - Aggregated content stats
- `content_search_performance` - Search visibility data

---

## Analytics Dimensions Framework

### 1. Traffic & Visitor Analytics

#### 1.1 Traffic Overview Dashboard
**Key Metrics:**
- Total visitors (unique fingerprints/user_ids)
- Total sessions (human vs bot separated)
- Total pageviews (from content_view_events)
- Avg session duration
- Avg pages per session
- Bounce rate (single-page sessions)
- Return visitor rate

**Segmentation:**
- Human vs Bot traffic composition
- New vs Returning visitors
- Anonymous vs Registered users
- By device type (mobile, desktop, tablet)
- By geography (country, region, city)
- By browser/OS

**Time Series:**
- Hourly patterns (time-of-day analysis)
- Daily trends (7d, 30d, 90d)
- Weekly seasonality
- Month-over-month growth

#### 1.2 Visitor Journey Analysis
**Entry Points:**
- Top landing pages
- Entry URL distribution
- Campaign entry analysis (UTM tracking)

**Exit Points:**
- Exit pages analysis
- Drop-off patterns
- Session completion rates

**Path Analysis:**
- Common page sequences (from endpoints_accessed)
- Content flow visualization
- User navigation patterns

#### 1.3 Traffic Sources & Attribution
**Channel Performance:**
- Direct traffic
- Referral traffic (by domain)
- Search traffic (organic vs paid - if detectable)
- Social traffic breakdown
- Email campaigns

**UTM Campaign Tracking:**
- Performance by utm_source
- Performance by utm_medium
- Performance by utm_campaign
- Campaign ROI (if conversion tracking exists)

**Referrer Quality:**
- Bounce rate by referrer
- Engagement by referrer
- Conversion by referrer

---

### 2. Content Performance Analytics

#### 2.1 Content Engagement Dashboard
**Per-Content Metrics:**
- Total views (from content_view_events)
- Unique viewers
- Avg time on page
- Avg scroll depth
- Engagement score (composite metric)
- Social shares (if tracked)
- Comments/interactions (if tracked)

**Content Rankings:**
- Most viewed content (7d, 30d, all-time)
- Most engaging content (by time/scroll)
- Fastest growing content
- Trending content (velocity analysis)

**Content Quality Indicators:**
- Bounce rate per content
- Exit rate per content
- Read completion rate (scroll depth 80%+)
- Return reader rate

#### 2.2 Content Categories Analysis
**Category Performance:**
- Views by category
- Engagement by category
- Growth rate by category
- Category affinity (what readers view together)

**Content Type Analysis:**
- Article vs Tutorial vs Guide performance
- Long-form vs Short-form engagement
- Visual vs Text-heavy content

#### 2.3 Content Lifecycle
**Publishing Impact:**
- Initial traffic surge (first 24h, 7d)
- Organic growth trajectory
- Content half-life (traffic decay)
- Evergreen vs Timely content identification

**Content Freshness:**
- Traffic by publish date
- Update impact analysis
- Content decay detection

---

### 3. User Behavior & Engagement

#### 3.1 Session Quality Metrics
**Engagement Scoring:**
- Session depth (pages viewed)
- Session duration quartiles
- Interaction intensity (requests per minute)
- Content consumption patterns

**User Segmentation:**
- Highly engaged (>5 min, >3 pages)
- Medium engaged (2-5 min, 2-3 pages)
- Low engaged (<2 min, 1 page)
- Bounce sessions (0 interaction)

**Behavioral Patterns:**
- Reading habits (time of day, day of week)
- Content preferences (categories, types)
- Return frequency (session intervals)
- Loyalty indicators (repeat visits)

#### 3.2 User Cohort Analysis
**Cohort Definition:**
- By acquisition date (first_seen)
- By acquisition channel (first referrer)
- By device type
- By geography

**Cohort Metrics:**
- Retention rates (D1, D7, D30 retention)
- Lifetime value (engagement over time)
- Churn analysis
- Cohort maturation

#### 3.3 Conversion Funnels
**Funnel Stages:**
- Landing → Content view
- Content view → Additional pages
- Anonymous → Registered conversion
- Visitor → AI user conversion (task_count > 0)

**Funnel Metrics:**
- Conversion rates per stage
- Drop-off analysis
- Time to convert
- Conversion attribution

---

### 4. Geographic & Device Analytics

#### 4.1 Geographic Distribution
**Country-Level:**
- Sessions by country
- Engagement by country
- Growth by country
- Content preferences by country

**City-Level:**
- Top cities by traffic
- City engagement metrics
- Regional patterns

#### 4.2 Device & Technology
**Device Analysis:**
- Mobile vs Desktop usage patterns
- Tablet traffic
- Device-specific engagement
- Cross-device behavior

**Browser & OS:**
- Browser distribution
- OS distribution
- Technical capability segmentation
- Performance by platform

---

### 5. Bot & Crawler Analytics

#### 5.1 SEO Health Monitoring
**Search Engine Crawlers:**
- Google crawler activity (frequency, coverage)
- Bing crawler activity
- Baidu, Yandex crawler tracking
- Crawl errors/issues

**Indexing Status:**
- Pages crawled today/week
- Crawl frequency trends
- Coverage analysis

#### 5.2 AI Scraper Monitoring
**AI Model Activity:**
- ChatGPT scraping patterns
- Claude scraping patterns
- Perplexity scraping patterns
- Content preferences of AI scrapers

**Bot Traffic Composition:**
- Human vs Bot ratio trends
- Bot type distribution
- Suspicious bot activity alerts

---

### 6. Performance & Quality Analytics

#### 6.1 Response Time Analysis
**Latency Metrics:**
- P50, P95, P99 response times
- Response time by endpoint
- Response time trends
- Performance degradation alerts

#### 6.2 Error & Success Tracking
**Error Analysis:**
- Error rate trends
- Error types distribution
- Error by endpoint
- User impact of errors

**Success Metrics:**
- Overall success rate
- Success rate by session type
- Success rate by client
- Quality score trends

#### 6.3 Cost Analytics
**AI Cost Tracking:**
- Total cost trends (daily, weekly, monthly)
- Cost per session (human sessions only)
- Cost per user
- Token efficiency metrics
- Model cost comparison

**Resource Utilization:**
- Tokens per request
- Request volume trends
- High-cost sessions identification
- Cost optimization opportunities

---

### 7. Real-Time & Predictive Analytics

#### 7.1 Real-Time Dashboard
**Live Metrics:**
- Users online now (last 15 min)
- Active sessions (last 1 hour)
- Requests per minute
- Current bot activity
- System health status

**Live Events:**
- Recent content views
- Recent sessions
- Recent errors
- Recent conversions

#### 7.2 Trend Forecasting
**Predictive Models:**
- Traffic growth projections
- Seasonal trend detection
- Anomaly detection (unusual patterns)
- Content performance predictions

**Alerting:**
- Traffic spike alerts
- Error rate alerts
- Cost threshold alerts
- SEO crawler health alerts

---

## Implementation Plan

### Phase 1: Enhanced Traffic & Visitor Analytics (Week 1-2)

#### Tool: Traffic Analytics (`traffic.rs`)

**Additions:**

##### 1. Visitor Journey Section
```rust
#[derive(serde::Serialize)]
struct VisitorJourney {
    total_visitors: i32,
    new_visitors: i32,
    returning_visitors: i32,
    avg_pages_per_session: f64,
    bounce_rate: f64,
    return_visitor_rate: f64,
}

async fn get_visitor_journey(pool: &DbPool, days: i32) -> Result<VisitorJourney> {
    let query = r#"
        WITH visitor_stats AS (
            SELECT
                COUNT(DISTINCT fingerprint_hash) as total_visitors,
                COUNT(DISTINCT CASE
                    WHEN request_count = 1 THEN session_id
                END) * 100.0 / COUNT(*) as bounce_rate,
                AVG(request_count) as avg_pages_per_session
            FROM user_sessions
            WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
              AND is_bot = false
        ),
        new_vs_returning AS (
            SELECT
                COUNT(DISTINCT CASE
                    WHEN first_session THEN fingerprint_hash
                END) as new_visitors,
                COUNT(DISTINCT CASE
                    WHEN NOT first_session THEN fingerprint_hash
                END) as returning_visitors
            FROM (
                SELECT
                    fingerprint_hash,
                    ROW_NUMBER() OVER (PARTITION BY fingerprint_hash ORDER BY started_at) = 1 as first_session
                FROM user_sessions
                WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
                  AND is_bot = false
            ) sub
        )
        SELECT
            vs.total_visitors,
            nvr.new_visitors,
            nvr.returning_visitors,
            vs.avg_pages_per_session,
            vs.bounce_rate,
            (nvr.returning_visitors::float / NULLIF(vs.total_visitors, 0) * 100) as return_visitor_rate
        FROM visitor_stats vs, new_vs_returning nvr
    "#;

    let row = pool.fetch_one(&query, &[&days]).await?;

    Ok(VisitorJourney {
        total_visitors: row.get("total_visitors").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        new_visitors: row.get("new_visitors").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        returning_visitors: row.get("returning_visitors").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        avg_pages_per_session: row.get("avg_pages_per_session").and_then(|v| v.as_f64()).unwrap_or(0.0),
        bounce_rate: row.get("bounce_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
        return_visitor_rate: row.get("return_visitor_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
    })
}
```

##### 2. Traffic Sources Section
```rust
#[derive(serde::Serialize)]
struct TrafficSource {
    source_name: String,
    session_count: i32,
    unique_visitors: i32,
    avg_engagement_seconds: f64,
    bounce_rate: f64,
    percentage: f64,
}

async fn get_traffic_sources(pool: &DbPool, days: i32) -> Result<Vec<TrafficSource>> {
    let query = r#"
        SELECT
            COALESCE(
                CASE
                    WHEN referrer_source IS NULL THEN 'Direct'
                    WHEN referrer_source ILIKE '%google%' THEN 'Google'
                    WHEN referrer_source ILIKE '%bing%' THEN 'Bing'
                    WHEN referrer_source ILIKE '%facebook%' THEN 'Facebook'
                    WHEN referrer_source ILIKE '%twitter%' OR referrer_source ILIKE '%t.co%' THEN 'Twitter'
                    WHEN referrer_source ILIKE '%linkedin%' THEN 'LinkedIn'
                    ELSE referrer_source
                END,
                'Direct'
            ) as source_name,
            COUNT(*) as session_count,
            COUNT(DISTINCT fingerprint_hash) as unique_visitors,
            AVG(duration_seconds) as avg_engagement_seconds,
            (COUNT(CASE WHEN request_count = 1 THEN 1 END) * 100.0 / COUNT(*)) as bounce_rate
        FROM user_sessions
        WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
          AND is_bot = false
        GROUP BY source_name
        ORDER BY session_count DESC
        LIMIT 10
    "#;

    let rows = pool.fetch_all(&query, &[&days]).await?;
    let total_sessions: i32 = rows.iter()
        .map(|r| r.get("session_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32)
        .sum();

    Ok(rows.iter().map(|r| {
        let count = r.get("session_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        TrafficSource {
            source_name: r.get("source_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            session_count: count,
            unique_visitors: r.get("unique_visitors").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            avg_engagement_seconds: r.get("avg_engagement_seconds").and_then(|v| v.as_f64()).unwrap_or(0.0),
            bounce_rate: r.get("bounce_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
            percentage: if total_sessions > 0 { (count as f64 / total_sessions as f64) * 100.0 } else { 0.0 },
        }
    }).collect())
}
```

##### 3. UTM Campaign Performance
```rust
#[derive(serde::Serialize)]
struct CampaignPerformance {
    campaign_name: String,
    source: String,
    medium: String,
    sessions: i32,
    unique_visitors: i32,
    avg_engagement: f64,
    conversion_rate: f64,
}

async fn get_campaign_performance(pool: &DbPool, days: i32) -> Result<Vec<CampaignPerformance>> {
    let query = r#"
        SELECT
            COALESCE(utm_campaign, '(not set)') as campaign_name,
            COALESCE(utm_source, '(not set)') as source,
            COALESCE(utm_medium, '(not set)') as medium,
            COUNT(*) as sessions,
            COUNT(DISTINCT fingerprint_hash) as unique_visitors,
            AVG(duration_seconds) as avg_engagement,
            (COUNT(CASE WHEN converted_at IS NOT NULL THEN 1 END) * 100.0 / COUNT(*)) as conversion_rate
        FROM user_sessions
        WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
          AND is_bot = false
          AND (utm_campaign IS NOT NULL OR utm_source IS NOT NULL OR utm_medium IS NOT NULL)
        GROUP BY utm_campaign, utm_source, utm_medium
        ORDER BY sessions DESC
        LIMIT 20
    "#;

    let rows = pool.fetch_all(&query, &[&days]).await?;

    Ok(rows.iter().map(|r| CampaignPerformance {
        campaign_name: r.get("campaign_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
        source: r.get("source").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
        medium: r.get("medium").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
        sessions: r.get("sessions").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        unique_visitors: r.get("unique_visitors").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        avg_engagement: r.get("avg_engagement").and_then(|v| v.as_f64()).unwrap_or(0.0),
        conversion_rate: r.get("conversion_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
    }).collect())
}
```

##### 4. Landing Page Analysis
```rust
#[derive(serde::Serialize)]
struct LandingPageAnalysis {
    landing_page: String,
    sessions: i32,
    unique_visitors: i32,
    bounce_rate: f64,
    avg_session_duration: f64,
    percentage: f64,
}

async fn get_landing_pages(pool: &DbPool, days: i32) -> Result<Vec<LandingPageAnalysis>> {
    let query = r#"
        SELECT
            COALESCE(landing_page, entry_url, '(not set)') as landing_page,
            COUNT(*) as sessions,
            COUNT(DISTINCT fingerprint_hash) as unique_visitors,
            (COUNT(CASE WHEN request_count = 1 THEN 1 END) * 100.0 / COUNT(*)) as bounce_rate,
            AVG(duration_seconds) as avg_session_duration
        FROM user_sessions
        WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
          AND is_bot = false
        GROUP BY landing_page
        ORDER BY sessions DESC
        LIMIT 15
    "#;

    let rows = pool.fetch_all(&query, &[&days]).await?;
    let total: i32 = rows.iter()
        .map(|r| r.get("sessions").and_then(|v| v.as_i64()).unwrap_or(0) as i32)
        .sum();

    Ok(rows.iter().map(|r| {
        let count = r.get("sessions").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        LandingPageAnalysis {
            landing_page: r.get("landing_page").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            sessions: count,
            unique_visitors: r.get("unique_visitors").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            bounce_rate: r.get("bounce_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
            avg_session_duration: r.get("avg_session_duration").and_then(|v| v.as_f64()).unwrap_or(0.0),
            percentage: if total > 0 { (count as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }).collect())
}
```

##### 5. Time-Series Trends (Hourly, Daily, Weekly)
```rust
#[derive(serde::Serialize)]
struct TrafficTrend {
    date: String,
    hour: Option<i32>,
    sessions: i32,
    unique_visitors: i32,
    pageviews: i32,
    avg_session_duration: f64,
    bot_sessions: i32,
}

async fn get_traffic_trend(pool: &DbPool, days: i32, granularity: &str) -> Result<Vec<TrafficTrend>> {
    let (date_trunc, date_format) = match granularity {
        "hour" => ("hour", "YYYY-MM-DD HH24:00"),
        "day" => ("day", "YYYY-MM-DD"),
        "week" => ("week", "YYYY-\"W\"IW"),
        _ => ("day", "YYYY-MM-DD"),
    };

    let query = format!(r#"
        SELECT
            TO_CHAR(DATE_TRUNC('{}', started_at), '{}') as date,
            {} as hour,
            COUNT(CASE WHEN is_bot = false THEN 1 END) as sessions,
            COUNT(DISTINCT CASE WHEN is_bot = false THEN fingerprint_hash END) as unique_visitors,
            SUM(CASE WHEN is_bot = false THEN request_count ELSE 0 END) as pageviews,
            AVG(CASE WHEN is_bot = false THEN duration_seconds END) as avg_session_duration,
            COUNT(CASE WHEN is_bot = true THEN 1 END) as bot_sessions
        FROM user_sessions
        WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        GROUP BY date{}
        ORDER BY date DESC
    "#,
        date_trunc,
        date_format,
        if granularity == "hour" { "EXTRACT(HOUR FROM started_at)" } else { "NULL" },
        if granularity == "hour" { ", hour" } else { "" }
    );

    let rows = pool.fetch_all(&query, &[&days]).await?;

    Ok(rows.iter().map(|r| TrafficTrend {
        date: r.get("date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        hour: r.get("hour").and_then(|v| v.as_i64()).map(|v| v as i32),
        sessions: r.get("sessions").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        unique_visitors: r.get("unique_visitors").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        pageviews: r.get("pageviews").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        avg_session_duration: r.get("avg_session_duration").and_then(|v| v.as_f64()).unwrap_or(0.0),
        bot_sessions: r.get("bot_sessions").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
    }).collect())
}
```

---

### Phase 2: Content Performance Analytics (Week 3-4)

#### New Tool: Content Analytics (`content.rs`)

**Purpose:** Comprehensive content performance tracking

**Input Schema:**
```rust
pub fn content_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "time_range": {
                "type": "string",
                "enum": ["7d", "30d", "90d", "all"],
                "default": "30d"
            },
            "content_type": {
                "type": "string",
                "enum": ["all", "article", "tutorial", "guide"],
                "default": "all"
            },
            "sort_by": {
                "type": "string",
                "enum": ["views", "engagement", "trending"],
                "default": "views"
            }
        }
    })
}
```

**Sections:**

##### 1. Top Content by Views
```rust
#[derive(serde::Serialize)]
struct ContentPerformance {
    content_id: String,
    title: String,
    slug: String,
    content_type: String,
    category: Option<String>,
    total_views: i32,
    unique_viewers: i32,
    avg_time_on_page: f64,
    avg_scroll_depth: f64,
    engagement_score: f64,
    bounce_rate: f64,
}

async fn get_top_content(pool: &DbPool, days: i32, sort_by: &str) -> Result<Vec<ContentPerformance>> {
    let order_by = match sort_by {
        "engagement" => "engagement_score DESC",
        "trending" => "views_growth DESC",
        _ => "total_views DESC",
    };

    let query = format!(r#"
        SELECT
            mc.id as content_id,
            mc.title,
            mc.slug,
            mc.content_type,
            mc.category_id as category,
            COUNT(DISTINCT cv.id) as total_views,
            COUNT(DISTINCT cv.session_id) as unique_viewers,
            AVG(cv.time_on_page_seconds) as avg_time_on_page,
            AVG(cv.scroll_depth_percent) as avg_scroll_depth,
            (
                (AVG(cv.time_on_page_seconds) / 60.0) * 0.4 +
                (AVG(cv.scroll_depth_percent) / 100.0) * 0.3 +
                (COUNT(DISTINCT cv.session_id)::float / GREATEST(COUNT(DISTINCT cv.id), 1)) * 0.3
            ) * 100 as engagement_score,
            (
                COUNT(DISTINCT CASE
                    WHEN cv.time_on_page_seconds < 30 AND cv.scroll_depth_percent < 25
                    THEN cv.session_id
                END) * 100.0 / GREATEST(COUNT(DISTINCT cv.session_id), 1)
            ) as bounce_rate
        FROM markdown_content mc
        LEFT JOIN content_view_events cv ON mc.id = cv.content_id
        WHERE cv.viewed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        GROUP BY mc.id, mc.title, mc.slug, mc.content_type, mc.category_id
        ORDER BY {}
        LIMIT 20
    "#, order_by);

    let rows = pool.fetch_all(&query, &[&days]).await?;

    Ok(rows.iter().map(|r| ContentPerformance {
        content_id: r.get("content_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        title: r.get("title").and_then(|v| v.as_str()).unwrap_or("Untitled").to_string(),
        slug: r.get("slug").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        content_type: r.get("content_type").and_then(|v| v.as_str()).unwrap_or("article").to_string(),
        category: r.get("category").and_then(|v| v.as_str()).map(|s| s.to_string()),
        total_views: r.get("total_views").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        unique_viewers: r.get("unique_viewers").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        avg_time_on_page: r.get("avg_time_on_page").and_then(|v| v.as_f64()).unwrap_or(0.0),
        avg_scroll_depth: r.get("avg_scroll_depth").and_then(|v| v.as_f64()).unwrap_or(0.0),
        engagement_score: r.get("engagement_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
        bounce_rate: r.get("bounce_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
    }).collect())
}
```

##### 2. Content Categories Performance
```rust
#[derive(serde::Serialize)]
struct CategoryPerformance {
    category_name: String,
    article_count: i32,
    total_views: i32,
    avg_engagement_score: f64,
    avg_time_on_page: f64,
}

async fn get_category_performance(pool: &DbPool, days: i32) -> Result<Vec<CategoryPerformance>> {
    let query = r#"
        SELECT
            COALESCE(mcat.name, 'Uncategorized') as category_name,
            COUNT(DISTINCT mc.id) as article_count,
            COUNT(DISTINCT cv.id) as total_views,
            AVG(
                (cv.time_on_page_seconds / 60.0) * 0.5 +
                (cv.scroll_depth_percent / 100.0) * 0.5
            ) * 100 as avg_engagement_score,
            AVG(cv.time_on_page_seconds) as avg_time_on_page
        FROM markdown_content mc
        LEFT JOIN markdown_categories mcat ON mc.category_id = mcat.id
        LEFT JOIN content_view_events cv ON mc.id = cv.content_id
        WHERE cv.viewed_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
        GROUP BY mcat.name
        ORDER BY total_views DESC
    "#;

    let rows = pool.fetch_all(&query, &[&days]).await?;

    Ok(rows.iter().map(|r| CategoryPerformance {
        category_name: r.get("category_name").and_then(|v| v.as_str()).unwrap_or("Uncategorized").to_string(),
        article_count: r.get("article_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        total_views: r.get("total_views").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        avg_engagement_score: r.get("avg_engagement_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
        avg_time_on_page: r.get("avg_time_on_page").and_then(|v| v.as_f64()).unwrap_or(0.0),
    }).collect())
}
```

##### 3. Content Trends (New vs Evergreen)
```rust
#[derive(serde::Serialize)]
struct ContentTrend {
    content_id: String,
    title: String,
    published_at: String,
    days_since_publish: i32,
    views_7d: i32,
    views_30d: i32,
    growth_rate: f64,
    content_status: String, // "new", "trending", "evergreen", "declining"
}

async fn get_content_trends(pool: &DbPool) -> Result<Vec<ContentTrend>> {
    let query = r#"
        SELECT
            mc.id as content_id,
            mc.title,
            mc.published_at,
            EXTRACT(DAY FROM CURRENT_TIMESTAMP - mc.published_at::timestamp) as days_since_publish,
            COUNT(DISTINCT CASE
                WHEN cv.viewed_at >= CURRENT_TIMESTAMP - INTERVAL '7 days'
                THEN cv.id
            END) as views_7d,
            COUNT(DISTINCT CASE
                WHEN cv.viewed_at >= CURRENT_TIMESTAMP - INTERVAL '30 days'
                THEN cv.id
            END) as views_30d,
            CASE
                WHEN mc.published_at::timestamp >= CURRENT_TIMESTAMP - INTERVAL '7 days' THEN 'new'
                WHEN COUNT(DISTINCT CASE WHEN cv.viewed_at >= CURRENT_TIMESTAMP - INTERVAL '7 days' THEN cv.id END) >
                     COUNT(DISTINCT CASE WHEN cv.viewed_at >= CURRENT_TIMESTAMP - INTERVAL '14 days' AND cv.viewed_at < CURRENT_TIMESTAMP - INTERVAL '7 days' THEN cv.id END) * 1.5
                THEN 'trending'
                WHEN mc.published_at::timestamp < CURRENT_TIMESTAMP - INTERVAL '90 days'
                     AND COUNT(DISTINCT CASE WHEN cv.viewed_at >= CURRENT_TIMESTAMP - INTERVAL '7 days' THEN cv.id END) > 10
                THEN 'evergreen'
                ELSE 'declining'
            END as content_status
        FROM markdown_content mc
        LEFT JOIN content_view_events cv ON mc.id = cv.content_id
        WHERE cv.viewed_at >= CURRENT_TIMESTAMP - INTERVAL '30 days'
        GROUP BY mc.id, mc.title, mc.published_at
        ORDER BY views_7d DESC
        LIMIT 50
    "#;

    let rows = pool.fetch_all(&query, &[]).await?;

    Ok(rows.iter().map(|r| {
        let views_7d = r.get("views_7d").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let views_30d = r.get("views_30d").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

        ContentTrend {
            content_id: r.get("content_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            title: r.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            published_at: r.get("published_at").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            days_since_publish: r.get("days_since_publish").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            views_7d,
            views_30d,
            growth_rate: if views_30d > 0 { (views_7d as f64 / views_30d as f64) * 100.0 } else { 0.0 },
            content_status: r.get("content_status").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        }
    }).collect())
}
```

---

### Phase 3: Enhanced Dashboard with Real-Time Monitoring (Week 5)

#### Tool: Dashboard (`dashboard.rs` enhancements)

**New Sections:**

##### 1. Real-Time Activity (15-minute window)
```rust
#[derive(serde::Serialize)]
struct RealTimeActivity {
    users_online_now: i32,
    sessions_last_hour: i32,
    pageviews_last_15min: i32,
    top_content_now: Vec<String>,
    active_countries: Vec<String>,
}

async fn get_realtime_activity(pool: &DbPool) -> Result<RealTimeActivity> {
    // Users online (last 15 min activity)
    let users_query = r#"
        SELECT COUNT(DISTINCT fingerprint_hash) as count
        FROM user_sessions
        WHERE last_activity_at >= CURRENT_TIMESTAMP - INTERVAL '15 minutes'
          AND is_bot = false
    "#;
    let users_row = pool.fetch_one(&users_query, &[]).await?;
    let users_online = users_row.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    // Sessions last hour
    let sessions_query = r#"
        SELECT COUNT(*) as count
        FROM user_sessions
        WHERE started_at >= CURRENT_TIMESTAMP - INTERVAL '1 hour'
          AND is_bot = false
    "#;
    let sessions_row = pool.fetch_one(&sessions_query, &[]).await?;
    let sessions = sessions_row.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    // Pageviews last 15 min
    let pageviews_query = r#"
        SELECT COUNT(*) as count
        FROM content_view_events
        WHERE viewed_at >= CURRENT_TIMESTAMP - INTERVAL '15 minutes'
    "#;
    let pageviews_row = pool.fetch_one(&pageviews_query, &[]).await?;
    let pageviews = pageviews_row.get("count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;

    // Top content right now
    let top_content_query = r#"
        SELECT mc.title
        FROM content_view_events cv
        JOIN markdown_content mc ON cv.content_id = mc.id
        WHERE cv.viewed_at >= CURRENT_TIMESTAMP - INTERVAL '15 minutes'
        GROUP BY mc.title
        ORDER BY COUNT(*) DESC
        LIMIT 5
    "#;
    let top_content_rows = pool.fetch_all(&top_content_query, &[]).await?;
    let top_content = top_content_rows.iter()
        .map(|r| r.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string())
        .collect();

    // Active countries
    let countries_query = r#"
        SELECT DISTINCT country
        FROM user_sessions
        WHERE last_activity_at >= CURRENT_TIMESTAMP - INTERVAL '15 minutes'
          AND is_bot = false
          AND country IS NOT NULL
        LIMIT 10
    "#;
    let countries_rows = pool.fetch_all(&countries_query, &[]).await?;
    let countries = countries_rows.iter()
        .map(|r| r.get("country").and_then(|v| v.as_str()).unwrap_or("").to_string())
        .collect();

    Ok(RealTimeActivity {
        users_online_now: users_online,
        sessions_last_hour: sessions,
        pageviews_last_15min: pageviews,
        top_content_now: top_content,
        active_countries: countries,
    })
}
```

##### 2. Key Performance Indicators (KPIs)
```rust
#[derive(serde::Serialize)]
struct KPIMetrics {
    daily_active_users: i32,
    weekly_active_users: i32,
    monthly_active_users: i32,
    dau_wau_ratio: f64, // Engagement stickiness
    avg_session_quality_score: f64,
    content_engagement_rate: f64,
    conversion_rate: f64,
}

async fn get_kpi_metrics(pool: &DbPool) -> Result<KPIMetrics> {
    let query = r#"
        WITH user_activity AS (
            SELECT
                COUNT(DISTINCT CASE
                    WHEN last_activity_at >= CURRENT_TIMESTAMP - INTERVAL '1 day'
                    THEN fingerprint_hash
                END) as dau,
                COUNT(DISTINCT CASE
                    WHEN last_activity_at >= CURRENT_TIMESTAMP - INTERVAL '7 days'
                    THEN fingerprint_hash
                END) as wau,
                COUNT(DISTINCT CASE
                    WHEN last_activity_at >= CURRENT_TIMESTAMP - INTERVAL '30 days'
                    THEN fingerprint_hash
                END) as mau,
                AVG(
                    (duration_seconds / 60.0) * 0.3 +
                    request_count * 0.3 +
                    (success_rate * 100) * 0.2 +
                    CASE WHEN ai_request_count > 0 THEN 20 ELSE 0 END
                ) as avg_quality_score,
                COUNT(CASE WHEN converted_at IS NOT NULL THEN 1 END) * 100.0 / COUNT(*) as conversion_rate
            FROM user_sessions
            WHERE is_bot = false
        ),
        content_engagement AS (
            SELECT
                COUNT(CASE WHEN time_on_page_seconds >= 30 AND scroll_depth_percent >= 25 THEN 1 END) * 100.0 /
                NULLIF(COUNT(*), 0) as engagement_rate
            FROM content_view_events
            WHERE viewed_at >= CURRENT_TIMESTAMP - INTERVAL '7 days'
        )
        SELECT
            ua.dau,
            ua.wau,
            ua.mau,
            CASE WHEN ua.wau > 0 THEN (ua.dau::float / ua.wau) ELSE 0 END as dau_wau_ratio,
            ua.avg_quality_score,
            ce.engagement_rate,
            ua.conversion_rate
        FROM user_activity ua, content_engagement ce
    "#;

    let row = pool.fetch_one(&query, &[]).await?;

    Ok(KPIMetrics {
        daily_active_users: row.get("dau").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        weekly_active_users: row.get("wau").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        monthly_active_users: row.get("mau").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        dau_wau_ratio: row.get("dau_wau_ratio").and_then(|v| v.as_f64()).unwrap_or(0.0),
        avg_session_quality_score: row.get("avg_quality_score").and_then(|v| v.as_f64()).unwrap_or(0.0),
        content_engagement_rate: row.get("engagement_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
        conversion_rate: row.get("conversion_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
    })
}
```

---

### Phase 4: User Behavior & Cohort Analysis (Week 6)

#### New Tool: User Insights (`user_insights.rs`)

**Sections:**

##### 1. User Segmentation
```rust
#[derive(serde::Serialize)]
struct UserSegment {
    segment_name: String,
    user_count: i32,
    avg_sessions_per_user: f64,
    avg_engagement_score: f64,
    conversion_rate: f64,
    percentage: f64,
}

async fn get_user_segments(pool: &DbPool, days: i32) -> Result<Vec<UserSegment>> {
    let query = r#"
        WITH user_metrics AS (
            SELECT
                fingerprint_hash,
                COUNT(*) as session_count,
                AVG(duration_seconds) as avg_duration,
                AVG(request_count) as avg_pageviews,
                MAX(CASE WHEN converted_at IS NOT NULL THEN 1 ELSE 0 END) as converted
            FROM user_sessions
            WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
              AND is_bot = false
            GROUP BY fingerprint_hash
        ),
        segmented AS (
            SELECT
                CASE
                    WHEN session_count >= 10 AND avg_duration >= 300 THEN 'Power Users'
                    WHEN session_count >= 5 OR avg_pageviews >= 5 THEN 'Engaged Users'
                    WHEN session_count >= 2 THEN 'Regular Users'
                    WHEN avg_duration < 60 AND avg_pageviews <= 1 THEN 'Bounced Users'
                    ELSE 'Casual Users'
                END as segment,
                session_count,
                (avg_duration / 60.0 + avg_pageviews) / 2 as engagement_score,
                converted
            FROM user_metrics
        )
        SELECT
            segment as segment_name,
            COUNT(*) as user_count,
            AVG(session_count) as avg_sessions,
            AVG(engagement_score) as avg_engagement,
            SUM(converted) * 100.0 / COUNT(*) as conversion_rate
        FROM segmented
        GROUP BY segment
        ORDER BY
            CASE segment
                WHEN 'Power Users' THEN 1
                WHEN 'Engaged Users' THEN 2
                WHEN 'Regular Users' THEN 3
                WHEN 'Casual Users' THEN 4
                WHEN 'Bounced Users' THEN 5
            END
    "#;

    let rows = pool.fetch_all(&query, &[&days]).await?;
    let total: i32 = rows.iter()
        .map(|r| r.get("user_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32)
        .sum();

    Ok(rows.iter().map(|r| {
        let count = r.get("user_count").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        UserSegment {
            segment_name: r.get("segment_name").and_then(|v| v.as_str()).unwrap_or("Unknown").to_string(),
            user_count: count,
            avg_sessions_per_user: r.get("avg_sessions").and_then(|v| v.as_f64()).unwrap_or(0.0),
            avg_engagement_score: r.get("avg_engagement").and_then(|v| v.as_f64()).unwrap_or(0.0),
            conversion_rate: r.get("conversion_rate").and_then(|v| v.as_f64()).unwrap_or(0.0),
            percentage: if total > 0 { (count as f64 / total as f64) * 100.0 } else { 0.0 },
        }
    }).collect())
}
```

##### 2. Cohort Retention Analysis
```rust
#[derive(serde::Serialize)]
struct CohortRetention {
    cohort_date: String,
    cohort_size: i32,
    day_1_retention: f64,
    day_7_retention: f64,
    day_30_retention: f64,
}

async fn get_cohort_retention(pool: &DbPool) -> Result<Vec<CohortRetention>> {
    let query = r#"
        WITH first_sessions AS (
            SELECT
                fingerprint_hash,
                DATE(MIN(started_at)) as cohort_date,
                MIN(started_at) as first_seen
            FROM user_sessions
            WHERE is_bot = false
              AND started_at >= CURRENT_DATE - INTERVAL '90 days'
            GROUP BY fingerprint_hash
        ),
        return_activity AS (
            SELECT
                fs.cohort_date,
                fs.fingerprint_hash,
                MAX(CASE
                    WHEN us.started_at >= fs.first_seen + INTERVAL '1 day'
                     AND us.started_at < fs.first_seen + INTERVAL '2 days'
                    THEN 1 ELSE 0
                END) as returned_d1,
                MAX(CASE
                    WHEN us.started_at >= fs.first_seen + INTERVAL '7 days'
                     AND us.started_at < fs.first_seen + INTERVAL '8 days'
                    THEN 1 ELSE 0
                END) as returned_d7,
                MAX(CASE
                    WHEN us.started_at >= fs.first_seen + INTERVAL '30 days'
                     AND us.started_at < fs.first_seen + INTERVAL '31 days'
                    THEN 1 ELSE 0
                END) as returned_d30
            FROM first_sessions fs
            LEFT JOIN user_sessions us ON fs.fingerprint_hash = us.fingerprint_hash
                AND us.is_bot = false
            GROUP BY fs.cohort_date, fs.fingerprint_hash
        )
        SELECT
            cohort_date::text,
            COUNT(DISTINCT fingerprint_hash) as cohort_size,
            SUM(returned_d1) * 100.0 / COUNT(*) as day_1_retention,
            SUM(returned_d7) * 100.0 / COUNT(*) as day_7_retention,
            SUM(returned_d30) * 100.0 / COUNT(*) as day_30_retention
        FROM return_activity
        GROUP BY cohort_date
        ORDER BY cohort_date DESC
        LIMIT 30
    "#;

    let rows = pool.fetch_all(&query, &[]).await?;

    Ok(rows.iter().map(|r| CohortRetention {
        cohort_date: r.get("cohort_date").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        cohort_size: r.get("cohort_size").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
        day_1_retention: r.get("day_1_retention").and_then(|v| v.as_f64()).unwrap_or(0.0),
        day_7_retention: r.get("day_7_retention").and_then(|v| v.as_f64()).unwrap_or(0.0),
        day_30_retention: r.get("day_30_retention").and_then(|v| v.as_f64()).unwrap_or(0.0),
    }).collect())
}
```

---

## Tool Structure & Organization

### Updated Tool List

1. **dashboard** - Real-time overview + KPIs
2. **traffic** - Visitor analytics, sources, attribution
3. **content** - Content performance, engagement, trends (NEW)
4. **user_insights** - Segmentation, cohorts, behavior (NEW)
5. **conversations** - Agent/AI conversation analytics (existing)
6. **subjects** - Conversation topics (existing)
7. **users** - User management (existing)
8. **activity_summary** - Quick text summary (existing)

---

## Success Metrics

### Analytics Quality Benchmarks

**Completeness:**
- ✅ 100% session tracking (all visits recorded)
- ✅ Bot/human separation (66.7% bot detection rate)
- ✅ Geographic coverage (IP → country/city mapping)
- ✅ Device detection (mobile/desktop/tablet)
- ✅ Content engagement tracking (time, scroll depth)
- ✅ Attribution tracking (referrer, UTM parameters)

**Actionability:**
- Traffic insights drive content strategy decisions
- Engagement metrics identify top-performing content
- Cohort analysis reveals retention patterns
- Attribution data optimizes marketing spend
- Bot monitoring ensures SEO health

**Performance:**
- Dashboard load < 500ms
- Real-time updates (15-minute lag max)
- Historical trend analysis (90+ days)
- Query optimization with indexes

---

## Implementation Timeline

| Phase | Weeks | Deliverables | Effort |
|-------|-------|--------------|--------|
| **Phase 1** | 1-2 | Enhanced Traffic Analytics | 16h |
| **Phase 2** | 3-4 | Content Analytics Tool | 20h |
| **Phase 3** | 5 | Real-Time Dashboard | 12h |
| **Phase 4** | 6 | User Insights & Cohorts | 16h |
| **Testing** | 7 | Integration & Performance | 8h |
| **Documentation** | 8 | User guides, examples | 4h |
| **Total** | 8 weeks | Complete analytics platform | **76h** |

---

## Database Optimizations Required

### New Indexes
```sql
-- Traffic source analysis
CREATE INDEX IF NOT EXISTS idx_sessions_referrer ON user_sessions(referrer_source, started_at);
CREATE INDEX IF NOT EXISTS idx_sessions_utm ON user_sessions(utm_source, utm_campaign, started_at);

-- Landing page analysis
CREATE INDEX IF NOT EXISTS idx_sessions_landing ON user_sessions(landing_page, is_bot);

-- Engagement metrics
CREATE INDEX IF NOT EXISTS idx_sessions_engagement ON user_sessions(duration_seconds, request_count, is_bot);

-- Content views optimization
CREATE INDEX IF NOT EXISTS idx_content_views_time ON content_view_events(viewed_at DESC, content_id);
CREATE INDEX IF NOT EXISTS idx_content_views_engagement ON content_view_events(time_on_page_seconds, scroll_depth_percent);

-- Cohort analysis
CREATE INDEX IF NOT EXISTS idx_sessions_fingerprint_time ON user_sessions(fingerprint_hash, started_at);
```

### New Views
```sql
-- Daily traffic summary view
CREATE VIEW v_daily_traffic_summary AS
SELECT
    DATE(started_at) as date,
    COUNT(CASE WHEN is_bot = false THEN 1 END) as human_sessions,
    COUNT(DISTINCT CASE WHEN is_bot = false THEN fingerprint_hash END) as unique_visitors,
    SUM(CASE WHEN is_bot = false THEN request_count ELSE 0 END) as pageviews,
    AVG(CASE WHEN is_bot = false THEN duration_seconds END) as avg_duration
FROM user_sessions
GROUP BY DATE(started_at)
ORDER BY date DESC;

-- Content performance summary view
CREATE VIEW v_content_performance AS
SELECT
    mc.id,
    mc.title,
    mc.slug,
    mc.content_type,
    COUNT(DISTINCT cv.id) as total_views,
    COUNT(DISTINCT cv.session_id) as unique_viewers,
    AVG(cv.time_on_page_seconds) as avg_time_on_page,
    AVG(cv.scroll_depth_percent) as avg_scroll_depth
FROM markdown_content mc
LEFT JOIN content_view_events cv ON mc.id = cv.content_id
WHERE cv.viewed_at >= CURRENT_DATE - INTERVAL '30 days'
GROUP BY mc.id, mc.title, mc.slug, mc.content_type;
```

---

## Visualization Recommendations

### Dashboard Layouts

**Traffic Tool:**
- Top: KPI cards (visitors, sessions, pageviews, bounce rate)
- Middle: Traffic trend chart (time-series)
- Sections: Sources table, Geo map, Device pie chart, Landing pages list

**Content Tool:**
- Top: Content KPIs (total articles, avg engagement, top performer)
- Middle: Top content table with engagement scores
- Sections: Category performance, Trending content, Content lifecycle chart

**User Insights:**
- Top: Segmentation pie chart
- Middle: Segment comparison table
- Bottom: Cohort retention heatmap

**Dashboard:**
- Top: Real-time activity cards (live users, active sessions)
- Middle: Multi-dimensional KPIs
- Sections: Traffic overview, Content highlights, Bot monitoring, System health

---

## Next Steps

1. **Review & Approve** this plan
2. **Prioritize phases** based on business needs
3. **Allocate resources** (developer time)
4. **Create database indexes** (can be done immediately)
5. **Begin Phase 1** implementation
6. **Iterate based on feedback**

---

## References

**Existing Analytics:**
- Traffic queries: `/core/crates/modules/core/src/queries/analytics/traffic/`
- Core stats: `/core/crates/modules/core/src/queries/analytics/core_stats/`
- Content analytics: `/core/crates/modules/blog/src/queries/core/analytics/`

**Database Schema:**
- Sessions: `user_sessions` (38 columns)
- Content views: `content_view_events` (11 columns)
- Content metadata: `markdown_content` (18 columns)

**Analytics Inspiration:**
- Google Analytics (GA4) - Industry standard
- Plausible Analytics - Privacy-focused
- PostHog - Product analytics
- Mixpanel - Event tracking

---

**Status:** ✅ Ready for Review & Implementation
**Last Updated:** 2025-11-10
**Prepared By:** SystemPrompt Analytics Team
