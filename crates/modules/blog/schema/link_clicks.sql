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
    clicked_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- User context (copied from session for performance)
    user_agent TEXT,
    ip_address TEXT,
    device_type TEXT,
    country TEXT,

    -- Journey tracking
    is_first_click BOOLEAN DEFAULT FALSE, -- First time this session clicked this link
    is_conversion BOOLEAN DEFAULT FALSE, -- Did this click lead to a conversion?
    conversion_at TIMESTAMP,

    -- Analytics
    time_on_page_seconds INTEGER, -- How long they spent on the page before clicking
    scroll_depth_percent INTEGER, -- How far they scrolled (if we track it client-side)

    -- Foreign keys
    FOREIGN KEY (link_id) REFERENCES campaign_links(id) ON DELETE CASCADE
    -- Note: session_id does not have FK constraint (removed in migration 004)
    -- This allows clicks to be recorded even if session doesn't exist yet
);

-- Indexes for analytics queries
CREATE INDEX IF NOT EXISTS idx_link_clicks_link_id ON link_clicks(link_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_session_id ON link_clicks(session_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_user_id ON link_clicks(user_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_context_id ON link_clicks(context_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_task_id ON link_clicks(task_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_clicked_at ON link_clicks(clicked_at DESC);
CREATE INDEX IF NOT EXISTS idx_link_clicks_conversion ON link_clicks(is_conversion) WHERE is_conversion = TRUE;
CREATE INDEX IF NOT EXISTS idx_link_clicks_link_session ON link_clicks(link_id, session_id);
