import { Page, expect } from '@playwright/test';

import { KoofrApiClient } from './koofrApiClient';

export async function createRepo(client: KoofrApiClient) {
  await client.createTestVaultRepo();
}

export async function unlockRepo(page: Page) {
  await expect(
    page.getByText('Enter your Safe Key to continue', { exact: true })
  ).toBeVisible();
  // firefox needs click before fill
  await page.getByLabel('Safe Key').click();
  await page.getByLabel('Safe Key').fill('password');
  await page.getByRole('button', { name: 'Continue' }).click();
}

export async function createUnlockedRepo(page: Page, client: KoofrApiClient) {
  await createRepo(client);
  await unlockRepo(page);
}
