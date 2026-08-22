// The requirements evidence pack: one full-page screenshot per REQ row of
// requirements/compliance-register.md, written into the tracked
// requirements/evidence/ directory alongside an index.md recording the URL and
// principal each came from.
//
// Written straight into the repo rather than into a gitignored scratch
// directory that someone then hand-copies: the previous pack under
// docs/screenshots-* had no record of which URL or principal produced a given
// PNG, so it could not be re-derived or trusted.
//
// Reuses the storageStates minted by global-setup; run `just e2e` (or any spec)
// first if .auth/ is empty, or this script mints them itself via the same setup.
import { mkdirSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { chromium } from '@playwright/test';
import globalSetup, { AUTH } from '../setup/global-setup';

const HERE = __dirname;
const BASE = process.env.GATEWAY_URL ?? 'http://localhost:8080';

// Named by requirement, not by page: this directory is the evidence pack for
// requirements/compliance-register.md, and a reviewer reading a register row
// needs to find its screenshot by its REQ id.
const PAGES: { name: string; path: string; state: string }[] = [
  { name: 'req-002-login-invite-only', path: '/admin/login', state: '' },

  { name: 'req-001-users-roster', path: '/admin/access/users', state: AUTH.admin },
  {
    name: 'req-001-user-detail',
    path: '/admin/access/user?id=e2e-member-1',
    state: AUTH.platformAdmin,
  },

  {
    name: 'req-003-overview',
    path: '/admin/analytics?tab=overview&preset=30d&org=e2e-corp',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-003-usage-trends',
    path: '/admin/analytics?tab=overview&preset=30d&bucket=week&org=e2e-corp',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-004-spend',
    path: '/admin/analytics?tab=spend&org=e2e-corp',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-004-model-mix',
    path: '/admin/entities/requests?tab=models',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-005-seats',
    path: '/admin/analytics?tab=seats&org=e2e-corp&inactive_days=30',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-005-inactive-seats',
    path: '/admin/analytics?tab=seats&org=e2e-corp&inactive_days=90',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-006-drilldown',
    path: '/admin/analytics?tab=usage&preset=30d&org=e2e-corp&department=Engineering',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-006-user-drilldown',
    path: '/admin/analytics/users/e2e-member-1',
    state: AUTH.platformAdmin,
  },
  {
    // One capture for both rows: the Code tab is where REQ-007's labelled
    // proxies and REQ-008's commit activity are read side by side, and two
    // files of the same URL would be duplicate bytes pretending to be
    // independent evidence.
    name: 'req-007-008-code-tab',
    path: '/admin/analytics?tab=code&preset=30d&org=e2e-corp',
    state: AUTH.platformAdmin,
  },
  {
    name: 'req-009-spend-meters',
    path: '/admin/analytics?tab=spend&org=e2e-corp',
    state: AUTH.platformAdmin,
  },

  { name: 'requests-log', path: '/admin/entities/requests', state: AUTH.platformAdmin },
];

async function main() {
  // Always re-mint. Checking only that the files EXIST used a previous run's
  // expired tokens, which the server silently downgrades to anonymous -- so
  // every capture became a photograph of the login page and the script still
  // reported success. globalSetup is idempotent, so re-running it is cheap.
  await globalSetup({ projects: [{ use: { baseURL: BASE } }] } as never);
  const stamp = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19);
  const outDir = join(HERE, '..', '..', 'requirements', 'evidence');
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

    // An authenticated capture that landed on the login page is a failed
    // capture, however good the PNG looks. Fail loudly rather than shipping a
    // pack of identical login screenshots as requirement evidence.
    if (p.state && new URL(page.url()).pathname === '/admin/login') {
      throw new Error(
        `${p.name}: expected ${p.path} as an authenticated principal but landed on /admin/login. ` +
          'The storage state did not authenticate.',
      );
    }
    await page.screenshot({ path: join(outDir, `${p.name}.png`), fullPage: true });
    await context.close();
    console.log(`captured ${p.name}`);
  }
  await browser.close();

  // Why a manifest ships beside the images: docs/screenshots-* was a hand-copy
  // out of this gitignored directory with nothing recording where a given PNG
  // came from, so a reviewer could not tell which URL or principal produced it.
  const manifest = [
    `# Requirements evidence pack`,
    ``,
    `Screenshot evidence for [compliance-register.md](../compliance-register.md).`,
    `Regenerate with \`just e2e-screens\` against a running \`just start\`; this file`,
    `is written by the same script, so it cannot drift from the images beside it.`,
    ``,
    `Captured ${stamp} against ${BASE}.`,
    `Principals come from the deterministic e2e seed (\`just e2e-seed --reset\`).`,
    ``,
    `| File | Path | Principal |`,
    `|------|------|-----------|`,
    ...PAGES.map(
      (p) =>
        `| ${p.name}.png | \`${p.path}\` | ${
          p.state === '' ? 'anonymous' : p.state.endsWith('platform-admin.json') ? 'platform admin' : 'admin'
        } |`,
    ),
  ].join('\n');
  writeFileSync(join(outDir, 'index.md'), `${manifest}\n`);

  console.log(`screenshots in ${outDir}`);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
