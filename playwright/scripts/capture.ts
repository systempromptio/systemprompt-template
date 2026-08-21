// Deliverable full-page screenshots of the key admin pages, written to the
// gitignored playwright/screenshots/<timestamp>/ directory. Reuses the
// storageStates minted by global-setup; run `just e2e` (or any spec) first if
// .auth/ is empty, or this script will mint them itself via the same setup.
import { mkdirSync, existsSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from '@playwright/test';
import globalSetup, { AUTH } from '../setup/global-setup';

const HERE = __dirname;
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8080';

const PAGES: { name: string; path: string; state: string }[] = [
  { name: 'login', path: '/admin/login', state: '' },
  { name: 'analytics-overview', path: '/admin/analytics?tab=overview', state: AUTH.platformAdmin },
  { name: 'analytics-usage', path: '/admin/analytics?tab=usage', state: AUTH.platformAdmin },
  { name: 'analytics-seats', path: '/admin/analytics?tab=seats', state: AUTH.platformAdmin },
  { name: 'analytics-spend', path: '/admin/analytics?tab=spend&org=e2e-corp', state: AUTH.platformAdmin },
  { name: 'analytics-code', path: '/admin/analytics?tab=code', state: AUTH.platformAdmin },
  {
    name: 'analytics-user-drilldown',
    path: '/admin/analytics/users/e2e-member-1',
    state: AUTH.platformAdmin,
  },
  { name: 'users-roster', path: '/admin/access/users', state: AUTH.admin },
  { name: 'user-detail', path: '/admin/access/user?id=e2e-member-1', state: AUTH.platformAdmin },
  { name: 'requests-log', path: '/admin/entities/requests', state: AUTH.platformAdmin },
];

async function main() {
  if (!existsSync(AUTH.admin)) {
    await globalSetup({ projects: [{ use: { baseURL: BASE } }] } as never);
  }
  const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const outDir = join(HERE, '..', 'screenshots', stamp);
  mkdirSync(outDir, { recursive: true });

  const browser = await chromium.launch();
  for (const p of PAGES) {
    const context = await browser.newContext({
      baseURL: BASE,
      viewport: { width: 1440, height: 900 },
      ...(p.state ? { storageState: p.state } : {}),
    });
    const page = await context.newPage();
    await page.goto(p.path, { waitUntil: 'networkidle' });
    await page.screenshot({ path: join(outDir, `${p.name}.png`), fullPage: true });
    await context.close();
    console.log(`captured ${p.name}`);
  }
  await browser.close();
  console.log(`screenshots in ${outDir}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
