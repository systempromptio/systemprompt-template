-- Blog extension: link_clicks table
-- Records individual click events for campaign links

CREATE TABLE IF NOT EXISTS link_clicks (
    id TEXT PRIMARY KEY,

    link_id TEXT NOT NULL,

    session_id TEXT NOT NULL,
    user_id TEXT,
    context_id TEXT,
    task_id TEXT,

    referrer_page TEXT,
    referrer_url TEXT,
    clicked_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,

    user_agent TEXT,
    ip_address TEXT,
    device_type TEXT,
    country TEXT,

    is_first_click BOOLEAN NOT NULL DEFAULT FALSE,
    is_conversion BOOLEAN NOT NULL DEFAULT FALSE,
    conversion_at TIMESTAMPTZ,

    time_on_page_seconds INTEGER,
    scroll_depth_percent INTEGER,

    FOREIGN KEY (link_id) REFERENCES campaign_links(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_link_clicks_link_id ON link_clicks(link_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_session_id ON link_clicks(session_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_user_id ON link_clicks(user_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_context_id ON link_clicks(context_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_task_id ON link_clicks(task_id);
CREATE INDEX IF NOT EXISTS idx_link_clicks_clicked_at ON link_clicks(clicked_at DESC);
CREATE INDEX IF NOT EXISTS idx_link_clicks_conversion ON link_clicks(is_conversion) WHERE is_conversion = TRUE;
CREATE INDEX IF NOT EXISTS idx_link_clicks_link_session ON link_clicks(link_id, session_id);
