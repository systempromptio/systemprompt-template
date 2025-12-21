-- SystemPrompt PostgreSQL Extensions
-- This script runs automatically when the container is first created

-- Enable UUID generation
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enable full-text search with unaccent
CREATE EXTENSION IF NOT EXISTS "unaccent";

-- Enable trigram similarity for fuzzy matching
CREATE EXTENSION IF NOT EXISTS "pg_trgm";
