-- Generated Images Table
-- Tracks images created via text-to-image AI services
CREATE TABLE IF NOT EXISTS generated_images (
    id SERIAL PRIMARY KEY,
    uuid VARCHAR(36) NOT NULL UNIQUE,

    -- Link to AI request
    request_id VARCHAR(36) NOT NULL,

    -- Image metadata
    prompt TEXT NOT NULL,
    model VARCHAR(255) NOT NULL,
    provider VARCHAR(50) NOT NULL,

    -- Storage information
    file_path TEXT NOT NULL,
    public_url TEXT NOT NULL,
    file_size_bytes INTEGER,
    mime_type VARCHAR(50) DEFAULT 'image/png',

    -- Generation configuration
    resolution VARCHAR(10),  -- e.g., '1K', '2K', '4K'
    aspect_ratio VARCHAR(10), -- e.g., '1:1', '16:9', '9:16'

    -- Performance metrics
    generation_time_ms INTEGER,

    -- Cost tracking
    cost_estimate DECIMAL(10, 6),

    -- User context (denormalized for analytics)
    user_id VARCHAR(255),
    session_id VARCHAR(255),
    trace_id VARCHAR(255),

    -- Lifecycle management
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,

    -- Indexes
    CONSTRAINT fk_request FOREIGN KEY (request_id) REFERENCES ai_requests(request_id) ON DELETE CASCADE
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_generated_images_request_id ON generated_images(request_id);
CREATE INDEX IF NOT EXISTS idx_generated_images_user_id ON generated_images(user_id);
CREATE INDEX IF NOT EXISTS idx_generated_images_session_id ON generated_images(session_id);
CREATE INDEX IF NOT EXISTS idx_generated_images_created_at ON generated_images(created_at);
CREATE INDEX IF NOT EXISTS idx_generated_images_expires_at ON generated_images(expires_at) WHERE expires_at IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_generated_images_provider_model ON generated_images(provider, model);

-- View for analytics
CREATE OR REPLACE VIEW image_generation_stats AS
SELECT
    provider,
    model,
    resolution,
    aspect_ratio,
    COUNT(*) as total_images,
    AVG(generation_time_ms) as avg_generation_time_ms,
    SUM(file_size_bytes) as total_storage_bytes,
    SUM(cost_estimate) as total_cost,
    DATE(created_at) as generation_date
FROM generated_images
WHERE deleted_at IS NULL
GROUP BY provider, model, resolution, aspect_ratio, DATE(created_at);
