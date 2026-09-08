// The governance hooks page: the hook contract this instance exposes to a
// Claude Code plugin, with the export a developer pastes into their settings.
import { type Locator, type Page } from '@playwright/test';
import { PATHS } from '../paths';
import { BasePage } from './base.page';

export class GovernanceHooksPage extends BasePage {
  constructor(page: Page) {
    super(page, PATHS.governanceHooks);
  }

  /** The rendered hook configuration block. */
  exportBlock(): Locator {
    return this.page.locator('.sp-p-hooks__export');
  }

  /** The copy control beside the export, if the page offers one. */
  copyButton(): Locator {
    return this.page.getByRole('button', { name: /copy/i }).first();
  }
}
