-- Gateway routes carry an ordered `fallbacks: [{provider, upstream_model}]`
-- chain and optional `by_scope` per-scope chains. A stored single
-- `fallback_provider` / `fallback_upstream_model` pair becomes a one-entry
-- chain, and the two columns are dropped.

ALTER TABLE gateway_routes ADD COLUMN IF NOT EXISTS fallbacks JSONB NOT NULL DEFAULT '[]'::jsonb;
ALTER TABLE gateway_routes ADD COLUMN IF NOT EXISTS by_scope JSONB;

DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
         WHERE table_schema = current_schema()
           AND table_name = 'gateway_routes'
           AND column_name = 'fallback_provider'
    ) THEN
        UPDATE gateway_routes
           SET fallbacks = jsonb_build_array(jsonb_strip_nulls(jsonb_build_object(
                   'provider', fallback_provider,
                   'upstream_model', fallback_upstream_model)))
         WHERE fallback_provider IS NOT NULL;
    END IF;
END
$$;

ALTER TABLE gateway_routes DROP COLUMN IF EXISTS fallback_provider;
ALTER TABLE gateway_routes DROP COLUMN IF EXISTS fallback_upstream_model;
