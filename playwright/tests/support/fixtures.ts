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

const AUTH_DIR = join(__dirname, '..', '..', '.auth');

export const AUTH = {
  admin: join(AUTH_DIR, 'admin.json'),
  platformAdmin: join(AUTH_DIR, 'platform-admin.json'),
  user: join(AUTH_DIR, 'user.json'),
};

export type Principal = keyof typeof AUTH;

async function pageFor(browser: Browser, principal: Principal, use: (p: Page) => Promise<void>) {
  const context = await browser.newContext({ storageState: AUTH[principal] });
  const page = await context.newPage();
  await use(page);
  await context.close();
}

interface PrincipalPages {
  adminPage: Page;
  platformAdminPage: Page;
  userPage: Page;
  anonPage: Page;
}

export const test = base.extend<PrincipalPages>({
  adminPage: async ({ browser }, use) => {
    await pageFor(browser, 'admin', use);
  },
  platformAdminPage: async ({ browser }, use) => {
    await pageFor(browser, 'platformAdmin', use);
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
