import { Page, expect } from '@playwright/test';

export async function createRepo(page: Page) {
  // firefox needs click before fill
  await page.getByLabel('Safe Key').click();
  await page.getByLabel('Safe Key').fill('password');
  await page.getByRole('button', { name: 'Create' }).click();
  await page.getByRole('button', { name: 'Copy' }).click();
  await expect(page.getByRole('button', { name: 'Copied' })).toBeVisible();
  await page.getByRole('button', { name: 'Continue' }).click();
}

export async function unlockRepo(page: Page) {
  await expect(
    page.getByText('Enter your Safe Key to unlock', { exact: true })
  ).toBeVisible();
  // firefox needs click before fill
  await page.getByLabel('Safe Key').click();
  await page.getByLabel('Safe Key').fill('password');
  await page.getByRole('button', { name: 'Unlock' }).click();
}

export async function createUnlockedRepo(page: Page) {
  await createRepo(page);
  await unlockRepo(page);
}
