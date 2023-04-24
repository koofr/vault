import { expect } from '@playwright/test';

import { test } from './base';
import { hideIntro } from './helpersIntro';

test('create a new safe box', async ({ page }) => {
  await page.goto('/');
  await page.goto('/');
  hideIntro(page);

  // firefox needs click before fill
  await page.getByLabel('Safe Key').click();
  await page.getByLabel('Safe Key').fill('password');
  await page.getByRole('button', { name: 'Create' }).click();
  await page.getByRole('button', { name: 'Copy' }).click();
  await expect(page.getByRole('button', { name: 'Copied' })).toBeVisible();
  await page.getByRole('button', { name: 'Continue' }).click();
  // firefox needs click before fill
  await page.getByLabel('Safe Key').click();
  await page.getByLabel('Safe Key').fill('password');
  await page.getByRole('button', { name: 'Unlock' }).click();
  await expect(
    page.getByRole('link', { name: 'My private documents' })
  ).toBeVisible();
});
