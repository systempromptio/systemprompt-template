// The declarative traffic dataset: sessions, contexts, AI requests, MCP tool
// executions and skill invocations, all at fixed offsets from T0.
//
// The shape is exact and asserted on: 240 requests over a 6-model wheel and a
// 10-slot outcome wheel, which over eight full cycles gives every model the
// same outcome mix — 144 completed, and 24 each of policy rejection, quota
// rejection, upstream error and safety block.
import type { Client } from 'pg';
import { ACTORS } from './principals';
import { ID, ago, minutesAgo, prng } from './kit';

export const LEGACY_CONTEXT_ID = '00000000-0000-0000-0000-4c4547414359';

export const SESSION_COUNT = 12;
export const CONTEXT_COUNT = 6;
export const REQUEST_COUNT = 240;
export const EXECUTION_COUNT = 60;
export const SKILL_COUNT = 40;

export const MODELS: { model: string; provider: string; reasoning: boolean }[] = [
  { model: 'claude-opus-5', provider: 'anthropic', reasoning: true },
  { model: 'claude-sonnet-5', provider: 'anthropic', reasoning: false },
  { model: 'claude-haiku-4-5', provider: 'anthropic', reasoning: false },
  { model: 'gemini-2.5-pro', provider: 'vertex', reasoning: true },
  { model: 'gpt-5', provider: 'openai', reasoning: true },
  { model: 'claude-sonnet-5-bedrock', provider: 'bedrock', reasoning: false },
];

export type Outcome = 'ok' | 'rejected_policy' | 'rejected_quota' | 'upstream_error' | 'safety_block';

// Six of ten slots succeed. `rejected` is the only status the ai_requests
// CHECK constraint lets carry a NULL provider/model, which is exactly right:
// a request refused before routing never chose one.
const OUTCOME_WHEEL: Outcome[] = [
  'ok', 'ok', 'ok', 'ok', 'ok', 'ok',
  'rejected_policy', 'rejected_quota', 'upstream_error', 'safety_block',
];

export const SERVERS = ['systemprompt', 'filesystem', 'github'];
export const TOOLS = ['list_skills', 'get_user_report', 'read_file', 'create_issue'];
export const SKILLS = ['systematic_debugging', 'code_review', 'git_commit', 'meeting_summary'];
export const PLUGINS = ['enterprise-demo', 'systemprompt-dev', 'systemprompt-admin'];

export function outcomeOf(n: number): Outcome {
  return OUTCOME_WHEEL[n % OUTCOME_WHEEL.length];
}

export function modelOf(n: number) {
  return MODELS[n % MODELS.length];
}

export function actorOf(n: number): string {
  return ACTORS[n % ACTORS.length];
}

export function sessionOf(n: number): string {
  return ID.session(n % SESSION_COUNT);
}

async function seedSessions(db: Client) {
  for (let n = 0; n < SESSION_COUNT; n += 1) {
    const startedAt = ago(n % 14, 6 + (n % 5));
    await db.query(
      `INSERT INTO user_sessions (session_id, user_id, user_type, started_at, expires_at, last_activity_at)
       VALUES ($1, $2, 'registered', $3::timestamptz, $3::timestamptz + INTERVAL '7 days',
               $3::timestamptz + INTERVAL '40 minutes')
       ON CONFLICT (session_id) DO UPDATE SET last_activity_at = EXCLUDED.last_activity_at`,
      [ID.session(n), ACTORS[n % ACTORS.length], startedAt],
    );
  }
  for (let n = 0; n < CONTEXT_COUNT; n += 1) {
    await db.query(
      `INSERT INTO user_contexts (context_id, user_id, session_id, name, kind, created_at, updated_at)
       VALUES ($1, $2, $3, $4, 'cli_session', $5::timestamptz, $5::timestamptz)
       ON CONFLICT (context_id) DO UPDATE SET name = EXCLUDED.name`,
      [ID.context(n), ACTORS[n % ACTORS.length], ID.session(n), `Checkout refactor ${n + 1}`, ago(n, 4)],
    );
  }
}

function contextFor(n: number): string {
  const slot = n % SESSION_COUNT;
  return slot < CONTEXT_COUNT ? ID.context(slot) : LEGACY_CONTEXT_ID;
}

