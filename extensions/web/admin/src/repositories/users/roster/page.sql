WITH base AS (
               SELECT u.id,
                      COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                      u.email, u.roles, (u.status = 'active') AS is_active, u.created_at
                 FROM users u
                WHERE NOT ('anonymous' = ANY(u.roles))
                  AND u.email NOT LIKE '%@anonymous.local'
                  AND ($1::TEXT[] IS NULL OR u.id = ANY($1))
                  AND ($3::TEXT IS NULL OR $3 = ANY(u.roles))
                  AND ($4::TEXT IS NULL OR position(lower($4) in lower(
                      COALESCE(u.display_name, u.full_name, u.name, '') || ' ' || COALESCE(u.email, '') || ' ' || u.id)) > 0)
           ), enriched AS (
               SELECT b.id, b.display_name, b.email, b.roles, b.is_active, b.created_at,
                      COALESCE(g.ids, ARRAY[]::TEXT[]) AS group_ids,
                      COALESCE(p.ids, ARRAY[]::TEXT[]) AS project_ids,
                      GREATEST(ls.last_session, a.last_seen) AS last_active,
                      CASE
                        WHEN GREATEST(ls.last_session, a.last_seen) IS NULL THEN NULL
                        WHEN ls.last_session = GREATEST(ls.last_session, a.last_seen) THEN 'session'
                        ELSE 'console'
                      END AS last_active_source
                 FROM base b
                 LEFT JOIN LATERAL (
                     SELECT array_agg(DISTINCT ug.group_id) AS ids
                       FROM user_groups ug WHERE ug.user_id = b.id) g ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT array_agg(DISTINCT pm.project_id) AS ids
                       FROM project_members pm WHERE pm.user_id = b.id) p ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT MAX(ua.created_at) AS last_seen
                       FROM user_activity ua WHERE ua.user_id = b.id) a ON TRUE
                 LEFT JOIN LATERAL (
                     SELECT MAX(us.last_activity_at) AS last_session
                       FROM user_sessions us WHERE us.user_id = b.id) ls ON TRUE
           ), filtered AS (
               SELECT * FROM enriched e
                WHERE CASE $2::TEXT
                        WHEN 'unassigned' THEN
                            cardinality(e.group_ids) = 0 OR 'unassigned' = ANY(e.group_ids)
                        WHEN 'no-role' THEN
                            cardinality(array_remove(array_remove(e.roles, 'user'), 'anonymous')) = 0
                        WHEN 'inactive-30d' THEN
                            e.last_active IS NULL OR e.last_active < NOW() - INTERVAL '30 days'
                        ELSE TRUE
                      END
                  AND ($3::TEXT IS NULL OR $3 = ANY(e.roles))
                  AND ($4::TEXT IS NULL OR position(lower($4) in lower(
                          COALESCE(e.display_name, '') || ' ' || COALESCE(e.email, '') || ' ' || e.id)) > 0)
           )
           , page AS MATERIALIZED (
               SELECT f.*, COUNT(*) OVER ()::bigint AS total_rows,
                   ROW_NUMBER() OVER (            ORDER BY
              CASE WHEN $5::TEXT = 'name'     AND     $6::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END DESC,
              CASE WHEN $5::TEXT = 'name'     AND NOT $6::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END ASC,
              CASE WHEN $5::TEXT = 'email'    AND     $6::BOOLEAN THEN lower(COALESCE(f.email, '')) END DESC,
              CASE WHEN $5::TEXT = 'email'    AND NOT $6::BOOLEAN THEN lower(COALESCE(f.email, '')) END ASC,
              CASE WHEN $5::TEXT = 'roles'    AND     $6::BOOLEAN THEN cardinality(f.roles) END DESC,
              CASE WHEN $5::TEXT = 'roles'    AND NOT $6::BOOLEAN THEN cardinality(f.roles) END ASC,
              CASE WHEN $5::TEXT = 'groups'   AND     $6::BOOLEAN THEN cardinality(f.group_ids) END DESC,
              CASE WHEN $5::TEXT = 'groups'   AND NOT $6::BOOLEAN THEN cardinality(f.group_ids) END ASC,
              CASE WHEN $5::TEXT = 'projects' AND     $6::BOOLEAN THEN cardinality(f.project_ids) END DESC,
              CASE WHEN $5::TEXT = 'projects' AND NOT $6::BOOLEAN THEN cardinality(f.project_ids) END ASC,
              CASE WHEN $5::TEXT = 'seen'     AND     $6::BOOLEAN THEN f.last_active END DESC NULLS LAST,
              CASE WHEN $5::TEXT = 'seen'     AND NOT $6::BOOLEAN THEN f.last_active END ASC NULLS FIRST,
              CASE WHEN $5::TEXT = 'status'   AND     $6::BOOLEAN THEN f.is_active::INT END DESC,
              CASE WHEN $5::TEXT = 'status'   AND NOT $6::BOOLEAN THEN f.is_active::INT END ASC,
              f.id
) AS position
             FROM filtered f
            ORDER BY
              CASE WHEN $5::TEXT = 'name'     AND     $6::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END DESC,
              CASE WHEN $5::TEXT = 'name'     AND NOT $6::BOOLEAN THEN lower(COALESCE(f.display_name, f.id)) END ASC,
              CASE WHEN $5::TEXT = 'email'    AND     $6::BOOLEAN THEN lower(COALESCE(f.email, '')) END DESC,
              CASE WHEN $5::TEXT = 'email'    AND NOT $6::BOOLEAN THEN lower(COALESCE(f.email, '')) END ASC,
              CASE WHEN $5::TEXT = 'roles'    AND     $6::BOOLEAN THEN cardinality(f.roles) END DESC,
              CASE WHEN $5::TEXT = 'roles'    AND NOT $6::BOOLEAN THEN cardinality(f.roles) END ASC,
              CASE WHEN $5::TEXT = 'groups'   AND     $6::BOOLEAN THEN cardinality(f.group_ids) END DESC,
              CASE WHEN $5::TEXT = 'groups'   AND NOT $6::BOOLEAN THEN cardinality(f.group_ids) END ASC,
              CASE WHEN $5::TEXT = 'projects' AND     $6::BOOLEAN THEN cardinality(f.project_ids) END DESC,
              CASE WHEN $5::TEXT = 'projects' AND NOT $6::BOOLEAN THEN cardinality(f.project_ids) END ASC,
              CASE WHEN $5::TEXT = 'seen'     AND     $6::BOOLEAN THEN f.last_active END DESC NULLS LAST,
              CASE WHEN $5::TEXT = 'seen'     AND NOT $6::BOOLEAN THEN f.last_active END ASC NULLS FIRST,
              CASE WHEN $5::TEXT = 'status'   AND     $6::BOOLEAN THEN f.is_active::INT END DESC,
              CASE WHEN $5::TEXT = 'status'   AND NOT $6::BOOLEAN THEN f.is_active::INT END ASC,
              f.id
            LIMIT $7 OFFSET $8
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
                  page.total_rows AS "total_rows!"
             FROM page
             ORDER BY page.position
