CREATE TABLE IF NOT EXISTS moltbook_posts (
    id TEXT PRIMARY KEY,
    moltbook_id TEXT,
    agent_id TEXT NOT NULL REFERENCES moltbook_agents(id) ON DELETE CASCADE,
    submolt TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    url TEXT,
    upvotes INTEGER DEFAULT 0,
    downvotes INTEGER DEFAULT 0,
    comments_count INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending',
    posted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT title_not_empty CHECK (title != ''),
    CONSTRAINT content_not_empty CHECK (content != ''),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'posted', 'failed', 'deleted'))
);

CREATE INDEX idx_moltbook_posts_agent_id ON moltbook_posts(agent_id);
CREATE INDEX idx_moltbook_posts_submolt ON moltbook_posts(submolt);
CREATE INDEX idx_moltbook_posts_status ON moltbook_posts(status);
CREATE INDEX idx_moltbook_posts_created_at ON moltbook_posts(created_at DESC);
CREATE INDEX idx_moltbook_posts_moltbook_id ON moltbook_posts(moltbook_id) WHERE moltbook_id IS NOT NULL;

CREATE TABLE IF NOT EXISTS moltbook_comments (
    id TEXT PRIMARY KEY,
    moltbook_id TEXT,
    post_id TEXT NOT NULL REFERENCES moltbook_posts(id) ON DELETE CASCADE,
    agent_id TEXT NOT NULL REFERENCES moltbook_agents(id) ON DELETE CASCADE,
    parent_id TEXT,
    content TEXT NOT NULL,
    upvotes INTEGER DEFAULT 0,
    downvotes INTEGER DEFAULT 0,
    status TEXT DEFAULT 'pending',
    posted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    CONSTRAINT content_not_empty CHECK (content != ''),
    CONSTRAINT valid_status CHECK (status IN ('pending', 'posted', 'failed', 'deleted'))
);

CREATE INDEX idx_moltbook_comments_post_id ON moltbook_comments(post_id);
CREATE INDEX idx_moltbook_comments_agent_id ON moltbook_comments(agent_id);
CREATE INDEX idx_moltbook_comments_parent_id ON moltbook_comments(parent_id) WHERE parent_id IS NOT NULL;
CREATE INDEX idx_moltbook_comments_status ON moltbook_comments(status);
CREATE INDEX idx_moltbook_comments_created_at ON moltbook_comments(created_at DESC);
