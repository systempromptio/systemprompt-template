// The e2e principals, their roles, their sessions, and the users the traffic
// dataset attributes rows to.
//
// Roles live in users.roles — the minted JWT's own role claim is ignored by the
// server — so this file is what decides what every authorization assertion in
// the suite sees. This instance knows two roles: `admin` reaches the console,
// `user` reaches only their own profile and settings.
import type { Client } from 'pg';
import { ID, ago } from './kit';

// Why: core's session middleware attests that a cookie names a session the
// server issued *to that user*; a token carrying an unknown session_id is
// treated as stale and silently replaced with an anonymous one. Each principal
// that signs in therefore needs a real user_sessions row whose id the minted
// JWT repeats.
export const E2E_SESSIONS: Record<string, string> = {
  'e2e-admin': 'e2e-session-admin',
  'e2e-user': 'e2e-session-user',
};

export const E2E = {
  admin: { id: 'e2e-admin', email: 'e2e-admin@e2e.local' },
  user: { id: 'e2e-user', email: 'e2e-user@e2e.local' },
  members: [1, 2, 3, 4, 5, 6, 7].map((n) => ({
    id: `e2e-member-${n}`,
    email: `e2e-member-${n}@e2e.local`,
  })),
  // A user no spec reads through the UI, so a mutating spec has something to
  // move without disturbing the rows every other assertion counts.
  victim: { id: 'e2e-victim', email: 'e2e-victim@e2e.local' },
  departments: {
    engineering: 'Engineering',
    product: 'Product',
    support: 'Support',
  },
};

// Which department each principal sits in. `user_profile_ext.department`
// holds the department NAME, so a rename in departments.ts must be mirrored
// here. Anyone absent stays in the catch-all `Default` department, which is
// what keeps that row populated on the departments page.
export const DEPARTMENT_OF: Record<string, string> = {
  'e2e-admin': E2E.departments.engineering,
  'e2e-member-1': E2E.departments.engineering,
  'e2e-member-2': E2E.departments.engineering,
  'e2e-member-3': E2E.departments.engineering,
  'e2e-member-4': E2E.departments.product,
  'e2e-member-5': E2E.departments.product,
  'e2e-member-6': E2E.departments.support,
  'e2e-member-7': E2E.departments.support,
};

// The ten users the declarative traffic dataset attributes rows to, in a
// fixed order: request n belongs to ACTORS[n % 10], which is what makes the
// per-user and per-department totals exact rather than approximate.
export const ACTORS: string[] = [
  E2E.members[0].id,
  E2E.members[1].id,
  E2E.members[2].id,
  E2E.members[3].id,
  E2E.members[4].id,
  E2E.members[5].id,
  E2E.members[6].id,
  E2E.user.id,
  E2E.admin.id,
  E2E.members[0].id,
];

const PRINCIPALS: { id: string; email: string; roles: string[] }[] = [
  { ...E2E.admin, roles: ['admin', 'user'] },
  { ...E2E.user, roles: ['user'] },
  { ...E2E.victim, roles: ['user'] },
  ...E2E.members.map((m) => ({ ...m, roles: ['user'] })),
];

// Why the retry: several agents share one database on this clone, and two
// seeds running at once both insert the same principal. ON CONFLICT arbitrates
// on `id`, so a peer's in-flight row can still surface as a violation of the
// `email` or `name` unique index instead. The row the peer is writing is byte
// for byte the row we want, so one retry after its commit is the whole fix.
async function upsertUser(db: Client, id: string, email: string, roles: string[]) {
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      const r = await db.query(
        `INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
         VALUES ($1, $2, $2, $3, 'active', true, $4)
         ON CONFLICT (id) DO UPDATE SET roles = EXCLUDED.roles, status = 'active' RETURNING id`,
        [id, email, email.split('@')[0], roles],
      );
      if (r.rowCount !== 1) throw new Error(`upsertUser(${id}) affected ${r.rowCount} rows`);
      return;
    } catch (e) {
      if ((e as { code?: string }).code !== '23505' || attempt === 1) throw e;
      await new Promise((resolve) => setTimeout(resolve, 250));
    }
  }
}

// Why: the user detail page's Activity tab lists an account's *work*
// sessions — the plugin_session_summaries rows the bridge writes when a
// Claude Code session ends — not the sign-in rows in user_sessions. The
// traffic seed attributes session n to ACTORS[n % 10] under ID.session(n), so
// a summary under the same id and owner makes the tab list what the sessions
// page links to. Count mirrors traffic.ts's SESSION_COUNT.
const WORK_SESSION_COUNT = 12;

async function seedWorkSessions(db: Client) {
  for (let n = 0; n < WORK_SESSION_COUNT; n += 1) {
    const startedAt = ago(n % 14, 6 + (n % 5));
    await db.query(
      `INSERT INTO plugin_session_summaries
           (id, session_id, user_id, plugin_id, started_at, ended_at,
            total_events, tool_uses, prompts, errors, status, client_source)
       VALUES ($1, $1, $2, 'e2e-plugin', $3::timestamptz, $3::timestamptz + INTERVAL '40 minutes',
               $4, $5, $6, $7, 'completed', 'e2e')
       ON CONFLICT (session_id) DO UPDATE
          SET user_id = EXCLUDED.user_id,
              started_at = EXCLUDED.started_at,
              ended_at = EXCLUDED.ended_at,
              total_events = EXCLUDED.total_events,
              tool_uses = EXCLUDED.tool_uses,
              prompts = EXCLUDED.prompts,
              errors = EXCLUDED.errors`,
      [ID.session(n), ACTORS[n % ACTORS.length], startedAt, 20 + n * 3, 8 + n, 4 + (n % 3), n % 4],
    );
  }
}

export async function seedPrincipals(db: Client) {
  await seedWorkSessions(db);
  for (const p of PRINCIPALS) {
    await upsertUser(db, p.id, p.email, p.roles);
    // The department is written here rather than in departments.ts so a
    // principal always has its profile row. seed.ts upserts the department
    // rows first, so the name is never dangling.
    await db.query(
      `INSERT INTO user_profile_ext (user_id, department) VALUES ($1, $2)
       ON CONFLICT (user_id) DO UPDATE SET department = EXCLUDED.department`,
      [p.id, DEPARTMENT_OF[p.id] ?? 'Default'],
    );
  }
  for (const [userId, sessionId] of Object.entries(E2E_SESSIONS)) {
    await db.query(
      `INSERT INTO user_sessions (session_id, user_id, user_type, expires_at, last_activity_at)
       VALUES ($1, $2, 'registered', NOW() + INTERVAL '7 days', NOW())
       ON CONFLICT (session_id) DO UPDATE
          SET expires_at = NOW() + INTERVAL '7 days',
              last_activity_at = NOW(),
              ended_at = NULL`,
      [sessionId, userId],
    );
  }
}
