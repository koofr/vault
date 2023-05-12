import { Locator, Page } from '@playwright/test';

export function navbarLocator(page: Page): Locator {
  return page.getByRole('banner');
}

export function breadcrumbsLocator(page: Page): Locator {
  return navbarLocator(page).getByLabel('Breadcrumb');
}

export function navbarLogoLocator(page: Page): Locator {
  return navbarLocator(page).getByLabel('Koofr Vault logo');
}

export function navbarCloseLocator(page: Page): Locator {
  return navbarLocator(page).getByLabel('Close');
}
