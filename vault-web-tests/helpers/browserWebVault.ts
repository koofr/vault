import { Page } from '@playwright/test';

import { waitFor } from './time';

export async function browserWebVaultWait(page: Page) {
  await waitFor(async () => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    return await page.evaluate(() => (window as any).webVault !== undefined);
  }, 5000);
}
