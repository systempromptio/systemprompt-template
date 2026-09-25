-- What counts as a skill invocation.
--
-- A skill run reaches us as one of two client signals, and the page must count
-- both or it under-reports badly:
--
--   slash  The user typed `/plugin:skill`. Claude Code sends no tool call for
--          this at all -- the text survives only in `prompt_preview`, which is
--          the first 200 chars of the prompt kept by `generate_prompt_preview`
--          while `sanitize_metadata` strips the prompt from `metadata`. This is
--          the common case: the model reads the skill body and starts working,
--          so no `Skill` tool call is ever emitted. Measuring skills by the
--          tool call alone missed 16 of the 20 real invocations on this
--          instance.
--
--   tool   The model dispatched the `Skill` tool. Carries `tool_use_id` and the
--          `plugin:skill` string in `tool_input`.
--
-- The `tool` arm additionally requires a matching PreToolUse decision in
-- `governance_decisions`. Every genuine tool call is governed before it runs,
-- so a `Skill` row with no decision beside it was posted straight to
-- `/hooks/track` -- local seeding, not a client. The `slash` arm gets no such
-- check and must not: `UserPromptSubmit` is not a tool call and never reaches
-- `/hooks/govern`, and the `user_prompt` rows that do exist in that table come
-- from the inference gateway under a separate `sess_...` session-id namespace,
-- so joining them would delete every real invocation instead of filtering it.
--
-- Both arms name the skill the same way, `plugin:skill` with underscores
-- rendered as hyphens, because `plugin_resolvers.rs` builds the slash command
-- as `format!("/{}:{}", plugin.id, skill_id.replace('_', "-"))`.
CREATE OR REPLACE VIEW skill_invocation_events AS
WITH raw AS (
    SELECT
        e.user_id,
        e.session_id,
        e.plugin_id,
        substring(e.prompt_preview from '^/([A-Za-z0-9._-]+:[A-Za-z0-9._-]+)') AS skill,
        NULL::text AS tool_use_id,
        'slash'::text AS source,
        e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type = 'UserPromptSubmit'
      AND e.prompt_preview ~ '^/[A-Za-z0-9._-]+:[A-Za-z0-9._-]+'

    UNION ALL

    SELECT
        e.user_id,
        e.session_id,
        e.plugin_id,
        e.metadata->'tool_input'->>'skill' AS skill,
        e.metadata->>'tool_use_id' AS tool_use_id,
        'tool'::text AS source,
        e.created_at AS invoked_at
    FROM plugin_usage_events e
    WHERE e.event_type IN ('PostToolUse', 'PostToolUseFailure')
      AND e.tool_name = 'Skill'
      AND e.metadata->'tool_input'->>'skill' IS NOT NULL
      AND EXISTS (
          SELECT 1
          FROM governance_decisions g
          WHERE g.session_id = e.session_id
            AND g.tool_name = 'Skill'
            AND g.created_at BETWEEN e.created_at - interval '5 seconds'
                                 AND e.created_at + interval '5 seconds'
      )
)
-- Why: when a slash command and a Skill tool call describe the same run, they
-- arrive within a second or two of each other. Keeping both would double-count
-- it, so the earlier row wins and the later duplicate is dropped.
SELECT user_id, session_id, plugin_id, skill, tool_use_id, source, invoked_at
FROM (
    SELECT raw.*,
           LAG(invoked_at) OVER (
               PARTITION BY session_id, skill ORDER BY invoked_at
           ) AS prev_at
    FROM raw
) d
WHERE prev_at IS NULL
   OR invoked_at - prev_at > interval '5 seconds';