async function insertRequest(db: Client, n: number, rand: () => number) {
  const outcome = outcomeOf(n);
  const mix = modelOf(n);
  const rejected = outcome === 'rejected_policy' || outcome === 'rejected_quota';
  const status = rejected ? 'rejected' : outcome === 'upstream_error' ? 'failed' : 'completed';
  const inputTokens = 800 + Math.floor(rand() * 3200);
  const outputTokens = 120 + Math.floor(rand() * 900);
  const cacheHit = outcome === 'ok' && n % 3 === 0;
  await db.query(
    `INSERT INTO ai_requests (
         id, request_id, user_id, session_id, trace_id, context_id,
         provider, model, requested_model, input_tokens, output_tokens, tokens_used,
         cost_microdollars, latency_ms, status, error_message,
         cache_hit, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
         actor_kind, actor_id, created_at, updated_at, completed_at)
     VALUES ($1::TEXT, $1::TEXT, $2::TEXT, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15,
             $16, $17, $18, $19, 'user', $2::TEXT, $20::timestamptz, $20::timestamptz, $20::timestamptz)
     ON CONFLICT (id) DO NOTHING`,
    [
      ID.request(n),
      actorOf(n),
      sessionOf(n),
      `e2e-dtrace-${n % 40}`,
      contextFor(n),
      rejected ? null : mix.provider,
      rejected ? null : mix.model,
      mix.model,
      rejected ? 0 : inputTokens,
      status === 'completed' ? outputTokens : 0,
      rejected ? 0 : inputTokens + outputTokens,
      rejected ? 0 : 200 + Math.floor(rand() * 9000),
      rejected ? 12 : 500 + Math.floor(rand() * 7000),
      status,
      errorFor(outcome),
      cacheHit,
      cacheHit ? Math.floor(inputTokens * 0.7) : null,
      outcome === 'ok' && n % 5 === 0 ? Math.floor(inputTokens * 0.4) : null,
      mix.reasoning && outcome === 'ok' ? 200 + Math.floor(rand() * 1500) : null,
      ago(n % 21, 1 + (n % 12)),
    ],
  );
}

function errorFor(outcome: Outcome): string | null {
  if (outcome === 'rejected_policy') return 'blocked by policy: tool_blocklist';
  if (outcome === 'rejected_quota') return 'quota exceeded: daily spend cap reached';
  if (outcome === 'upstream_error') return 'upstream 529: provider overloaded';
  if (outcome === 'safety_block') return 'response withheld: safety scanner';
  return null;
}

async function seedRequests(db: Client) {
  const rand = prng(0x51ee9);
  for (let n = 0; n < REQUEST_COUNT; n += 1) await insertRequest(db, n, rand);
}

async function seedExecutions(db: Client) {
  for (let n = 0; n < EXECUTION_COUNT; n += 1) {
    const startedAt = ago(n % 14, 2 + (n % 9));
    const failed = n % 7 === 0;
    await db.query(
      `INSERT INTO mcp_tool_executions
           (mcp_execution_id, tool_name, server_name, started_at, completed_at,
            execution_time_ms, input, output, status, error_message, user_id, session_id,
            context_id, trace_id, actor_kind, actor_id, created_at)
       VALUES ($1, $2, $3, $4::timestamptz, $4::timestamptz + INTERVAL '2 seconds', $5, '{}', '{}', $6, $7,
               $8::TEXT, $9, $10, $11, 'user', $8::TEXT, $4::timestamptz)
       ON CONFLICT (mcp_execution_id) DO NOTHING`,
      [
        ID.execution(n),
        TOOLS[n % TOOLS.length],
        SERVERS[n % SERVERS.length],
        startedAt,
        400 + n * 17,
        failed ? 'failed' : 'success',
        failed ? 'tool returned a non-zero exit status' : null,
        actorOf(n),
        sessionOf(n),
        contextFor(n),
        `e2e-dtrace-${n % 40}`,
      ],
    );
    // One in four executions is also an assistant tool call on a request, so
    // the request detail view has a tool trail to render.
    if (n % 4 === 0) {
      await db.query(
        `INSERT INTO ai_request_tool_calls
             (id, request_id, tool_name, tool_input, mcp_execution_id, sequence_number, created_at)
         VALUES ($1, $2, $3, '{}', $4, 0, $5::timestamptz)
         ON CONFLICT (request_id, sequence_number) DO NOTHING`,
        [ID.toolCall(n), ID.request(n), TOOLS[n % TOOLS.length], ID.execution(n), startedAt],
      );
    }
  }
  for (let n = 0; n < SERVERS.length; n += 1) {
    await db.query(
      `INSERT INTO mcp_sessions (session_id, user_id, mcp_server_id, status, created_at, last_activity_at)
       VALUES ($1, $2, $3, 'active', $4::timestamptz, $4::timestamptz)
       ON CONFLICT (session_id) DO NOTHING`,
      [ID.mcpSession(n), ACTORS[n], SERVERS[n], ago(0, n + 1)],
    );
  }
}

// Skill invocations are plugin_usage_events rows whose event_type is
// UserPromptSubmit and whose prompt starts /plugin:skill.
async function seedSkills(db: Client) {
  for (let n = 0; n < SKILL_COUNT; n += 1) {
    const plugin = PLUGINS[n % PLUGINS.length];
    await db.query(
      `INSERT INTO plugin_usage_events
           (id, user_id, session_id, event_type, plugin_id, prompt_preview, created_at)
       VALUES ($1, $2, $3, 'UserPromptSubmit', $4, $5, $6::timestamptz)
       ON CONFLICT (id) DO NOTHING`,
      [
        ID.skill(n),
        actorOf(n),
        sessionOf(n),
        plugin,
        `/${plugin}:${SKILLS[n % SKILLS.length]}`,
        minutesAgo(30 + n * 47),
      ],
    );
  }
}

export async function seedTraffic(db: Client) {
  await seedSessions(db);
  await seedRequests(db);
  await seedExecutions(db);
  await seedSkills(db);
}
