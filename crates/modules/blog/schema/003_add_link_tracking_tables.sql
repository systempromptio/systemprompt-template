-- Migration: Add link tracking tables for campaign analytics
-- Creates: campaign_links, link_clicks tables
-- Dependencies: markdown_content, user_sessions

-- Create campaign_links table
CREATE TABLE IF NOT EXISTS campaign_links (
    id TEXT PRIMARY KEY,

    -- Link configuration
    short_code TEXT NOT NULL UNIQUE,
    target_url TEXT NOT NULL,
    link_type TEXT NOT NULL, -- 'redirect', 'utm', 'both'

    -- Campaign attribution
    campaign_id TEXT,
    campaign_name TEXT,
    source_content_id TEXT, -- ID of content that contains this link
    source_page TEXT, -- URL/slug where link appears

    -- UTM parameters (JSON)
    utm_params TEXT, -- JSON: {"source": "twitter", "medium": "social", "campaign": "post123"}

    -- Link metadata
    link_text TEXT, -- The anchor text or CTA text
    link_position TEXT, -- 'header', 'body', 'footer', 'cta'
    destination_type TEXT, -- 'internal', 'external'

    -- Tracking
    click_count INTEGER DEFAULT 0,
    unique_click_count INTEGER DEFAULT 0,
    conversion_count INTEGER DEFAULT 0,

    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    expires_at TIMESTAMPTZ,

    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    -- Foreign keys
    FOREIGN KEY (source_content_id) REFERENCES markdown_content(id) ON DELETE SET NULL
);

-- Indexes for campaign_links
CREATE INDEX IF NOT EXISTS idx_campaign_links_short_code ON campaign_links(short_code);
CREATE INDEX IF NOT EXISTS idx_campaign_links_campaign_id ON campaign_links(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_links_source_content ON campaign_links(source_content_id);
CREATE INDEX IF NOT EXISTS idx_campaign_links_target_url ON campaign_links(target_url);
CREATE INDEX IF NOT EXISTS idx_campaign_links_created ON campaign_links(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_campaign_links_active ON campaign_links(is_active) WHERE is_active = TRUE;

-- Create link_clicks table
CREATE TABLE IF NOT EXISTS link_clicks (
    id TEXT PRIMARY KEY,

    -- Link reference
    link_id TEXT NOT NULL,

    -- Session/User context
    session_id TEXT NOT NULL,
    user_id TEXT,
    context_id TEXT,
    task_id TEXT,

    -- Click context
    referrer_page TEXT, -- Page user was on when they clicked
    referrer_url TEXT, -- Full referrer URL
    clicked_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,

    -- User context (copied from session for performance)
    user_agent TEXT,
    ip_address TEXT,
    device_type TEXT,
    country TEXT,

    -- Journey tracking
    is_first_click BOOLEAN DEFAULT FALSE, -- First time this session clicked this link
    is_conversion BOOLEAN DEFAULT FALSE, -- Did this click lead to a conversion?
    conversion_at TIMESTAMPTZ,

    -- Analytics
    time_on_page_seconds INTEGER, -- How long they spent on the page before clicking
    scroll_depth_percent INTEGER, -- How far they scrolled (if we track it client-side)

    -- Foreign keys
    FOREIGN KEY (link_id) REFERENCES campaign_links(id) ON DELETE CASCADE,
    FOREIGN KEY (session_id) REFERENCES user_sessions(session_id) ON DELETE CASCADE
);

-- Indexes for link_clicks
CREATE INDEX IF NOT EXISTS idx_link_clicks_link_id ON link_clicks(link_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_session_id ON link_clicks(session_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_user_id ON link_clicks(user_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_context_id ON link_clicks(context_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_task_id ON link_clicks(task_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_clicked_at ON link_clicks(clicked_at DESC);
CREATE INDEX IF NOT EXISTS idx_link_clicks_conversion ON link_clicks(is_conversion) WHERE is_conversion = TRUE;
CREATE INDEX IF NOT EXISTS idx_link_clicks_link_session ON link_clicks(link_id, session_id);
