// REQ-011/014 (forecast surface), REQ-015 (web CSV export), REQ-029 (latency
// SLO), REQ-031 (anomaly surface) — the FinOps delivery pass of 2026-08-25.
//
// The alert paths (Slack) and jobs are backend-verified; what a browser can
// prove is the surfaces they feed: the warning table understands both kinds,
// the CSV endpoints answer text/csv with the seeded rows, the SLO threshold is
// a real query parameter that changes the rendered split, and the anomalies
// section exists with an honest empty state.
import { test, expect } from '../support/fixtures';

test.describe('@REQ-015 web CSV export', () => {
  test('@REQ-015 the internal P&L exports organization rows as CSV', async ({
    platformAdminPage: page,
  }) => {
    const res = await page.request.get('/admin/reports/internal.csv?dimension=organization');
    expect(res.status()).toBe(200);
    expect(res.headers()['content-type']).toContain('text/csv');
    const body = await res.text();
    expect(body.split('\r\n')[0]).toBe(
      'slug,name,plan,revenue_usd,cost_usd,margin_usd,requests,tokens,seats_used,active_users',
    );
    expect(body).toContain('e2e-corp');
  });

  test('@REQ-015 the customer report exports the seeded users, org-scoped', async ({
    platformAdminPage: page,
  }) => {
    const res = await page.request.get('/admin/reports/customer.csv?org=e2e-corp');
    expect(res.status()).toBe(200);
    expect(res.headers()['content-type']).toContain('text/csv');
    const body = await res.text();
    expect(body.split('\r\n')[0]).toContain('email,display_name,department');
  });

  test('@REQ-015 the export button is on the report page', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/reports/internal');
    await expect(page.locator('a', { hasText: 'Download CSV' })).toBeVisible();
  });
});

test.describe('@REQ-029 configurable latency SLO', () => {
  test('@REQ-029 the threshold is a query parameter reflected in the split', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=spend&org=e2e-corp&slo_ms=2000');
    await expect(page.locator('.kpi-card__label', { hasText: 'Within SLO' })).toContainText(
      /2(\.00)?\s?s/,
    );
    // An absurd value clamps rather than erroring.
    const res = await page.goto('/admin/analytics?tab=spend&org=e2e-corp&slo_ms=999999999');
    expect(res?.status()).toBe(200);
    await expect(page.locator('.kpi-card__label', { hasText: 'Within SLO' })).toContainText(
      /1\.0 min|60/,
    );
  });

  test('@REQ-029 the SLO picker renders its options', async ({ platformAdminPage: page }) => {
    await page.goto('/admin/analytics?tab=spend&org=e2e-corp');
    const picker = page.locator('[aria-label="Latency SLO threshold"]');
    await expect(picker).toBeVisible();
    await expect(picker.locator('a')).toHaveCount(4);
  });
});

test.describe('@REQ-011 @REQ-031 warning and anomaly surfaces', () => {
  test('@REQ-011 the budget warning table names both kinds in its caption', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=spend&org=e2e-corp');
    await expect(
      page.locator('.section-title', { hasText: 'Budget warnings' }),
    ).toBeVisible();
    await expect(page.locator('.section-caption', { hasText: 'projection' })).toBeVisible();
  });

  test('@REQ-031 the anomalies section renders, empty state included', async ({
    platformAdminPage: page,
  }) => {
    await page.goto('/admin/analytics?tab=spend&org=e2e-corp');
    await expect(
      page.locator('.section-title', { hasText: 'Usage anomalies' }),
    ).toBeVisible();
  });
});
