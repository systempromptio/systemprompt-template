// One completed judge run over the seeded traffic, so the evals page and the
// run detail page have a run to render and a verdict mix to count.
//
// eval_results is unique on (run_id, ai_request_id), so every result names a
// distinct request from traffic.ts; the verdict wheel gives the four verdicts
// in a fixed ratio.
import type { Client } from 'pg';
import { ID, ago } from './kit';
import { E2E } from './principals';
import { REQUEST_COUNT, actorOf, modelOf, outcomeOf, sessionOf } from './traffic';

export const EVAL_RUN_ID = ID.evalRun(0);
export const EVAL_RESULT_COUNT = 24;

const VERDICTS = ['pass', 'pass', 'pass', 'partial', 'fail', 'skipped'] as const;

export async function seedEvals(db: Client) {
  await db.query(
    `INSERT INTO eval_runs
         (id, kind, status, judge_provider, judge_model, filter, sample_size, scored_count,
          failed_count, cost_microdollars, created_by, created_at, completed_at, trigger_source)
     VALUES ($1, 'judge', 'completed', 'anthropic', 'claude-sonnet-5', '{}'::jsonb, $2, $3, $4,
             42000, $5, $6::timestamptz, $6::timestamptz + INTERVAL '4 minutes', 'manual')
     ON CONFLICT (id) DO UPDATE SET scored_count = EXCLUDED.scored_count`,
    [
      EVAL_RUN_ID,
      EVAL_RESULT_COUNT,
      EVAL_RESULT_COUNT - 4,
      4,
      E2E.admin.id,
      ago(0, 2),
    ],
  );

  // Only completed requests carry a response worth judging, and the outcome
  // wheel puts those at slots 0..5 of every ten.
  let written = 0;
  for (let n = 0; n < REQUEST_COUNT && written < EVAL_RESULT_COUNT; n += 1) {
    if (outcomeOf(n) !== 'ok') continue;
    const verdict = VERDICTS[written % VERDICTS.length];
    const score = verdict === 'pass' ? 4 + (written % 2) : verdict === 'partial' ? 3 : verdict === 'fail' ? 1 + (written % 2) : null;
    await db.query(
      `INSERT INTO eval_results
           (id, run_id, ai_request_id, user_id, session_id, provider, model, overall_score,
            dimension_scores, verdict, rationale, prompt_excerpt, response_excerpt,
            latency_ms, cost_microdollars, judge_cost_microdollars, created_at)
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9::jsonb, $10, $11, $12, $13, $14, $15, 1750,
               $16::timestamptz)
       ON CONFLICT (id) DO NOTHING`,
      [
        ID.evalResult(written),
        EVAL_RUN_ID,
        ID.request(n),
        actorOf(n),
        sessionOf(n),
        modelOf(n).provider,
        modelOf(n).model,
        score,
        JSON.stringify({ accuracy: score ?? 0, helpfulness: score ?? 0 }),
        verdict,
        verdict === 'skipped' ? 'No assistant text to judge.' : `Deterministic e2e verdict ${written}.`,
        `Prompt ${n}: summarise the release notes`,
        `Reply ${n}: the release adds…`,
        600 + n * 13,
        1200 + n * 7,
        ago(1, 2),
      ],
    );
    written += 1;
  }
}
