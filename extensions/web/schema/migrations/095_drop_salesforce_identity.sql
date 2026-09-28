-- Drop salesforce_user_identities.
--
-- The table mapped a user to their Salesforce Username for the RFC 7523
-- JWT-bearer pre-authorization of a Salesforce MCP org (created by
-- 067_dashboard_salesforce_identity and later declared by the retired
-- schema/21_salesforce_identity.sql). The Salesforce identity, org registry and
-- connector provider were removed from the template with it, so nothing reads
-- or writes the table any more. Its only index is the primary key, which goes
-- with the table.
DROP TABLE IF EXISTS salesforce_user_identities;
