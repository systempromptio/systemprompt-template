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
    expires_at TIMESTAMP,

    -- Timestamps
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Foreign keys
    FOREIGN KEY (source_content_id) REFERENCES markdown_content(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_campaign_links_short_code ON campaign_links(short_code);
CREATE INDEX IF NOT EXISTS idx_campaign_links_campaign_id ON campaign_links(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_links_source_content ON campaign_links(source_content_id);
CREATE INDEX IF NOT EXISTS idx_campaign_links_target_url ON campaign_links(target_url);
CREATE INDEX IF NOT EXISTS idx_campaign_links_created ON campaign_links(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_campaign_links_active ON campaign_links(is_active) WHERE is_active = TRUE;
