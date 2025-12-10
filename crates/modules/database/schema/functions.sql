-- ============================================================================
-- SHARED DATABASE FUNCTIONS
-- ============================================================================
-- These functions are used by multiple modules and must be created before
-- any module that depends on them.

-- Function to automatically update the updated_at column on row update (BEFORE trigger)
CREATE OR REPLACE FUNCTION update_timestamp_trigger()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
