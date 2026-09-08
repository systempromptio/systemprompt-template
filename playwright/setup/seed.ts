// Idempotent e2e seed: the principals, their departments and tokens, and a
// declarative demo dataset placed at fixed offsets from the instant the seed
// started.
//
// Every row this script owns carries an `e2e-` id prefix or an `@e2e.local`
// email; `--reset` deletes exactly those rows (children before parents) and
// nothing else — never TRUNCATE, never a developer's data. Safe to run
// repeatedly: every statement upserts.
//
// The dataset itself lives in setup/seed/*.ts; this file is the connection, the
// order, and the reset. Column shapes mirror tests/contract/admin/src/seed.rs
// and the declarative schema under extensions/web/schema/ (plus core's own
// schema for the shared tables) — if an insert breaks, diff against those first.
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Client } from 'pg';
import { seedDepartments } from './seed/departments';
import { seedEvals } from './seed/evals';
import { seedGovernance } from './seed/governance';
import { seedPrincipals } from './seed/principals';
import { seedTokens } from './seed/tokens';
import { seedTraffic } from './seed/traffic';

const REPO = join(__dirname, '..', '..');

export { E2E, E2E_SESSIONS, DEPARTMENT_OF } from './seed/principals';
export { departmentId } from './seed/departments';
export { EVAL_RUN_ID } from './seed/evals';
export { T0 } from './seed/kit';

export function databaseUrl(): string {
  if (process.env.E2E_DATABASE_URL) return process.env.E2E_DATABASE_URL;
  const secrets = JSON.parse(
    readFileSync(join(REPO, '.systemprompt', 'profiles', 'local', 'secrets.json'), 'utf8'),
  );
  if (!secrets.database_url) throw new Error('no database_url in local profile secrets.json');
  return secrets.database_url;
}

// Every e2e account, whichever way it was keyed. A spec that creates a user
// through the console gets a generated id and an `@e2e.local` email, so the
// email is the predicate that also catches those.
const E2E_USER_IDS = `SELECT id FROM users WHERE email LIKE '%@e2e.local'`;
const E2E_REQUEST_IDS = `SELECT id FROM ai_requests WHERE user_id IN (${E2E_USER_IDS})`;

// Children before parents; every predicate is anchored to the e2e prefix.
//
// The `departments` rows are deliberately NOT deleted: a developer may have
// hand-assigned a real user to Engineering, and the assignments (all on
// `e2e-%` profiles) are what this seed actually owns. The department rows are
// upserted on every run, so they cannot drift.
const RESET_STATEMENTS = [
  `DELETE FROM eval_results WHERE run_id LIKE 'e2e-%' OR id LIKE 'e2e-%'`,
  `DELETE FROM eval_runs WHERE id LIKE 'e2e-%'`,
  `DELETE FROM user_api_keys WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM ai_request_tool_calls WHERE id LIKE 'e2e-%'`,
  `DELETE FROM ai_safety_findings WHERE id LIKE 'e2e-%'`,
  `DELETE FROM approval_requests WHERE call_id LIKE 'e2e-%'`,
  `DELETE FROM mcp_tool_executions WHERE mcp_execution_id LIKE 'e2e-%'`,
  `DELETE FROM mcp_sessions WHERE session_id LIKE 'e2e-%'`,
  // Only rules a spec minted itself. Rules declared in services/ are ingested
  // at boot and never carry an e2e- id, so this cannot revoke a real grant.
  `DELETE FROM access_control_rules WHERE id LIKE 'e2e-rule-%'`,
  `DELETE FROM session_analyses WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM session_ratings WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM session_entity_links WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM skill_ratings WHERE id LIKE 'e2e-%'`,
  `DELETE FROM plugin_usage_daily WHERE id LIKE 'e2e-%'`,
  `DELETE FROM plugin_session_summaries WHERE id LIKE 'e2e-%'`,
  `DELETE FROM plugin_usage_events WHERE id LIKE 'e2e-%'`,
  `DELETE FROM governance_decisions WHERE id LIKE 'e2e-%'`,
  `DELETE FROM ai_requests WHERE id LIKE 'e2e-%'`,
  `DELETE FROM ai_request_tool_calls WHERE request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_request_messages WHERE request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_request_payloads WHERE ai_request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_safety_findings WHERE ai_request_id IN (${E2E_REQUEST_IDS})`,
  `DELETE FROM ai_requests WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM user_contexts WHERE context_id LIKE 'e2e00000-dc7c-%' OR user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM secret_audit_log WHERE id LIKE 'e2e-%'`,
  `DELETE FROM user_sessions WHERE session_id LIKE 'e2e-%'`,
  `DELETE FROM user_sessions WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM user_profile_ext WHERE user_id IN (${E2E_USER_IDS})`,
  `DELETE FROM users WHERE email LIKE '%@e2e.local'`,
];

async function reset(db: Client) {
  for (const stmt of RESET_STATEMENTS) {
    // A few side tables are optional on an older database (secret_audit_log,
    // the eval tables); a missing table must not abort a reset that has
    // already deleted rows.
    try {
      await db.query(stmt);
    } catch (e) {
      if ((e as { code?: string }).code !== '42P01') throw e;
    }
  }
}

export async function seed(opts: { reset?: boolean } = {}): Promise<void> {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    if (opts.reset) await reset(db);
    await seedDepartments(db);
    await seedPrincipals(db);
    await seedTokens(db);
    await seedTraffic(db);
    await seedGovernance(db);
    await seedEvals(db);
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
