// Page object for /admin/setup — the four-phase onboarding table.
import { type Locator } from '@playwright/test';
import { BasePage } from './base.page';
import { PATHS } from '../paths';

export class SetupPage extends BasePage {
  constructor(page: import('@playwright/test').Page) {
    super(page, PATHS.setup);
  }

  /** The phase rows, in order. */
  phases(): Locator {
    return this.page.locator('.sp-table__el tbody tr');
  }

  /** The phase currently highlighted as the one to do next, if any. */
  currentPhase(): Locator {
    return this.page.locator('.sp-table__el tbody tr.is-active');
  }

  guideLinks(): Locator {
    return this.page.locator('.sp-table__cell--actions a');
  }
}
