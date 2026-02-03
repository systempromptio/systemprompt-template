-- Blog extension: engagement_events table
-- Stores client-side engagement tracking data

CREATE TABLE IF NOT EXISTS engagement_events (
    id TEXT PRIMARY KEY,
    session_id TEXT,
    page_url TEXT NOT NULL,
    event_type TEXT NOT NULL,  -- page_view, page_exit, scroll, link_click
    referrer TEXT,

    -- Scroll metrics
    scroll_depth INTEGER,
    scroll_velocity_avg INTEGER,
    scroll_direction_changes INTEGER,

    -- Time metrics
    time_on_page_ms INTEGER,
    time_to_first_interaction_ms INTEGER,
    time_to_first_scroll_ms INTEGER,
    visible_time_ms INTEGER,
    hidden_time_ms INTEGER,

    -- Interaction metrics
    mouse_move_distance_px INTEGER,
    keyboard_events INTEGER,
    copy_events INTEGER,
    click_count INTEGER,

    -- Behavioral flags
    is_rage_click BOOLEAN DEFAULT FALSE,
    is_dead_click BOOLEAN DEFAULT FALSE,
    reading_pattern TEXT,  -- bounce, skimmer, scanner, reader, engaged

    -- Link click specific (when event_type = 'link_click')
    target_url TEXT,
    link_text TEXT,
    is_external BOOLEAN,

    -- Additional data as JSON
    data JSONB,

    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_engagement_events_session_id ON engagement_events(session_id);
CREATE INDEX IF NOT EXISTS idx_engagement_events_page_url ON engagement_events(page_url);
CREATE INDEX IF NOT EXISTS idx_engagement_events_event_type ON engagement_events(event_type);
CREATE INDEX IF NOT EXISTS idx_engagement_events_created_at ON engagement_events(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_engagement_events_reading_pattern ON engagement_events(reading_pattern);

-- View for page-level engagement summary
CREATE OR REPLACE VIEW v_page_engagement_summary AS
SELECT
    page_url,
    COUNT(*) FILTER (WHERE event_type = 'page_view') as page_views,
    COUNT(*) FILTER (WHERE event_type = 'page_exit') as page_exits,
    AVG(scroll_depth) FILTER (WHERE event_type = 'page_exit') as avg_scroll_depth,
    AVG(time_on_page_ms) FILTER (WHERE event_type = 'page_exit') as avg_time_on_page_ms,
    COUNT(*) FILTER (WHERE reading_pattern = 'engaged') as engaged_readers,
    COUNT(*) FILTER (WHERE reading_pattern = 'reader') as readers,
    COUNT(*) FILTER (WHERE reading_pattern = 'scanner') as scanners,
    COUNT(*) FILTER (WHERE reading_pattern = 'skimmer') as skimmers,
    COUNT(*) FILTER (WHERE reading_pattern = 'bounce') as bounces,
    COUNT(*) FILTER (WHERE is_rage_click = true) as rage_clicks,
    COUNT(*) FILTER (WHERE is_dead_click = true) as dead_clicks
FROM engagement_events
GROUP BY page_url;
