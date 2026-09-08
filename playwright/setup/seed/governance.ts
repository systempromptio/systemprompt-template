// The governance plane: a decision per tool execution with denials at all four
// chain stages, safety findings on the blocked requests, pending approvals, and
// the session-quality rows the reviewer surfaces read.
import type { Client } from 'pg';
import { ID, ago, hasTable, minutesAgo, prng } from './kit';
import { ACTORS } from './principals';
import {
  EXECUTION_COUNT,
  LEGACY_CONTEXT_ID,
  REQUEST_COUNT,
  TOOLS,
  actorOf,
  outcomeOf,
  sessionOf,
} from './traffic';

// The four policies services/governance/config.yaml declares, by id, in
// evaluation order. Every one of them denies something in this dataset, so a
// filter by policy is never empty and the four cannot be confused for one
// another — and the policies page links each id to a decision log that has rows.
export const DENY_POLICIES = ['secret_scan', 'scope_check', 'tool_blocklist', 'rate_limit'];

const DENY_REASONS: Record<string, string> = {
  secret_scan: 'an AWS access key id was detected in the tool input',
  scope_check: 'tool is outside the scope granted to this principal',
  tool_blocklist: 'Bash is blocked for this department by policy',
  rate_limit: 'tool call budget for this session is exhausted',
};

export const FINDING_CATEGORIES = [
  'prompt_injection',
  'pii_email',
  'pii_credit_card',
  'jailbreak',
  'secret_key',
  'toxicity',
];

async function seedDecisions(db: Client) {
  for (let n = 0; n < EXECUTION_COUNT; n += 1) {
    const denied = n % 5 === 0;
    const warned = !denied && n % 11 === 0;
    const policy = DENY_POLICIES[Math.floor(n / 5) % DENY_POLICIES.length];
    const decision = denied ? 'deny' : warned ? 'warn' : 'allow';
    await db.query(
      `INSERT INTO governance_decisions (
           id, user_id, session_id, context_id, tool_name, decision, policy, reason,
           plugin_id, actor_kind, actor_id, trace_id, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'enterprise-demo', 'user', $2, $9, $10)
       ON CONFLICT (id) DO NOTHING`,
      [
        ID.decision(n),
        actorOf(n),
        sessionOf(n),
        LEGACY_CONTEXT_ID,
        TOOLS[n % TOOLS.length],
        decision,
        denied || warned ? policy : 'governance_allow',
        denied || warned ? DENY_REASONS[policy] : 'no rule matched',
        `e2e-dtrace-${n % 40}`,
        ago(n % 14, 2 + (n % 9)),
      ],
    );
  }
}

// Findings hang off ai_requests by foreign key, so every one names a request
// this dataset actually wrote. The 24 safety_block requests carry a blocking
// finding and successful requests carry audit-only ones up to a total of 30,
// which is what makes "scanned" and "blocked" visibly different numbers.
async function seedFindings(db: Client) {
  const rand = prng(0xf1d5);
  let written = 0;
  for (let n = 0; n < REQUEST_COUNT && written < 30; n += 1) {
    const outcome = outcomeOf(n);
    const blocking = outcome === 'safety_block';
    const auditOnly = outcome === 'ok' && n % 25 === 0;
    if (!blocking && !auditOnly) continue;
    await db.query(
      `INSERT INTO ai_safety_findings
           (id, ai_request_id, phase, severity, category, scanner, excerpt, blocked, created_at)
       VALUES ($1, $2, $3, $4, $5, 'regex_scanner', $6, $7, $8)
       ON CONFLICT (id) DO NOTHING`,
      [
        ID.finding(written),
        ID.request(n),
        blocking ? 'response' : 'request',
        blocking ? 'critical' : 'medium',
        FINDING_CATEGORIES[written % FINDING_CATEGORIES.length],
        `match at offset ${Math.floor(rand() * 400)}`,
        blocking,
        ago(n % 21, 1 + (n % 12)),
      ],
    );
    written += 1;
  }
}

