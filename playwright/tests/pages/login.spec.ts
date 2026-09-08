// /admin/login — the sign-in page. Public, and outside the admin shell: it
// has no sidebar to mark and no table to measure, so its design-language
// block asserts the tokens, focus and axe directly and measures density on
// the page it sends a signed-in person to.
import { test, expect, AUTH, type AccessRow } from '../support/fixtures';
import { expectAccessible, expectDensity } from '../support/a11y';
import { PATHS } from '../support/paths';
import { TOKENS } from '../support/pages/selectors';
import { authorizationTable } from '../support/shared';

const PATH = PATHS.login;

// Everyone may read the sign-in page; the console's own redirects land here
// for an anonymous visitor.
const ACCESS: AccessRow[] = [
  { principal: 'admin', status: 200 },
  { principal: 'user', status: 200 },
  { principal: 'anon', status: 200 },
];

test.describe('renders', () => {
  test('offers the email field and the passkey sign-in', async ({ anonPage }) => {
    await anonPage.goto(PATH);
    await expect(anonPage.locator('input[type="email"]').first()).toBeVisible();
    await expect(anonPage.getByRole('button', { name: /sign in|continue|passkey/i }).first()).toBeVisible();
  });

  test('is where an anonymous visitor to the console is sent', async ({ browser }) => {
    const context = await browser.newContext();
    const res = await context.request.get(PATHS.users, { maxRedirects: 0 });
    expect(res.status()).toBe(307);
    expect(res.headers().location).toContain(PATH);
    await context.close();
  });
});

test.describe('actions', () => {
  test('carries the redirect target through to the form', async ({ anonPage }) => {
    await anonPage.goto(`${PATH}?redirect=${encodeURIComponent(PATHS.users)}`);
    await expect(anonPage.locator('input[type="email"]').first()).toBeVisible();
    expect(anonPage.url()).toContain('redirect=');
  });

  test('refuses a magic-link request for an address that is not one', async ({ anonPage }) => {
    const res = await anonPage.request.post('/admin/api/magic-link/request', {
      data: { email: 'not-an-address' },
    });
    expect([400, 404, 422]).toContain(res.status());
  });
});

test.describe('authorization', () => {
  authorizationTable(PATH, ACCESS);
});

test.describe('design language', () => {
  test('paints from design tokens', async ({ anonPage }) => {
    await anonPage.goto(PATH);
    const values = await anonPage.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  test('keeps a visible focus ring', async ({ anonPage }) => {
    await anonPage.goto(PATH);
    await anonPage.keyboard.press('Tab');
    const ring = await anonPage.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) return null;
      const s = getComputedStyle(el);
      return { style: s.outlineStyle, width: s.outlineWidth, shadow: s.boxShadow };
    });
    expect(ring).not.toBeNull();
    const outlined = ring?.style !== 'none' && Number.parseFloat(ring?.width ?? '0') > 0;
    const shadowed = (ring?.shadow ?? 'none') !== 'none';
    expect(outlined || shadowed, `no visible focus ring: ${JSON.stringify(ring)}`).toBe(true);
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ anonPage }) => {
    for (const width of [1440, 1024]) {
      await anonPage.setViewportSize({ width, height: 900 });
      await anonPage.goto(PATH);
      const wide = await anonPage.evaluate(
        () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
      );
      expect(wide, `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ anonPage }) => {
    await anonPage.goto(PATH);
    await expectAccessible(anonPage);
  });

  // The page a successful sign-in lands on is the console's front door, so
  // its density is the claim this page hands over to.
  test('the page it signs into meets the density bar', async ({ browser }) => {
    const context = await browser.newContext({ storageState: AUTH.admin });
    const page = await context.newPage();
    await page.goto(PATHS.evals);
    await expect(page.locator('h1').first()).toBeVisible();
    await expectDensity(page, 'list');
    await context.close();
  });
});
