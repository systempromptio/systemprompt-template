# Person and team activity

Answer "what has <person> been working on", "how much is <person/team> using the platform",
"who in <team> is active", "how intensively", or "whose work relates to <topic>". Reads
conversations, skills used and active days, not raw requests.

The unit is the **conversation**, not the request. One Claude Code session is hundreds of
requests, so request counts and request timestamps say nothing about what someone did or
how long they worked.

## Steps

1. **Resolve the people.** `user_activity {"user":"<name, email or id>","since":"<window>"}`
   matches part of a display name, so a first name finds the full name. When several people
   match, name them all and say which you used. For a team there is no team field on a user:
   say so, then use the names the user gives or a group from the admin console. Never claim
   membership you inferred from a name.
2. **Activity, one call per window.** `user_activity` returns per person: `conversations`,
   `active_days`, `per_day` (conversations and turns each day), `skills`,
   `main_conversations` (date + title), `cost_usd`, `errors`, `denied` and `facts_as_of`.
   Quote these. An empty `user` returns everyone, active or not.
3. **The work itself.** `conversation_list {"user":"<id>","since":"<window>","limit":25}` gives
   one row per conversation with its title. Group titles into two to five themes (a project,
   a customer, a kind of task). Read at most two representative conversations with
   `conversation_audit {"request_id":"<row request_id>","limit":8,"max_chars":600}`.
4. **Other sources.** If the caller holds a connector for an issue tracker or a wiki, look the
   person up there by email, not by first name, and link what you find to the conversation
   themes where the names match. Without one, say that only platform activity was read.
5. **Answer.** Per person: window, active days out of the window, conversations, main themes
   with evidence (titles, and any issue keys or page titles), skills used, spend, and anything
   failed or denied.

## Rules

- **Never conclude from one page.** "Busiest day", totals and trends come from
  `user_activity` (it aggregates the whole window) or from paging to the end. If a list tool
  returned `next_cursor`, you have not seen everything; say so.
- **No hours per day.** Nothing measures time worked, and a conversation can stay open for
  days. Report active days and conversations and turns per day (`per_day`).
- **When two sources disagree, report both figures and their sources.** Never invent a cause
  such as a lag or a rollup delay. `facts_as_of` is the only freshness fact you have: activity
  after it is not in `user_activity` yet, and a null `facts_as_of` means the rollup has not run
  on this instance — say so rather than reporting zero work.
- **Say what was missing.** Name any source that was denied, failed or returned nothing; a
  missing source is not zero work.
- `request_log` is for debugging one request, never for describing someone's work.
- Personal or off-topic conversations: count them, do not describe them.
- Read only. Do not write memory files, comment, or change anything.
