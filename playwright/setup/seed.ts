// Idempotent e2e seed: principals plus a deterministic 14-day analytics trail.
//
// Every row this script owns has an `e2e-` id prefix or an `@e2e.local` email;
// `--reset` deletes exactly those rows (children before parents) and nothing
// else — never TRUNCATE, never a developer's data. Safe to run repeatedly:
// every statement upserts.
//
// Column shapes mirror tests/contract/admin/src/seed.rs and the declarative
// schema under extensions/web/schema/ — if a seed insert breaks, diff against
// those files first.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Client } from 'pg';

const REPO = join(__dirname, '..', '..');

// ai_requests.context_id is NOT NULL; core's sentinel stands in for "no
// known context" (same constant as tests/contract/admin/src/seed.rs).
const LEGACY_CONTEXT_ID = '00000000-0000-0000-0000-4c4547414359';

// Why: core's session middleware attests that a cookie names a session the
// server issued *to that user* — a token with an unknown session_id is treated
// as stale and silently replaced with an anonymous one. The e2e principals
// therefore need real `user_sessions` rows, and the minted JWT must carry the
// matching session id.
export const E2E_SESSIONS: Record<string, string> = {
  'e2e-admin': 'e2e-session-admin',
  'e2e-platform-admin': 'e2e-session-platform-admin',
  'e2e-user': 'e2e-session-user',
};

export const E2E = {
  org: 'e2e-corp',
  orgB: 'e2e-corp-b',
  plan: 'e2e-plan',
  admin: { id: 'e2e-admin', email: 'e2e-admin@e2e.local' },
  platformAdmin: { id: 'e2e-platform-admin', email: 'e2e-platform-admin@e2e.local' },
  user: { id: 'e2e-user', email: 'e2e-user@e2e.local' },
  members: [1, 2, 3].map((n) => ({ id: `e2e-member-${n}`, email: `e2e-member-${n}@e2e.local` })),
  victim: { id: 'e2e-victim', email: 'e2e-victim@e2e.local' },
};

export function databaseUrl(): string {
  if (process.env.E2E_DATABASE_URL) return process.env.E2E_DATABASE_URL;
  const secrets = JSON.parse(
    readFileSync(join(REPO, '.systemprompt', 'profiles', 'local', 'secrets.json'), 'utf8'),
  );
  if (!secrets.database_url) throw new Error('no database_url in local profile secrets.json');
  return secrets.database_url;
}

async function upsertUser(db: Client, id: string, email: string, roles: string[]) {
  // users.name is unique; the email doubles as the name, as production
  // provisioning does (see tests/contract/admin/src/seed.rs::insert_user).
  await db.query(
    `INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
     VALUES ($1, $2, $2, $3, 'active', true, $4)
     ON CONFLICT (id) DO UPDATE SET roles = EXCLUDED.roles, status = 'active'`,
    [id, email, email.split('@')[0], roles],
  );
}

async function setMembership(db: Client, userId: string, orgId: string, orgRole: string) {
  await db.query(
    `INSERT INTO organization_members (user_id, org_id, org_role) VALUES ($1, $2, $3)
     ON CONFLICT (user_id) DO UPDATE SET org_id = EXCLUDED.org_id, org_role = EXCLUDED.org_role`,
    [userId, orgId, orgRole],
  );
}

async function setDepartmentOf(db: Client, userId: string, department: string) {
  await db.query(
    `INSERT INTO user_profile_ext (user_id, department) VALUES ($1, $2)
     ON CONFLICT (user_id) DO UPDATE SET department = EXCLUDED.department`,
    [userId, department],
  );
}

