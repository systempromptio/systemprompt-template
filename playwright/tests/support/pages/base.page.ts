// The base page object every admin page spec extends.
//
// It owns the shell — nav, breadcrumb, heading, toasts, the confirm dialog —
// plus the time-range control that appears on nearly every listing.
// Page-specific objects add only what is genuinely unique to their page.
import { expect, type Locator, type Page } from '@playwright/test';
import { SEL } from './selectors';
import { TableHandle } from './table';

export class BasePage {
  constructor(
    readonly page: Page,
    readonly path: string,
  ) {}

  // Waits on any h1, not on the design-system title. "The page rendered" and
  // "the page wears the design system" are different claims, and collapsing
  // them means a page with a bespoke header cannot be driven at all. The
  // second claim belongs in the design-language block, which asserts
  // heading() directly.
  async goto(query: Record<string, string> = {}): Promise<void> {
    const qs = new URLSearchParams(query).toString();
    const separator = this.path.includes('?') ? '&' : '?';
    const url = qs ? `${this.path}${separator}${qs}` : this.path;
    await this.page.goto(url);
    await expect(this.page.locator('h1').first()).toBeVisible();
  }

  nav(): Locator {
    return this.page.locator(SEL.nav);
  }

  activeNavItem(): Locator {
    return this.page.locator(`${SEL.navLinkActive}:not([href=""]), ${SEL.nav} a.is-ancestor`);
  }

  breadcrumb(): Locator {
    return this.page.locator(SEL.breadcrumb);
  }

  /** The design-system page title. Not every page has one yet. */
  heading(): Locator {
    return this.page.locator(SEL.heading).first();
  }

  toast(): Locator {
    return this.page.locator(SEL.toast).first();
  }

  emptyState(): Locator {
    return this.page.locator(SEL.empty).first();
  }

  tabs(): Locator {
    return this.page.locator(SEL.tab);
  }

  table(name?: string): TableHandle {
    return new TableHandle(this.page, name);
  }

  /** The KPI tile whose label matches `label`, as value + locator. */
  kpi(label: string): KpiHandle {
    return new KpiHandle(this.page, label);
  }

  /** Follow a time-range link (`7d`, `30d`, …) and wait for the re-render. */
  async range(value: string): Promise<void> {
    await this.page.locator(`${SEL.rangeLink}[data-scope-range="${value}"]`).first().click();
    await this.page.waitForLoadState('networkidle');
  }

  /** The open confirm dialog, addressed through its shadow content. */
  confirmDialog(): Locator {
    return this.page.locator(SEL.dialog);
  }

  /** Answer the confirm dialog. Fails if no dialog is open.
   *
   *  Why not the host element: sp-confirm-dialog is zero-size and its overlay
   *  lives in the shadow root, so toBeVisible() on the host is false even with
   *  is-open set — this could never pass. The role="dialog" node inside is the
   *  thing on screen, and Playwright pierces the shadow root to reach it.
   *
   *  The buttons are matched by their data-role, which is what the component
   *  binds its own listeners to; `name` narrows by accessible name for a page
   *  that renders more than one confirmation at once.
   */
  async confirm(accept = true, name?: RegExp): Promise<void> {
    const dialog = this.confirmDialog();
    await expect(dialog).toBeVisible();
    const button = this.page.locator(accept ? SEL.dialogConfirm : SEL.dialogCancel);
    await (name ? button.filter({ hasText: name }) : button).first().click();
    await expect(dialog).toBeHidden();
  }

  /** True when the document is wider than the viewport at the current size. */
  async hasHorizontalScroll(): Promise<boolean> {
    return this.page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1,
    );
  }
}

export class KpiHandle {
  readonly locator: Locator;

  constructor(page: Page, readonly label: string) {
    this.locator = page.locator(SEL.kpi).filter({ hasText: label }).first();
  }

  async value(): Promise<string> {
    return (await this.locator.locator(SEL.kpiValue).innerText()).trim();
  }

  /** The KPI's value parsed as a number, with thousands separators and units stripped. */
  async number(): Promise<number> {
    const raw = (await this.value()).replace(/[^0-9.\-]/g, '');
    const n = Number(raw);
    if (Number.isNaN(n)) throw new Error(`KPI "${this.label}" is not numeric: ${await this.value()}`);
    return n;
  }
}
