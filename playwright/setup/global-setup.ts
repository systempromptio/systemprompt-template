// Global setup: fail fast on a dead stack, seed deterministic e2e data, mint
// one JWT per principal, and write storageState files the fixtures consume.
//
// It never boots a server (the stack may be shared with other agents on this
// clone — `just start` is the operator's job) and the minted cookies live only
// in the gitignored playwright/.auth/ directory.
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import type { FullConfig } from '@playwright/test';
import { mintToken } from './mint-token';
import { E2E, E2E_SESSIONS, seed } from './seed';

const REPO = join(__dirname, '..', '..');
const AUTH_DIR = join(__dirname, '..', '.auth');

export const AUTH = {
  admin: join(AUTH_DIR, 'admin.json'),
  user: join(AUTH_DIR, 'user.json'),
};

// The issuer must be the one the server under test verifies against, and this
// clone's profile is not always that server's profile: pointed at another
// worktree's port, reading jwt_issuer from here mints tokens a perfectly
// healthy server rejects with a 401 that says nothing about why.
//
// So: an explicit override wins, then the profile — but only when it names the
// same origin we are testing. Otherwise the base URL is the better guess,
// because jwt_issuer is the deployment's own address.
function resolveIssuer(baseURL: string): string {
  if (process.env.E2E_JWT_ISSUER) return process.env.E2E_JWT_ISSUER;
  try {
    const profile = readFileSync(
      join(REPO, '.systemprompt', 'profiles', 'local', 'profile.yaml'),
      'utf8',
    );
    const m = profile.match(/^\s*jwt_issuer:\s*(\S+)/m);
    if (m && new URL(m[1]).origin === new URL(baseURL).origin) return m[1];
  } catch {
    // fall through to the base URL
  }
  return new URL(baseURL).origin;
}

function storageState(baseURL: string, token: string): string {
  const host = new URL(baseURL).hostname;
  return JSON.stringify(
    {
      cookies: [
        {
          name: 'access_token',
          value: token,
          domain: host,
          path: '/',
          httpOnly: true,
          secure: false,
          sameSite: 'Lax',
          expires: Math.floor(Date.now() / 1000) + 2 * 60 * 60,
        },
      ],
      origins: [],
    },
    null,
    2,
  );
}

async function sanityCheck(baseURL: string, statePath: string, path: string, expect: number[]) {
  const state = JSON.parse(readFileSync(statePath, 'utf8'));
  const cookie = state.cookies[0];
  const res = await fetch(`${baseURL}${path}`, {
    headers: { cookie: `${cookie.name}=${cookie.value}` },
    redirect: 'manual',
  });
  if (!expect.includes(res.status)) {
    // A 404 is a different fault from a rejected token and wants a different
    // fix: the route moved or was never mounted on this build. Saying
    // "the key was rotated" about a missing page sends the reader to the wrong
    // place.
    const cause =
      res.status === 404
        ? `no such route on this build. Either the console's paths moved and this check ` +
          `wants updating, or the admin SSR router failed to mount — one unparseable ` +
          `template takes out every admin page at once, and the server log names it`
        : `a token minted from ${process.env.E2E_SIGNING_KEY ?? 'signing_key.pem'} was rejected; ` +
          `if the key was rotated, restart the stack, and check E2E_JWT_ISSUER matches the server`;
    throw new Error(
      `auth sanity check failed: ${path} returned ${res.status}, expected ${expect.join('/')} — ${cause}.`,
    );
  }
}

export default async function globalSetup(config: FullConfig) {
  const baseURL =
    config.projects[0]?.use?.baseURL ?? process.env.GATEWAY_URL ?? 'http://localhost:8080';

  let health: Response;
  try {
    health = await fetch(`${baseURL}/health`);
  } catch {
    throw new Error(
      `E2E needs a running stack at ${baseURL}: run 'just start' (or set GATEWAY_URL). ` +
        `Not booting one here — the server may be shared with other agents.`,
    );
  }
  if (!health.ok) throw new Error(`health check at ${baseURL}/health returned ${health.status}`);

  await seed();

  const jwks = (await (await fetch(`${baseURL}/.well-known/jwks.json`)).json()) as {
    keys: { kid: string }[];
  };
  const kid = jwks.keys[0]?.kid;
  if (!kid) throw new Error('no kid in /.well-known/jwks.json');

  const issuer = resolveIssuer(baseURL);
  let signingPem: string;
  if (process.env.E2E_SIGNING_KEY) {
    signingPem = readFileSync(process.env.E2E_SIGNING_KEY, 'utf8');
  } else {
    const secrets = JSON.parse(readFileSync(
      join(REPO, '.systemprompt', 'profiles', 'local', 'secrets.json'), 'utf8',
    ));
    signingPem = secrets.signing_key_pem
      ? Buffer.from(secrets.signing_key_pem, 'base64').toString('utf8')
      : readFileSync(join(REPO, 'signing_key.pem'), 'utf8');
  }

  mkdirSync(AUTH_DIR, { recursive: true });
  const principals = [
    { p: E2E.admin, out: AUTH.admin },
    { p: E2E.user, out: AUTH.user },
  ];
  for (const { p, out } of principals) {
    const token = await mintToken({
      userId: p.id,
      email: p.email,
      issuer,
      kid,
      signingPem,
      sessionId: E2E_SESSIONS[p.id],
    });
    writeFileSync(out, storageState(baseURL, token));
  }

  // Fail loudly here rather than as thirty cryptic spec failures: the DB (not
  // the token) decides roles, so these responses prove seeding + minting +
  // cookie extraction all line up. The admin reaches the API and the console;
  // the plain user is refused by the API and bounced by the console's
  // non-admin gate to their profile, which they may read.
  await sanityCheck(baseURL, AUTH.admin, '/api/public/admin/users', [200]);
  await sanityCheck(baseURL, AUTH.admin, '/admin/users', [200]);
  await sanityCheck(baseURL, AUTH.user, '/api/public/admin/users', [401, 403]);
  await sanityCheck(baseURL, AUTH.user, '/admin/users', [303, 302]);
  await sanityCheck(baseURL, AUTH.user, '/admin/profile', [200]);
}