async function seedPrincipals(db: Client) {
  // Why: the seat limit has headroom over the seeded principals — the roster
  // spec creates a user, and a limit sized exactly to the seed would make that
  // fail with a (correct) 409 seat-limit refusal.
  await db.query(
    `INSERT INTO plans (id, name, description, seat_limit,
                        monthly_cost_cap_microdollars, monthly_cost_warn_microdollars)
     VALUES ($1, 'E2E Plan', 'deterministic e2e fixture plan', 25, 50000000, 30000000)
     ON CONFLICT (id) DO UPDATE SET seat_limit = 25,
        monthly_cost_cap_microdollars = 50000000, monthly_cost_warn_microdollars = 30000000`,
    [E2E.plan],
  );

  for (const [id, name] of [
    [E2E.org, 'E2E Corp'],
    [E2E.orgB, 'E2E Corp B'],
  ] as const) {
    await db.query(
      `INSERT INTO organizations (id, slug, name, plan_id, status, email_domains)
       VALUES ($1, $1, $2, $3, 'active', ARRAY['e2e.local'])
       ON CONFLICT (slug) DO UPDATE SET plan_id = EXCLUDED.plan_id, status = 'active'`,
      [id, name, E2E.plan],
    );
  }
  // Only e2e-corp claims the e2e.local email domain (org-by-email resolution
  // must be unambiguous); orgB is purely a membership-move target.
  await db.query(`UPDATE organizations SET email_domains = ARRAY[]::TEXT[] WHERE id = $1`, [
    E2E.orgB,
  ]);

  for (const dep of ['Engineering', 'Sales']) {
    await db.query(
      `INSERT INTO departments (id, name, org_id) VALUES ($1, $2, $3)
       ON CONFLICT (id) DO NOTHING`,
      [`e2e-dep-${dep.toLowerCase()}`, dep, E2E.org],
    );
  }

  // Adopt the existing platform org (exactly one exists per install); only
  // create one when the database has none — never a second.
  let platformOrg = (
    await db.query(`SELECT id FROM organizations WHERE is_platform = true LIMIT 1`)
  ).rows[0]?.id;
  if (!platformOrg) {
    platformOrg = 'e2e-platform';
    await db.query(
      `INSERT INTO organizations (id, slug, name, status, is_platform)
       VALUES ($1, $1, 'E2E Platform', 'active', true) ON CONFLICT (slug) DO NOTHING`,
      [platformOrg],
    );
  }

  await upsertUser(db, E2E.admin.id, E2E.admin.email, ['admin', 'user']);
  await setMembership(db, E2E.admin.id, E2E.org, 'owner');
  await setDepartmentOf(db, E2E.admin.id, 'Engineering');

  await upsertUser(db, E2E.platformAdmin.id, E2E.platformAdmin.email, ['admin', 'user']);
  await setMembership(db, E2E.platformAdmin.id, platformOrg, 'admin');

  await upsertUser(db, E2E.user.id, E2E.user.email, ['user']);
  await setMembership(db, E2E.user.id, E2E.org, 'member');

  for (const [i, m] of E2E.members.entries()) {
    await upsertUser(db, m.id, m.email, ['user']);
    await setMembership(db, m.id, E2E.org, 'member');
    await setDepartmentOf(db, m.id, i < 2 ? 'Engineering' : 'Sales');
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

  // The org-membership spec moves this user; re-homing it here makes that
  // test self-healing across runs.
  await upsertUser(db, E2E.victim.id, E2E.victim.email, ['user']);
  await setMembership(db, E2E.victim.id, E2E.org, 'member');
  await setDepartmentOf(db, E2E.victim.id, 'Sales');
}

// Deterministic per-member, per-day request counts: enough shape for trends
// (weekday hump) without randomness, so charts and visual baselines are stable.
const MODELS = ['claude-opus-5', 'claude-sonnet-5', 'claude-haiku-4-5'];

async function seedAnalyticsTrail(db: Client) {
  for (const [mi, m] of E2E.members.entries()) {
    for (let day = 0; day < 14; day++) {
      const perDay = 1 + ((day + mi) % 4); // 1..4 requests/day
      const sessionId = `e2e-session-${m.id}-${day}`;
      await db.query(
        `INSERT INTO user_sessions (session_id, user_id) VALUES ($1, $2)
         ON CONFLICT (session_id) DO NOTHING`,
        [sessionId, m.id],
      );
      for (let r = 0; r < perDay; r++) {
        const id = `e2e-req-${m.id}-${day}-${r}`;
        const model = MODELS[(day + r + mi) % MODELS.length];
        // The app's success vocabulary is completed/pending/streaming — anything
        // else counts as an error, so 'success' here would read as a 100% error
        // rate on the dashboard.
        const status = (day + r) % 9 === 0 ? 'failed' : 'completed';
        const cost = 900 + 350 * ((r + mi) % 3); // microdollars
        const latency = 400 + 5200 * (r % 2); // exercises the fast/slow split
        await db.query(
          `INSERT INTO ai_requests (
               id, request_id, user_id, session_id, trace_id, context_id,
               provider, model, input_tokens, output_tokens, tokens_used,
               cost_microdollars, latency_ms, status, actor_kind, actor_id,
               created_at, updated_at)
           VALUES ($1::TEXT, $1::TEXT, $2::TEXT, $3, $4, $5, 'anthropic', $6, 120, 30, 150,
                   $7, $8, $9, 'user', $2::TEXT,
                   NOW() - ($10 || ' days')::interval - ($11 || ' hours')::interval,
                   NOW() - ($10 || ' days')::interval)
           ON CONFLICT (id) DO NOTHING`,
          [id, m.id, sessionId, `e2e-trace-${m.id}-${day}`, LEGACY_CONTEXT_ID,
           model, cost, latency, status, String(day), String(9 + r)],
        );
      }
      const decisionId = `e2e-gov-${m.id}-${day}`;
      const decision = day % 5 === 0 ? 'deny' : 'allow';
      await db.query(
        `INSERT INTO governance_decisions (
             id, user_id, session_id, context_id, tool_name, decision, policy, reason,
             actor_kind, actor_id, created_at)
         VALUES ($1, $2, $3, $4, 'Edit', $5, $6, 'e2e fixture', 'user', $2,
                 NOW() - ($7 || ' days')::interval)
         ON CONFLICT (id) DO NOTHING`,
        [decisionId, m.id, sessionId, LEGACY_CONTEXT_ID, decision,
         decision === 'deny' ? 'tool_blocklist' : 'governance_disabled', String(day)],
      );
      await db.query(
        `INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, created_at)
         VALUES ($1, $2, $3, 'claude_code_PostToolUse', 'Edit',
                 NOW() - ($4 || ' days')::interval)
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-evt-${m.id}-${day}`, m.id, sessionId, String(day)],
      );
      // A permission request and the tool use that answers it, so the Usage
      // tab's grant-rate proxy has both halves of its pair. Every third day
      // the request goes unanswered, which is what makes the rate < 100%.
      await db.query(
        `INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, created_at)
         VALUES ($1, $2, $3, 'PermissionRequest', $4,
                 NOW() - ($5 || ' days')::interval - INTERVAL '2 minutes')
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-perm-${m.id}-${day}`, m.id, sessionId, 'Bash', String(day)],
      );
      if (day % 3 !== 0) {
        await db.query(
          `INSERT INTO plugin_usage_events (id, user_id, session_id, event_type, tool_name, created_at)
           VALUES ($1, $2, $3, 'PostToolUse', $4,
                   NOW() - ($5 || ' days')::interval - INTERVAL '1 minute')
           ON CONFLICT (id) DO NOTHING`,
          [`e2e-perm-ok-${m.id}-${day}`, m.id, sessionId, 'Bash', String(day)],
        );
      }
      await db.query(
        `INSERT INTO plugin_session_summaries
             (id, session_id, user_id, started_at, tool_uses, prompts, errors,
              model, status, ai_title, total_events)
         VALUES ($1, $2, $3, NOW() - ($4 || ' days')::interval, 3, 2, 0,
                 'claude-opus-5', 'active', $5, 5)
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-sum-${m.id}-${day}`, sessionId, m.id, String(day), `e2e session ${day}`],
      );
      // Code tab sources: daily rollups (commits + AI LOC) and per-tool daily
      // edit operations.
      await db.query(
        `INSERT INTO admin_usage_daily_rollups
             (user_id, date, sessions_count, prompts, tool_uses, errors,
              loc_added_ai, loc_removed_ai, commits_count, commit_insertions,
              commit_deletions, ai_requests_count, input_tokens, output_tokens,
              cost_microdollars)
         VALUES ($1, (NOW() - ($2 || ' days')::interval)::date, 1, 4, 6, 0,
                 $3, $4, $5, $6, $7, $8, 480, 120, $9)
         ON CONFLICT (user_id, date) DO UPDATE SET
             loc_added_ai = EXCLUDED.loc_added_ai,
             commits_count = EXCLUDED.commits_count`,
        [m.id, String(day), 40 + 10 * (day % 3), 8, 1 + (day % 2),
         60 + 12 * (day % 4), 9, perDay, perDay * 1200],
      );
      await db.query(
        `INSERT INTO plugin_usage_daily
             (id, date, event_type, tool_name, user_id, event_count, loc_added, loc_removed)
         VALUES ($1, (NOW() - ($2 || ' days')::interval)::date,
                 'claude_code_PostToolUse', 'Edit', $3, $4, $5, $6)
         ON CONFLICT (id) DO NOTHING`,
        [`e2e-daily-${m.id}-${day}`, String(day), m.id, 6, 40 + 10 * (day % 3), 8],
      );
    }
    // Client-reported statusline snapshot (Usage tab cache/context cards).
    await db.query(
      `INSERT INTO session_cost_snapshots
           (session_id, user_id, model, total_cost_microdollars, context_window_size,
            input_tokens, output_tokens, cache_creation_input_tokens, cache_read_input_tokens)
       VALUES ($1, $2, 'claude-opus-5', 15000, 180000, 5000, 1500, 20000, 45000)
       ON CONFLICT (session_id) DO UPDATE SET updated_at = NOW()`,
      [`e2e-session-${m.id}-0`, m.id],
    );
  }
}

