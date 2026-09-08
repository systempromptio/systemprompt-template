// The evals page — its tabs over one window — and one run's detail.
//
// Tabs are links carrying `?tab=`, so switching is a navigation; the run
// launcher is the one real mutation and is driven by the spec.
import { expect, type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';
import { SEL } from './selectors';

export type EvalsTab = 'overview' | 'traffic' | 'judge' | 'head-to-head' | 'golden-set';

export class EvalsPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.evals);
  }

  async openTab(tab: EvalsTab, preset = '30d'): Promise<void> {
    await this.page.goto(`${PATHS.evals}?tab=${tab}&preset=${preset}`);
    await expect(this.page.locator('h1').first()).toBeVisible();
  }

  activeTab(): Locator {
    return this.page.locator(`${SEL.tab}[aria-selected="true"]`);
  }

  /** Every run link on the page, in row order. */
  runLinks(): Locator {
    return this.page.locator(`${SEL.tableRow} a[href^="/admin/evals/runs/"]`);
  }

  async openFirstRun(): Promise<EvalRunDetailPage> {
    await this.runLinks().first().click();
    await expect(this.page).toHaveURL(/\/admin\/evals\/runs\/.+/);
    return new EvalRunDetailPage(this.page, this.page.url());
  }

  /** The form that launches a judge run. */
  runForm(): Locator {
    return this.page.locator('form[action$="/evals/run"]');
  }
}

export class EvalRunDetailPage extends BasePage {
  constructor(page: Page, path: string) {
    super(page, path);
  }

  verdictBadges(): Locator {
    return this.page.locator(`${SEL.tableRow} ${SEL.badge}`);
  }
}
