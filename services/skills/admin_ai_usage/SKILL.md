# AI spend and adoption

Answer questions about AI spend and adoption on this platform - "what did we spend on AI this
week", "top users by cost", "which skills / models / plugins are used most", "were there failed
or denied requests", "how does this week compare to last". For what a person or team has been
working on, use `admin_person_activity`.

All tools are on the admin `systemprompt` server and read this platform's own audit tables,
so figures are exact; quote them as returned.

## Which tool answers what

| Question | Tool |
|---|---|
| Totals, trend, provider/model split | `admin_report {"report":"costs","days":7}` — call it fresh every run |
| Spend per person | `usage_by_user {"since":"7d","limit":100}` then `users` for names |
| What people did, active days, skills per person | `user_activity {"since":"30d"}` (empty `user` = everyone) |
| Which skills are used, by whom | `conversation_list {"since":"30d","skill":"<name>"}`; `user_activity` `skills` column |
| Failed or denied requests | `conversation_list` rows with `errors`/`denied` > 0, then `conversation_audit` on one or two |
| One request's detail | `request_log` then `conversation_audit` — debugging only |
| Anything else | the `systemprompt` CLI tool (`analytics costs breakdown --by model --since 7d`, `analytics requests stats`); flags are tabled in `systemprompt_cli` |

For a change, run the same tool again with `since`/`until` set to the previous period of
equal length and report both.

`user_activity` and `conversation_list` read the conversation rollup. `facts_as_of` is when it
last ran; activity after it is not counted yet, and an instance whose rollup has never run
returns no rows from them — fall back to `usage_by_user` and `admin_report` and say so.

## Rules

- **Rank people by conversations and spend, never by requests.** One Claude Code session is
  hundreds of requests.
- **Never conclude from one page.** Totals, "top", "busiest" and trends come from the
  aggregate tools (`admin_report`, `usage_by_user`, `user_activity`) or from paging a list to
  its end. A page that returned `next_cursor` is not the whole window; say so if you stop.
- **When two sources disagree, report both figures with their sources.** Do not invent a
  reason. `usage_by_user` excludes deleted users; `request_log` does not.
- **Say what was missing.** State the window, and name any source that was denied, failed or
  empty. A missing source is not a zero.
- Money: quote `*_usd`, never convert `*_microdollars` yourself. A model's request share is not
  its spend share.
- Do not paste full prompts; quote at most a trimmed line when it supports a judgement.

## Changes

Reporting authorises no writes. When a change is explicitly requested: inspect the current
state, run the exact command, read the result back, and report it. Never retry a write after a
timeout before checking whether it applied.