async function seedApprovals(db: Client) {
  for (let n = 0; n < 3; n += 1) {
    await db.query(
      `INSERT INTO approval_requests
           (call_id, tool_name, server_name, arguments, args_digest, requested_by,
            session_id, trace_id, rule, status, expires_at, created_at)
       VALUES ($1, $2, 'systemprompt', '{"path":"/etc/hosts"}', $3, $4, $5, $6,
               'require_approval:write_file', 'pending', $7, $8)
       ON CONFLICT (call_id) DO NOTHING`,
      [
        ID.approval(n),
        ['write_file', 'run_migration', 'rotate_secret'][n],
        `e2e-digest-${n}`,
        ACTORS[n],
        sessionOf(n),
        `e2e-dtrace-${n}`,
        new Date(Date.now() + 3_600_000),
        minutesAgo(15 + n * 20),
      ],
    );
  }
}

// Session quality rows: the reviewer surfaces read analyses, ratings and the
// entity links that say which files and skills a session touched.
async function seedSessionQuality(db: Client) {
  for (let n = 0; n < 6; n += 1) {
    const sessionId = ID.session(n);
    const userId = ACTORS[n % ACTORS.length];
    await db.query(
      `INSERT INTO session_analyses
           (session_id, user_id, title, summary, goal_achieved, quality_score, outcome, category)
       VALUES ($1, $2, $3, 'Deterministic e2e session analysis.', 'yes', $4, 'success', 'engineering')
       ON CONFLICT (session_id) DO UPDATE SET quality_score = EXCLUDED.quality_score`,
      [sessionId, userId, `Checkout refactor ${n + 1}`, 60 + n * 5],
    );
    await db.query(
      `INSERT INTO session_ratings (id, user_id, session_id, rating, outcome, notes, created_at)
       VALUES ($1, $2, $3, $4, 'success', 'e2e fixture', $5)
       ON CONFLICT (user_id, session_id) DO UPDATE SET rating = EXCLUDED.rating`,
      [`e2e-drate-${n}`, userId, sessionId, 3 + (n % 3), ago(n, 3)],
    );
    await db.query(
      `INSERT INTO session_entity_links
           (id, user_id, session_id, entity_type, entity_name, usage_count)
       VALUES ($1, $2, $3, 'skill', $4, $5)
       ON CONFLICT (user_id, session_id, entity_type, entity_name) DO NOTHING`,
      [`e2e-dlink-${n}`, userId, sessionId, 'systematic_debugging', 1 + n],
    );
  }
  for (let n = 0; n < 4; n += 1) {
    await db.query(
      `INSERT INTO skill_ratings (id, user_id, skill_name, rating, notes)
       VALUES ($1, $2, $3, $4, 'e2e fixture')
       ON CONFLICT (user_id, skill_name) DO UPDATE SET rating = EXCLUDED.rating`,
      [`e2e-dskillrate-${n}`, ACTORS[n], ['code_review', 'git_commit', 'meeting_summary', 'systematic_debugging'][n], 3 + (n % 3)],
    );
  }
}

async function seedSecretAudit(db: Client) {
  if (!(await hasTable(db, 'secret_audit_log'))) return;
  const actions = ['created', 'updated', 'accessed', 'rotated'];
  for (let n = 0; n < 4; n += 1) {
    await db.query(
      `INSERT INTO secret_audit_log (id, user_id, plugin_id, var_name, action, actor_id, created_at)
       VALUES ($1, $2, 'enterprise-demo', 'GITHUB_TOKEN', $3, $2, $4)
       ON CONFLICT (id) DO NOTHING`,
      [`e2e-dsecret-${n}`, ACTORS[n], actions[n], minutesAgo(90 + n * 33)],
    );
  }
}

export async function seedGovernance(db: Client) {
  await seedDecisions(db);
  await seedFindings(db);
  await seedApprovals(db);
  await seedSessionQuality(db);
  await seedSecretAudit(db);
}
