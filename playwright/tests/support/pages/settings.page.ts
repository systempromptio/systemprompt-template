// Page object for /admin/settings — the account form and the danger zone.
import { type Locator } from '@playwright/test';
import { BasePage } from './base.page';
import { PATHS } from '../paths';

export class SettingsPage extends BasePage {
  constructor(page: import('@playwright/test').Page) {
    super(page, PATHS.settings);
  }

  displayName(): Locator {
    return this.page.locator('#settings-display-name');
  }

  email(): Locator {
    return this.page.locator('#settings-email');
  }

  timezone(): Locator {
    return this.page.locator('#settings-timezone');
  }

  saveButton(): Locator {
    return this.page.locator('#save-settings-btn');
  }

  saveStatus(): Locator {
    return this.page.locator('#save-status');
  }

  deleteAccountButton(): Locator {
    return this.page.locator('#delete-account-btn');
  }
}
