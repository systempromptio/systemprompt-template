// The two describe-block bodies every page spec shares.
//
// The authorization block is a table, not hand-written tests: a literal
// expected status per principal is the only form in which a reviewer can see
// at a glance that the matrix is complete and that nothing was quietly
// relaxed. The design-language block asserts the shell, the tokens, focus,
// reflow and axe the same way on every page, so those live here too — a page
// spec adds the density measurement itself (scripts/check-spec-shape.sh wants
// to see the call in the file) and anything the page does differently.
import type { Page } from '@playwright/test';
import { expectAccessible } from './a11y';
import { AUTH, expect, test, type AccessRow } from './fixtures';
import { BasePage } from './pages/base.page';
import { SEL, TOKENS } from './pages/selectors';

// A detail page's path carries a generated id the spec resolves in
// beforeAll, after these tests are registered — so a path may be a thunk,
// read when the test runs rather than when it is declared.
export type PathLike = string | (() => string);

function resolve(path: PathLike): string {
  return typeof path === 'function' ? path() : path;
}

/** Register one test per principal, asserting the literal status. */
export function authorizationTable(path: PathLike, rows: AccessRow[]): void {
  for (const { principal, status } of rows) {
    test(`${principal} gets ${status}`, async ({ browser }) => {
      const context = await browser.newContext(
        principal === 'anon' ? {} : { storageState: AUTH[principal] },
      );
      const res = await context.request.get(resolve(path), { maxRedirects: 0 });
      expect(res.status()).toBe(status);
      await context.close();
    });
  }
}

export interface DesignOptions {
  /** Which signed-in page drives the checks; the admin unless the page is an account page. */
  as?: 'adminPage' | 'userPage';
  /** The sidebar href the page must mark current, when it differs from the path. */
  navPath?: string;
  /** Query the page needs to render its populated state. */
  query?: Record<string, string>;
}

/** Register the shell, token, focus, reflow and axe tests for one page. */
export function designLanguageTests(path: PathLike, opts: DesignOptions = {}): void {
  const as = opts.as ?? 'adminPage';
  const query = opts.query ?? {};
  const navPath = () => opts.navPath ?? resolve(path);
  const open = async (page: Page): Promise<BasePage> => {
    const object = new BasePage(page, resolve(path));
    await object.goto(query);
    return object;
  };

  // Playwright insists the callback destructure its fixtures, so both pages
  // are named and the one this page belongs to is picked afterwards.
  const pick = (pages: { adminPage: Page; userPage: Page }): Page => pages[as];

  test('wears the shell', async ({ adminPage, userPage }) => {
    const page = await open(pick({ adminPage, userPage }));
    // scope.js legitimately appends the active preset to AI-activity links
    // when the URL carries one, so only the path is the shell's claim.
    const href = await page.activeNavItem().getAttribute('href');
    expect(new URL(href ?? '', 'http://shell.local').pathname).toBe(navPath());
    await expect(page.breadcrumb()).toBeVisible();
    await expect(page.heading()).toBeVisible();
  });

  test('paints from design tokens', async ({ adminPage, userPage }) => {
    const driver = pick({ adminPage, userPage });
    await open(driver);
    const values = await driver.evaluate((tokens) => {
      const style = getComputedStyle(document.documentElement);
      return tokens.map((t) => style.getPropertyValue(t).trim());
    }, Object.values(TOKENS));
    for (const value of values) expect(value).not.toBe('');
  });

  // The first Tab lands on the skip link, which is a real control but not one
  // that represents the page — so tab past it to the first control a reader
  // would actually reach. The ring is not always an outline: the skip link
  // and some controls draw theirs with a box shadow, so either satisfies
  // WCAG 1.4.11 and either is accepted here.
  test('keeps a visible focus ring', async ({ adminPage, userPage }) => {
    const driver = pick({ adminPage, userPage });
    await open(driver);
    await driver.keyboard.press('Tab');
    if (await driver.locator('.sp-skip-link:focus').count()) {
      await driver.keyboard.press('Tab');
    }
    const ring = await driver.evaluate(() => {
      const el = document.activeElement;
      if (!el || el === document.body) return null;
      const s = getComputedStyle(el);
      return { outline: `${s.outlineStyle} ${s.outlineWidth}`, shadow: s.boxShadow, tag: el.tagName };
    });
    expect(ring, 'nothing took focus on Tab').not.toBeNull();
    const hasOutline =
      !ring?.outline.startsWith('none') && Number.parseFloat(ring?.outline.split(' ')[1] ?? '0') > 0;
    const hasShadow = !!ring?.shadow && ring.shadow !== 'none';
    expect(hasOutline || hasShadow, `no focus ring on <${ring?.tag}>: ${JSON.stringify(ring)}`).toBe(
      true,
    );
  });

  test('does not scroll horizontally at 1440 or 1024', async ({ adminPage, userPage }) => {
    const driver = pick({ adminPage, userPage });
    const page = new BasePage(driver, resolve(path));
    for (const width of [1440, 1024]) {
      await driver.setViewportSize({ width, height: 900 });
      await page.goto(query);
      expect(await page.hasHorizontalScroll(), `horizontal scroll at ${width}px`).toBe(false);
    }
  });

  test('has no serious or critical axe violations', async ({ adminPage, userPage }) => {
    const driver = pick({ adminPage, userPage });
    await open(driver);
    await expect(driver.locator(SEL.pageHeader)).toBeVisible();
    await expectAccessible(driver);
  });
}
