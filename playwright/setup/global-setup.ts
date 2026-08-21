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
  platformAdmin: join(AUTH_DIR, 'platform-admin.json'),
  user: join(AUTH_DIR, 'user.json'),
};

function issuerFromProfile(): string {
  try {
    const profile = readFileSync(
      join(REPO, '.systemprompt', 'profiles', 'local', 'profile.yaml'),
      'utf8',
    );
    const m = profile.match(/^\s*jwt_issuer:\s*(\S+)/m);
    if (m) return m[1];
  } catch {
    // fall through to the default
  }
  return 'http://localhost:8080';
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
    throw new Error(
      `auth sanity check failed: ${path} returned ${res.status}, expected ${expect.join('/')}. ` +
        `A token minted from signing_key.pem was rejected — if the key was rotated, restart the stack.`,
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

  const issuer = issuerFromProfile();
  const pemPath = process.env.E2E_SIGNING_KEY ?? join(REPO, 'signing_key.pem');

  mkdirSync(AUTH_DIR, { recursive: true });
  const principals = [
    { p: E2E.admin, out: AUTH.admin },
    { p: E2E.platformAdmin, out: AUTH.platformAdmin },
    { p: E2E.user, out: AUTH.user },
  ];
  for (const { p, out } of principals) {
    const token = await mintToken({
      userId: p.id,
      email: p.email,
      issuer,
      kid,
      pemPath,
      sessionId: E2E_SESSIONS[p.id],
    });
    writeFileSync(out, storageState(baseURL, token));
  }

  // Fail loudly here rather than as thirty cryptic spec failures: the DB (not
  // the token) decides roles, so these three responses prove seeding + minting
  // + cookie extraction all line up.
  await sanityCheck(baseURL, AUTH.admin, '/api/public/admin/users', [200]);
  await sanityCheck(baseURL, AUTH.platformAdmin, '/api/public/admin/management/organizations', [200]);
  await sanityCheck(baseURL, AUTH.user, '/api/public/admin/users', [401, 403]);
}
