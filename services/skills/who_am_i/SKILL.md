# Who am I

Use when someone asks who they are on this instance, what they have access to, why
a skill or connector they expected is missing, or what they are allowed to do here.
It is the first thing a new user runs and the fastest answer to "why can't I see X".

## What you report

Answer from what this session can actually observe — the signed-in identity and the
signed plugin manifest the client is already holding. Do not guess at entitlement
from a job title or a team name.

1. **Identity** — the account this session is authenticated as: the email or user
   id, and the roles it carries. If the session is anonymous or the identity is
   not visible, say exactly that rather than inventing a user.
2. **Workspaces** — the marketplaces this account reaches, by name. Say what each
   one is for in a sentence.
3. **Skills** — the skills those workspaces grant, grouped by workspace. Give the
   count and then the names; a list of forty with no shape is not an answer.
4. **Connectors** — the MCP servers available, and for each one whether it is
   *connected* (the account has linked it and calls will work) or merely
   *granted* (it appears in the manifest but has no credential yet). This
   distinction is the single most common source of "the skill didn't work".
5. **Where to see usage** — the admin console profile page (`/admin/profile`) shows this account's own
   request history and spend. Point the user there for figures; do not attempt to
   produce usage numbers yourself.

## Guardrails

- **You report on the caller, and only the caller.** Never enumerate other users,
  other accounts' access, or instance-wide totals. That is the administrator's
  control plane, not this skill.
- **Absence is a finding, not a failure.** A workspace or connector the user
  expected and does not have is the answer they came for. Name it, say which gate
  it sits behind — group membership, a role, or an unlinked connector account —
  and stop. Do not attempt to grant anything, and do not speculate about who could.
- **Do not read the platform's own configuration** to answer this. The manifest
  the session holds is the truth about what this user reaches; a YAML file on disk
  describes what the instance ships, which is a different question and will
  mislead when the two disagree.
- **Never state a usage figure you did not read.** If you cannot see the numbers,
  the honest answer is where to find them.
