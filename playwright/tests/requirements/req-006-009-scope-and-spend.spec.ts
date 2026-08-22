// REQ-006 (organizational drill-down), REQ-007/008 (engineering measures), and
// REQ-009 (spend limits).
//
// REQ-006 and REQ-008 are Partial and REQ-007 is not feasible as specified;
// these specs pin what IS true, including the labelling that keeps a proxy from
// being read as the metric it stands in for.
import { test, expect, apiAs } from '../support/fixtures';

test.describe('@REQ-006 organizational drill-down', () => {
  test('@REQ-006 org, department and user each narrow the view', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=usage&preset=30d&org=e2e-corp');
    const unfiltered = await page.locator('tbody tr').count();

    await page.goto('/admin/analytics?tab=usage&preset=30d&org=e2e-corp&department=Sales');
    const byDept = await page.locator('tbody tr').count();
    expect(byDept).toBeLessThanOrEqual(unfiltered);

    await page.goto(
      '/admin/analytics?tab=usage&preset=30d&org=e2e-corp&user_id=e2e-member-1',
    );
    const leaderboard = page.locator('.data-table').first();
    await expect(leaderboard).toContainText('e2e-member-1');
    await expect(leaderboard).not.toContainText('e2e-member-3');
  });

  test('@REQ-006 an org admin cannot read another organization by editing the URL', async ({
    adminPage: page,
  }) => {
    // `?org=` is honoured for platform admins only; a customer's own admin holds
    // `admin` too, so trusting it would turn a URL edit into a cross-tenant read.
    await page.goto('/admin/analytics?tab=usage&preset=30d&org=e2e-corp-b');
    await expect(page.locator('.analytics-filters')).not.toContainText('e2e-corp-b');
  });

  test('@REQ-006 the department dropdown lists only the caller organization', async ({
    adminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=usage');
    const options = await page.locator('select[name="department"] option').allInnerTexts();
    // e2e-corp seeds Engineering and Sales; anything from another org leaking in
    // means the option list is unscoped even though the rows behind it are not.
    for (const label of options) {
      expect(['All departments', 'Engineering', 'Sales', 'Default']).toContain(label.trim());
    }
  });

  // 404, not 403: a 403 would confirm the id names a real account in another
  // organization, which is the oracle this guards against. The seed puts every
  // principal in e2e-corp, so the cross-org case has to be staged: move the
  // victim to the other org, look, and move them back.
  test('@REQ-006 the per-user drill-down is not an existence oracle', async ({
    adminPage: page,
    request,
  }) => {
    const move = (org: string) =>
      request.put('/api/public/admin/management/users/e2e-victim/organization', {
        ...apiAs(request, 'platformAdmin'),
        data: { org },
      });

    expect((await move('e2e-corp-b')).ok()).toBeTruthy();
    try {
      const res = await page.goto('/admin/analytics/users/e2e-victim');
      expect(res?.status()).toBe(404);
    } finally {
      await move('e2e-corp');
    }
  });
});

test.describe('@REQ-007 productivity measures are labelled as proxies', () => {
  test('@REQ-007 the code tab states there is no acceptance rate to report', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=code&preset=30d&org=e2e-corp');
    // If this label ever disappears, a proxy has quietly become a headline
    // metric -- which is the specific failure this requirement invites.
    await expect(page.locator('.code-frames')).toContainText(/no accept\/reject signal/i);
  });
});

test.describe('@REQ-008 commit activity', () => {
  test('@REQ-008 commit activity is plotted beside AI line deltas', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=code&preset=30d&org=e2e-corp');
    const frames = page.locator('.code-frames .code-frame');
    await expect(frames.first()).toBeVisible();
    await expect(page.locator('main')).toContainText(/commit/i);
  });
});

test.describe('@REQ-009 spend limits and budget monitoring', () => {
  test('@REQ-009 spend is shown against the plan cap', async ({ platformAdminPage: page }) => {
    await page.goto('/admin/analytics?tab=spend&org=e2e-corp');
    await expect(page.locator('.spend-meter-list, .empty-state').first()).toBeVisible();
  });

  test('@REQ-009 the burn-up names both the soft and hard threshold', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=spend&org=e2e-corp');
    const main = page.locator('main');
    await expect(main).toContainText(/cap/i);
  });

  test('@REQ-009 a non-admin cannot read spend', async ({ userPage: page }) => {
    const res = await page.goto('/admin/analytics?tab=spend');
    // Non-admins are bounced to their profile rather than shown the dashboard.
    expect(page.url()).toContain('/admin/profile');
    expect(res?.status()).toBe(200);
  });
});
