import { Page } from '@playwright/test';

import { hideIntro } from './helpersIntro';

export async function openApp(page: Page) {
  await page.goto('/');

  await hideIntro(page);
}
