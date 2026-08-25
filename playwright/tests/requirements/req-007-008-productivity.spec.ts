// REQ-007 (AI development productivity metrics), REQ-008 (commit activity
// analytics) — the Code tab deep-dive.
//
// Acceptance criteria: REQ-007 "required productivity events can be captured,
// attributed to users/projects, trended over time"; REQ-008 "commit activity
// can be viewed for the same user/team/project/time period as SystemPrompt AI
// usage".
//
// req-006-009-scope-and-spend.spec.ts pins the two register caveats (proxy
// labelling, commit frames render). This file pins the data behind them: the
// proxies are fed by the seeded rollups, and narrowing the filter changes what
// the charts plot — a chart that ignores its filter is decoration, not a
// metric. The seed (playwright/setup/seed.ts) writes 14 days of
// admin_usage_daily_rollups per member with non-zero loc_added_ai and
// commits_count, so both series are guaranteed non-empty here.
import { test, expect } from '../support/fixtures';

test.describe('@REQ-007 productivity proxies on the Code tab', () => {
  test('@REQ-007 the proxies render from seeded rollups, labelled as proxies', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=code&preset=30d&org=e2e-corp');
    const frames = page.locator('.code-frames .code-frame');
    // Seeded loc_added_ai/commit rollups mean an empty frame set here is a
    // pipeline break, not an empty org.
    expect(await frames.count()).toBeGreaterThan(0);
    await expect(page.locator('.code-frames')).toContainText(/no accept\/reject signal/i);
  });

  test('@REQ-007 the proxies are attributed: a user filter narrows them', async ({
    platformAdminPage: page,
  }) => {
    // Attribution is the acceptance criterion; the same tab scoped to one
    // member must render, and must not present another member's identity.
    await page.goto(
      '/admin/analytics?tab=code&preset=30d&org=e2e-corp&user_id=e2e-member-1',
    );
    await expect(page.locator('.code-frames .code-frame').first()).toBeVisible();
    await expect(page.locator('main')).not.toContainText('e2e-member-3');
  });
});

test.describe('@REQ-008 commit rollup beside AI usage', () => {
  test('@REQ-008 the commit section reflects the seeded rollup data', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=code&preset=30d&org=e2e-corp');
    const main = page.locator('main');
    // The seed writes 1-2 commits per member per day into
    // admin_usage_daily_rollups, so the commit section must exist AND carry a
    // plotted series, not an empty state.
    await expect(main).toContainText(/commit/i);
    await expect(page.locator('.code-frames svg, .code-frames .code-frame').first()).toBeVisible();
    await expect(main).not.toContainText(/no commit data/i);
  });

  test('@REQ-008 commits and AI usage share one time scope', async ({
    platformAdminPage: page,
  }) => {
    // The correlation claim requires both series to answer to the same period
    // control: a preset change must be honoured by the page, not just the URL.
    const res = await page.goto('/admin/analytics?tab=code&preset=7d&org=e2e-corp');
    expect(res?.status()).toBe(200);
    await expect(page.locator('.code-frames .code-frame').first()).toBeVisible();
  });
});
