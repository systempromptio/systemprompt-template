// Drives real governed traffic through the stack so evidence screenshots show
// data that actually came through the system: for each principal, issue a PAT,
// call /v1/messages on the e2e-mock route (answered by the mock inference
// upstream), then post the transcript through the same capture webhook the
// Claude Code hook uses. No direct DB writes.
import { readFileSync } from 'node:fs';
import { AUTH } from './global-setup';
import { E2E_SESSIONS } from './seed';

const SCENARIOS = [
  {
    state: 'admin',
    session: E2E_SESSIONS['e2e-admin'],
    prompt:
      'Draft the rollout checklist for the Astound governance gateway: quotas, budget alerts, and the audit trail owners need to review before go-live.',
  },
  {
    state: 'platformAdmin',
    session: E2E_SESSIONS['e2e-platform-admin'],
    prompt:
      'Summarise this month’s AI spend by organization and flag any team trending over its soft budget threshold before the period closes.',
  },
  {
    state: 'user',
    session: E2E_SESSIONS['e2e-user'],
    prompt:
      'Explain how to connect a Salesforce identity from the profile page and which MCP tools that unlocks for account queries.',
  },
] as const;

function cookieToken(statePath: string): string {
  const state = JSON.parse(readFileSync(statePath, 'utf8')) as {
    cookies: { name: string; value: string }[];
  };
  const cookie = state.cookies.find((c) => c.name === 'access_token');
  if (!cookie) throw new Error(`no access_token cookie in ${statePath}`);
  return cookie.value;
}

async function issuePat(baseURL: string, jwt: string): Promise<string | null> {
  const res = await fetch(`${baseURL}/admin/devices/pats`, {
    method: 'POST',
    headers: { cookie: `access_token=${jwt}`, 'content-type': 'application/json' },
    body: JSON.stringify({ name: `e2e-flow-${Date.now()}` }),
  });
  if (!res.ok || !res.headers.get('content-type')?.includes('json')) return null;
  try {
    const body = (await res.json()) as { secret: string };
    return body.secret ?? null;
  } catch {
    return null;
  }
}

export async function driveFlows(baseURL: string): Promise<void> {
  for (const scenario of SCENARIOS) {
    const jwt = cookieToken(AUTH[scenario.state]);
    const pat = await issuePat(baseURL, jwt);
    if (pat === null) {
      console.log(`flow(${scenario.state}): PAT refused, skipping`);
      continue;
    }
    const inference = await fetch(`${baseURL}/v1/messages`, {
      method: 'POST',
      headers: {
        authorization: `Bearer ${pat}`,
        'x-session-id': scenario.session,
        'anthropic-version': '2023-06-01',
        'content-type': 'application/json',
      },
      body: JSON.stringify({
        model: 'e2e-mock-sonnet',
        max_tokens: 512,
        messages: [{ role: 'user', content: scenario.prompt }],
      }),
    });
    if (!inference.ok) {
      throw new Error(`flow(${scenario.state}): /v1/messages returned ${inference.status}`);
    }
    const reply = (await inference.json()) as {
      model: string;
      content: { type: string; text: string }[];
      usage: { input_tokens: number; output_tokens: number };
    };
    const capture = await fetch(`${baseURL}/api/public/hooks/transcript`, {
      method: 'POST',
      headers: { authorization: `Bearer ${jwt}`, 'content-type': 'application/json' },
      body: JSON.stringify({
        session_id: scenario.session,
        transcript: [
          { role: 'user', content: scenario.prompt },
          {
            role: 'assistant',
            model: reply.model,
            usage: reply.usage,
            content: reply.content,
          },
        ],
      }),
    });
    if (capture.status !== 204) {
      throw new Error(`flow(${scenario.state}): transcript capture returned ${capture.status}`);
    }
    console.log(`flow(${scenario.state}): governed call + transcript captured`);
  }
}
