-- Blog extension: campaign_links table
-- Trackable campaign links with UTM parameters and analytics

CREATE TABLE IF NOT EXISTS campaign_links (
    id TEXT PRIMARY KEY,

    short_code TEXT NOT NULL UNIQUE,
    target_url TEXT NOT NULL,
    link_type TEXT NOT NULL,

    campaign_id TEXT,
    campaign_name TEXT,
    source_content_id TEXT,
    source_page TEXT,

    utm_params TEXT,

    link_text TEXT,
    link_position TEXT,
    destination_type TEXT,

    click_count INTEGER NOT NULL DEFAULT 0,
    unique_click_count INTEGER NOT NULL DEFAULT 0,
    conversion_count INTEGER NOT NULL DEFAULT 0,

    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    expires_at TIMESTAMPTZ,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (source_content_id) REFERENCES markdown_content(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS idx_campaign_links_short_code ON campaign_links(short_code);
CREATE INDEX IF NOT EXISTS idx_campaign_links_campaign_id ON campaign_links(campaign_id);
CREATE INDEX IF NOT EXISTS idx_campaign_links_source_content ON campaign_links(source_content_id);
CREATE INDEX IF NOT EXISTS idx_campaign_links_target_url ON campaign_links(target_url);
CREATE INDEX IF NOT EXISTS idx_campaign_links_created ON campaign_links(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_campaign_links_active ON campaign_links(is_active) WHERE is_active = TRUE;
