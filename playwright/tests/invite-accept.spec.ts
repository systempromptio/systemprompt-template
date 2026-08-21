// The full invite journey through the production sign-in machinery: mint an
// invite as admin, open the public accept page in a fresh anonymous context,
// enrol a real passkey via CDP's virtual authenticator, and land signed in.
// Serial: WebAuthn + PKCE redirects are the flake hotspot; keep one at a time.
import { test as base, expect } from '@playwright/test';
import { apiAs, uniqueEmail } from './support/fixtures';

base.describe.configure({ mode: 'serial' });

base.describe('invite accept journey', () => {
  base('happy path: accept, enrol passkey, auto sign-in', async ({ browser, playwright }) => {
    // Mint the invite as admin over the API.
    const api = await playwright.request.newContext({
      baseURL: base.info().project.use.baseURL as string,
    });
    const email = uniqueEmail('journey');
    const minted = await api.post('/api/public/admin/invites', {
      ...apiAs(api, 'admin'),
      data: { email },
    });
    expect(minted.status()).toBe(201);
    const { invite_path } = (await minted.json()) as { invite_path: string };

    // Fresh anonymous context with a virtual authenticator.
    const context = await browser.newContext();
    const page = await context.newPage();
    const cdp = await context.newCDPSession(page);
    await cdp.send('WebAuthn.enable');
    await cdp.send('WebAuthn.addVirtualAuthenticator', {
      options: {
        protocol: 'ctap2',
        transport: 'internal',
        hasResidentKey: true,
        hasUserVerification: true,
        isUserVerified: true,
        automaticPresenceSimulation: true,
      },
    });

    await page.goto(invite_path);
    await expect(page.locator('#pane-invite')).toBeVisible();
    await expect(page.locator('#pane-invite')).toContainText(email);

    await page.locator('#accept-btn').click();

    // Passkey enrolment + PKCE sign-in chain; generous timeout by design.
    await page.waitForURL(/\/admin\/(profile|analytics)/, { timeout: 30_000 });
    const cookies = await context.cookies();
    expect(cookies.some((c) => c.name === 'access_token')).toBe(true);

    await context.close();
    await api.dispose();
  });

  base('an invalid token shows the invalid pane', async ({ page }) => {
    await page.goto('/admin/invite/this-token-does-not-exist');
    await expect(page.locator('#pane-invalid')).toBeVisible();
  });

  base('a consumed token cannot be accepted twice', async ({ browser, playwright }) => {
    const api = await playwright.request.newContext({
      baseURL: base.info().project.use.baseURL as string,
    });
    const email = uniqueEmail('reuse');
    const minted = await api.post('/api/public/admin/invites', {
      ...apiAs(api, 'admin'),
      data: { email },
    });
    const { invite_path } = (await minted.json()) as { invite_path: string };
    const token = invite_path.split('/').pop();

    // Consume it directly over the accept endpoint.
    const first = await api.post('/admin/auth/invite/accept', { data: { token } });
    expect(first.status()).toBe(200);

    const context = await browser.newContext();
    const page = await context.newPage();
    await page.goto(invite_path);
    await expect(page.locator('#pane-invalid')).toBeVisible();
    await context.close();
    await api.dispose();
  });
});