async function reset(db: Client) {
  // Children before parents; every predicate is anchored to the e2e prefix.
  const stmts = [
    `DELETE FROM session_cost_snapshots WHERE session_id LIKE 'e2e-%'`,
    `DELETE FROM plugin_usage_daily WHERE id LIKE 'e2e-%'`,
    `DELETE FROM admin_usage_daily_rollups WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM plugin_session_summaries WHERE id LIKE 'e2e-%'`,
    `DELETE FROM plugin_usage_events WHERE id LIKE 'e2e-%'`,
    `DELETE FROM governance_decisions WHERE id LIKE 'e2e-%'`,
    `DELETE FROM ai_requests WHERE id LIKE 'e2e-%'`,
    `DELETE FROM user_commits WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM user_invites WHERE email LIKE '%@e2e.local'`,
    `DELETE FROM user_sessions WHERE session_id LIKE 'e2e-%'`,
    `DELETE FROM user_profile_ext WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM organization_members WHERE user_id LIKE 'e2e-%'`,
    `DELETE FROM users WHERE email LIKE '%@e2e.local'`,
    `DELETE FROM departments WHERE id LIKE 'e2e-%'`,
    `DELETE FROM organizations WHERE slug IN ('e2e-corp', 'e2e-corp-b') `,
    `DELETE FROM plans WHERE id = 'e2e-plan'`,
  ];
  for (const s of stmts) await db.query(s);
}

export async function seed(opts: { reset?: boolean } = {}) {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    if (opts.reset) await reset(db);
    await seedPrincipals(db);
    await seedAnalyticsTrail(db);
  } finally {
    await db.end();
  }
}

if (require.main === module) {
  seed({ reset: process.argv.includes('--reset') })
    .then(() => console.log('e2e seed complete'))
    .catch((e) => {
      console.error('e2e seed failed:', e.message);
      console.error(e.stack);
      process.exit(1);
    });
}
