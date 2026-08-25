// Invite lifecycle from the roster page: mint via UI, list, revoke, plus the
// API contract for the invite path shape and the non-admin refusal. Every
// invite uses a unique @e2e.local email and is revoked in-test.
import { test, expect, apiAs, uniqueEmail } from './support/fixtures';

test.describe('invites panel', () => {
  test('mints an invite from the UI and lists it', async ({ adminPage: page }) => {
    const email = uniqueEmail('invite-ui');
    await page.goto('/admin/access/users');
    await page.locator('#btn-invite-user').click();
    await expect(page.locator('#invites-section')).toBeVisible();

    await page.locator('#invite-email').fill(email);
    await page.locator('#invite-department').fill('Engineering');
    await page.locator('#btn-create-invite').click();

    const row = page.locator('#invites-list tr', { hasText: email });
    await expect(row).toBeVisible();

    // Revoke it so reruns stay clean. Scoped to THIS invite's row: a parallel
    // worker can have its own invite listed first, and `.first()` would revoke
    // that one instead.
    await row.locator('[data-revoke]').click();
    await expect(page.locator('#invites-list')).not.toContainText(email);
  });

  test('API returns an invite_path with the raw token', async ({ request }) => {
    const email = uniqueEmail('invite-api');
    const res = await request.post('/api/public/admin/invites', {
      ...apiAs(request, 'admin'),
      data: { email },
    });
    expect(res.status()).toBe(201);
    const body = (await res.json()) as { id: string; invite_path: string };
    expect(body.invite_path).toMatch(/^\/admin\/invite\/.+/);
    await request.delete(`/api/public/admin/invites/${body.id}`, apiAs(request, 'admin'));
  });

  test('a second live invite for the same email conflicts', async ({ request }) => {
    const email = uniqueEmail('invite-dup');
    const first = await request.post('/api/public/admin/invites', {
      ...apiAs(request, 'admin'),
      data: { email },
    });
    expect(first.status()).toBe(201);
    const second = await request.post('/api/public/admin/invites', {
      ...apiAs(request, 'admin'),
      data: { email },
    });
    expect(second.status()).toBe(409);
    const { id } = (await first.json()) as { id: string };
    await request.delete(`/api/public/admin/invites/${id}`, apiAs(request, 'admin'));
  });
});

test('non-admins cannot mint invites', async ({ request }) => {
  const res = await request.post('/api/public/admin/invites', {
    ...apiAs(request, 'user'),
    data: { email: uniqueEmail('refused') },
  });
  expect([401, 403]).toContain(res.status());
});

test('anonymous cannot mint invites', async ({ request }) => {
  const res = await request.post('/api/public/admin/invites', {
    data: { email: uniqueEmail('anon') },
  });
  expect([401, 403]).toContain(res.status());
});
