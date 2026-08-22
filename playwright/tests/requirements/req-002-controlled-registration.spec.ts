// REQ-002 — Controlled User Registration.
//
// Acceptance criteria: "An unapproved user cannot create an account or connect
// a Bridge. Approved/provisioned users can access the platform based on
// assigned entitlements."
//
// The negative cases are the requirement. An in-domain address being refused is
// the assertion that matters most: before closed enrolment it was the one that
// would have succeeded.
import { test, expect, apiAs, uniqueEmail } from '../support/fixtures';

test.describe('@REQ-002 controlled registration', () => {
  test('@REQ-002 an out-of-domain address cannot self-register', async ({ request }) => {
    const resp = await request.post('/admin/auth/passkey/register', {
      data: { email: 'stranger@not-astound.example', display_name: 'Stranger' },
    });
    expect(resp.status()).toBe(403);
  });

  test('@REQ-002 an allow-listed domain cannot self-register either', async ({ request }) => {
    // The domain list says an address *could* belong to someone who should have
    // access. It never says anyone approved them.
    const resp = await request.post('/admin/auth/passkey/register', {
      data: { email: uniqueEmail('selfreg').replace('@e2e.local', '@astounddigital.com') },
    });
    expect(resp.status()).toBe(403);
  });

  test('@REQ-002 the login page offers no self-registration path', async ({ anonPage: page }) => {
    await page.goto('/admin/login');
    await expect(page.locator('#register-form')).toHaveCount(0);
    await expect(page.locator('#show-register')).toHaveCount(0);
    await expect(page.locator('.login-switch').first()).toContainText('invite');
  });

  test('@REQ-002 an admin invite is the way in', async ({ request }) => {
    const email = uniqueEmail('invited');
    const created = await request.post('/api/public/admin/invites', {
      ...apiAs(request, 'platformAdmin'),
      data: { email, org: 'e2e-corp', department: 'Engineering', roles: ['user'] },
    });
    expect(created.ok()).toBeTruthy();

    const listed = await request.get('/api/public/admin/invites', apiAs(request, 'platformAdmin'));
    const body = (await listed.json()) as { invites?: { email: string }[] } | { email: string }[];
    const invites = Array.isArray(body) ? body : (body.invites ?? []);
    expect(invites.some((i) => i.email === email)).toBeTruthy();
  });

  test('@REQ-002 an anonymous caller cannot mint a bridge connect code', async ({ request }) => {
    // maxRedirects: 0 or the client follows the bounce to /admin/login and
    // reports the login page's 200, which would pass while proving nothing.
    const resp = await request.post('/admin/api/profile/bridge-code', { maxRedirects: 0 });
    expect([401, 303, 307]).toContain(resp.status());
  });

  test('@REQ-002 the profile page mints no code until asked', async ({ userPage: page }) => {
    await page.goto('/admin/profile');
    // A connect code is a bearer credential; it must not be in the HTML of a
    // page people leave open and reload.
    await expect(page.locator('#connect-code-output')).toBeHidden();
    await expect(page.locator('#issue-connect-code')).toBeVisible();

    await page.locator('#issue-connect-code').click();
    await expect(page.locator('#connect-code-output')).toBeVisible();
    await expect(page.locator('[data-connect-field="code"]')).not.toBeEmpty();
  });
});
