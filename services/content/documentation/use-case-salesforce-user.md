---
title: "Use Case — Your Salesforce Day, in Plain English"
description: "The Salesforce user journey: sign in once, then run your pipeline, briefings, call logs and follow-ups by asking in plain English — under your own permissions, with confirmation before anything changes."
author: "Astound Digital"
slug: "use-case-salesforce-user"
keywords: "salesforce user, sales, plain english, pipeline, account briefing, log a call, bridge, getting started, skills"
kind: "guide"
public: true
tags: ["salesforce", "getting-started", "skills"]
published_at: "2026-07-31"
updated_at: "2026-07-31"
after_reading_this:
  - "Get set up in about two minutes with no admin help"
  - "Know the two promises: it acts as you, and it confirms before changing anything"
  - "Run a full working day through plain-English questions"
  - "Know what to do when something is missing and who to ask"
related_playbooks:
  - title: "Use Case — Standing Up the Gateway"
    url: "/documentation/use-case-admin"
  - title: "All Skills"
    url: "/skills/"
  - title: "Step 5 — Rolling Out the Bridge"
    url: "/documentation/salesforce-bridge-rollout"
---

# Use Case — Your Salesforce Day, in Plain English

**Who this is for:** anyone who works in Salesforce — sales, service, account
management. No technical setup, no configuration, nothing to learn about field
names or reports.

**What you need:** your normal Salesforce login. That is genuinely all.

You ask for what you want in your own words. It answers from live Salesforce
data, shows you exactly what it will change before it changes anything, and
records what it did.

## Getting Set Up — About Two Minutes

1. **[Install the bridge](/documentation/bridge-install).** Your admin will
   send you the download, or the Windows download is on the [homepage](/). It
   runs quietly in your menu bar or system tray.
2. **Click "Sign in with Salesforce."** Your browser opens and you log in to
   Salesforce exactly as you normally do — same password, same MFA. The bridge
   never sees your credentials.
3. **Approve the device link.** One click, confirming that the app on your
   machine is the one asking.
4. **Wait a moment.** Your skills appear automatically. You will see the
   Salesforce ones plus anything else your role is entitled to.

There is no account to create, no form to fill in, and no admin step for your
account specifically. Your Salesforce login *is* your account.

If something goes wrong here, jump to [When Something Is
Missing](#when-something-is-missing).

## Two Things to Know

**It acts as you.** Everything it reads or changes happens under your own
Salesforce login and permissions. It can only see and touch what you can see and
touch — your sharing rules, field-level security and record access all apply
exactly as they do when you use Salesforce directly. Nothing is elevated.

**It always confirms before changing anything.** Reading is instant. But before
it creates, edits or deletes a record it shows you the record and every field as
old value → new value, and waits for your explicit yes. Nothing reaches
Salesforce until you approve it.

If a request is ambiguous — which "Acme"? whose pipeline? — it asks a quick
clarifying question rather than guessing. If something is not in Salesforce or
you do not have access to it, it says so plainly instead of making something up.

## A Working Day

Everything below is a real question you can type as written.

### Morning — what am I walking into

> "What do I need to follow up on today?"

> "How's my pipeline looking this quarter?"

> "Any cases escalated overnight?"

You get your due and overdue tasks, your open deals by stage with a forecast,
and your support queue by priority. No report to build, no filter to set.

### Before a call — brief me

> "Give me a briefing on the Acme account before my call."

One question pulls the account, its key contacts and decision-makers, every open
opportunity, and any open cases — the whole picture in one answer rather than
five tabs.

> "Who do we know at Globex, and who's the decision-maker?"

### During and after — capture it while it's fresh

> "Log a call I just had with Jane at Globex — she wants pricing by Friday."

> "Set a follow-up task for Thursday to send the Acme proposal."

> "Move the Initech renewal to Closed Won."

Each of these is a change, so each one comes back with a confirmation first:
here is the record, here are the fields, here is what they will become. You say
yes, and it writes.

### End of week — what's slipping

> "Which of my accounts haven't we contacted in the last 60 days?"

> "Which deals in my pipeline are at risk?"

> "Which leads came in this month and still have no follow-up?"

## Your Dashboards

Alongside the questions, six dashboards are installed into your workspace the
first time you connect:

| Dashboard | What it shows |
|---|---|
| Open Deals | Your live pipeline by stage |
| Book of Business | Your accounts at a glance |
| People | Your contacts, by account |
| Inbound Prospects | Leads and where they came from |
| Support Queue | Open cases by priority and age |
| Tasks and Meetings | What is due and what is overdue |

They are live views of your own Salesforce data, subject to the same
permissions as everything else. If you already have them, they are left alone —
setup can be re-run safely.

## What Else It Can Do

The Salesforce skills cover six areas — deals, accounts, contacts, leads, cases,
and activities — and each has a focused helper for finding things, summarising
them, and updating them. You do not choose between them; you ask naturally and
the right one steps in.

Browse the **[full skills catalogue](/skills/)** — every skill lists what it does
and gives example questions.

## When Something Is Missing

Almost everything that goes wrong here is a permission or a seat, and all of it
is fixed by your admin rather than by you.

| What you see | What it means | What to do |
|---|---|---|
| `?sso=seat_limit` when signing in | Your organization is out of seats | Ask your admin for a seat |
| `?sso=not_provisioned` when signing in | Your account has not been set up and auto-provisioning is off | Ask your admin to provision you |
| You sign in fine but see nothing | Your email domain is not attached to an organization | Ask your admin — it is a one-line config fix |
| The Salesforce tools appear but every request fails | Your Salesforce user is not authorized on the app | Ask your admin to re-run the provisioning step for you |
| "Invalid redirect" on the device link | Something is intercepting the sign-in flow | Stop and tell your admin — treat it as a security issue, not a glitch |
| It says you do not have access to a record | You genuinely do not, in Salesforce | Request access in Salesforce as you normally would |

That last row is worth dwelling on: the assistant cannot see more than you can.
If it says a record is not visible to you, the answer is in Salesforce's
permissions, not here.

## For Your Admin

If you are the person who has to make the above work, start at
**[Standing Up the Gateway](/documentation/use-case-admin)**.
