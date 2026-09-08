WITH classified AS MATERIALIZED (
            SELECT cr.* FROM conversation_requests cr
            WHERE cr.context_id IN (
                SELECT DISTINCT context_id FROM ai_requests
                WHERE ($2::text[] IS NULL OR user_id = ANY($2)) AND context_id <> $5
            )
        ), latest AS (
            SELECT DISTINCT ON (st.session_id)
                st.id, st.session_id, st.user_id, st.model, st.captured_at,
                COALESCE(st.entries_counted, 0) AS entries_counted,
                COALESCE(st.total_input_tokens, 0) AS total_input_tokens,
                COALESCE(st.total_output_tokens, 0) AS total_output_tokens,
                CASE WHEN $1::text IS NULL THEN NULL
                     ELSE ts_rank(st.search_tsv, websearch_to_tsquery('english', $1))
                END AS rank
            FROM session_transcripts st
            WHERE ($2::text[] IS NULL OR st.user_id = ANY($2))
              AND ($1::text IS NULL
                   OR st.search_tsv @@ websearch_to_tsquery('english', $1))
            ORDER BY st.session_id, st.captured_at DESC
        ),
        transcripts AS (
            SELECT
                'transcript'::text                       AS source,
                l.session_id                             AS session_id,
                NULL::text                               AS context_id,
                l.user_id                                AS user_id,
                pss.ai_title                             AS title,
                NULL::text                               AS preview,
                l.model                                  AS model,
                COALESCE(pss.started_at, l.captured_at)  AS started_at,
                l.captured_at                            AS last_at,
                l.entries_counted::bigint                AS turns,
                l.total_input_tokens::bigint             AS tokens_in,
                l.total_output_tokens::bigint            AS tokens_out,
                0::bigint                                AS cost_microdollars,
                0::bigint                                AS side_calls,
                l.rank                                   AS rank,
                l.id AS transcript_id
            FROM latest l
            LEFT JOIN plugin_session_summaries pss ON pss.session_id = l.session_id
        ),
        gw AS (
            SELECT
                context_id,
                MAX(client_session_id)                      AS client_session_id,
                COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint  AS turns,
                COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint AS side_calls,
                COALESCE(SUM(input_tokens), 0)::bigint      AS tokens_in,
                COALESCE(SUM(output_tokens), 0)::bigint     AS tokens_out,
                COALESCE(SUM(cost_microdollars), 0)::bigint AS cost_microdollars,
                MIN(created_at)                             AS started_at,
                MAX(created_at)                             AS last_at
            FROM classified
            WHERE context_id <> $5
              AND ($2::text[] IS NULL OR user_id = ANY($2))
            GROUP BY context_id
        ),
        gw_latest AS (
            SELECT DISTINCT ON (context_id) context_id, user_id, model
            FROM classified
            WHERE context_id <> $5
            ORDER BY context_id, created_at DESC
        ),
        gateway AS (
            SELECT
                'gateway'::text        AS source,
                NULL::text             AS session_id,
                g.context_id           AS context_id,
                l.user_id              AS user_id,
                COALESCE(pss.ai_title, NULLIF(uc.name, 'Gateway conversation')) AS title,
                CASE WHEN $6::text IS NOT NULL THEN conversation_opening_prompt(g.context_id, 200) END AS preview,
                l.model                AS model,
                g.started_at           AS started_at,
                g.last_at              AS last_at,
                g.turns                AS turns,
                g.tokens_in            AS tokens_in,
                g.tokens_out           AS tokens_out,
                g.cost_microdollars    AS cost_microdollars,
                g.side_calls           AS side_calls,
                NULL::real             AS rank,
                NULL::text             AS transcript_id
            FROM gw g
            JOIN gw_latest l ON l.context_id = g.context_id
            LEFT JOIN user_contexts uc ON uc.context_id = g.context_id
            LEFT JOIN plugin_session_summaries pss ON pss.session_id = g.client_session_id
            WHERE ($7::bool OR g.turns > 0)
              AND ($6::text IS NULL
                   OR pss.ai_title ILIKE $6
                   OR uc.name ILIKE $6
                   OR conversation_opening_prompt(g.context_id, 200) ILIKE $6
                   OR g.context_id ILIKE $6
                   OR l.model ILIKE $6)
        ),
        unified AS (
            SELECT * FROM transcripts
            UNION ALL
            SELECT * FROM gateway
        )
        , counted AS MATERIALIZED (
            SELECT *, COUNT(*) OVER ()::bigint AS total_count FROM unified
        ), page AS MATERIALIZED (
            SELECT * FROM counted
            ORDER BY rank DESC NULLS LAST, last_at DESC, source, COALESCE(session_id, context_id)
            LIMIT $3 OFFSET $4
        )
        , enriched AS (SELECT
            source                          AS source,
            session_id                      AS session_id,
            context_id                      AS context_id,
            COALESCE(user_id, '')           AS user_id,
            title                           AS title,
            COALESCE(preview, CASE WHEN source = 'gateway'
                THEN conversation_opening_prompt(context_id, 200) END) AS preview,
            model                           AS model,
            started_at                      AS started_at,
            last_at                         AS last_at,
            turns                           AS turns,
            tokens_in                       AS total_input_tokens,
            tokens_out                      AS total_output_tokens,
            cost_microdollars               AS cost_microdollars,
            side_calls                      AS side_call_count,
            rank                            AS rank,
            CASE WHEN $1::text IS NOT NULL AND transcript_id IS NOT NULL THEN
                (SELECT ts_headline('english', left(st.transcript::text, 262144),
                    websearch_to_tsquery('english', $1),
                    'StartSel=[, StopSel=], MaxWords=24, MinWords=8, MaxFragments=2')
                 FROM session_transcripts st WHERE st.id = transcript_id)
            END AS snippet,
            total_count AS total_count
        FROM page
        ORDER BY rank DESC NULLS LAST, last_at DESC, source, COALESCE(session_id, context_id)
        ) SELECT jsonb_build_object(
            'items', COALESCE((SELECT jsonb_agg(to_jsonb(e) ORDER BY rank DESC NULLS LAST,
                last_at DESC, source, COALESCE(session_id, context_id)) FROM enriched e), '[]'::jsonb),
            'total', (SELECT COUNT(*) FROM counted)
        ) AS "payload!: Json<HistoryPageResult>",
          ARRAY(SELECT context_id FROM page WHERE context_id IS NOT NULL) AS "context_ids!: Vec<ContextId>"
