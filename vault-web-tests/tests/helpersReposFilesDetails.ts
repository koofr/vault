import { Locator, Page } from '@playwright/test';

export function viewerIframeLocator(page: Page): Locator {
  return page.getByRole('main').getByTitle('Viewer');
}
