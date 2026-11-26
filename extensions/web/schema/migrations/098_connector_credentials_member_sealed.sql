-- A connector grant is sealed whole under the web member's cipher: the
-- envelope carries its own nonce, so the nonce column holds the 12-byte
-- nonce of a value written before the member cipher, until
-- `admin secrets reseal --member web --apply` re-seals it, and is empty for
-- every value written since.

ALTER TABLE mcp_connector_credentials DROP CONSTRAINT IF EXISTS mcp_connector_credentials_nonce_check;
ALTER TABLE mcp_connector_credentials DROP CONSTRAINT IF EXISTS mcp_connector_credentials_nonce_length;
ALTER TABLE mcp_connector_credentials
    ADD CONSTRAINT mcp_connector_credentials_nonce_length CHECK (octet_length(nonce) IN (0, 12));

ALTER TABLE mcp_connector_oauth_states DROP CONSTRAINT IF EXISTS mcp_connector_oauth_states_nonce_check;
ALTER TABLE mcp_connector_oauth_states DROP CONSTRAINT IF EXISTS mcp_connector_oauth_states_nonce_length;
ALTER TABLE mcp_connector_oauth_states
    ADD CONSTRAINT mcp_connector_oauth_states_nonce_length CHECK (octet_length(nonce) IN (0, 12));
