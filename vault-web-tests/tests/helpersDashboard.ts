import { Locator, Page } from '@playwright/test';

export function navbarHeaderLocator(page: Page): Locator {
  return page.getByRole('banner');
}

export function breadcrumbsLocator(page: Page): Locator {
  return navbarHeaderLocator(page).getByLabel('Breadcrumb');
}
