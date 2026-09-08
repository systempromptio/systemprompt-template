// Per-principal fixtures.
//
// Playwright resolves `storageState` per FILE, not per test — so a spec that
// drives two principals cannot get them by overriding that option (one
// declaration silently wins for the whole file and the other principal runs
// anonymous). Each principal is therefore its own browser context, exposed as
// a page fixture, which composes freely inside one file.
import { test as base, type APIRequestContext, type Browser, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { Client, type QueryResultRow } from 'pg';
import { databaseUrl } from '../../setup/seed';

const AUTH_DIR = join(__dirname, '..', '..', '.auth');

export const AUTH = {
  admin: join(AUTH_DIR, 'admin.json'),
  user: join(AUTH_DIR, 'user.json'),
};

export type Principal = keyof typeof AUTH;

// The authorization block every page spec carries, as one table: the console
// pages render for the admin, bounce the plain user to their profile (303)
// and send an anonymous visitor to sign in (307). The account pages override
// the middle row to 200; the sign-in page overrides all three.
export type AccessRow = { principal: Principal | 'anon'; status: number };
export const CONSOLE_ACCESS: AccessRow[] = [
  { principal: 'admin', status: 200 },
  { principal: 'user', status: 303 },
  { principal: 'anon', status: 307 },
];
export const ACCOUNT_ACCESS: AccessRow[] = [
  { principal: 'admin', status: 200 },
  { principal: 'user', status: 200 },
  { principal: 'anon', status: 307 },
];

async function pageFor(browser: Browser, principal: Principal, use: (p: Page) => Promise<void>) {
  const context = await browser.newContext({ storageState: AUTH[principal] });
  const page = await context.newPage();
  await use(page);
  await context.close();
}

interface PrincipalPages {
  adminPage: Page;
  userPage: Page;
  anonPage: Page;
}

export const test = base.extend<PrincipalPages>({
  adminPage: async ({ browser }, use) => {
    await pageFor(browser, 'admin', use);
  },
  userPage: async ({ browser }, use) => {
    await pageFor(browser, 'user', use);
  },
  anonPage: async ({ browser }, use) => {
    const context = await browser.newContext();
    const page = await context.newPage();
    await use(page);
    await context.close();
  },
});

export const expect = base.expect;

/** Cookie header for API-level calls as a given principal. */
export function cookieFor(principal: Principal): { cookie: string } {
  const state = JSON.parse(readFileSync(AUTH[principal], 'utf8'));
  const c = state.cookies[0];
  return { cookie: `${c.name}=${c.value}` };
}

/** API helper: request options carrying a principal's cookie. */
export function apiAs(
  _request: APIRequestContext,
  principal: Principal,
): { headers: { cookie: string } } {
  return { headers: cookieFor(principal) };
}

let runId = '';
/** Unique, per-run e2e email that the seed's --reset backstop will clean up. */
export function uniqueEmail(prefix: string): string {
  if (!runId) runId = Math.random().toString(36).slice(2, 8);
  return `e2e-${prefix}-${runId}-${Date.now() % 100000}@e2e.local`;
}

/** Visual comparison for a page or region.
 *
 *  Baselines are per project and platform (see snapshotPathTemplate), so a
 *  developer on macOS never rewrites the Linux baselines CI compares against;
 *  a run on any other platform asserts nothing rather than failing on font
 *  rendering it cannot control.
 */
export async function snapshot(page: Page, name: string): Promise<void> {
  if (process.platform !== 'linux') return;
  await page.waitForLoadState('networkidle');
  await expect(page).toHaveScreenshot(`${name}.png`, { fullPage: true });
}

/** Read one row straight from the database the seed wrote.
 *
 *  For assertions the UI cannot make honestly — that a delete actually removed
 *  the row rather than only the table cell, that a warn-mode call was audited.
 */
export async function dbRow<T extends QueryResultRow>(
  sql: string,
  params: unknown[] = [],
): Promise<T | null> {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    const { rows } = await db.query<T>(sql, params);
    return rows[0] ?? null;
  } finally {
    await db.end();
  }
}

/** Every row of a query, for count and ordering assertions. */
export async function dbRows<T extends QueryResultRow>(
  sql: string,
  params: unknown[] = [],
): Promise<T[]> {
  const db = new Client({ connectionString: databaseUrl() });
  await db.connect();
  try {
    const { rows } = await db.query<T>(sql, params);
    return rows;
  } finally {
    await db.end();
  }
}
