*Admin analytics + user management — what's live*

Everything below is running in the admin site now. Screenshots attached, one per section.

*Trend charts* — `analytics-overview.png`
• Request volume and cost over time, as line charts with filled areas
• Day / week bucket toggle on every trend
• Each KPI card shows the change vs the prior period (↑ +19.2%, colour-coded — green where up is good, red where up is spend) with a sparkline underneath
• Daily commit activity gets its own trend on the Code tab

*Model distribution* — `analytics-overview.png`
• Pie chart of model usage with a legend showing requests, tokens and cost per model
• Beside it, a stacked bar of *cost by model over time* — so a spend spike can be traced to the model that caused it, without reading two charts
• Top six models keep their own colour, the tail folds into "Other", and a model keeps the same colour across both charts

*Leaderboards and tables* — `analytics-usage.png`, `analytics-seats.png`
• Top users by request volume, with share bars, req/day, tokens, cost and last-active
• Sortable by requests, cost, tokens or last active; paginated
• Seat utilisation per organisation
• Wasted seats — anyone with zero requests in 30 days, listed by name with their last request date

*Per-user drill-down* — `analytics-user-drilldown.png`
New page. Click any user in the leaderboard to get their own dashboard:
• Their requests, spend, tokens, each with period-over-period change
• Their requests-per-day trend and their personal model mix
• Their daily usage records — sessions, prompts, tool uses, requests, AI lines, commits, cost
• Recent sessions with cost, context window and cache reads
• Links straight through to their raw request log and their account page

*Adoption* — `analytics-overview.png`, `analytics-usage.png`
• Weekly active users, with the change vs the prior week
• Requests per user per day
• Prompt cache hit rate and context-window pressure (average and peak)

*Cost and limits* — `analytics-spend.png`
• Total spend and cost per request
• Month-to-date burn-up chart against each plan's soft cap (amber dashed) and hard cap (red dashed), so proximity to the limit is visible at a glance
• "On pace for ~$X" projection on the same chart
• History of every month where spend crossed the soft cap
• Fast vs slow request split (under/over 5s) with p50 and p95

*Code impact* — `analytics-code.png`
• Commit activity trend
• AI lines added vs lines actually committed, plotted together
• Totals for AI lines added, AI lines removed, committed lines, commits, and applied AI edit operations

One honest note on this section: Claude Code emits no accept/reject signal, so there is no tab-acceptance rate to report and we don't invent one. What we show instead is the permission grant rate — the share of permission requests that were followed by the tool actually running — which is the real, measurable equivalent. Same for AI vs manual lines: AI lines and committed lines come from two different measurement systems, so we show both side by side and never subtract one from the other to fabricate a "manual" number.

*Filtering and drill-down* — every tab
• Filter by organisation, department, or individual user
• Time range: 15m / 1h / 24h / 7d / 30d / custom
• Active filters show as removable chips with a clear-all
• Org admins are locked to their own organisation; platform admins can pick any

*Managing users from the admin site* — `users-roster.png`, `user-detail.png`
• Roster grouped by department, with search
• *Add User* — creates the account and hands you a sign-in link to pass on (previously a new account had no way to sign in at all)
• *Invite by Link* — mint an invite, copy the link, revoke it, or regenerate it if the link gets lost
• The invited person clicks the link, creates a passkey, and is signed in — no password, no email round-trip
• Roles are now tick-boxes rather than a free-text field, so a typo can't create a role that doesn't exist
• Department is picked from the real department list
• Move a user between organisations and set their org role, with the current organisation shown correctly
• Deactivate or delete from the roster, behind a confirmation

*Look and feel*
• All charts are drawn by the server as SVG — no charting library, no CDN, nothing to load before a chart appears
• Charts use the existing design tokens, so they match the rest of the admin site and follow it into dark mode
• Every chart and table has a proper empty state rather than a blank panel
• Filters stay pinned to the top while scrolling long tables

*Screenshots attached*
`analytics-overview.png` · `analytics-usage.png` · `analytics-seats.png` · `analytics-spend.png` · `analytics-code.png` · `analytics-user-drilldown.png` · `users-roster.png` · `user-detail.png`

Files are in `docs/screenshots-2026-08-21/`.
