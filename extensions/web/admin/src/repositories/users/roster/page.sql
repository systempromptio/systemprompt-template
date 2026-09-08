WITH base AS (
               SELECT u.id,
                      COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                      u.email, u.roles, (u.status = 'active') AS is_active, u.created_at
                 FROM users u
                WHERE NOT ('anonymous' = ANY(u.roles))
                  AND u.email NOT LIKE '%@anonymous.local'
                  AND ($1::TEXT[] IS NULL OR u.id = ANY($1))
                  AND ($4::TEXT IS NULL OR $4 = ANY(u.roles))
                  AND ($5::TEXT IS NULL OR position(lower($5) in lower(
                      COALESCE(u.display_name, u.full_name, u.name, '') || ' ' || COALESCE(u.email, '') || ' ' || u.id)) > 0)
           ), spend AS (
               SELECT ar.user_id, COUNT(*)::bigint AS requests,
                      COALESCE(SUM(ar.cost_microdollars), 0)::bigint AS cost,
                      COALESCE(SUM(COALESCE(ar.tokens_used,
                          COALESCE(ar.input_tokens, 0) + COALESCE(ar.output_tokens, 0))), 0)::bigint AS tokens
               FROM ai_requests ar JOIN base b ON b.id = ar.user_id
               WHERE ar.created_at >= NOW() - make_interval(days => $2::int)
               GROUP BY ar.user_id
           ), enriched AS (
               SELECT b.id, b.display_name, b.email, b.roles, b.is_active, b.created_at,
                      COALESCE(g.ids, ARRAY[]::TEXT[]) AS group_ids,
                      COALESCE(p.ids, ARRAY[]::TEXT[]) AS project_ids,
                      COALESCE(r.requests, 0)::BIGINT AS requests,
                      COALESCE(r.cost, 0)::BIGINT AS cost_microdollars,
                      COALESCE(r.tokens, 0)::BIGINT AS tokens,
                      GREATEST(lr.last_request, ls.last_session, a.last_seen) AS last_active,
                      CASE
                        WHEN GREATEST(lr.last_request, ls.last_session, a.last_seen) IS NULL THEN NULL
                        WHEN lr.last_request = GREATEST(lr.last_request, ls.last_session, a.last_seen) THEN 'gateway'
                        WHEN ls.last_session = GREATEST(lr.last_request, ls.last_session, a.last_seen) THEN 'session'
                        ELSE 'console'
                      END AS last_active_source
                 FROM base b
                 LEFT JOIN LATERAL (
                     SELECT array_agg(DISTINCT ug.group_id) AS ids
                       FROM user_groups ug WHERE ug.user_id = b.id) g ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT array_agg(DISTINCT pm.project_id) AS ids
                       FROM project_members pm WHERE pm.user_id = b.id) p ON TRUE
                 LEFT JOIN spend r ON r.user_id = b.id
                 LEFT JOIN LATERAL (
                     SELECT MAX(ua.created_at) AS last_seen
                       FROM user_activity ua WHERE ua.user_id = b.id) a ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT MAX(ar.created_at) AS last_request
                       FROM ai_requests ar WHERE ar.user_id = b.id) lr ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT MAX(us.last_activity_at) AS last_session
                       FROM user_sessions us WHERE us.user_id = b.id) ls ON TRUE
           ), filtered AS (
               SELECT * FROM enriched e
                WHERE CASE $3::TEXT
                        WHEN 'unassigned' THEN
                            cardinality(e.group_ids) = 0 OR 'unassigned' = ANY(e.group_ids)
                        WHEN 'no-role' THEN
                            cardinality(array_remove(array_remove(e.roles, 'user'), 'anonymous')) = 0
                        WHEN 'inactive-30d' THEN
                            e.last_active IS NULL OR e.last_active < NOW() - INTERVAL '30 days'
                        ELSE TRUE
                      END
                  AND ($4::TEXT IS NULL OR $4 = ANY(e.roles))
                  AND ($5::TEXT IS NULL OR position(lower($5) in lower(
                          COALESCE(e.display_name, '') || ' ' || COALESCE(e.email, '') || ' ' || e.id)) > 0)
           )
           , page AS MATERIALIZED (
               SELECT f.*, COUNT(*) OVER ()::bigint AS total_rows,
                   ROW_NUMBER() OVER (            ORDER BY
              CASE WHEN $6::TEXT = 'name'     AND     $7::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END DESC,
              CASE WHEN $6::TEXT = 'name'     AND NOT $7::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END ASC,
              CASE WHEN $6::TEXT = 'email'    AND     $7::BOOLEAN THEN lower(COALESCE(f.email, '')) END DESC,
              CASE WHEN $6::TEXT = 'email'    AND NOT $7::BOOLEAN THEN lower(COALESCE(f.email, '')) END ASC,
              CASE WHEN $6::TEXT = 'roles'    AND     $7::BOOLEAN THEN cardinality(f.roles) END DESC,
              CASE WHEN $6::TEXT = 'roles'    AND NOT $7::BOOLEAN THEN cardinality(f.roles) END ASC,
              CASE WHEN $6::TEXT = 'groups'   AND     $7::BOOLEAN THEN cardinality(f.group_ids) END DESC,
              CASE WHEN $6::TEXT = 'groups'   AND NOT $7::BOOLEAN THEN cardinality(f.group_ids) END ASC,
              CASE WHEN $6::TEXT = 'projects' AND     $7::BOOLEAN THEN cardinality(f.project_ids) END DESC,
              CASE WHEN $6::TEXT = 'projects' AND NOT $7::BOOLEAN THEN cardinality(f.project_ids) END ASC,
              CASE WHEN $6::TEXT = 'seen'     AND     $7::BOOLEAN THEN f.last_active END DESC NULLS LAST,
              CASE WHEN $6::TEXT = 'seen'     AND NOT $7::BOOLEAN THEN f.last_active END ASC NULLS FIRST,
              CASE WHEN $6::TEXT = 'cost'     AND     $7::BOOLEAN THEN f.cost_microdollars END DESC,
              CASE WHEN $6::TEXT = 'cost'     AND NOT $7::BOOLEAN THEN f.cost_microdollars END ASC,
              CASE WHEN $6::TEXT = 'requests' AND     $7::BOOLEAN THEN f.requests END DESC,
              CASE WHEN $6::TEXT = 'requests' AND NOT $7::BOOLEAN THEN f.requests END ASC,
              CASE WHEN $6::TEXT = 'status'   AND     $7::BOOLEAN THEN f.is_active::INT END DESC,
              CASE WHEN $6::TEXT = 'status'   AND NOT $7::BOOLEAN THEN f.is_active::INT END ASC,
              f.id
) AS position
             FROM filtered f
            ORDER BY
              CASE WHEN $6::TEXT = 'name'     AND     $7::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END DESC,
              CASE WHEN $6::TEXT = 'name'     AND NOT $7::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END ASC,
              CASE WHEN $6::TEXT = 'email'    AND     $7::BOOLEAN THEN lower(COALESCE(f.email, '')) END DESC,
              CASE WHEN $6::TEXT = 'email'    AND NOT $7::BOOLEAN THEN lower(COALESCE(f.email, '')) END ASC,
              CASE WHEN $6::TEXT = 'roles'    AND     $7::BOOLEAN THEN cardinality(f.roles) END DESC,
              CASE WHEN $6::TEXT = 'roles'    AND NOT $7::BOOLEAN THEN cardinality(f.roles) END ASC,
              CASE WHEN $6::TEXT = 'groups'   AND     $7::BOOLEAN THEN cardinality(f.group_ids) END DESC,
              CASE WHEN $6::TEXT = 'groups'   AND NOT $7::BOOLEAN THEN cardinality(f.group_ids) END ASC,
              CASE WHEN $6::TEXT = 'projects' AND     $7::BOOLEAN THEN cardinality(f.project_ids) END DESC,
              CASE WHEN $6::TEXT = 'projects' AND NOT $7::BOOLEAN THEN cardinality(f.project_ids) END ASC,
              CASE WHEN $6::TEXT = 'seen'     AND     $7::BOOLEAN THEN f.last_active END DESC NULLS LAST,
              CASE WHEN $6::TEXT = 'seen'     AND NOT $7::BOOLEAN THEN f.last_active END ASC NULLS FIRST,
              CASE WHEN $6::TEXT = 'cost'     AND     $7::BOOLEAN THEN f.cost_microdollars END DESC,
              CASE WHEN $6::TEXT = 'cost'     AND NOT $7::BOOLEAN THEN f.cost_microdollars END ASC,
              CASE WHEN $6::TEXT = 'requests' AND     $7::BOOLEAN THEN f.requests END DESC,
              CASE WHEN $6::TEXT = 'requests' AND NOT $7::BOOLEAN THEN f.requests END ASC,
              CASE WHEN $6::TEXT = 'status'   AND     $7::BOOLEAN THEN f.is_active::INT END DESC,
              CASE WHEN $6::TEXT = 'status'   AND NOT $7::BOOLEAN THEN f.is_active::INT END ASC,
              f.id
            LIMIT $8 OFFSET $9
           )
           SELECT page.id AS "user_id!: UserId",
                  page.display_name,
                  page.email AS "email?: Email",
                  page.roles AS "roles!: Vec<String>",
                  page.group_ids AS "group_ids!: Vec<String>",
                  page.project_ids AS "project_ids!: Vec<String>",
                  page.is_active AS "is_active!",
                  page.created_at AS "created_at!",
                  page.last_active,
                  page.last_active_source AS "last_active_source?",
                  lc.context_id AS "last_context_id?: ContextId",
                  page.requests AS "requests!",
                  page.tokens AS "tokens!",
                  page.cost_microdollars AS "cost_microdollars!",
                  page.total_rows AS "total_rows!"
             FROM page
             LEFT JOIN LATERAL (
                 SELECT cr.context_id FROM conversation_requests cr
                 WHERE cr.user_id = page.id AND cr.effective_kind = 'turn'
                 ORDER BY cr.created_at DESC, cr.id DESC LIMIT 1
             ) lc ON TRUE
             ORDER BY page.position
