SELECT redirect_uri FROM oauth_client_redirect_uris WHERE client_id = $1 ORDER BY is_primary DESC, redirect_uri
