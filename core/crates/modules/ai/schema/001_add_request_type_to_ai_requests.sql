-- Extend ai_requests table to distinguish between text and image generation

-- Add request_type enum column
ALTER TABLE ai_requests
ADD COLUMN IF NOT EXISTS request_type VARCHAR(50) NOT NULL DEFAULT 'text_generation';

-- Add image_count for tracking batch image generation
ALTER TABLE ai_requests
ADD COLUMN IF NOT EXISTS image_count INTEGER DEFAULT 0;

-- Create index for filtering by request type
CREATE INDEX IF NOT EXISTS idx_ai_requests_request_type ON ai_requests(request_type);

-- Update comment
COMMENT ON COLUMN ai_requests.request_type IS 'Type of AI request: text_generation, image_generation';
COMMENT ON COLUMN ai_requests.image_count IS 'Number of images generated in this request (for batch generation)';
